use std::time::{Duration, Instant};
use crate::tabs::tab::{Tab, TabState};
use crate::tabs::tab_manager::TabManager;
use crate::performance::pressure::{MemoryPressureLevel, MemoryPressureMonitor};
use crate::performance::memory::trim_working_set;

/// Smart Suspension Engine orchestrating idle detection, pressure adaptation,
/// and tab memory reclamation.
pub struct SmartSuspensionEngine {
    pub base_timeout_minutes: u32,
    pub enabled: bool,
    monitor: MemoryPressureMonitor,
}

impl Default for SmartSuspensionEngine {
    fn default() -> Self {
        Self {
            base_timeout_minutes: 10,
            enabled: true,
            monitor: MemoryPressureMonitor::new(),
        }
    }
}

impl SmartSuspensionEngine {
    pub fn new(base_timeout_minutes: u32, enabled: bool) -> Self {
        Self {
            base_timeout_minutes,
            enabled,
            monitor: MemoryPressureMonitor::new(),
        }
    }

    /// Evaluates all inactive tabs and suspends eligible candidates exceeding the idle threshold.
    /// Returns the list of tab IDs that were suspended in this sweep.
    pub fn sweep_and_suspend(&self, tab_manager: &mut TabManager) -> Vec<usize> {
        if !self.enabled {
            return Vec::new();
        }

        let pressure = self.monitor.current_pressure_level().unwrap_or(MemoryPressureLevel::Normal);
        let effective_timeout_minutes = self.monitor.calculate_adaptive_timeout(self.base_timeout_minutes, pressure);
        let timeout_duration = Duration::from_secs((effective_timeout_minutes as u64) * 60);

        let mut suspended_ids = Vec::new();
        let now = Instant::now();

        for tab in tab_manager.tabs_mut() {
            // Strict exemption checks:
            // 1. Must not be currently active
            // 2. Must not be playing audio/video
            // 3. Must not be loading
            // 4. Must not already be suspended
            // 5. Must not be an internal or blank page
            if tab.is_active() || tab.is_playing_audio || tab.is_loading || tab.is_suspended() {
                continue;
            }

            if tab.url.is_empty() || tab.url == "about:blank" {
                continue;
            }

            // Check idle duration
            let idle_duration = now.saturating_duration_since(tab.last_accessed);
            if idle_duration >= timeout_duration {
                if tab.suspend() {
                    log::info!(
                        "SmartSuspension: Suspended tab #{} '{}' (idle for {}s, pressure: {:?})",
                        tab.id,
                        tab.title,
                        idle_duration.as_secs(),
                        pressure
                    );
                    suspended_ids.push(tab.id);
                }
            }
        }

        // If any tabs were suspended, trim working set to immediately return pages to Windows
        if !suspended_ids.is_empty() {
            let _ = trim_working_set();
        }

        suspended_ids
    }

    /// Explicitly suspend a specific tab by ID (manual user action or tab context menu)
    pub fn force_suspend_tab(&self, tab_manager: &mut TabManager, tab_id: usize) -> bool {
        if let Some(tab) = tab_manager.get_tab_mut(tab_id) {
            if tab.can_be_suspended() {
                let success = tab.suspend();
                if success {
                    log::info!("Manually suspended tab #{} '{}'", tab.id, tab.title);
                    let _ = trim_working_set();
                }
                return success;
            }
        }
        false
    }

    /// Restore a suspended tab, updating its state and returning its target URL
    pub fn restore_tab(&self, tab_manager: &mut TabManager, tab_id: usize) -> Option<String> {
        let tab = tab_manager.get_tab_mut(tab_id)?;
        if tab.is_suspended() {
            tab.restore();
            log::info!("Restored suspended tab #{} '{}'", tab.id, tab.title);
            Some(tab.url.clone())
        } else {
            Some(tab.url.clone())
        }
    }

    /// Generates lightweight, standalone HTML markup for a suspended tab placeholder
    pub fn generate_suspended_html(tab: &Tab) -> String {
        let title_escaped = html_escape(&tab.title);
        let url_escaped = html_escape(&tab.url);

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Suspended: {title_escaped}</title>
    <style>
        :root {{
            --bg-color: #12151a;
            --card-bg: #1c212a;
            --text-main: #f0f3f6;
            --text-dim: #8b949e;
            --accent: #3b82f6;
            --accent-hover: #2563eb;
            --border: #2a3342;
        }}
        @media (prefers-color-scheme: light) {{
            :root {{
                --bg-color: #f6f8fa;
                --card-bg: #ffffff;
                --text-main: #1f2328;
                --text-dim: #656d76;
                --accent: #0969da;
                --accent-hover: #0853b0;
                --border: #d0d7de;
            }}
        }}
        body {{
            margin: 0;
            padding: 0;
            height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            background-color: var(--bg-color);
            color: var(--text-main);
            user-select: none;
            cursor: pointer;
        }}
        .card {{
            background-color: var(--card-bg);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 36px 44px;
            max-width: 480px;
            text-align: center;
            box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
            transition: transform 0.2s ease, box-shadow 0.2s ease;
        }}
        .card:hover {{
            transform: translateY(-2px);
            box-shadow: 0 14px 40px rgba(0, 0, 0, 0.28);
        }}
        .icon {{
            font-size: 40px;
            margin-bottom: 16px;
            display: inline-block;
        }}
        h2 {{
            margin: 0 0 10px 0;
            font-size: 20px;
            font-weight: 600;
        }}
        .meta-url {{
            color: var(--text-dim);
            font-size: 13px;
            word-break: break-all;
            margin-bottom: 24px;
        }}
        .badge {{
            display: inline-block;
            background: rgba(59, 130, 246, 0.12);
            color: var(--accent);
            border: 1px solid rgba(59, 130, 246, 0.3);
            border-radius: 20px;
            padding: 4px 12px;
            font-size: 11px;
            font-weight: 600;
            margin-bottom: 20px;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }}
        .wake-btn {{
            background-color: var(--accent);
            color: #ffffff;
            border: none;
            border-radius: 8px;
            padding: 10px 24px;
            font-size: 14px;
            font-weight: 600;
            cursor: pointer;
            transition: background-color 0.15s ease;
        }}
        .wake-btn:hover {{
            background-color: var(--accent-hover);
        }}
        .hint {{
            margin-top: 16px;
            color: var(--text-dim);
            font-size: 12px;
        }}
    </style>
</head>
<body onclick="wakeTab()">
    <div class="card" onclick="event.stopPropagation(); wakeTab();">
        <div class="icon">💤</div>
        <div class="badge">Tab Suspended by Nexora</div>
        <h2>{title_escaped}</h2>
        <div class="meta-url">{url_escaped}</div>
        <button class="wake-btn" onclick="wakeTab()">Wake Tab</button>
        <div class="hint">Suspended to conserve CPU & memory. Click anywhere to restore.</div>
    </div>
    <script>
        function wakeTab() {{
            window.location.href = "{url_escaped}";
        }}
    </script>
</body>
</html>"#
        )
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suspension_engine_rules() {
        let engine = SmartSuspensionEngine::new(10, true);
        let mut mgr = TabManager::new();

        // Create 3 tabs
        let t1 = mgr.create_tab("https://youtube.com", Some("YouTube"));
        let t2 = mgr.create_tab("https://reddit.com", Some("Reddit"));
        let t3 = mgr.create_tab("https://news.ycombinator.com", Some("Hacker News"));

        // t3 is currently Active
        // Mark t1 as playing audio
        if let Some(tab1) = mgr.get_tab_mut(t1) {
            tab1.is_playing_audio = true;
        }

        // Simulate idle time for t2 (inactive tab) past 11 minutes
        if let Some(tab2) = mgr.get_tab_mut(t2) {
            tab2.last_accessed = Instant::now() - Duration::from_secs(12 * 60);
        }

        let suspended = engine.sweep_and_suspend(&mut mgr);

        // Only t2 should be suspended:
        // t3 is active -> exempt
        // t1 is playing audio -> exempt
        assert_eq!(suspended, vec![t2]);
        assert_eq!(mgr.get_tab(t2).unwrap().state, TabState::Suspended);
        assert_eq!(mgr.get_tab(t1).unwrap().state, TabState::Inactive);
        assert_eq!(mgr.get_tab(t3).unwrap().state, TabState::Active);
    }

    #[test]
    fn test_manual_suspend_and_restore() {
        let engine = SmartSuspensionEngine::new(10, true);
        let mut mgr = TabManager::new();

        let t1 = mgr.create_tab("https://wikipedia.org", Some("Wikipedia"));
        let _t2 = mgr.create_tab("https://google.com", Some("Google")); // active tab

        // t1 is inactive
        let suspended = engine.force_suspend_tab(&mut mgr, t1);
        assert!(suspended, "Manual suspend on inactive tab must succeed");
        assert!(mgr.get_tab(t1).unwrap().is_suspended());

        let url = engine.restore_tab(&mut mgr, t1);
        assert_eq!(url.as_deref(), Some("https://wikipedia.org"));
        assert!(!mgr.get_tab(t1).unwrap().is_suspended());
    }

    #[test]
    fn test_generate_suspended_html() {
        let tab = Tab::new(1, "https://example.com/test?a=1&b=2", Some("Example <Title>"));
        let html = SmartSuspensionEngine::generate_suspended_html(&tab);
        assert!(html.contains("Example &lt;Title&gt;"));
        assert!(html.contains("https://example.com/test?a=1&amp;b=2"));
        assert!(html.contains("Tab Suspended by Nexora"));
        assert!(html.contains("Wake Tab"));
    }
}
