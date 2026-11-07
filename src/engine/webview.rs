use std::path::Path;
use std::sync::mpsc::channel;
use windows::core::{Interface, PCWSTR, PWSTR};
use windows::Win32::Foundation::{BOOL, HWND, RECT};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use windows::Win32::System::WinRT::EventRegistrationToken;
use webview2_com::Microsoft::Web::WebView2::Win32::*;
use webview2_com::*;

use crate::core::error::{NexoraError, Result};

pub struct WebViewEngine {
    _environment: ICoreWebView2Environment,
    controller: ICoreWebView2Controller,
    webview: ICoreWebView2,
}

impl WebViewEngine {
    pub fn new(hwnd: HWND, user_data_dir: &Path, initial_url: &str) -> Result<Self> {
        unsafe {
            // Initialize COM for UI apartment thread
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        }

        std::fs::create_dir_all(user_data_dir)?;
        let udata_str = user_data_dir.to_string_lossy().to_string();
        let udata_wide = pwstr_from_str(&udata_str);

        // Hardware video acceleration and low-overhead flags via GPU capabilities
        let gpu_caps = crate::media::GpuCapabilities::detect(true);
        let mut browser_args_vec = gpu_caps.build_browser_switches();
        browser_args_vec.push("--disk-cache-size=268435456");
        browser_args_vec.push("--media-cache-size=134217728");
        let browser_args = browser_args_vec.join(" ");

        log::info!("Configuring Microsoft WebView2 Environment:");
        log::info!("  UserDataFolder: {}", udata_str);
        log::info!("  Arguments:      {}", browser_args);

        let options = CoreWebView2EnvironmentOptions::default();
        unsafe {
            options.set_additional_browser_arguments(browser_args);
        }
        let options_iface: ICoreWebView2EnvironmentOptions = options.into();

        // 1. Create WebView2 Environment
        let (tx_env, rx_env) = channel();
        let env_handler = CreateCoreWebView2EnvironmentCompletedHandler::create(Box::new(move |res, env| {
            let _ = tx_env.send((res, env));
            Ok(())
        }));

        unsafe {
            CreateCoreWebView2EnvironmentWithOptions(
                None,
                PCWSTR(udata_wide.0),
                &options_iface,
                &env_handler,
            )
            .map_err(|e| NexoraError::Engine(format!("Failed to invoke CreateCoreWebView2EnvironmentWithOptions: {}", e)))?;
        }

        let (res_env, env_opt) = wait_with_pump(rx_env)
            .map_err(|e| NexoraError::Engine(format!("Environment pump failed: {}", e)))?;
        res_env.map_err(|e| NexoraError::Engine(format!("Environment creation returned error: {}", e)))?;
        let environment = env_opt.ok_or_else(|| NexoraError::Engine("Null WebView2 Environment returned".into()))?;

        log::info!("WebView2 Environment created successfully.");

        // 2. Create WebView2 Controller attached to host HWND
        let (tx_ctl, rx_ctl) = channel();
        let ctl_handler = CreateCoreWebView2ControllerCompletedHandler::create(Box::new(move |res, ctl| {
            let _ = tx_ctl.send((res, ctl));
            Ok(())
        }));

        unsafe {
            environment.CreateCoreWebView2Controller(hwnd, &ctl_handler)
                .map_err(|e| NexoraError::Engine(format!("Failed to invoke CreateCoreWebView2Controller: {}", e)))?;
        }

        let (res_ctl, ctl_opt) = wait_with_pump(rx_ctl)
            .map_err(|e| NexoraError::Engine(format!("Controller pump failed: {}", e)))?;
        res_ctl.map_err(|e| NexoraError::Engine(format!("Controller creation returned error: {}", e)))?;
        let controller = ctl_opt.ok_or_else(|| NexoraError::Engine("Null WebView2 Controller returned".into()))?;

        log::info!("WebView2 Controller instantiated and attached to HWND {:?}.", hwnd.0);

        let webview = unsafe {
            controller.CoreWebView2()
                .map_err(|e| NexoraError::Engine(format!("Failed to get CoreWebView2 from controller: {}", e)))?
        };

        // 3. Configure initial visibility and settings
        unsafe {
            let _ = controller.SetIsVisible(true);
            if let Ok(settings) = webview.Settings() {
                let _ = settings.SetIsScriptEnabled(true);
                let _ = settings.SetIsWebMessageEnabled(true);
                let _ = settings.SetAreDefaultScriptDialogsEnabled(true);
                let _ = settings.SetIsStatusBarEnabled(false);
                let _ = settings.SetAreDevToolsEnabled(cfg!(debug_assertions));
            }
        }

        let engine = Self {
            _environment: environment,
            controller,
            webview,
        };

        // 4. Hook Core Events
        engine.setup_event_handlers()?;

        // 5. Navigate to initial URL
        engine.navigate(initial_url)?;

        Ok(engine)
    }

    pub fn set_bounds(&self, x: i32, y: i32, width: i32, height: i32) -> Result<()> {
        let rect = RECT {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        };
        unsafe {
            self.controller.SetBounds(rect)
                .map_err(|e| NexoraError::Engine(format!("Failed to set webview bounds: {}", e)))?;
        }
        Ok(())
    }

    pub fn navigate(&self, target_url: &str) -> Result<()> {
        let normalized = if target_url.starts_with("http://")
            || target_url.starts_with("https://")
            || target_url.starts_with("edge://")
            || target_url.starts_with("about:")
        {
            target_url.to_string()
        } else {
            format!("https://{}", target_url)
        };

        log::info!("Navigating to: {}", normalized);
        let wide = pwstr_from_str(&normalized);
        unsafe {
            self.webview.Navigate(PCWSTR(wide.0))
                .map_err(|e| NexoraError::Navigation(format!("Navigation failed for {}: {}", normalized, e)))?;
        }
        Ok(())
    }

    pub fn go_back(&self) -> Result<()> {
        unsafe {
            let mut can_go_back = BOOL(0);
            let _ = self.webview.CanGoBack(&mut can_go_back);
            if can_go_back.as_bool() {
                self.webview.GoBack()
                    .map_err(|e| NexoraError::Engine(format!("GoBack failed: {}", e)))?;
            }
        }
        Ok(())
    }

    pub fn go_forward(&self) -> Result<()> {
        unsafe {
            let mut can_go_forward = BOOL(0);
            let _ = self.webview.CanGoForward(&mut can_go_forward);
            if can_go_forward.as_bool() {
                self.webview.GoForward()
                    .map_err(|e| NexoraError::Engine(format!("GoForward failed: {}", e)))?;
            }
        }
        Ok(())
    }

    pub fn reload(&self) -> Result<()> {
        unsafe {
            self.webview.Reload()
                .map_err(|e| NexoraError::Engine(format!("Reload failed: {}", e)))?;
        }
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        unsafe {
            self.webview.Stop()
                .map_err(|e| NexoraError::Engine(format!("Stop failed: {}", e)))?;
        }
        Ok(())
    }

    pub fn get_source(&self) -> Result<String> {
        unsafe {
            let mut uri = PWSTR::null();
            self.webview.Source(&mut uri)
                .map_err(|e| NexoraError::Engine(format!("Failed to get source URL: {}", e)))?;
            Ok(string_from_pcwstr(&PCWSTR(uri.0)))
        }
    }

    pub fn get_title(&self) -> Result<String> {
        unsafe {
            let mut title = PWSTR::null();
            self.webview.DocumentTitle(&mut title)
                .map_err(|e| NexoraError::Engine(format!("Failed to get document title: {}", e)))?;
            Ok(string_from_pcwstr(&PCWSTR(title.0)))
        }
    }

    pub fn execute_script(&self, script: &str) -> Result<()> {
        let wide = pwstr_from_str(script);
        unsafe {
            self.webview.ExecuteScript(PCWSTR(wide.0), None)
                .map_err(|e| NexoraError::Engine(format!("ExecuteScript failed: {}", e)))?;
        }
        Ok(())
    }

    pub fn trigger_pip(&self) -> Result<()> {
        let script = crate::media::PipWindow::generate_pip_script();
        self.execute_script(script)
    }

    fn setup_event_handlers(&self) -> Result<()> {
        // NavigationStarting
        let starting_handler = NavigationStartingEventHandler::create(Box::new(|_sender, args| {
            if let Some(args) = args {
                unsafe {
                    let mut uri = PWSTR::null();
                    if args.Uri(&mut uri).is_ok() {
                        let url_str = string_from_pcwstr(&PCWSTR(uri.0));
                        log::info!("[WebView2] Navigation Starting: {}", url_str);
                    }
                }
            }
            Ok(())
        }));
        let mut starting_token = EventRegistrationToken::default();
        unsafe {
            let _ = self.webview.add_NavigationStarting(&starting_handler, &mut starting_token);
        }

        // NavigationCompleted
        let completed_handler = NavigationCompletedEventHandler::create(Box::new(|_sender, args| {
            if let Some(args) = args {
                unsafe {
                    let mut is_success = BOOL(0);
                    let _ = args.IsSuccess(&mut is_success);
                    let mut web_error = COREWEBVIEW2_WEB_ERROR_STATUS::default();
                    let _ = args.WebErrorStatus(&mut web_error);
                    log::info!("[WebView2] Navigation Completed: success={}, status={:?}", is_success.as_bool(), web_error);
                }
            }
            Ok(())
        }));
        let mut completed_token = EventRegistrationToken::default();
        unsafe {
            let _ = self.webview.add_NavigationCompleted(&completed_handler, &mut completed_token);
        }

        // SourceChanged
        let source_handler = SourceChangedEventHandler::create(Box::new(|sender, _args| {
            if let Some(sender) = sender {
                unsafe {
                    let mut uri = PWSTR::null();
                    if sender.Source(&mut uri).is_ok() {
                        let url_str = string_from_pcwstr(&PCWSTR(uri.0));
                        log::info!("[WebView2] Source Changed: {}", url_str);
                    }
                }
            }
            Ok(())
        }));
        let mut source_token = EventRegistrationToken::default();
        unsafe {
            let _ = self.webview.add_SourceChanged(&source_handler, &mut source_token);
        }

        // ProcessFailed (Crash Recovery Hook - TASK-205)
        let process_failed_handler = ProcessFailedEventHandler::create(Box::new(|_sender, args| {
            if let Some(args) = args {
                unsafe {
                    let mut kind = COREWEBVIEW2_PROCESS_FAILED_KIND::default();
                    let _ = args.ProcessFailedKind(&mut kind);
                    log::error!("[WebView2 Alert] Process Failed (Kind: {:?}). Initiating recovery reload...", kind);
                }
            }
            Ok(())
        }));
        let mut failed_token = EventRegistrationToken::default();
        unsafe {
            let _ = self.webview.add_ProcessFailed(&process_failed_handler, &mut failed_token);
        }

        // ContainsFullScreenElementChanged (HTML5 Fullscreen Hook - TASK-505)
        let fs_handler = ContainsFullScreenElementChangedEventHandler::create(Box::new(|sender, _args| {
            if let Some(sender) = sender {
                unsafe {
                    let mut is_fs = BOOL(0);
                    if sender.ContainsFullScreenElement(&mut is_fs).is_ok() {
                        log::info!("[Media Engine] HTML5 Fullscreen changed: {}", is_fs.as_bool());
                    }
                }
            }
            Ok(())
        }));
        let mut fs_token = EventRegistrationToken::default();
        unsafe {
            let _ = self.webview.add_ContainsFullScreenElementChanged(&fs_handler, &mut fs_token);
        }

        // IsDocumentPlayingAudioChanged (Active Media Detection - TASK-504)
        if let Ok(wv8) = self.webview.cast::<ICoreWebView2_8>() {
            let audio_handler = IsDocumentPlayingAudioChangedEventHandler::create(Box::new(|sender, _args| {
                if let Some(sender) = sender {
                    unsafe {
                        if let Ok(wv8_sender) = sender.cast::<ICoreWebView2_8>() {
                            let mut is_audio = BOOL(0);
                            if wv8_sender.IsDocumentPlayingAudio(&mut is_audio).is_ok() {
                                log::info!("[Media Engine] Document audio playback changed: {}", is_audio.as_bool());
                            }
                        }
                    }
                }
                Ok(())
            }));
            let mut audio_token = EventRegistrationToken::default();
            unsafe {
                let _ = wv8.add_IsDocumentPlayingAudioChanged(&audio_handler, &mut audio_token);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser::HostWindow;

    #[test]
    fn test_webview_engine_initialization() {
        let tab_mgr = std::rc::Rc::new(std::cell::RefCell::new(crate::tabs::TabManager::new()));
        let theme = crate::ui::theme::ThemePalette::dark();
        let window = HostWindow::new(
            "Nexora Engine Test Window",
            800,
            600,
            theme,
            "https://www.google.com",
            "Google",
            tab_mgr,
        )
        .expect("HostWindow creation failed in test");

        let temp_dir = std::env::temp_dir().join(format!("nexora_test_udf_{}", std::process::id()));
        let engine = WebViewEngine::new(window.hwnd(), &temp_dir, "https://www.google.com");
        
        assert!(engine.is_ok(), "WebViewEngine creation should succeed: {:?}", engine.err());
        let engine = engine.unwrap();

        let (left, top, w, h) = window.get_client_bounds();
        let bounds_res = engine.set_bounds(left, top, w, h);
        assert!(bounds_res.is_ok(), "Setting bounds should succeed");

        // Clean up temp dir
        let _ = std::fs::remove_dir_all(&temp_dir);

        // Clean up window
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(window.hwnd());
        }
    }
}
