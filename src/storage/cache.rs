use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::core::error::Result;

pub struct CacheManager;

impl CacheManager {
    pub fn calculate_dir_size(dir: &Path) -> u64 {
        if !dir.exists() || !dir.is_dir() {
            return 0;
        }

        let mut total_size = 0;
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(meta) = entry.metadata() {
                        total_size += meta.len();
                    }
                } else if path.is_dir() {
                    total_size += Self::calculate_dir_size(&path);
                }
            }
        }
        total_size
    }

    pub fn enforce_boundary(dir: &Path, max_mb: u32) -> Result<u64> {
        let max_bytes = (max_mb as u64) * 1024 * 1024;
        let current_size = Self::calculate_dir_size(dir);

        if current_size <= max_bytes {
            log::debug!(
                "Cache size {} MB within boundary {} MB",
                current_size / (1024 * 1024),
                max_mb
            );
            return Ok(0);
        }

        log::warn!(
            "Cache size {} MB exceeds limit {} MB. Enforcing boundary pruning...",
            current_size / (1024 * 1024),
            max_mb
        );

        // Collect all files with metadata
        let mut files = Vec::new();
        Self::collect_files(dir, &mut files);

        // Sort by last modified ascending (oldest first)
        files.sort_by_key(|f| f.modified);

        let target_size = (max_bytes as f64 * 0.8) as u64; // Trim down to 80%
        let mut freed_bytes = 0;
        let mut remaining_size = current_size;

        for file in files {
            if remaining_size <= target_size {
                break;
            }
            if fs::remove_file(&file.path).is_ok() {
                freed_bytes += file.size;
                remaining_size = remaining_size.saturating_sub(file.size);
            }
        }

        log::info!(
            "Cache boundary enforced: freed {} MB, current size {} MB",
            freed_bytes / (1024 * 1024),
            remaining_size / (1024 * 1024)
        );

        Ok(freed_bytes)
    }

    pub fn clear_cache(dir: &Path) -> Result<u64> {
        let size = Self::calculate_dir_size(dir);
        if dir.exists() {
            let _ = fs::remove_dir_all(dir);
            let _ = fs::create_dir_all(dir);
        }
        log::info!("Cache cleared ({} bytes reclaimed)", size);
        Ok(size)
    }

    fn collect_files(dir: &Path, files: &mut Vec<FileInfo>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(meta) = entry.metadata() {
                        let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                        files.push(FileInfo {
                            path,
                            size: meta.len(),
                            modified,
                        });
                    }
                } else if path.is_dir() {
                    Self::collect_files(&path, files);
                }
            }
        }
    }
}

struct FileInfo {
    path: std::path::PathBuf,
    size: u64,
    modified: SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_size_calculation_and_boundary() {
        let temp_dir = std::env::temp_dir().join(format!("nexora_test_cache_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();

        let f1 = temp_dir.join("cache_1.bin");
        let f2 = temp_dir.join("cache_2.bin");
        fs::write(&f1, vec![0u8; 1024 * 100]).unwrap(); // 100 KB
        fs::write(&f2, vec![1u8; 1024 * 200]).unwrap(); // 200 KB

        let total = CacheManager::calculate_dir_size(&temp_dir);
        assert_eq!(total, 1024 * 300);

        // Clear cache
        let cleared = CacheManager::clear_cache(&temp_dir).unwrap();
        assert_eq!(cleared, 1024 * 300);
        assert_eq!(CacheManager::calculate_dir_size(&temp_dir), 0);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}

