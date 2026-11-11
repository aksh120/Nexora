//! Update Notification Banner Component
//!
//! Generates the interactive, native-styled update notification banner
//! injected into the browser viewport when a newer release is detected.

use crate::updater::detector::ReleaseInfo;

pub struct UpdateBanner;

impl UpdateBanner {
    /// Generates self-contained JavaScript to inject the update notification banner into the active page.
    pub fn script_show(release: &ReleaseInfo) -> String {
        let version_escaped = js_escape(&release.version);
        let _tag_escaped = js_escape(&release.tag_name);
        let _name_escaped = js_escape(&release.name);
        let notes_raw: String = release.body.lines().take(3).collect::<Vec<&str>>().join(" ");
        let notes_escaped = js_escape(&notes_raw);
        let download_url = release
            .download_url
            .as_deref()
            .unwrap_or("");
        let download_url_escaped = js_escape(download_url);

        format!(
            r###"(function() {{
    if (document.getElementById('nexora-update-banner')) return;

    const banner = document.createElement('div');
    banner.id = 'nexora-update-banner';
    banner.innerHTML = `
        <div class="nexora-banner-wrap">
            <div class="nexora-banner-left">
                <span class="nexora-badge">UPDATE AVAILABLE</span>
                <span class="nexora-banner-title">Nexora <strong>v{version}</strong> is ready to install</span>
                <span class="nexora-banner-notes">{notes}</span>
            </div>
            <div class="nexora-banner-actions">
                <button id="nexora-update-action-btn" class="nexora-btn-primary" onclick="window.__nexora_start_update()">
                    Update & Restart
                </button>
                <button class="nexora-btn-dismiss" onclick="window.__nexora_dismiss_update()" title="Dismiss update notification">
                    &times;
                </button>
            </div>
        </div>
        <div id="nexora-update-progress-container" class="nexora-progress-container" style="display: none;">
            <div class="nexora-progress-bar">
                <div id="nexora-update-progress-fill" class="nexora-progress-fill" style="width: 0%;"></div>
            </div>
            <div id="nexora-update-status" class="nexora-status-text">Starting download...</div>
        </div>
    `;

    const style = document.createElement('style');
    style.id = 'nexora-update-banner-styles';
    style.textContent = `
        #nexora-update-banner {{
            position: fixed !important;
            top: 0 !important;
            left: 0 !important;
            right: 0 !important;
            z-index: 2147483646 !important;
            background: rgba(15, 15, 18, 0.96) !important;
            backdrop-filter: blur(16px) !important;
            border-bottom: 1px solid rgba(225, 29, 72, 0.35) !important;
            box-shadow: 0 4px 20px rgba(0, 0, 0, 0.6) !important;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif !important;
            color: #f4f4f5 !important;
            padding: 8px 16px !important;
            box-sizing: border-box !important;
            animation: nexoraBannerSlideDown 0.22s ease-out !important;
        }}

        @keyframes nexoraBannerSlideDown {{
            from {{ transform: translateY(-100%); opacity: 0; }}
            to {{ transform: translateY(0); opacity: 1; }}
        }}

        .nexora-banner-wrap {{
            display: flex !important;
            align-items: center !important;
            justify-content: space-between !important;
            gap: 16px !important;
        }}

        .nexora-banner-left {{
            display: flex !important;
            align-items: center !important;
            gap: 10px !important;
            overflow: hidden !important;
        }}

        .nexora-badge {{
            background: rgba(225, 29, 72, 0.15) !important;
            border: 1px solid rgba(225, 29, 72, 0.45) !important;
            color: #f43f5e !important;
            padding: 2px 7px !important;
            border-radius: 4px !important;
            font-size: 10px !important;
            font-weight: 700 !important;
            letter-spacing: 0.5px !important;
            flex-shrink: 0 !important;
        }}

        .nexora-banner-title {{
            font-size: 13px !important;
            color: #e4e4e7 !important;
            flex-shrink: 0 !important;
        }}

        .nexora-banner-title strong {{
            color: #ffffff !important;
        }}

        .nexora-banner-notes {{
            font-size: 12px !important;
            color: #a1a1aa !important;
            white-space: nowrap !important;
            overflow: hidden !important;
            text-overflow: ellipsis !important;
            max-width: 480px !important;
        }}

        .nexora-banner-actions {{
            display: flex !important;
            align-items: center !important;
            gap: 8px !important;
            flex-shrink: 0 !important;
        }}

        .nexora-btn-primary {{
            background: #e11d48 !important;
            color: #ffffff !important;
            border: none !important;
            padding: 5px 12px !important;
            border-radius: 5px !important;
            font-size: 12px !important;
            font-weight: 600 !important;
            cursor: pointer !important;
            transition: all 0.12s ease !important;
        }}

        .nexora-btn-primary:hover {{
            background: #be123c !important;
        }}

        .nexora-btn-dismiss {{
            background: transparent !important;
            color: #71717a !important;
            border: none !important;
            font-size: 18px !important;
            line-height: 1 !important;
            padding: 4px 6px !important;
            cursor: pointer !important;
            border-radius: 4px !important;
            transition: color 0.12s ease !important;
        }}

        .nexora-btn-dismiss:hover {{
            color: #ffffff !important;
            background: rgba(255, 255, 255, 0.08) !important;
        }}

        .nexora-progress-container {{
            margin-top: 8px !important;
            display: flex !important;
            flex-direction: column !important;
            gap: 4px !important;
        }}

        .nexora-progress-bar {{
            height: 4px !important;
            background: #27272a !important;
            border-radius: 2px !important;
            overflow: hidden !important;
            width: 100% !important;
        }}

        .nexora-progress-fill {{
            height: 100% !important;
            background: #f43f5e !important;
            border-radius: 2px !important;
            transition: width 0.15s ease !important;
        }}

        .nexora-status-text {{
            font-size: 11px !important;
            color: #a1a1aa !important;
        }}
    `;

    document.head.appendChild(style);
    document.body.appendChild(banner);

    window.__nexora_start_update = function() {{
        const btn = document.getElementById('nexora-update-action-btn');
        if (btn) {{
            btn.disabled = true;
            btn.textContent = "Downloading...";
            btn.style.opacity = "0.7";
        }}
        const prog = document.getElementById('nexora-update-progress-container');
        if (prog) prog.style.display = 'flex';

        if (window.chrome && window.chrome.webview) {{
            window.chrome.webview.postMessage(JSON.stringify({{
                action: "trigger_update",
                download_url: "{download_url}",
                version: "{version}"
            }}));
        }}
    }};

    window.__nexora_dismiss_update = function() {{
        const el = document.getElementById('nexora-update-banner');
        if (el) el.remove();
        const st = document.getElementById('nexora-update-banner-styles');
        if (st) st.remove();

        if (window.chrome && window.chrome.webview) {{
            window.chrome.webview.postMessage(JSON.stringify({{
                action: "dismiss_update"
            }}));
        }}
    }};

    window.__nexora_on_update_progress = function(percent, statusText) {{
        const fill = document.getElementById('nexora-update-progress-fill');
        if (fill) fill.style.width = percent + '%';
        const status = document.getElementById('nexora-update-status');
        if (status) status.textContent = statusText;
    }};
}})();
"###,
            version = version_escaped,
            notes = notes_escaped,
            download_url = download_url_escaped,
        )
    }

    /// Script to update the progress bar in the injected banner.
    pub fn script_update_progress(percent: f64, status_text: &str) -> String {
        format!(
            "if (window.__nexora_on_update_progress) {{ window.__nexora_on_update_progress({:.1}, '{}'); }}",
            percent,
            js_escape(status_text)
        )
    }
}

fn js_escape(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('"', "\\\"")
        .replace('\n', " ")
        .replace('\r', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_show_generation() {
        let rel = ReleaseInfo {
            tag_name: "v0.2.0".into(),
            version: "0.2.0".into(),
            name: "Release 0.2.0".into(),
            body: "Major speed enhancements".into(),
            html_url: "https://github.com/aksh120/Nexora/releases/v0.2.0".into(),
            download_url: Some("https://github.com/aksh120/Nexora/releases/download/v0.2.0/Nexora.exe".into()),
            asset_name: Some("Nexora.exe".into()),
            asset_size: Some(1646080),
        };

        let script = UpdateBanner::script_show(&rel);
        assert!(script.contains("nexora-update-banner"));
        assert!(script.contains("v0.2.0"));
        assert!(script.contains("Update & Restart"));
        assert!(script.contains("trigger_update"));
    }
}
