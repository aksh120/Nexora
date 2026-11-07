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
    pub history: Vec<String>,
    pub history_index: usize,
}

impl Tab {
    pub fn new(id: usize, url: &str, title: Option<&str>) -> Self {
        let now = Instant::now();
        let display_title = title.unwrap_or(if url.is_empty() || url == "about:blank" {
            "New Tab"
        } else {
            url
        });

        let initial_history = if !url.is_empty() && url != "about:blank" {
            vec![url.to_string()]
        } else {
            Vec::new()
        };

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
            history: initial_history,
            history_index: 0,
        }
    }

    pub fn push_history(&mut self, new_url: &str) {
        if new_url.is_empty() || new_url == "about:blank" {
            return;
        }
        if !self.history.is_empty() && self.history_index < self.history.len() {
            if self.history[self.history_index] == new_url {
                return;
            }
            // Branch history
            self.history.truncate(self.history_index + 1);
        }
        self.history.push(new_url.to_string());
        self.history_index = self.history.len().saturating_sub(1);
    }

    pub fn can_go_back(&self) -> bool {
        self.history_index > 0 && !self.history.is_empty()
    }

    pub fn can_go_forward(&self) -> bool {
        !self.history.is_empty() && self.history_index + 1 < self.history.len()
    }

    pub fn go_back(&mut self) -> Option<String> {
        if self.can_go_back() {
            self.history_index -= 1;
            let url = self.history[self.history_index].clone();
            self.url = url.clone();
            Some(url)
        } else {
            None
        }
    }

    pub fn go_forward(&mut self) -> Option<String> {
        if self.can_go_forward() {
            self.history_index += 1;
            let url = self.history[self.history_index].clone();
            self.url = url.clone();
            Some(url)
        } else {
            None
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

