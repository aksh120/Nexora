use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateSolidBrush, HBRUSH};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
    PostQuitMessage, RegisterClassExW, SetWindowLongPtrW, GetWindowLongPtrW, ShowWindow,
    TranslateMessage, CW_USEDEFAULT, GWLP_USERDATA, MSG, SW_SHOW, WINDOW_EX_STYLE,
    WM_CLOSE, WM_DESTROY, WM_DPICHANGED, WM_ERASEBKGND, WM_SETFOCUS, WM_SIZE,
    WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
};

use crate::core::error::{NexoraError, Result};

pub type ResizeCallback = Box<dyn Fn(i32, i32, i32, i32) + 'static>;

struct WindowContext {
    on_resize: Option<ResizeCallback>,
    is_closing: Arc<AtomicBool>,
}

pub struct HostWindow {
    hwnd: HWND,
    is_closing: Arc<AtomicBool>,
}

impl HostWindow {
    pub fn new(title: &str, width: i32, height: i32) -> Result<Self> {
        let is_closing = Arc::new(AtomicBool::new(false));

        unsafe {
            let hinstance: HINSTANCE = GetModuleHandleW(None)
                .map_err(|e| NexoraError::Win32(format!("Failed to get module handle: {}", e)))?
                .into();

            let class_name = w!("NexoraBrowserHostWindowClass");

            // Dark-neutral background brush (#1E1E1E)
            let bg_brush: HBRUSH = CreateSolidBrush(COLORREF(0x001E1E1E));

            let wnd_class = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: windows::Win32::UI::WindowsAndMessaging::WNDCLASS_STYLES(0),
                lpfnWndProc: Some(Self::wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance,
                hIcon: windows::Win32::UI::WindowsAndMessaging::HICON::default(),
                hCursor: windows::Win32::UI::WindowsAndMessaging::LoadCursorW(
                    None,
                    windows::Win32::UI::WindowsAndMessaging::IDC_ARROW,
                )
                .unwrap_or_default(),
                hbrBackground: bg_brush,
                lpszMenuName: PCWSTR::null(),
                lpszClassName: class_name,
                hIconSm: windows::Win32::UI::WindowsAndMessaging::HICON::default(),
            };

            let _ = RegisterClassExW(&wnd_class);

            let wide_title: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class_name,
                PCWSTR(wide_title.as_ptr()),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                width,
                height,
                None,
                None,
                hinstance,
                None,
            )
            .map_err(|e| NexoraError::Win32(format!("Failed to create host window: {}", e)))?;

            let context = Box::new(WindowContext {
                on_resize: None,
                is_closing: is_closing.clone(),
            });

            SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(context) as isize);

            log::info!("Native Win32 Host Window created (HWND: {:?}, size: {}x{})", hwnd.0, width, height);

            Ok(Self { hwnd, is_closing })
        }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn show(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_SHOW);
        }
    }

    pub fn set_resize_callback<F>(&self, callback: F)
    where
        F: Fn(i32, i32, i32, i32) + 'static,
    {
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut WindowContext;
            if !ptr.is_null() {
                (*ptr).on_resize = Some(Box::new(callback));
            }
        }
    }

    pub fn get_client_bounds(&self) -> (i32, i32, i32, i32) {
        unsafe {
            let mut rect = RECT::default();
            let _ = GetClientRect(self.hwnd, &mut rect);
            (rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top)
        }
    }

    pub fn is_closing(&self) -> bool {
        self.is_closing.load(Ordering::SeqCst)
    }

    pub fn run_message_loop(&self) {
        unsafe {
            let mut msg = MSG::default();
            log::info!("Starting native Win32 message pump...");
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            log::info!("Native Win32 message pump exited.");
        }
    }

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowContext;

        match msg {
            WM_SIZE => {
                if !ptr.is_null() {
                    let mut rect = RECT::default();
                    if GetClientRect(hwnd, &mut rect).is_ok() {
                        let width = rect.right - rect.left;
                        let height = rect.bottom - rect.top;
                        if let Some(ref cb) = (*ptr).on_resize {
                            cb(rect.left, rect.top, width, height);
                        }
                    }
                }
                LRESULT(0)
            }
            WM_DPICHANGED => {
                let suggested_rect = *(lparam.0 as *const RECT);
                let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                    hwnd,
                    HWND::default(),
                    suggested_rect.left,
                    suggested_rect.top,
                    suggested_rect.right - suggested_rect.left,
                    suggested_rect.bottom - suggested_rect.top,
                    windows::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                        | windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE,
                );
                LRESULT(0)
            }
            WM_SETFOCUS => {
                // Return 0
                LRESULT(0)
            }
            WM_ERASEBKGND => {
                // Prevent flicker
                LRESULT(1)
            }
            WM_CLOSE => {
                if !ptr.is_null() {
                    (*ptr).is_closing.store(true, Ordering::SeqCst);
                }
                let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd);
                LRESULT(0)
            }
            WM_DESTROY => {
                if !ptr.is_null() {
                    let _ = Box::from_raw(ptr);
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                }
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_window_lifecycle() {
        let window = HostWindow::new("Nexora Test Window", 800, 600);
        assert!(window.is_ok(), "HostWindow creation should succeed");
        let window = window.unwrap();
        assert!(!window.hwnd().0.is_null(), "HWND should be valid and non-null");

        let (left, top, width, height) = window.get_client_bounds();
        assert!(width > 0, "Client width should be positive");
        assert!(height > 0, "Client height should be positive");
        log::info!("Test window bounds: ({}, {}, {}, {})", left, top, width, height);

        // Cleanly close
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(window.hwnd());
        }
    }
}
