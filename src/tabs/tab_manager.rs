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

    pub fn tabs_mut(&mut self) -> &mut [Tab] {
        &mut self.tabs
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

    // --- Audio Control Methods ---
    pub fn set_tab_muted(&mut self, id: usize, muted: bool) {
        if let Some(tab) = self.get_tab_mut(id) {
            tab.set_muted(muted);
        }
    }

    pub fn toggle_tab_mute(&mut self, id: usize) -> Option<bool> {
        let tab = self.get_tab_mut(id)?;
        Some(tab.toggle_mute())
    }

    pub fn mute_other_tabs(&mut self, keep_id: usize) {
        for tab in self.tabs.iter_mut() {
            if tab.id != keep_id {
                tab.set_muted(true);
            }
        }
    }

    pub fn unmute_all_tabs(&mut self) {
        for tab in self.tabs.iter_mut() {
            tab.set_muted(false);
        }
    }

    // --- Pinned Tabs Methods ---
    pub fn pinned_tab_count(&self) -> usize {
        self.tabs.iter().filter(|t| t.is_pinned).count()
    }

    pub fn is_tab_pinned(&self, id: usize) -> bool {
        self.get_tab(id).map(|t| t.is_pinned).unwrap_or(false)
    }

    pub fn pin_tab(&mut self, id: usize) -> bool {
        let current_idx = match self.tabs.iter().position(|t| t.id == id) {
            Some(i) => i,
            None => return false,
        };

        if self.tabs[current_idx].is_pinned {
            return false;
        }

        self.tabs[current_idx].is_pinned = true;
        let tab = self.tabs.remove(current_idx);

        // Insert at the end of the pinned section
        let insert_idx = self.tabs.iter().position(|t| !t.is_pinned).unwrap_or(self.tabs.len());
        self.tabs.insert(insert_idx, tab);
        log::info!("Pinned tab #{} moved to index {}", id, insert_idx);
        true
    }

    pub fn unpin_tab(&mut self, id: usize) -> bool {
        let current_idx = match self.tabs.iter().position(|t| t.id == id) {
            Some(i) => i,
            None => return false,
        };

        if !self.tabs[current_idx].is_pinned {
            return false;
        }

        self.tabs[current_idx].is_pinned = false;
        let tab = self.tabs.remove(current_idx);

        // Insert at the beginning of the unpinned section
        let insert_idx = self.tabs.iter().position(|t| !t.is_pinned).unwrap_or(self.tabs.len());
        self.tabs.insert(insert_idx, tab);
        log::info!("Unpinned tab #{} moved to index {}", id, insert_idx);
        true
    }

    // --- Duplicate Tab ---
    pub fn duplicate_tab(&mut self, id: usize) -> Option<usize> {
        let (source_idx, source_url, source_title, zoom, muted) = {
            let (idx, tab) = self.tabs.iter().enumerate().find(|(_, t)| t.id == id)?;
            (idx, tab.url.clone(), tab.title.clone(), tab.zoom_factor, tab.is_muted)
        };

        let new_id = self.next_tab_id;
        self.next_tab_id += 1;

        let mut new_tab = Tab::new(new_id, &source_url, Some(&source_title));
        new_tab.zoom_factor = zoom;
        new_tab.is_muted = muted;
        new_tab.state = TabState::Inactive;

        // Insert immediately after source tab
        let insert_idx = (source_idx + 1).min(self.tabs.len());
        self.tabs.insert(insert_idx, new_tab);
        log::info!("Duplicated tab #{} as #{} at index {}", id, new_id, insert_idx);
        Some(new_id)
    }

    // --- Drag and Drop Tab Reordering ---
    pub fn move_tab(&mut self, from_index: usize, to_index: usize) -> bool {
        if from_index >= self.tabs.len() || to_index >= self.tabs.len() || from_index == to_index {
            return false;
        }

        let mut tab = self.tabs.remove(from_index);

        // Check if crossed pinned boundary
        let pinned_count = self.tabs.iter().filter(|t| t.is_pinned).count();
        if tab.is_pinned {
            if to_index > pinned_count {
                // Dragged out of pinned section -> unpin
                tab.is_pinned = false;
                log::info!("Tab #{} unpinned via drag past pinned zone", tab.id);
            }
        } else {
            if to_index < pinned_count {
                // Dragged into pinned section -> pin
                tab.is_pinned = true;
                log::info!("Tab #{} pinned via drag into pinned zone", tab.id);
            }
        }

        self.tabs.insert(to_index, tab);
        true
    }

    // --- Tab Close Variants ---
    pub fn close_other_tabs(&mut self, keep_id: usize) -> Vec<usize> {
        let mut closed_ids = Vec::new();
        let mut remaining = Vec::new();
        for tab in self.tabs.drain(..) {
            if tab.id == keep_id {
                remaining.push(tab);
            } else {
                closed_ids.push(tab.id);
            }
        }
        self.tabs = remaining;
        self.active_tab_id = Some(keep_id);
        if let Some(tab) = self.get_active_tab_mut() {
            tab.state = TabState::Active;
        }
        closed_ids
    }

    pub fn close_tabs_to_right(&mut self, pivot_id: usize) -> Vec<usize> {
        let pivot_idx = match self.tabs.iter().position(|t| t.id == pivot_id) {
            Some(i) => i,
            None => return Vec::new(),
        };

        let mut closed_ids = Vec::new();
        let mut kept = Vec::new();
        for (i, tab) in self.tabs.drain(..).enumerate() {
            if i <= pivot_idx {
                kept.push(tab);
            } else {
                closed_ids.push(tab.id);
            }
        }
        self.tabs = kept;

        // If the active tab was among the closed ones, set pivot as active
        if let Some(active) = self.active_tab_id {
            if closed_ids.contains(&active) {
                self.active_tab_id = Some(pivot_id);
                if let Some(tab) = self.get_active_tab_mut() {
                    tab.state = TabState::Active;
                }
            }
        }
        closed_ids
    }

    // --- Zoom & State ---
    pub fn set_tab_zoom(&mut self, id: usize, zoom: f64) {
        if let Some(tab) = self.get_tab_mut(id) {
            tab.set_zoom(zoom);
        }
    }

    pub fn get_tab_zoom(&self, id: usize) -> f64 {
        self.get_tab(id).map(|t| t.zoom_factor).unwrap_or(1.0)
    }

    pub fn set_tab_pip(&mut self, id: usize, active: bool) {
        if let Some(tab) = self.get_tab_mut(id) {
            tab.set_active_pip(active);
        }
    }

    pub fn set_tab_thumbnail(&mut self, id: usize, data: Option<Vec<u8>>) {
        if let Some(tab) = self.get_tab_mut(id) {
            tab.set_thumbnail(data);
        }
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

    #[test]
    fn test_tab_pinning_and_boundaries() {
        let mut manager = TabManager::new();
        let _t1 = manager.create_tab("https://t1.com", Some("T1"));
        let t2 = manager.create_tab("https://t2.com", Some("T2"));
        let t3 = manager.create_tab("https://t3.com", Some("T3"));

        assert_eq!(manager.pinned_tab_count(), 0);
        assert!(!manager.is_tab_pinned(t2));

        // Pin t2 -> should move to index 0
        assert!(manager.pin_tab(t2));
        assert_eq!(manager.pinned_tab_count(), 1);
        assert!(manager.is_tab_pinned(t2));
        assert_eq!(manager.tabs()[0].id, t2);

        // Pin t3 -> should move to index 1 (end of pinned group)
        assert!(manager.pin_tab(t3));
        assert_eq!(manager.pinned_tab_count(), 2);
        assert_eq!(manager.tabs()[1].id, t3);

        // Unpin t2 -> moves to beginning of unpinned
        assert!(manager.unpin_tab(t2));
        assert_eq!(manager.pinned_tab_count(), 1);
        assert!(!manager.is_tab_pinned(t2));
    }

    #[test]
    fn test_tab_audio_and_muting() {
        let mut manager = TabManager::new();
        let t1 = manager.create_tab("https://yt.com", Some("YouTube"));
        let t2 = manager.create_tab("https://sp.com", Some("Spotify"));
        let t3 = manager.create_tab("https://news.com", Some("News"));

        assert_eq!(manager.toggle_tab_mute(t1), Some(true));
        assert!(manager.tabs().iter().find(|t| t.id == t1).unwrap().is_muted);
        assert_eq!(manager.toggle_tab_mute(t1), Some(false));
        assert!(!manager.tabs().iter().find(|t| t.id == t1).unwrap().is_muted);

        // Mute others
        manager.mute_other_tabs(t1);
        assert!(!manager.tabs().iter().find(|t| t.id == t1).unwrap().is_muted);
        assert!(manager.tabs().iter().find(|t| t.id == t2).unwrap().is_muted);
        assert!(manager.tabs().iter().find(|t| t.id == t3).unwrap().is_muted);

        // Unmute all
        manager.unmute_all_tabs();
        assert!(!manager.tabs().iter().find(|t| t.id == t2).unwrap().is_muted);
        assert!(!manager.tabs().iter().find(|t| t.id == t3).unwrap().is_muted);
    }

    #[test]
    fn test_tab_duplication() {
        let mut manager = TabManager::new();
        let t1 = manager.create_tab("https://github.com", Some("GitHub"));
        manager.set_tab_zoom(t1, 1.25);
        manager.set_tab_muted(t1, true);

        let dup = manager.duplicate_tab(t1);
        assert!(dup.is_some());
        let dup_id = dup.unwrap();

        assert_eq!(manager.tab_count(), 2);
        let dup_tab = manager.get_tab(dup_id).unwrap();
        assert_eq!(dup_tab.url, "https://github.com");
        assert_eq!(dup_tab.title, "GitHub");
        assert_eq!(dup_tab.zoom_factor, 1.25);
        assert!(dup_tab.is_muted);
    }

    #[test]
    fn test_tab_drag_reordering() {
        let mut manager = TabManager::new();
        let t1 = manager.create_tab("https://1.com", Some("1"));
        let t2 = manager.create_tab("https://2.com", Some("2"));
        let t3 = manager.create_tab("https://3.com", Some("3"));

        // Move index 0 to index 2
        assert!(manager.move_tab(0, 2));
        assert_eq!(manager.tabs()[0].id, t2);
        assert_eq!(manager.tabs()[1].id, t3);
        assert_eq!(manager.tabs()[2].id, t1);
    }

    #[test]
    fn test_tab_close_others_and_right() {
        let mut manager = TabManager::new();
        let t1 = manager.create_tab("https://1.com", Some("1"));
        let t2 = manager.create_tab("https://2.com", Some("2"));
        let t3 = manager.create_tab("https://3.com", Some("3"));
        let t4 = manager.create_tab("https://4.com", Some("4"));

        // Close right of t2 -> closes t3, t4
        let closed = manager.close_tabs_to_right(t2);
        assert_eq!(closed, vec![t3, t4]);
        assert_eq!(manager.tab_count(), 2);

        // Close others of t2 -> closes t1
        let closed2 = manager.close_other_tabs(t2);
        assert_eq!(closed2, vec![t1]);
        assert_eq!(manager.tab_count(), 1);
        assert_eq!(manager.tabs()[0].id, t2);
    }
}
