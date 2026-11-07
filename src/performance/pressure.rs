use std::mem;
use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
use crate::core::error::{NexoraError, Result};

/// Categorized system memory pressure level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryPressureLevel {
    /// Plentiful memory available (<75% load, >2GB free RAM)
    Normal,
    /// Memory load elevated (75%-85% load, or <=2GB free RAM)
    Moderate,
    /// Severe memory constraints (>85% load, or <1GB free RAM, or <15% free)
    Critical,
}

impl MemoryPressureLevel {
    pub fn is_under_pressure(&self) -> bool {
        *self != MemoryPressureLevel::Normal
    }

    pub fn is_critical(&self) -> bool {
        *self == MemoryPressureLevel::Critical
    }
}

/// System-wide physical and virtual memory statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemMemoryInfo {
    pub total_phys_bytes: u64,
    pub avail_phys_bytes: u64,
    pub total_pagefile_bytes: u64,
    pub avail_pagefile_bytes: u64,
    pub memory_load_percent: u32,
}

impl SystemMemoryInfo {
    pub fn total_phys_mb(&self) -> f64 {
        self.total_phys_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn avail_phys_mb(&self) -> f64 {
        self.avail_phys_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn avail_phys_percent(&self) -> f64 {
        if self.total_phys_bytes == 0 {
            0.0
        } else {
            (self.avail_phys_bytes as f64 / self.total_phys_bytes as f64) * 100.0
        }
    }
}

/// System Memory Pressure Monitor
#[derive(Debug, Clone)]
pub struct MemoryPressureMonitor {
    pub moderate_threshold_percent: u32,
    pub critical_threshold_percent: u32,
    pub critical_min_free_mb: u64,
}

impl Default for MemoryPressureMonitor {
    fn default() -> Self {
        Self {
            moderate_threshold_percent: 75,
            critical_threshold_percent: 85,
            critical_min_free_mb: 1024,
        }
    }
}

impl MemoryPressureMonitor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Query global memory statistics from Windows OS
    pub fn query_system_memory() -> Result<SystemMemoryInfo> {
        unsafe {
            let mut mem_status = MEMORYSTATUSEX {
                dwLength: mem::size_of::<MEMORYSTATUSEX>() as u32,
                dwMemoryLoad: 0,
                ullTotalPhys: 0,
                ullAvailPhys: 0,
                ullTotalPageFile: 0,
                ullAvailPageFile: 0,
                ullTotalVirtual: 0,
                ullAvailVirtual: 0,
                ullAvailExtendedVirtual: 0,
            };

            GlobalMemoryStatusEx(&mut mem_status)
                .map_err(|e| NexoraError::Win32(format!("Failed to query GlobalMemoryStatusEx: {:?}", e)))?;

            Ok(SystemMemoryInfo {
                total_phys_bytes: mem_status.ullTotalPhys,
                avail_phys_bytes: mem_status.ullAvailPhys,
                total_pagefile_bytes: mem_status.ullTotalPageFile,
                avail_pagefile_bytes: mem_status.ullAvailPageFile,
                memory_load_percent: mem_status.dwMemoryLoad,
            })
        }
    }

    /// Evaluates current memory stats against defined pressure boundaries
    pub fn evaluate_pressure(&self, info: &SystemMemoryInfo) -> MemoryPressureLevel {
        let avail_mb = (info.avail_phys_bytes / (1024 * 1024)) as u64;
        let avail_percent = info.avail_phys_percent();

        if info.memory_load_percent >= self.critical_threshold_percent
            || avail_mb < self.critical_min_free_mb
            || avail_percent < 15.0
        {
            MemoryPressureLevel::Critical
        } else if info.memory_load_percent >= self.moderate_threshold_percent || avail_mb < 2048 {
            MemoryPressureLevel::Moderate
        } else {
            MemoryPressureLevel::Normal
        }
    }

    /// Query current pressure level in real-time
    pub fn current_pressure_level(&self) -> Result<MemoryPressureLevel> {
        let info = Self::query_system_memory()?;
        Ok(self.evaluate_pressure(&info))
    }

    /// Adaptively scale the tab idle timeout (in minutes) based on current memory pressure.
    /// Under Critical pressure, idle timeout drops dramatically (e.g. from 10m to 2-3m).
    pub fn calculate_adaptive_timeout(&self, base_timeout_minutes: u32, pressure: MemoryPressureLevel) -> u32 {
        match pressure {
            MemoryPressureLevel::Normal => base_timeout_minutes,
            MemoryPressureLevel::Moderate => (base_timeout_minutes / 2).max(3),
            MemoryPressureLevel::Critical => (base_timeout_minutes / 3).clamp(1, 3),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_system_memory() {
        let mem = MemoryPressureMonitor::query_system_memory();
        assert!(mem.is_ok(), "Querying system memory must succeed on Windows");
        let info = mem.unwrap();
        assert!(info.total_phys_bytes > 0);
        assert!(info.avail_phys_bytes > 0);
        assert!(info.total_phys_mb() > 512.0);
        assert!(info.memory_load_percent <= 100);
    }

    #[test]
    fn test_evaluate_pressure_levels() {
        let monitor = MemoryPressureMonitor::default();

        // Normal scenario: 16GB total, 8GB free, 50% load
        let normal_info = SystemMemoryInfo {
            total_phys_bytes: 16 * 1024 * 1024 * 1024,
            avail_phys_bytes: 8 * 1024 * 1024 * 1024,
            total_pagefile_bytes: 20 * 1024 * 1024 * 1024,
            avail_pagefile_bytes: 12 * 1024 * 1024 * 1024,
            memory_load_percent: 50,
        };
        assert_eq!(monitor.evaluate_pressure(&normal_info), MemoryPressureLevel::Normal);

        // Moderate scenario: 16GB total, 3.2GB free (20%), 80% load
        let moderate_info = SystemMemoryInfo {
            total_phys_bytes: 16 * 1024 * 1024 * 1024,
            avail_phys_bytes: 3200 * 1024 * 1024,
            total_pagefile_bytes: 20 * 1024 * 1024 * 1024,
            avail_pagefile_bytes: 5 * 1024 * 1024 * 1024,
            memory_load_percent: 80,
        };
        assert_eq!(monitor.evaluate_pressure(&moderate_info), MemoryPressureLevel::Moderate);

        // Critical scenario: 16GB total, 800MB free, 92% load
        let critical_info = SystemMemoryInfo {
            total_phys_bytes: 16 * 1024 * 1024 * 1024,
            avail_phys_bytes: 800 * 1024 * 1024,
            total_pagefile_bytes: 20 * 1024 * 1024 * 1024,
            avail_pagefile_bytes: 1 * 1024 * 1024 * 1024,
            memory_load_percent: 92,
        };
        assert_eq!(monitor.evaluate_pressure(&critical_info), MemoryPressureLevel::Critical);
    }

    #[test]
    fn test_adaptive_timeout_scaling() {
        let monitor = MemoryPressureMonitor::default();
        let base_timeout = 10;

        assert_eq!(monitor.calculate_adaptive_timeout(base_timeout, MemoryPressureLevel::Normal), 10);
        assert_eq!(monitor.calculate_adaptive_timeout(base_timeout, MemoryPressureLevel::Moderate), 5);
        assert_eq!(monitor.calculate_adaptive_timeout(base_timeout, MemoryPressureLevel::Critical), 3);
    }
}
