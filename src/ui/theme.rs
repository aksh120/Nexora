use windows::core::w;
use windows::Win32::Foundation::{BOOL, COLORREF, HWND};
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE};
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ, REG_DWORD,
    REG_VALUE_TYPE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
    System,
}

#[inline]
pub const fn rgb(r: u8, g: u8, b: u8) -> COLORREF {
    COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16))
}

#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub is_dark: bool,
    pub window_bg: COLORREF,
    pub tab_bar_bg: COLORREF,
    pub tab_active_bg: COLORREF,
    pub tab_inactive_bg: COLORREF,
    pub tab_hover_bg: COLORREF,
    pub tab_text_active: COLORREF,
    pub tab_text_inactive: COLORREF,
    pub toolbar_bg: COLORREF,
    pub omnibox_bg: COLORREF,
    pub omnibox_border: COLORREF,
    pub omnibox_text: COLORREF,
    pub button_hover_bg: COLORREF,
    pub accent_color: COLORREF,
    pub divider_color: COLORREF,
}

impl ThemePalette {
    pub fn dark() -> Self {
        Self {
            is_dark: true,
            window_bg: rgb(0, 0, 0),             // Pure AMOLED Black #000000
            tab_bar_bg: rgb(0, 0, 0),            // Pure AMOLED Black #000000
            tab_active_bg: rgb(12, 7, 8),        // #0C0708 (Dark carbon card surface)
            tab_inactive_bg: rgb(0, 0, 0),       // #000000
            tab_hover_bg: rgb(22, 13, 15),       // #160D0F (Carbon hover)
            tab_text_active: rgb(247, 240, 241), // #F7F0F1 (Warm crisp off-white)
            tab_text_inactive: rgb(153, 127, 131),// #997F83 (Muted warm slate)
            toolbar_bg: rgb(0, 0, 0),            // #000000
            omnibox_bg: rgb(12, 7, 8),           // #0C0708
            omnibox_border: rgb(41, 21, 24),     // #291518 (1px crimson outline)
            omnibox_text: rgb(247, 240, 241),    // #F7F0F1
            button_hover_bg: rgb(22, 13, 15),    // #160D0F
            accent_color: rgb(225, 29, 72),      // #E11D48 (Eclipse Crimson)
            divider_color: rgb(41, 21, 24),      // #291518
        }
    }

    pub fn light() -> Self {
        Self {
            is_dark: false,
            window_bg: rgb(243, 243, 243),        // #F3F3F3
            tab_bar_bg: rgb(232, 232, 232),       // #E8E8E8
            tab_active_bg: rgb(255, 255, 255),    // #FFFFFF
            tab_inactive_bg: rgb(232, 232, 232),  // #E8E8E8
            tab_hover_bg: rgb(240, 240, 240),     // #F0F0F0
            tab_text_active: rgb(20, 20, 20),     // #141414
            tab_text_inactive: rgb(110, 110, 110),// #6E6E6E
            toolbar_bg: rgb(248, 248, 248),       // #F8F8F8
            omnibox_bg: rgb(255, 255, 255),       // #FFFFFF
            omnibox_border: rgb(210, 210, 210),   // #D2D2D2
            omnibox_text: rgb(20, 20, 20),        // #141414
            button_hover_bg: rgb(225, 225, 225),  // #E1E1E1
            accent_color: rgb(0, 120, 215),       // #0078D7
            divider_color: rgb(220, 220, 220),    // #DCDCDC
        }
    }

    pub fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::dark(),
            ThemeMode::Light => Self::light(),
            ThemeMode::System => {
                if Self::detect_system_is_dark() {
                    Self::dark()
                } else {
                    Self::light()
                }
            }
        }
    }

    pub fn detect_system_is_dark() -> bool {
        unsafe {
            let mut hkey = HKEY::default();
            let subkey = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
            if RegOpenKeyExW(HKEY_CURRENT_USER, subkey, 0, KEY_READ, &mut hkey).is_ok() {
                let mut data: u32 = 0;
                let mut data_len = std::mem::size_of::<u32>() as u32;
                let mut val_type = REG_VALUE_TYPE::default();
                let status = RegQueryValueExW(
                    hkey,
                    w!("AppsUseLightTheme"),
                    None,
                    Some(&mut val_type),
                    Some(&mut data as *mut u32 as *mut u8),
                    Some(&mut data_len),
                );
                let _ = RegCloseKey(hkey);

                if status.is_ok() && val_type == REG_DWORD {
                    // AppsUseLightTheme == 0 means Dark Mode
                    return data == 0;
                }
            }
            // Default to dark mode for Nexora
            true
        }
    }

    pub fn apply_to_window(hwnd: HWND, is_dark: bool) {
        unsafe {
            let dark_flag = BOOL(if is_dark { 1 } else { 0 });
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                &dark_flag as *const _ as *const _,
                std::mem::size_of::<BOOL>() as u32,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_palettes() {
        let dark = ThemePalette::dark();
        assert!(dark.is_dark);
        assert_eq!(dark.window_bg.0, 0); // Pure AMOLED #000000

        let light = ThemePalette::light();
        assert!(!light.is_dark);
        assert_ne!(light.window_bg.0, dark.window_bg.0);
    }

    #[test]
    fn test_system_theme_detection() {
        // Should execute safely without panic on Windows
        let is_dark = ThemePalette::detect_system_is_dark();
        log::info!("System is dark mode: {}", is_dark);
    }
}
