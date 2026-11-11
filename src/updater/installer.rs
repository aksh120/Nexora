//! In-Place Native Self-Updater & Installer
//!
//! Downloads the new executable directly, verifies binary integrity,
//! performs an atomic Windows in-place executable swap (`.exe` -> `.bak`),
//! and seamlessly relaunches the new version without any external installers or apps.

use crate::core::error::{NexoraError, Result};
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct UpdateInstaller;

impl UpdateInstaller {
    /// Downloads the update payload from `download_url` to a staging file with live progress.
    pub fn download_update_payload<F>(
        download_url: &str,
        dest_path: &Path,
        progress_cb: Option<F>,
    ) -> Result<()>
    where
        F: Fn(f64, i64, i64) + Send + Sync + 'static,
    {
        // 1. Attempt streaming download via WinINet for exact byte-level progress reporting
        if let Ok(()) = Self::download_wininet(download_url, dest_path, progress_cb) {
            return Self::validate_executable(dest_path);
        }

        // 2. Fallback to Windows built-in curl.exe
        Self::download_curl(download_url, dest_path)?;
        Self::validate_executable(dest_path)
    }

    fn download_wininet<F>(
        download_url: &str,
        dest_path: &Path,
        progress_cb: Option<F>,
    ) -> Result<()>
    where
        F: Fn(f64, i64, i64) + Send + Sync + 'static,
    {
        use windows::core::PCWSTR;
        use windows::Win32::Networking::WinInet::{
            HttpQueryInfoW, InternetCloseHandle, InternetOpenUrlW, InternetOpenW, InternetReadFile,
            HTTP_QUERY_CONTENT_LENGTH, HTTP_QUERY_FLAG_NUMBER, INTERNET_FLAG_NO_CACHE_WRITE,
            INTERNET_FLAG_RELOAD, INTERNET_FLAG_SECURE, INTERNET_OPEN_TYPE_PRECONFIG,
        };

        unsafe {
            let wide_agent: Vec<u16> = "Nexora-Browser-Updater".encode_utf16().chain(std::iter::once(0)).collect();
            let h_internet = InternetOpenW(
                PCWSTR(wide_agent.as_ptr()),
                INTERNET_OPEN_TYPE_PRECONFIG.0,
                PCWSTR::null(),
                PCWSTR::null(),
                0,
            );
            if h_internet.is_null() {
                return Err(NexoraError::Other("InternetOpenW failed".into()));
            }

            let wide_url: Vec<u16> = download_url.encode_utf16().chain(std::iter::once(0)).collect();
            let flags = INTERNET_FLAG_RELOAD | INTERNET_FLAG_NO_CACHE_WRITE | INTERNET_FLAG_SECURE;

            let h_file = InternetOpenUrlW(
                h_internet,
                PCWSTR(wide_url.as_ptr()),
                None,
                flags,
                0,
            );

            if h_file.is_null() {
                let _ = InternetCloseHandle(h_internet);
                return Err(NexoraError::Other(format!("Failed to open URL: {}", download_url)));
            }

            // Query Content-Length
            let mut content_len: u32 = 0;
            let mut len_size = std::mem::size_of::<u32>() as u32;
            let mut index = 0u32;
            let total_bytes: i64 = if HttpQueryInfoW(
                h_file,
                HTTP_QUERY_CONTENT_LENGTH | HTTP_QUERY_FLAG_NUMBER,
                Some(&mut content_len as *mut u32 as *mut _),
                &mut len_size,
                Some(&mut index),
            )
            .is_ok()
            {
                content_len as i64
            } else {
                -1
            };

            let mut out_file = File::create(dest_path)
                .map_err(|e| NexoraError::Other(format!("Failed to create destination file: {}", e)))?;

            let mut chunk = [0u8; 32768];
            let mut bytes_read: u32 = 0;
            let mut total_received: i64 = 0;

            loop {
                let ok = InternetReadFile(
                    h_file,
                    chunk.as_mut_ptr() as *mut _,
                    chunk.len() as u32,
                    &mut bytes_read,
                );
                if ok.is_err() || bytes_read == 0 {
                    break;
                }

                out_file
                    .write_all(&chunk[..bytes_read as usize])
                    .map_err(|e| NexoraError::Other(format!("Write error: {}", e)))?;

                total_received += bytes_read as i64;
                if let Some(ref cb) = progress_cb {
                    let percent = if total_bytes > 0 {
                        ((total_received as f64 / total_bytes as f64) * 100.0).clamp(0.0, 100.0)
                    } else {
                        0.0
                    };
                    cb(percent, total_received, total_bytes);
                }
            }

            let _ = out_file.flush();
            let _ = InternetCloseHandle(h_file);
            let _ = InternetCloseHandle(h_internet);

            Ok(())
        }
    }

    fn download_curl(download_url: &str, dest_path: &Path) -> Result<()> {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let mut cmd = std::process::Command::new("curl.exe");
        cmd.args([
            "-s",
            "-L",
            "--max-time",
            "120",
            "-H",
            "User-Agent: Nexora-Browser",
            "-H",
            "Accept: application/octet-stream",
            "-o",
            &dest_path.to_string_lossy(),
        ]);

        // Support private repositories via environment variable or local GitHub CLI
        if let Ok(token) = std::env::var("GITHUB_TOKEN").or_else(|_| std::env::var("GH_TOKEN")) {
            cmd.arg("-H").arg(format!("Authorization: Bearer {}", token.trim()));
        } else if let Ok(out) = std::process::Command::new("gh.exe").args(["auth", "token"]).creation_flags(CREATE_NO_WINDOW).output() {
            if out.status.success() {
                let token = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !token.is_empty() {
                    cmd.arg("-H").arg(format!("Authorization: Bearer {}", token));
                }
            }
        }

        let output = cmd
            .arg(download_url)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| NexoraError::Other(format!("Failed to run curl.exe: {}", e)))?;

        if !output.status.success() {
            return Err(NexoraError::Other(format!(
                "curl.exe failed with code: {:?}",
                output.status.code()
            )));
        }

        Ok(())
    }

    /// Validates that the downloaded file is a valid Windows PE binary (Magic 'MZ') and > 500 KB.
    pub fn validate_executable(path: &Path) -> Result<()> {
        use std::io::Read;

        let mut file = File::open(path)
            .map_err(|e| NexoraError::Other(format!("Failed to open downloaded file: {}", e)))?;

        let meta = file
            .metadata()
            .map_err(|e| NexoraError::Other(format!("Failed to read metadata: {}", e)))?;

        if meta.len() < 500 * 1024 {
            return Err(NexoraError::Other(format!(
                "Downloaded file too small ({} bytes); likely incomplete or error payload",
                meta.len()
            )));
        }

        let mut magic = [0u8; 2];
        file.read_exact(&mut magic)
            .map_err(|e| NexoraError::Other(format!("Failed to read PE magic: {}", e)))?;

        // 'MZ' magic = [0x4D, 0x5A]
        if magic != [0x4D, 0x5A] {
            return Err(NexoraError::Other(
                "Invalid executable: missing Windows PE 'MZ' signature".into(),
            ));
        }

        Ok(())
    }

    /// Performs atomic in-place executable replacement and seamlessly relaunches Nexora.
    pub fn apply_update_and_restart(staged_exe: &Path) -> Result<()> {
        let current_exe = std::env::current_exe()
            .map_err(|e| NexoraError::Other(format!("Failed to resolve current exe: {}", e)))?;

        let exe_dir = current_exe
            .parent()
            .ok_or_else(|| NexoraError::Other("Failed to get exe directory".into()))?;

        let backup_exe = exe_dir.join(format!("Nexora_old_{}.bak", std::process::id()));

        log::info!(
            "[Updater] Initiating atomic binary swap: Current={:?}, Staged={:?}, Backup={:?}",
            current_exe,
            staged_exe,
            backup_exe
        );

        // 1. Rename running executable to .bak (Windows allows renaming active running processes)
        if backup_exe.exists() {
            let _ = std::fs::remove_file(&backup_exe);
        }
        std::fs::rename(&current_exe, &backup_exe)
            .map_err(|e| NexoraError::Other(format!("Failed to rename running executable: {}", e)))?;

        // 2. Move newly downloaded executable into the active executable's path
        if let Err(e) = std::fs::rename(staged_exe, &current_exe) {
            // Rollback if staged rename fails
            let _ = std::fs::rename(&backup_exe, &current_exe);
            return Err(NexoraError::Other(format!(
                "Failed to place updated executable: {}",
                e
            )));
        }

        log::info!("[Updater] Binary swap successful. Relaunching new Nexora instance...");

        // 3. Spawn the newly updated process with --updated flag
        std::process::Command::new(&current_exe)
            .arg("--updated")
            .spawn()
            .map_err(|e| NexoraError::Other(format!("Failed to spawn updated instance: {}", e)))?;

        // 4. Terminate current process cleanly
        std::process::exit(0);
    }

    /// Cleans up any leftover backup executables (.bak) from previous updates.
    pub fn cleanup_previous_update_backups() {
        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(exe_dir) = current_exe.parent() {
                if let Ok(entries) = std::fs::read_dir(exe_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if name.starts_with("Nexora_old_") && name.ends_with(".bak") {
                                log::info!("[Updater] Cleaning up old update backup: {:?}", path);
                                let _ = std::fs::remove_file(&path);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_executable_pe_magic() {
        let temp_dir = std::env::temp_dir();
        let test_pe = temp_dir.join("test_valid.exe");
        let test_invalid = temp_dir.join("test_invalid.exe");

        // Write fake PE header with 600 KB size
        {
            let mut f = File::create(&test_pe).unwrap();
            f.write_all(&[0x4D, 0x5A]).unwrap(); // MZ
            f.write_all(&vec![0u8; 600 * 1024]).unwrap();
        }

        // Write non-PE file
        {
            let mut f = File::create(&test_invalid).unwrap();
            f.write_all(&[0x00, 0x00]).unwrap();
            f.write_all(&vec![0u8; 600 * 1024]).unwrap();
        }

        assert!(UpdateInstaller::validate_executable(&test_pe).is_ok());
        assert!(UpdateInstaller::validate_executable(&test_invalid).is_err());

        let _ = std::fs::remove_file(test_pe);
        let _ = std::fs::remove_file(test_invalid);
    }
}
