use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateSolidBrush, HBRUSH};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, RegisterClassExW,
    ShowWindow, CW_USEDEFAULT, HICON, SW_HIDE, SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE,
    WM_CLOSE, WM_DESTROY, WM_ERASEBKGND, WNDCLASSEXW, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_POPUP, WS_THICKFRAME,
};

use crate::core::error::{NexoraError, Result};

pub const DEFAULT_PIP_WIDTH: i32 = 480;
pub const DEFAULT_PIP_HEIGHT: i32 = 270; // 16:9 aspect ratio

pub struct PipWindow {
    hwnd: HWND,
    is_active: bool,
}

impl PipWindow {
    pub fn new(parent: HWND) -> Result<Self> {
        unsafe {
            let hinstance: HINSTANCE = GetModuleHandleW(None)
                .map_err(|e| NexoraError::Win32(format!("Failed to get module handle: {}", e)))?
                .into();

            let class_name = w!("NexoraPipWindowClass");
            let bg_brush: HBRUSH = CreateSolidBrush(COLORREF(0x00101010));

            let wnd_class = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: windows::Win32::UI::WindowsAndMessaging::WNDCLASS_STYLES(0),
                lpfnWndProc: Some(Self::pip_wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance,
                hIcon: HICON::default(),
                hCursor: windows::Win32::UI::WindowsAndMessaging::LoadCursorW(
                    None,
                    windows::Win32::UI::WindowsAndMessaging::IDC_ARROW,
                )
                .unwrap_or_default(),
                hbrBackground: bg_brush,
                lpszMenuName: PCWSTR::null(),
                lpszClassName: class_name,
                hIconSm: HICON::default(),
            };

            let _ = RegisterClassExW(&wnd_class);

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE((WS_EX_TOPMOST.0 | WS_EX_TOOLWINDOW.0) as u32),
                class_name,
                w!("Nexora Picture-in-Picture"),
                WINDOW_STYLE((WS_POPUP.0 | WS_THICKFRAME.0) as u32),
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                DEFAULT_PIP_WIDTH,
                DEFAULT_PIP_HEIGHT,
                parent,
                None,
                hinstance,
                None,
            )
            .map_err(|e| NexoraError::Win32(format!("Failed to create PiP window: {}", e)))?;

            log::info!("Created Picture-in-Picture floating window (HWND: {:?})", hwnd.0);

            Ok(Self {
                hwnd,
                is_active: false,
            })
        }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn show(&mut self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_SHOW);
            self.is_active = true;
        }
    }

    pub fn hide(&mut self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
            self.is_active = false;
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn generate_pip_script() -> &'static str {
        r#"
        (function() {
            const video = document.querySelector('video');
            if (!video) return { success: false, reason: "no_video" };
            if (document.pictureInPictureElement) {
                document.exitPictureInPicture();
                return { success: true, action: "exited" };
            } else if (document.pictureInPictureEnabled) {
                video.requestPictureInPicture()
                    .then(() => {})
                    .catch((err) => console.error("PiP request failed:", err));
                return { success: true, action: "requested" };
            }
            return { success: false, reason: "pip_unsupported" };
        })();
        "#
    }

    unsafe extern "system" fn pip_wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_ERASEBKGND => LRESULT(1),
            WM_CLOSE => {
                let _ = ShowWindow(hwnd, SW_HIDE);
                LRESULT(0)
            }
            WM_DESTROY => LRESULT(0),
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

impl Drop for PipWindow {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pip_script_syntax() {
        let script = PipWindow::generate_pip_script();
        assert!(script.contains("requestPictureInPicture"));
        assert!(script.contains("exitPictureInPicture"));
    }

    #[test]
    fn test_pip_window_lifecycle() {
        let pip = PipWindow::new(HWND::default());
        assert!(pip.is_ok(), "PiP window creation should succeed");
        let mut pip = pip.unwrap();
        assert!(!pip.hwnd().0.is_null());
        assert!(!pip.is_active());

        pip.show();
        assert!(pip.is_active());

        pip.hide();
        assert!(!pip.is_active());
    }
}
