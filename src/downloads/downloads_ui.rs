//! Download Manager Interface
//!
//! Renders the clean, native `nexora://downloads` dashboard.

use crate::downloads::manager::{DownloadItem, DownloadState};
use crate::ui::design_system::DesignTokens;
use crate::ui::icons::LucideIcon;

pub struct DownloadsUi;

impl DownloadsUi {
    pub fn render_html(downloads: &[DownloadItem]) -> String {
        let shared_css = DesignTokens::shared_css(true);
        let download_icon = LucideIcon::Download.to_svg(20, 2.0, "var(--accent)");
        let folder_icon = LucideIcon::ExternalLink.to_svg(14, 2.0, "currentColor");
        let hash_icon = LucideIcon::ShieldCheck.to_svg(12, 2.0, "#10b981");

        let mut rows = String::new();
        if downloads.is_empty() {
            rows.push_str(
                r#"<div class="empty-state">
                    <p style="color:var(--text-secondary);font-size:14px;">No downloads yet</p>
                    <p style="color:var(--text-muted);font-size:12px;margin-top:4px;">Files you download will appear here.</p>
                </div>"#,
            );
        } else {
            for item in downloads {
                let percent = item.progress_percent();
                let status_text = match item.state {
                    DownloadState::InProgress => format!("{:.1}% • {:.1} MB/s", percent, item.speed_mb_s),
                    DownloadState::Completed => "Completed".to_string(),
                    DownloadState::Paused => "Paused".to_string(),
                    DownloadState::Cancelled => "Cancelled".to_string(),
                    DownloadState::Failed => "Failed".to_string(),
                };

                let hash_badge = if let Some(ref h) = item.sha256_hash {
                    let short_hash = if h.len() > 16 { &h[..16] } else { h.as_str() };
                    format!(r#"<span class="hash-tag" title="SHA-256: {}">{} SHA-256: {}...</span>"#, h, hash_icon, short_hash)
                } else {
                    String::new()
                };

                let escaped_name = html_escape(&item.file_name);
                let escaped_path = html_escape(&item.file_path.to_string_lossy());

                rows.push_str(&format!(
                    r###"<div class="download-card">
                        <div class="card-icon">{download_icon}</div>
                        <div class="card-info">
                            <div class="file-name" title="{escaped_path}">{escaped_name}</div>
                            <div class="file-meta">
                                <span>{}</span>
                                <span>•</span>
                                <span>{status_text}</span>
                                {hash_badge}
                            </div>
                            <div class="progress-bar-bg">
                                <div class="progress-bar-fill" style="width: {:.1}%"></div>
                            </div>
                        </div>
                        <div class="card-actions">
                            <button class="action-btn" onclick="openFile('{escaped_path}')" title="Open file">{folder_icon} Open</button>
                        </div>
                    </div>"###,
                    item.formatted_size(),
                    percent
                ));
            }
        }

        format!(
            r###"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Downloads - Nexora</title>
    <style>
        {shared_css}
        body {{
            background: var(--bg-primary);
            color: var(--text-primary);
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            margin: 0;
            padding: 32px 48px;
        }}
        .header {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 24px;
            padding-bottom: 16px;
            border-bottom: 1px solid var(--border-subtle);
        }}
        .title {{
            font-size: 20px;
            font-weight: 600;
            display: flex;
            align-items: center;
            gap: 10px;
        }}
        .download-card {{
            display: flex;
            align-items: center;
            gap: 16px;
            background: var(--bg-secondary);
            border: 1px solid var(--border-subtle);
            border-radius: 8px;
            padding: 14px 18px;
            margin-bottom: 12px;
        }}
        .card-info {{
            flex: 1;
            min-width: 0;
        }}
        .file-name {{
            font-size: 14px;
            font-weight: 500;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            margin-bottom: 4px;
        }}
        .file-meta {{
            font-size: 12px;
            color: var(--text-secondary);
            display: flex;
            align-items: center;
            gap: 8px;
            margin-bottom: 8px;
        }}
        .hash-tag {{
            display: inline-flex;
            align-items: center;
            gap: 4px;
            background: rgba(16, 185, 129, 0.1);
            color: #10b981;
            padding: 2px 6px;
            border-radius: 4px;
            font-family: Consolas, monospace;
            font-size: 11px;
        }}
        .progress-bar-bg {{
            height: 4px;
            background: #2a2a32;
            border-radius: 2px;
            overflow: hidden;
        }}
        .progress-bar-fill {{
            height: 100%;
            background: var(--accent);
            border-radius: 2px;
            transition: width 0.2s ease;
        }}
        .action-btn {{
            background: var(--bg-tertiary);
            border: 1px solid var(--border-subtle);
            color: var(--text-primary);
            padding: 6px 12px;
            border-radius: 6px;
            cursor: pointer;
            font-size: 12px;
            display: flex;
            align-items: center;
            gap: 6px;
        }}
        .action-btn:hover {{
            background: var(--border-subtle);
        }}
        .empty-state {{
            text-align: center;
            padding: 64px 0;
        }}
    </style>
</head>
<body>
    <div class="header">
        <div class="title">{download_icon} Downloads</div>
        <button class="action-btn" onclick="clearCompleted()">Clear Finished</button>
    </div>
    <div class="download-list">
        {rows}
    </div>
    <script>
        function openFile(path) {{
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage(JSON.stringify({{ action: "open_download_file", path: path }}));
            }}
        }}
        function clearCompleted() {{
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage(JSON.stringify({{ action: "clear_downloads" }}));
                window.location.reload();
            }}
        }}
    </script>
</body>
</html>"###
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

