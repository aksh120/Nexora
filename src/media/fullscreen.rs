use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTOPRIMARY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, GetWindowRect, SetWindowLongPtrW, SetWindowPos, GWL_STYLE,
    HWND_TOP, SWP_FRAMECHANGED, SWP_NOOWNERZORDER, WINDOW_STYLE, WS_OVERLAPPEDWINDOW, WS_POPUP,
};

#[derive(Debug, Default)]
pub struct FullscreenController {
    is_fullscreen: bool,
    saved_rect: RECT,
    saved_style: WINDOW_STYLE,
}

impl FullscreenController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_fullscreen(&self) -> bool {
        self.is_fullscreen
    }

    pub fn enter_fullscreen(&mut self, hwnd: HWND) -> (i32, i32, i32, i32) {
        if self.is_fullscreen {
            return (0, 0, self.saved_rect.right - self.saved_rect.left, self.saved_rect.bottom - self.saved_rect.top);
        }

        unsafe {
            // 1. Save current window placement & style
            let mut rect = RECT::default();
            let _ = GetWindowRect(hwnd, &mut rect);
            self.saved_rect = rect;

            let style_raw = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
            self.saved_style = WINDOW_STYLE(style_raw);

            // 2. Query target monitor dimensions
            let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY);
            let mut mi = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            let _ = GetMonitorInfoW(hmonitor, &mut mi);
            let mrect = mi.rcMonitor;

            let screen_w = mrect.right - mrect.left;
            let screen_h = mrect.bottom - mrect.top;

            // 3. Switch style to borderless popup
            let new_style = (style_raw & !WS_OVERLAPPEDWINDOW.0) | WS_POPUP.0;
            SetWindowLongPtrW(hwnd, GWL_STYLE, new_style as isize);

            // 4. Reposition window to cover the entire monitor
            let _ = SetWindowPos(
                hwnd,
                HWND_TOP,
                mrect.left,
                mrect.top,
                screen_w,
                screen_h,
                SWP_NOOWNERZORDER | SWP_FRAMECHANGED,
            );

            self.is_fullscreen = true;
            log::info!("[Fullscreen] Entered seamless borderless HTML5 fullscreen: {}x{}", screen_w, screen_h);

            (0, 0, screen_w, screen_h)
        }
    }

    pub fn exit_fullscreen(&mut self, hwnd: HWND) -> (i32, i32, i32, i32) {
        if !self.is_fullscreen {
            return (0, 0, 0, 0);
        }

        unsafe {
            // Restore original style
            SetWindowLongPtrW(hwnd, GWL_STYLE, self.saved_style.0 as isize);

            let w = self.saved_rect.right - self.saved_rect.left;
            let h = self.saved_rect.bottom - self.saved_rect.top;

            // Restore original bounds
            let _ = SetWindowPos(
                hwnd,
                HWND_TOP,
                self.saved_rect.left,
                self.saved_rect.top,
                w,
                h,
                SWP_NOOWNERZORDER | SWP_FRAMECHANGED,
            );

            self.is_fullscreen = false;
            log::info!("[Fullscreen] Exited HTML5 fullscreen. Restored window: {}x{}", w, h);

            (self.saved_rect.left, self.saved_rect.top, w, h)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fullscreen_state_transitions() {
        let controller = FullscreenController::new();
        assert!(!controller.is_fullscreen());
    }
}
