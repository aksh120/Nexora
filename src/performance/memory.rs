use std::mem;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentProcessId, OpenProcess, SetProcessWorkingSetSize,
    PROCESS_QUERY_INFORMATION, PROCESS_SET_QUOTA,
};
use crate::core::error::{NexoraError, Result};

/// Process memory consumption statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProcessMemoryInfo {
    pub working_set_bytes: usize,
    pub peak_working_set_bytes: usize,
    pub pagefile_usage_bytes: usize,
    pub peak_pagefile_usage_bytes: usize,
}

impl ProcessMemoryInfo {
    pub fn working_set_mb(&self) -> f64 {
        self.working_set_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn peak_working_set_mb(&self) -> f64 {
        self.peak_working_set_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn pagefile_usage_mb(&self) -> f64 {
        self.pagefile_usage_bytes as f64 / (1024.0 * 1024.0)
    }
}

/// Statistics reported after trimming working set
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkingSetTrimResult {
    pub before_bytes: usize,
    pub after_bytes: usize,
    pub bytes_freed: usize,
}

impl WorkingSetTrimResult {
    pub fn freed_mb(&self) -> f64 {
        self.bytes_freed as f64 / (1024.0 * 1024.0)
    }
}

/// Query current process memory usage via Win32 `GetProcessMemoryInfo`
pub fn get_process_memory() -> Result<ProcessMemoryInfo> {
    unsafe {
        let process = GetCurrentProcess();
        let mut counters = PROCESS_MEMORY_COUNTERS {
            cb: mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            PageFaultCount: 0,
            PeakWorkingSetSize: 0,
            WorkingSetSize: 0,
            QuotaPeakPagedPoolUsage: 0,
            QuotaPagedPoolUsage: 0,
            QuotaPeakNonPagedPoolUsage: 0,
            QuotaNonPagedPoolUsage: 0,
            PagefileUsage: 0,
            PeakPagefileUsage: 0,
        };

        GetProcessMemoryInfo(
            process,
            &mut counters,
            mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        ).map_err(|e| NexoraError::Win32(format!("Failed to query GetProcessMemoryInfo: {:?}", e)))?;

        Ok(ProcessMemoryInfo {
            working_set_bytes: counters.WorkingSetSize,
            peak_working_set_bytes: counters.PeakWorkingSetSize,
            pagefile_usage_bytes: counters.PagefileUsage,
            peak_pagefile_usage_bytes: counters.PeakPagefileUsage,
        })
    }
}

/// Instruct the Windows Memory Manager to trim unused physical pages from the working set
/// of the current process and all child WebView2 processes.
pub fn trim_working_set() -> Result<WorkingSetTrimResult> {
    let before = get_process_memory().unwrap_or_default();

    unsafe {
        let process = GetCurrentProcess();
        // Passing (usize::MAX, usize::MAX) indicates to Windows to flush working set pages to standby
        SetProcessWorkingSetSize(process, usize::MAX, usize::MAX)
            .map_err(|e| NexoraError::Win32(format!("Failed to trim working set: {:?}", e)))?;
    }

    // Also trim all child processes (WebView2 browser, renderer, GPU, and utility processes)
    let _ = trim_child_processes();

    let after = get_process_memory().unwrap_or_default();
    let bytes_freed = if before.working_set_bytes > after.working_set_bytes {
        before.working_set_bytes - after.working_set_bytes
    } else {
        0
    };

    log::info!(
        "Trimmed process working set: {:.2} MB -> {:.2} MB (freed {:.2} MB)",
        before.working_set_mb(),
        after.working_set_mb(),
        bytes_freed as f64 / (1024.0 * 1024.0)
    );

    Ok(WorkingSetTrimResult {
        before_bytes: before.working_set_bytes,
        after_bytes: after.working_set_bytes,
        bytes_freed,
    })
}

/// Trims the working sets of all child processes (including WebView2 browser & renderer processes)
pub fn trim_child_processes() -> usize {
    let parent_pid = unsafe { GetCurrentProcessId() };
    let mut trimmed_count = 0;

    unsafe {
        let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let mut entry = PROCESSENTRY32W {
            dwSize: mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        let mut child_pids = Vec::new();

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                if entry.th32ParentProcessID == parent_pid {
                    child_pids.push(entry.th32ProcessID);
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }

        // Also discover any grandchild processes spawned by WebView2 child processes
        if !child_pids.is_empty() {
            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    if child_pids.contains(&entry.th32ParentProcessID) && !child_pids.contains(&entry.th32ProcessID) {
                        child_pids.push(entry.th32ProcessID);
                    }
                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }
        }

        let _ = CloseHandle(snapshot);

        for pid in child_pids {
            if let Ok(h_proc) = OpenProcess(PROCESS_SET_QUOTA | PROCESS_QUERY_INFORMATION, false, pid) {
                if SetProcessWorkingSetSize(h_proc, usize::MAX, usize::MAX).is_ok() {
                    trimmed_count += 1;
                }
                let _ = CloseHandle(h_proc);
            }
        }
    }

    if trimmed_count > 0 {
        log::info!("Trimmed working set for {} child WebView2 processes", trimmed_count);
    }
    trimmed_count
}

/// Query memory usage for host process and sum of all child WebView2 processes
pub fn get_tree_memory() -> (usize, usize, usize) {
    let host_info = get_process_memory().unwrap_or_default();
    let parent_pid = unsafe { GetCurrentProcessId() };
    let mut children_ws = 0;
    let mut child_count = 0;

    unsafe {
        if let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            let mut entry = PROCESSENTRY32W {
                dwSize: mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };

            let mut child_pids = Vec::new();
            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    if entry.th32ParentProcessID == parent_pid {
                        child_pids.push(entry.th32ProcessID);
                    }
                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }

            // Descendant processes
            if !child_pids.is_empty() {
                if Process32FirstW(snapshot, &mut entry).is_ok() {
                    loop {
                        if child_pids.contains(&entry.th32ParentProcessID) && !child_pids.contains(&entry.th32ProcessID) {
                            child_pids.push(entry.th32ProcessID);
                        }
                        if Process32NextW(snapshot, &mut entry).is_err() {
                            break;
                        }
                    }
                }
            }

            let _ = CloseHandle(snapshot);

            child_count = child_pids.len();
            for pid in child_pids {
                if let Ok(h_proc) = OpenProcess(PROCESS_QUERY_INFORMATION, false, pid) {
                    let mut counters = PROCESS_MEMORY_COUNTERS {
                        cb: mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
                        ..Default::default()
                    };
                    if GetProcessMemoryInfo(h_proc, &mut counters, mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32).is_ok() {
                        children_ws += counters.WorkingSetSize;
                    }
                    let _ = CloseHandle(h_proc);
                }
            }
        }
    }

    (host_info.working_set_bytes, children_ws, child_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_process_memory() {
        let mem_info = get_process_memory();
        assert!(mem_info.is_ok(), "Getting process memory info should succeed");
        let info = mem_info.unwrap();
        assert!(info.working_set_bytes > 0, "Working set must be greater than 0");
        assert!(info.working_set_mb() > 0.0);
    }

    #[test]
    fn test_trim_working_set() {
        let trim_result = trim_working_set();
        assert!(trim_result.is_ok(), "Trimming working set must succeed on Windows");
        let result = trim_result.unwrap();
        assert!(result.before_bytes > 0);
        assert!(result.after_bytes > 0);
    }
}
