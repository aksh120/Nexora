use std::time::{Duration, Instant};

/// Action to take upon a renderer process failure
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrashRecoveryAction {
    /// Seamless automatic reload of the preserved URL
    SilentReload(String),
    /// Loop throttle exceeded; display informative crash recovery card HTML
    ShowCrashCard(String),
}

/// Renderer Process Crash & Recovery Controller
///
/// Handles WebView2 `ProcessFailed` events with rapid auto-recovery
/// while throttling infinite crash loops.
#[derive(Debug)]
pub struct CrashRecoveryController {
    crash_timestamps: Vec<Instant>,
    max_crashes: usize,
    window_duration: Duration,
}

impl Default for CrashRecoveryController {
    fn default() -> Self {
        Self {
            crash_timestamps: Vec::new(),
            max_crashes: 3,
            window_duration: Duration::from_secs(10),
        }
    }
}

impl CrashRecoveryController {
    pub fn new(max_crashes: usize, window_secs: u64) -> Self {
        Self {
            crash_timestamps: Vec::new(),
            max_crashes,
            window_duration: Duration::from_secs(window_secs),
        }
    }

    /// Evaluates a process failure and determines whether to silently reload
    /// or present the interactive crash fallback card.
    pub fn handle_failure(&mut self, url: &str, reason: &str) -> CrashRecoveryAction {
        let now = Instant::now();

        // Prune timestamps older than the evaluation window
        self.crash_timestamps.retain(|&t| now.saturating_duration_since(t) < self.window_duration);
        self.crash_timestamps.push(now);

        let count = self.crash_timestamps.len();
        log::warn!(
            "CrashRecoveryController: Process failure ({}) on '{}' (count in window: {}/{})",
            reason,
            url,
            count,
            self.max_crashes
        );

        if count >= self.max_crashes {
            log::error!(
                "Crash threshold exceeded for '{}' ({} crashes in {:?}). Showing recovery card.",
                url,
                count,
                self.window_duration
            );
            CrashRecoveryAction::ShowCrashCard(Self::generate_crash_card_html(url, reason))
        } else {
            log::info!("Initiating silent reload recovery for '{}'...", url);
            CrashRecoveryAction::SilentReload(url.to_string())
        }
    }

    pub fn reset(&mut self) {
        self.crash_timestamps.clear();
    }

    /// Generates lightweight standalone HTML markup for a crashed tab card
    pub fn generate_crash_card_html(url: &str, reason: &str) -> String {
        let url_escaped = html_escape(url);
        let reason_escaped = html_escape(reason);

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Tab Recovered - Nexora</title>
    <style>
        :root {{
            --bg-color: #0f1217;
            --card-bg: #1a1f29;
            --text-main: #f0f3f6;
            --text-dim: #8b949e;
            --accent: #ef4444;
            --accent-hover: #dc2626;
            --btn-bg: #2563eb;
            --btn-hover: #1d4ed8;
            --border: #2a3443;
        }}
        @media (prefers-color-scheme: light) {{
            :root {{
                --bg-color: #f6f8fa;
                --card-bg: #ffffff;
                --text-main: #1f2328;
                --text-dim: #656d76;
                --accent: #cf222e;
                --accent-hover: #a40e26;
                --btn-bg: #0969da;
                --btn-hover: #0853b0;
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
        }}
        .card {{
            background-color: var(--card-bg);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 36px 44px;
            max-width: 500px;
            text-align: center;
            box-shadow: 0 12px 36px rgba(0, 0, 0, 0.25);
        }}
        .icon {{
            font-size: 42px;
            margin-bottom: 16px;
        }}
        h2 {{
            margin: 0 0 10px 0;
            font-size: 20px;
            font-weight: 600;
        }}
        .meta {{
            color: var(--text-dim);
            font-size: 13px;
            word-break: break-all;
            margin-bottom: 20px;
        }}
        .reason-badge {{
            display: inline-block;
            background: rgba(239, 68, 68, 0.12);
            color: var(--accent);
            border: 1px solid rgba(239, 68, 68, 0.3);
            border-radius: 16px;
            padding: 4px 12px;
            font-size: 12px;
            font-weight: 500;
            margin-bottom: 24px;
        }}
        .reload-btn {{
            background-color: var(--btn-bg);
            color: #ffffff;
            border: none;
            border-radius: 8px;
            padding: 10px 24px;
            font-size: 14px;
            font-weight: 600;
            cursor: pointer;
            transition: background-color 0.15s ease;
        }}
        .reload-btn:hover {{
            background-color: var(--btn-hover);
        }}
        .hint {{
            margin-top: 16px;
            color: var(--text-dim);
            font-size: 12px;
        }}
    </style>
</head>
<body>
    <div class="card">
        <div class="icon">⚠️</div>
        <h2>Tab Crashed Repeatedly</h2>
        <div class="reason-badge">Reason: {reason_escaped}</div>
        <div class="meta">{url_escaped}</div>
        <button class="reload-btn" onclick="window.location.href='{url_escaped}'">Reload Page</button>
        <div class="hint">Nexora paused auto-reloading to prevent system instability.</div>
    </div>
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
    fn test_crash_controller_silent_reload_and_throttling() {
        let mut controller = CrashRecoveryController::new(3, 10);
        let url = "https://complex-site.com";

        // 1st crash -> Silent reload
        let action1 = controller.handle_failure(url, "RendererExit");
        assert_eq!(action1, CrashRecoveryAction::SilentReload(url.to_string()));

        // 2nd crash -> Silent reload
        let action2 = controller.handle_failure(url, "RendererExit");
        assert_eq!(action2, CrashRecoveryAction::SilentReload(url.to_string()));

        // 3rd crash -> Threshold reached, show fallback card
        let action3 = controller.handle_failure(url, "RendererExit");
        assert!(matches!(action3, CrashRecoveryAction::ShowCrashCard(_)));

        // Reset
        controller.reset();
        let action_after_reset = controller.handle_failure(url, "RendererExit");
        assert_eq!(action_after_reset, CrashRecoveryAction::SilentReload(url.to_string()));
    }

    #[test]
    fn test_generate_crash_card_html() {
        let html = CrashRecoveryController::generate_crash_card_html(
            "https://example.com/test?a=1&b=2",
            "Out of Memory",
        );
        assert!(html.contains("Tab Crashed Repeatedly"));
        assert!(html.contains("https://example.com/test?a=1&amp;b=2"));
        assert!(html.contains("Out of Memory"));
        assert!(html.contains("Reload Page"));
    }
}

