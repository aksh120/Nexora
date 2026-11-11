//! Native Download Manager
//!
//! Tracks file downloads, transfer speed, progress calculation,
//! pause/resume states, and post-download SHA-256 integrity hash verification.

use std::fs::File;
use std::io::{Read, Result as IoResult};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DownloadState {
    InProgress,
    Paused,
    Completed,
    Cancelled,
    Failed,
}

impl DownloadState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InProgress => "In Progress",
            Self::Paused => "Paused",
            Self::Completed => "Completed",
            Self::Cancelled => "Cancelled",
            Self::Failed => "Failed",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DownloadItem {
    pub id: u64,
    pub url: String,
    pub file_name: String,
    pub file_path: PathBuf,
    pub total_bytes: i64,
    pub received_bytes: i64,
    pub state: DownloadState,
    #[serde(skip, default = "Instant::now")]
    pub start_time: Instant,
    pub speed_mb_s: f64,
    pub sha256_hash: Option<String>,
}

impl DownloadItem {
    pub fn progress_percent(&self) -> f64 {
        if self.total_bytes <= 0 {
            0.0
        } else {
            ((self.received_bytes as f64 / self.total_bytes as f64) * 100.0).clamp(0.0, 100.0)
        }
    }

    pub fn formatted_size(&self) -> String {
        let rec_mb = self.received_bytes as f64 / (1024.0 * 1024.0);
        if self.total_bytes > 0 {
            let tot_mb = self.total_bytes as f64 / (1024.0 * 1024.0);
            format!("{:.1} / {:.1} MB", rec_mb, tot_mb)
        } else {
            format!("{:.1} MB", rec_mb)
        }
    }
}

#[derive(Debug, Clone)]
pub struct DownloadManager {
    downloads: Arc<Mutex<Vec<DownloadItem>>>,
    next_id: Arc<std::sync::atomic::AtomicU64>,
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self {
            downloads: Arc::new(Mutex::new(Vec::new())),
            next_id: Arc::new(std::sync::atomic::AtomicU64::new(1)),
        }
    }
}

impl DownloadManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_download(&self, url: &str, file_path: PathBuf, total_bytes: i64) -> u64 {
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let file_name = file_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "download".to_string());

        let item = DownloadItem {
            id,
            url: url.to_string(),
            file_name,
            file_path,
            total_bytes,
            received_bytes: 0,
            state: DownloadState::InProgress,
            start_time: Instant::now(),
            speed_mb_s: 0.0,
            sha256_hash: None,
        };

        if let Ok(mut lock) = self.downloads.lock() {
            lock.insert(0, item);
        }

        id
    }

    pub fn update_progress(&self, id: u64, received_bytes: i64, total_bytes: i64) {
        if let Ok(mut lock) = self.downloads.lock() {
            if let Some(item) = lock.iter_mut().find(|d| d.id == id) {
                let elapsed = item.start_time.elapsed().as_secs_f64();
                if elapsed > 0.1 {
                    let bytes_diff = received_bytes.saturating_sub(item.received_bytes);
                    item.speed_mb_s = (bytes_diff as f64 / (1024.0 * 1024.0)) / elapsed.max(1.0);
                }
                item.received_bytes = received_bytes;
                if total_bytes > 0 {
                    item.total_bytes = total_bytes;
                }
            }
        }
    }

    pub fn mark_completed(&self, id: u64) {
        let file_path = if let Ok(mut lock) = self.downloads.lock() {
            if let Some(item) = lock.iter_mut().find(|d| d.id == id) {
                item.state = DownloadState::Completed;
                item.received_bytes = item.total_bytes;
                item.speed_mb_s = 0.0;
                Some(item.file_path.clone())
            } else {
                None
            }
        } else {
            None
        };

        // Compute SHA-256 hash if file exists
        if let Some(path) = file_path {
            if path.exists() {
                if let Ok(hash) = Self::compute_sha256(&path) {
                    if let Ok(mut lock) = self.downloads.lock() {
                        if let Some(item) = lock.iter_mut().find(|d| d.id == id) {
                            item.sha256_hash = Some(hash);
                        }
                    }
                }
            }
        }
    }

    pub fn mark_state(&self, id: u64, state: DownloadState) {
        if let Ok(mut lock) = self.downloads.lock() {
            if let Some(item) = lock.iter_mut().find(|d| d.id == id) {
                item.state = state;
            }
        }
    }

    pub fn get_all(&self) -> Vec<DownloadItem> {
        self.downloads.lock().map(|l| l.clone()).unwrap_or_default()
    }

    pub fn clear_completed(&self) {
        if let Ok(mut lock) = self.downloads.lock() {
            lock.retain(|d| d.state != DownloadState::Completed && d.state != DownloadState::Cancelled);
        }
    }

    /// Fast standard-library SHA-256 checksum calculation without external cryptography crates
    pub fn compute_sha256(path: &Path) -> IoResult<String> {
        let mut file = File::open(path)?;
        let mut buffer = [0u8; 8192];
        let mut state = Sha256State::new();

        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            state.update(&buffer[..n]);
        }

        Ok(state.finalize())
    }
}

/// Minimal self-contained SHA-256 implementation
struct Sha256State {
    h: [u32; 8],
    len: u64,
    buffer: [u8; 64],
    buf_len: usize,
}

impl Sha256State {
    fn new() -> Self {
        Self {
            h: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
            ],
            len: 0,
            buffer: [0u8; 64],
            buf_len: 0,
        }
    }

    fn update(&mut self, data: &[u8]) {
        self.len += (data.len() as u64) * 8;
        let mut offset = 0;

        if self.buf_len > 0 {
            let to_fill = 64 - self.buf_len;
            if data.len() >= to_fill {
                self.buffer[self.buf_len..64].copy_from_slice(&data[..to_fill]);
                self.process_block();
                self.buf_len = 0;
                offset = to_fill;
            } else {
                self.buffer[self.buf_len..self.buf_len + data.len()].copy_from_slice(data);
                self.buf_len += data.len();
                return;
            }
        }

        while offset + 64 <= data.len() {
            self.buffer.copy_from_slice(&data[offset..offset + 64]);
            self.process_block();
            offset += 64;
        }

        if offset < data.len() {
            let remaining = data.len() - offset;
            self.buffer[..remaining].copy_from_slice(&data[offset..]);
            self.buf_len = remaining;
        }
    }

    fn process_block(&mut self) {
        const K: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
            0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
            0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
            0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
            0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
            0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
            0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
        ];

        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                self.buffer[i * 4],
                self.buffer[i * 4 + 1],
                self.buffer[i * 4 + 2],
                self.buffer[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }

        let mut a = self.h[0];
        let mut b = self.h[1];
        let mut c = self.h[2];
        let mut d = self.h[3];
        let mut e = self.h[4];
        let mut f = self.h[5];
        let mut g = self.h[6];
        let mut h = self.h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let temp1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        self.h[0] = self.h[0].wrapping_add(a);
        self.h[1] = self.h[1].wrapping_add(b);
        self.h[2] = self.h[2].wrapping_add(c);
        self.h[3] = self.h[3].wrapping_add(d);
        self.h[4] = self.h[4].wrapping_add(e);
        self.h[5] = self.h[5].wrapping_add(f);
        self.h[6] = self.h[6].wrapping_add(g);
        self.h[7] = self.h[7].wrapping_add(h);
    }

    fn finalize(mut self) -> String {
        self.buffer[self.buf_len] = 0x80;
        self.buf_len += 1;

        if self.buf_len > 56 {
            self.buffer[self.buf_len..64].fill(0);
            self.process_block();
            self.buf_len = 0;
        }

        self.buffer[self.buf_len..56].fill(0);
        self.buffer[56..64].copy_from_slice(&self.len.to_be_bytes());
        self.process_block();

        format!(
            "{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
            self.h[0], self.h[1], self.h[2], self.h[3],
            self.h[4], self.h[5], self.h[6], self.h[7]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_item_progress() {
        let item = DownloadItem {
            id: 1,
            url: "https://example.com/file.zip".into(),
            file_name: "file.zip".into(),
            file_path: PathBuf::from("C:\\Downloads\\file.zip"),
            total_bytes: 1000,
            received_bytes: 500,
            state: DownloadState::InProgress,
            start_time: Instant::now(),
            speed_mb_s: 1.5,
            sha256_hash: None,
        };
        assert_eq!(item.progress_percent(), 50.0);
    }

    #[test]
    fn test_sha256_empty_string() {
        let mut state = Sha256State::new();
        state.update(b"");
        let hash = state.finalize();
        assert_eq!(hash, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }
}

