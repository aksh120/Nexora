use windows::Win32::System::Threading::{
    GetCurrentProcess, SetPriorityClass, ABOVE_NORMAL_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS,
    PROCESS_CREATION_FLAGS,
};

#[derive(Debug, Default, Clone)]
pub struct MediaMonitor {
    pub is_playing_audio: bool,
    pub is_fullscreen: bool,
    pub is_priority_boosted: bool,
}

impl MediaMonitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_audio_playing(&mut self, playing: bool) {
        self.is_playing_audio = playing;
        self.update_priority_scheduling();
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.is_fullscreen = fullscreen;
        self.update_priority_scheduling();
    }

    pub fn should_exempt_from_suspension(&self) -> bool {
        self.is_playing_audio || self.is_fullscreen
    }

    fn update_priority_scheduling(&mut self) {
        let should_boost = self.is_playing_audio || self.is_fullscreen;
        if should_boost && !self.is_priority_boosted {
            Self::apply_priority_class(ABOVE_NORMAL_PRIORITY_CLASS);
            self.is_priority_boosted = true;
            log::info!("[Media Engine] Active media detected. Process priority boosted to ABOVE_NORMAL.");
        } else if !should_boost && self.is_priority_boosted {
            Self::apply_priority_class(NORMAL_PRIORITY_CLASS);
            self.is_priority_boosted = false;
            log::info!("[Media Engine] Media idle. Process priority restored to NORMAL.");
        }
    }

    fn apply_priority_class(priority: PROCESS_CREATION_FLAGS) {
        unsafe {
            let proc = GetCurrentProcess();
            let _ = SetPriorityClass(proc, priority);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_monitor_suspension_exemption() {
        let mut monitor = MediaMonitor::new();
        assert!(!monitor.should_exempt_from_suspension());

        monitor.set_audio_playing(true);
        assert!(monitor.should_exempt_from_suspension());
        assert!(monitor.is_priority_boosted);

        monitor.set_audio_playing(false);
        assert!(!monitor.should_exempt_from_suspension());
        assert!(!monitor.is_priority_boosted);

        monitor.set_fullscreen(true);
        assert!(monitor.should_exempt_from_suspension());
        assert!(monitor.is_priority_boosted);
    }
}

