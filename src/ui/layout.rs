pub const TAB_BAR_HEIGHT: i32 = 36;
pub const TOOLBAR_HEIGHT: i32 = 42;
pub const TOP_CHROME_HEIGHT: i32 = TAB_BAR_HEIGHT + TOOLBAR_HEIGHT; // 78px
pub const TAB_MIN_WIDTH: i32 = 110;
pub const TAB_MAX_WIDTH: i32 = 220;
pub const PINNED_TAB_WIDTH: i32 = 38;
pub const BUTTON_WIDTH: i32 = 32;
pub const BUTTON_HEIGHT: i32 = 30;
pub const BUTTON_GAP: i32 = 4;
pub const OMNIBOX_HEIGHT: i32 = 30;

use crate::tabs::Tab;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrowserLayout {
    pub window_width: i32,
    pub window_height: i32,
}

impl BrowserLayout {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            window_width: width.max(400),
            window_height: height.max(200),
        }
    }

    pub fn tab_bar_rect(&self) -> (i32, i32, i32, i32) {
        (0, 0, self.window_width, TAB_BAR_HEIGHT)
    }

    pub fn toolbar_rect(&self) -> (i32, i32, i32, i32) {
        (0, TAB_BAR_HEIGHT, self.window_width, TOOLBAR_HEIGHT)
    }

    pub fn webview_rect(&self) -> (i32, i32, i32, i32) {
        let h = (self.window_height - TOP_CHROME_HEIGHT).max(0);
        (0, TOP_CHROME_HEIGHT, self.window_width, h)
    }

    pub fn brand_logo_rect(&self) -> (i32, i32, i32, i32) {
        (8, (TAB_BAR_HEIGHT - 22) / 2, 22, 22)
    }

    pub fn calculate_tab_rects(&self, tab_count: usize) -> Vec<(i32, i32, i32, i32)> {
        if tab_count == 0 {
            return Vec::new();
        }

        // Available space for tabs leaving room for brand logo (38px), "+" button and caption buttons (138px)
        let available_width = (self.window_width - 150 - 50 - 38).max(100);
        let ideal_width = available_width / (tab_count as i32);
        let tab_width = ideal_width.clamp(TAB_MIN_WIDTH, TAB_MAX_WIDTH);

        let mut rects = Vec::with_capacity(tab_count);
        let y = 3;
        let h = TAB_BAR_HEIGHT - 3;

        for i in 0..tab_count {
            let x = 38 + (i as i32) * (tab_width + 2);
            rects.push((x, y, tab_width, h));
        }

        rects
    }

    pub fn calculate_tab_rects_for_tabs(&self, tabs: &[Tab]) -> Vec<(i32, i32, i32, i32)> {
        if tabs.is_empty() {
            return Vec::new();
        }
        let pinned_count = tabs.iter().filter(|t| t.is_pinned).count();
        let normal_count = tabs.len() - pinned_count;

        let pinned_total_width = (pinned_count as i32) * (PINNED_TAB_WIDTH + 2);
        let available_width = (self.window_width - 150 - 50 - 38 - pinned_total_width).max(100);

        let normal_tab_width = if normal_count > 0 {
            (available_width / (normal_count as i32)).clamp(TAB_MIN_WIDTH, TAB_MAX_WIDTH)
        } else {
            TAB_MAX_WIDTH
        };

        let mut rects = Vec::with_capacity(tabs.len());
        let y = 3;
        let h = TAB_BAR_HEIGHT - 3;
        let mut current_x = 38;

        for tab in tabs {
            let width = if tab.is_pinned {
                PINNED_TAB_WIDTH
            } else {
                normal_tab_width
            };
            rects.push((current_x, y, width, h));
            current_x += width + 2;
        }

        rects
    }

    pub fn new_tab_button_rect_for_tabs(&self, tabs: &[Tab]) -> (i32, i32, i32, i32) {
        let rects = self.calculate_tab_rects_for_tabs(tabs);
        let x = if let Some(last) = rects.last() {
            last.0 + last.2 + 4
        } else {
            38
        };
        (x, 5, 26, TAB_BAR_HEIGHT - 8)
    }

    pub fn caption_minimize_rect(&self) -> (i32, i32, i32, i32) {
        (self.window_width - 138, 0, 46, TAB_BAR_HEIGHT)
    }

    pub fn caption_maximize_rect(&self) -> (i32, i32, i32, i32) {
        (self.window_width - 92, 0, 46, TAB_BAR_HEIGHT)
    }

    pub fn caption_close_rect(&self) -> (i32, i32, i32, i32) {
        (self.window_width - 46, 0, 46, TAB_BAR_HEIGHT)
    }

    pub fn new_tab_button_rect(&self, tab_count: usize) -> (i32, i32, i32, i32) {
        let tabs = self.calculate_tab_rects(tab_count);
        let x = if let Some(last) = tabs.last() {
            last.0 + last.2 + 4
        } else {
            38
        };
        (x, 5, 26, TAB_BAR_HEIGHT - 8)
    }

    // Navigation buttons in toolbar
    pub fn back_button_rect(&self) -> (i32, i32, i32, i32) {
        let y = TAB_BAR_HEIGHT + (TOOLBAR_HEIGHT - BUTTON_HEIGHT) / 2;
        (8, y, BUTTON_WIDTH, BUTTON_HEIGHT)
    }

    pub fn forward_button_rect(&self) -> (i32, i32, i32, i32) {
        let back = self.back_button_rect();
        (back.0 + back.2 + BUTTON_GAP, back.1, BUTTON_WIDTH, BUTTON_HEIGHT)
    }

    pub fn reload_button_rect(&self) -> (i32, i32, i32, i32) {
        let fwd = self.forward_button_rect();
        (fwd.0 + fwd.2 + BUTTON_GAP, fwd.1, BUTTON_WIDTH, BUTTON_HEIGHT)
    }

    pub fn home_button_rect(&self) -> (i32, i32, i32, i32) {
        let reload = self.reload_button_rect();
        (reload.0 + reload.2 + BUTTON_GAP, reload.1, BUTTON_WIDTH, BUTTON_HEIGHT)
    }

    pub fn bookmark_button_rect(&self) -> (i32, i32, i32, i32) {
        let y = TAB_BAR_HEIGHT + (TOOLBAR_HEIGHT - BUTTON_HEIGHT) / 2;
        let x = self.window_width - BUTTON_WIDTH * 2 - 12;
        (x, y, BUTTON_WIDTH, BUTTON_HEIGHT)
    }

    pub fn menu_button_rect(&self) -> (i32, i32, i32, i32) {
        let y = TAB_BAR_HEIGHT + (TOOLBAR_HEIGHT - BUTTON_HEIGHT) / 2;
        let x = self.window_width - BUTTON_WIDTH - 8;
        (x, y, BUTTON_WIDTH, BUTTON_HEIGHT)
    }

    pub fn omnibox_rect(&self) -> (i32, i32, i32, i32) {
        let home = self.home_button_rect();
        let bm = self.bookmark_button_rect();
        let x = home.0 + home.2 + 10;
        let right = bm.0 - 10;
        let w = (right - x).max(120);
        let y = TAB_BAR_HEIGHT + (TOOLBAR_HEIGHT - OMNIBOX_HEIGHT) / 2;
        (x, y, w, OMNIBOX_HEIGHT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_layout_rects() {
        let layout = BrowserLayout::new(1280, 800);
        let (x, y, w, h) = layout.webview_rect();
        assert_eq!(x, 0);
        assert_eq!(y, TOP_CHROME_HEIGHT);
        assert_eq!(w, 1280);
        assert_eq!(h, 800 - TOP_CHROME_HEIGHT);

        let tabs = layout.calculate_tab_rects(3);
        assert_eq!(tabs.len(), 3);
        assert!(tabs[0].2 >= TAB_MIN_WIDTH);
        assert!(tabs[0].2 <= TAB_MAX_WIDTH);

        let omnibox = layout.omnibox_rect();
        assert!(omnibox.2 > 200, "Omnibox width should be wide enough");
        assert_eq!(omnibox.3, OMNIBOX_HEIGHT);
    }
}

