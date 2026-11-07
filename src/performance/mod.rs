// Performance Optimization and Smart Suspension Subsystems
#![allow(unused_imports)]

pub mod memory;
pub mod pressure;
pub mod suspension;
pub mod lazy_restore;

pub use memory::{get_process_memory, trim_working_set, ProcessMemoryInfo, WorkingSetTrimResult};
pub use pressure::{MemoryPressureLevel, MemoryPressureMonitor, SystemMemoryInfo};
pub use suspension::SmartSuspensionEngine;
pub use lazy_restore::{LazyRestorePipeline, LazyRestorePlan};
