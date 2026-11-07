use crate::tabs::tab::{Tab, TabState};

#[derive(Debug, Default)]
pub struct TabManager {
    tabs: Vec<Tab>,
    active_tab_id: Option<usize>,
    next_tab_id: usize,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab_id: None,
            next_tab_id: 1,
        }
    }

    pub fn create_tab(&mut self, url: &str, title: Option<&str>) -> usize {
        let id = self.next_tab_id;
        self.next_tab_id += 1;

        // Inactivate current active tab
        if let Some(active_id) = self.active_tab_id {
            if let Some(tab) = self.get_tab_mut(active_id) {
                if tab.state == TabState::Active {
                    tab.state = TabState::Inactive;
                }
            }
        }

        let mut new_tab = Tab::new(id, url, title);
        new_tab.state = TabState::Active;

        self.tabs.push(new_tab);
        self.active_tab_id = Some(id);

        log::info!("Created tab #{} ({})", id, url);
        id
    }

    pub fn close_tab(&mut self, id: usize) -> Option<usize> {
        let index = self.tabs.iter().position(|t| t.id == id)?;
        let was_active = self.active_tab_id == Some(id);

        self.tabs.remove(index);
        log::info!("Closed tab #{}", id);

        if self.tabs.is_empty() {
            self.active_tab_id = None;
            return None;
        }

        if was_active {
            // Select adjacent tab (prefer same index which is now the right neighbor, or previous index)
            let new_index = if index < self.tabs.len() {
                index
            } else {
                self.tabs.len() - 1
            };

            let new_active_id = self.tabs[new_index].id;
            self.tabs[new_index].state = TabState::Active;
            self.tabs[new_index].touch();
            self.active_tab_id = Some(new_active_id);
            log::info!("Active tab switched to #{} after closure", new_active_id);
            Some(new_active_id)
        } else {
            self.active_tab_id
        }
    }

    pub fn switch_tab(&mut self, target_id: usize) -> bool {
        if self.active_tab_id == Some(target_id) {
            return true;
        }

        let target_exists = self.tabs.iter().any(|t| t.id == target_id);
        if !target_exists {
            return false;
        }

        // Set previous active tab to inactive
        if let Some(curr_id) = self.active_tab_id {
            if let Some(tab) = self.get_tab_mut(curr_id) {
                if tab.state == TabState::Active {
                    tab.state = TabState::Inactive;
                }
            }
        }

        // Set target tab to active and restore if suspended
        if let Some(tab) = self.get_tab_mut(target_id) {
            if tab.state == TabState::Suspended {
                log::info!("Waking suspended tab #{} upon user switch", target_id);
            }
            tab.state = TabState::Active;
            tab.touch();
        }

        self.active_tab_id = Some(target_id);
        log::info!("Switched active tab to #{}", target_id);
        true
    }

    pub fn get_tab(&self, id: usize) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id == id)
    }

    pub fn get_tab_mut(&mut self, id: usize) -> Option<&mut Tab> {
        self.tabs.iter_mut().find(|t| t.id == id)
    }

    pub fn get_active_tab(&self) -> Option<&Tab> {
        self.active_tab_id.and_then(|id| self.get_tab(id))
    }

    pub fn get_active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.active_tab_id.and_then(|id| self.get_tab_mut(id))
    }

    pub fn active_tab_id(&self) -> Option<usize> {
        self.active_tab_id
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    pub fn update_title(&mut self, id: usize, title: &str) {
        if let Some(tab) = self.get_tab_mut(id) {
            tab.title = title.to_string();
        }
    }

    pub fn update_url(&mut self, id: usize, url: &str) {
        if let Some(tab) = self.get_tab_mut(id) {
            tab.url = url.to_string();
        }
    }

    pub fn set_audio_playing(&mut self, id: usize, playing: bool) {
        if let Some(tab) = self.get_tab_mut(id) {
            tab.is_playing_audio = playing;
        }
    }

    pub fn set_loading(&mut self, id: usize, loading: bool) {
        if let Some(tab) = self.get_tab_mut(id) {
            tab.is_loading = loading;
        }
    }

    pub fn suspend_tab(&mut self, id: usize) -> bool {
        if let Some(tab) = self.get_tab_mut(id) {
            let res = tab.suspend();
            if res {
                log::info!("Tab #{} successfully suspended", id);
            }
            res
        } else {
            false
        }
    }

    pub fn restore_tab(&mut self, id: usize) -> bool {
        if let Some(tab) = self.get_tab_mut(id) {
            if tab.is_suspended() {
                tab.restore();
                log::info!("Tab #{} restored from suspension", id);
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_manager_lifecycle() {
        let mut manager = TabManager::new();
        assert_eq!(manager.tab_count(), 0);
        assert_eq!(manager.active_tab_id(), None);

        // 1. Create first tab
        let tab1 = manager.create_tab("https://www.google.com", Some("Google"));
        assert_eq!(manager.tab_count(), 1);
        assert_eq!(manager.active_tab_id(), Some(tab1));
        assert_eq!(manager.get_active_tab().unwrap().title, "Google");
        assert_eq!(manager.get_active_tab().unwrap().state, TabState::Active);

        // 2. Create second tab
        let tab2 = manager.create_tab("https://www.youtube.com", Some("YouTube"));
        assert_eq!(manager.tab_count(), 2);
        assert_eq!(manager.active_tab_id(), Some(tab2));
        assert_eq!(manager.get_tab(tab1).unwrap().state, TabState::Inactive);
        assert_eq!(manager.get_tab(tab2).unwrap().state, TabState::Active);

        // 3. Switch back to tab1
        assert!(manager.switch_tab(tab1));
        assert_eq!(manager.active_tab_id(), Some(tab1));
        assert_eq!(manager.get_tab(tab1).unwrap().state, TabState::Active);
        assert_eq!(manager.get_tab(tab2).unwrap().state, TabState::Inactive);

        // 4. Test suspension rules
        // Cannot suspend active tab1
        assert!(!manager.suspend_tab(tab1));
        // Can suspend inactive tab2
        assert!(manager.suspend_tab(tab2));
        assert_eq!(manager.get_tab(tab2).unwrap().state, TabState::Suspended);

        // Audio playing tab cannot be suspended
        let tab3 = manager.create_tab("https://spotify.com", Some("Spotify"));
        manager.switch_tab(tab1);
        manager.set_audio_playing(tab3, true);
        assert!(!manager.suspend_tab(tab3));

        // 5. Switching to suspended tab restores it
        assert!(manager.switch_tab(tab2));
        assert_eq!(manager.get_tab(tab2).unwrap().state, TabState::Active);

        // 6. Close active tab
        let new_active = manager.close_tab(tab2);
        assert_eq!(manager.tab_count(), 2);
        assert!(new_active.is_some());
        assert_eq!(manager.active_tab_id(), new_active);
    }
}
