use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabState {
    Active,
    Inactive,
    Suspended,
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: usize,
    pub title: String,
    pub url: String,
    pub state: TabState,
    pub is_playing_audio: bool,
    pub is_loading: bool,
    pub favicon_url: Option<String>,
    pub scroll_position: (i32, i32),
    pub created_at: Instant,
    pub last_accessed: Instant,
}

impl Tab {
    pub fn new(id: usize, url: &str, title: Option<&str>) -> Self {
        let now = Instant::now();
        let display_title = title.unwrap_or(if url.is_empty() || url == "about:blank" {
            "New Tab"
        } else {
            url
        });

        Self {
            id,
            title: display_title.to_string(),
            url: url.to_string(),
            state: TabState::Active,
            is_playing_audio: false,
            is_loading: false,
            favicon_url: None,
            scroll_position: (0, 0),
            created_at: now,
            last_accessed: now,
        }
    }

    pub fn is_active(&self) -> bool {
        self.state == TabState::Active
    }

    pub fn is_suspended(&self) -> bool {
        self.state == TabState::Suspended
    }

    pub fn can_be_suspended(&self) -> bool {
        !self.is_active() && !self.is_playing_audio && !self.is_loading
    }

    pub fn suspend(&mut self) -> bool {
        if self.can_be_suspended() {
            self.state = TabState::Suspended;
            true
        } else {
            false
        }
    }

    pub fn restore(&mut self) {
        if self.is_suspended() {
            self.state = TabState::Inactive;
            self.last_accessed = Instant::now();
        }
    }

    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
    }
}

