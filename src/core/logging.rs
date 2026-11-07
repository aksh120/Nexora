use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use log::{LevelFilter, Log, Metadata, Record};

pub struct NexoraLogger {
    file: Option<Mutex<File>>,
    max_level: LevelFilter,
}

static LOGGER: Mutex<Option<&'static NexoraLogger>> = Mutex::new(None);

impl NexoraLogger {
    pub fn init(log_dir: &Path, max_level: LevelFilter) -> Result<(), crate::core::error::NexoraError> {
        std::fs::create_dir_all(log_dir)?;
        let log_path = log_dir.join("nexora.log");

        // Rotate or truncate if log exceeds 5MB
        if let Ok(metadata) = std::fs::metadata(&log_path) {
            if metadata.len() > 5 * 1024 * 1024 {
                let backup_path = log_dir.join("nexora.prev.log");
                let _ = std::fs::rename(&log_path, backup_path);
            }
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .ok()
            .map(Mutex::new);

        let logger = Box::leak(Box::new(NexoraLogger {
            file,
            max_level,
        }));

        log::set_logger(logger)
            .map(|()| log::set_max_level(max_level))
            .map_err(|e| crate::core::error::NexoraError::Other(e.to_string()))?;

        let mut lock = LOGGER.lock().unwrap();
        *lock = Some(logger);

        log::info!("====================================================");
        log::info!("NEXORA Browser Core Logging Initialized");
        log::info!("Version: {}", env!("CARGO_PKG_VERSION"));
        log::info!("Log Level: {}", max_level);
        log::info!("====================================================");

        Ok(())
    }
}

impl Log for NexoraLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.max_level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let now = std::time::SystemTime::now();
        let formatted = format!(
            "[{:?}] [{:5}] [{}] {}",
            now,
            record.level(),
            record.target(),
            record.args()
        );

        // 1. Output to console
        eprintln!("{}", formatted);

        // 2. Output to Windows debugger output (OutputDebugStringW)
        #[cfg(windows)]
        {
            use windows::core::HSTRING;
            use windows::Win32::System::Diagnostics::Debug::OutputDebugStringW;
            let hstring = HSTRING::from(format!("{}\n", formatted));
            unsafe {
                OutputDebugStringW(&hstring);
            }
        }

        // 3. Output to persistent log file
        if let Some(file_mutex) = &self.file {
            if let Ok(mut file) = file_mutex.lock() {
                let _ = writeln!(file, "{}", formatted);
                let _ = file.flush();
            }
        }
    }

    fn flush(&self) {
        if let Some(file_mutex) = &self.file {
            if let Ok(mut file) = file_mutex.lock() {
                let _ = file.flush();
            }
        }
    }
}
