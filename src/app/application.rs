use std::path::PathBuf;
use directories::ProjectDirs;
use crate::app::config::NexoraConfig;
use crate::core::error::{NexoraError, Result};

pub struct ApplicationPaths {
    pub app_data_dir: PathBuf,
    pub data_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub engine_data_dir: PathBuf,
}

impl ApplicationPaths {
    pub fn resolve() -> Result<Self> {
        let proj_dirs = ProjectDirs::from("com", "Nexora", "Nexora")
            .ok_or_else(|| NexoraError::Other("Failed to determine system application directories".into()))?;

        let app_data_dir = proj_dirs.config_dir().to_path_buf();
        let data_dir = app_data_dir.join("Data");
        let logs_dir = app_data_dir.join("Logs");

        // LocalAppData for large, ephemeral engine caches and user data folder
        let engine_data_dir = proj_dirs.data_local_dir().join("EngineData");

        std::fs::create_dir_all(&app_data_dir)?;
        std::fs::create_dir_all(&data_dir)?;
        std::fs::create_dir_all(&logs_dir)?;
        std::fs::create_dir_all(&engine_data_dir)?;

        Ok(Self {
            app_data_dir,
            data_dir,
            logs_dir,
            engine_data_dir,
        })
    }
}

pub struct Application {
    pub paths: ApplicationPaths,
    pub config: NexoraConfig,
    _single_instance_handle: Option<windows::Win32::Foundation::HANDLE>,
}

impl Application {
    pub fn init() -> Result<Self> {
        let paths = ApplicationPaths::resolve()?;

        // Initialize core logging
        crate::core::logging::NexoraLogger::init(&paths.logs_dir, log::LevelFilter::Debug)?;

        log::info!("Application directories initialized successfully:");
        log::info!("  AppData Dir:    {:?}", paths.app_data_dir);
        log::info!("  Data Dir:       {:?}", paths.data_dir);
        log::info!("  Logs Dir:       {:?}", paths.logs_dir);
        log::info!("  EngineData Dir: {:?}", paths.engine_data_dir);

        // Set Per-Monitor V2 DPI Awareness for crisp UI scaling on Windows 10 & 11
        #[cfg(windows)]
        unsafe {
            use windows::Win32::UI::HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2};
            if let Err(err) = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) {
                log::warn!("Could not set PerMonitorV2 DPI awareness (may have inherited): {}", err);
            } else {
                log::info!("Per-Monitor V2 DPI awareness successfully applied.");
            }
        }

        // Verify single instance via Win32 Mutex
        let single_instance_handle = Self::acquire_single_instance_lock()?;

        // Load configuration
        let config = NexoraConfig::load_or_default(&paths.app_data_dir);

        Ok(Self {
            paths,
            config,
            _single_instance_handle: Some(single_instance_handle),
        })
    }

    fn acquire_single_instance_lock() -> Result<windows::Win32::Foundation::HANDLE> {
        #[cfg(windows)]
        {
            use windows::core::w;
            use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
            use windows::Win32::System::Threading::CreateMutexW;

            unsafe {
                let mutex_name = w!("Local\\NexoraBrowserSingleInstanceMutex");
                let handle = CreateMutexW(None, true, mutex_name)
                    .map_err(|e| NexoraError::Win32(format!("Failed to create single-instance mutex: {}", e)))?;

                if GetLastError() == ERROR_ALREADY_EXISTS {
                    log::warn!("Another instance of Nexora is already running.");
                } else {
                    log::info!("Single-instance mutex successfully acquired.");
                }

                Ok(handle)
            }
        }
        #[cfg(not(windows))]
        {
            Ok(windows::Win32::Foundation::HANDLE(std::ptr::null_mut()))
        }
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        log::info!("Nexora Application is shutting down cleanly.");
        if let Some(handle) = self._single_instance_handle.take() {
            #[cfg(windows)]
            unsafe {
                use windows::Win32::Foundation::CloseHandle;
                let _ = CloseHandle(handle);
            }
        }
    }
}
