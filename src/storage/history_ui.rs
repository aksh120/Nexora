use crate::storage::history::HistoryEntry;

pub struct HistoryUi;

impl HistoryUi {
    pub fn render_html(history_entries: &[HistoryEntry]) -> String {
        let logo_svg = crate::ui::branding::BrandIdentity::monogram_svg(22);
        let mut entry_cards = String::new();
        let total_count = history_entries.len();

        if history_entries.is_empty() {
            entry_cards.push_str(r###"
                <div class="empty-state">
                    <svg viewBox="0 0 24 24" width="40" height="40" fill="none" stroke="#70757A" stroke-width="1.75">
                        <circle cx="12" cy="12" r="10"/>
                        <polyline points="12 6 12 12 16 14"/>
                    </svg>
                    <h3>No Browsing History Found</h3>
                    <p>Webpages you visit will appear here chronologically for quick access and search.</p>
                </div>
            "###);
        } else {
            for entry in history_entries {
                let escaped_title = escape_html(&entry.title);
                let escaped_url = escape_html(&entry.url);
                let domain = extract_domain(&entry.url);
                let escaped_domain = escape_html(&domain);
                let time_str = format_timestamp(entry.visit_time);
                let plural = if entry.visit_count == 1 { "" } else { "s" };

                entry_cards.push_str(&format!(
                    r###"
                    <div class="history-row" data-id="{id}" data-search="{search_term}">
                        <div class="time-col">{time_str}</div>
                        <div class="icon-col">
                            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="#9AA0A6" stroke-width="2">
                                <circle cx="12" cy="12" r="10"/>
                                <line x1="2" y1="12" x2="22" y2="12"/>
                                <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
                            </svg>
                        </div>
                        <div class="info-col">
                            <div class="title-text" onclick="openHistoryUrl('{escaped_url}')" title="{escaped_title}">{escaped_title}</div>
                            <div class="meta-row">
                                <span class="domain-text">{escaped_domain}</span>
                                <span class="visit-count">{visits} visit{plural}</span>
                            </div>
                        </div>
                        <div class="actions-col">
                            <button class="btn-action" onclick="openHistoryUrl('{escaped_url}')" title="Open page">
                                Open
                            </button>
                            <button class="btn-icon" onclick="deleteHistoryEntry({id})" title="Delete entry">
                                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                                    <line x1="18" y1="6" x2="6" y2="18"/>
                                    <line x1="6" y1="6" x2="18" y2="18"/>
                                </svg>
                            </button>
                        </div>
                    </div>
                    "###,
                    id = entry.id,
                    escaped_title = escaped_title,
                    escaped_url = escaped_url,
                    escaped_domain = escaped_domain,
                    time_str = time_str,
                    visits = entry.visit_count,
                    plural = plural,
                    search_term = format!("{} {}", escaped_title.to_lowercase(), escaped_url.to_lowercase()),
                ));
            }
        }

        format!(
            r###"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Browsing History - Nexora</title>
    <style>
        :root {{
            --bg-base: #000000;
            --bg-header: #050304;
            --bg-card: #0C0708;
            --bg-hover: #160D0F;
            --border: #291518;
            --border-subtle: #190B0D;
            --border-focus: #E11D48;
            --text-primary: #F7F0F1;
            --text-secondary: #997F83;
            --text-muted: #665256;
            --accent: #E11D48;
            --danger: #E11D48;
            --font-stack: "Segoe UI", -apple-system, BlinkMacSystemFont, Roboto, sans-serif;
        }}

        @media (prefers-color-scheme: light) {{
            :root {{
                --bg-base: #F3F3F3;
                --bg-header: #EBEBEB;
                --bg-card: #FFFFFF;
                --bg-hover: #F9F9F9;
                --border: #E0E0E0;
                --border-subtle: #EEEEEE;
                --border-focus: #0078D4;
                --text-primary: #202124;
                --text-secondary: #5F6368;
                --text-muted: #80868B;
            }}
        }}

        * {{
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }}

        body {{
            font-family: var(--font-stack);
            background-color: var(--bg-base);
            color: var(--text-primary);
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            -webkit-font-smoothing: antialiased;
        }}

        /* Header Bar */
        .top-bar {{
            background: var(--bg-header);
            border-bottom: 1px solid var(--border);
            padding: 14px 32px;
            display: flex;
            align-items: center;
            justify-content: space-between;
            position: sticky;
            top: 0;
            z-index: 100;
        }}

        .brand-section {{
            display: flex;
            align-items: center;
            gap: 12px;
        }}

        .brand-section h1 {{
            font-size: 16px;
            font-weight: 600;
            color: var(--text-primary);
        }}

        .count-pill {{
            font-size: 11px;
            padding: 2px 7px;
            background: rgba(255, 255, 255, 0.08);
            border-radius: 4px;
            color: var(--text-secondary);
        }}

        .search-box {{
            position: relative;
            width: 360px;
        }}

        .search-box input {{
            width: 100%;
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 6px;
            padding: 7px 12px 7px 34px;
            color: var(--text-primary);
            font-size: 13px;
            outline: none;
            transition: border-color 0.15s ease;
        }}

        .search-box input:focus {{
            border-color: var(--border-focus);
        }}

        .search-box svg {{
            position: absolute;
            left: 10px;
            top: 50%;
            transform: translateY(-50%);
            stroke: var(--text-muted);
        }}

        .header-actions {{
            display: flex;
            align-items: center;
            gap: 8px;
        }}

        .btn {{
            background: #2D2D2D;
            border: 1px solid var(--border);
            color: var(--text-primary);
            padding: 6px 14px;
            border-radius: 5px;
            font-size: 12.5px;
            font-weight: 500;
            cursor: pointer;
            transition: background 0.1s ease;
        }}

        .btn:hover {{
            background: #383838;
        }}

        .btn-danger {{
            color: #FF8F94;
            border-color: rgba(209, 52, 56, 0.4);
            background: rgba(209, 52, 56, 0.1);
        }}

        .btn-danger:hover {{
            background: var(--danger);
            color: #FFFFFF;
        }}

        /* Content List */
        .content-container {{
            flex: 1;
            padding: 28px 32px 60px 32px;
            max-width: 900px;
            width: 100%;
            margin: 0 auto;
        }}

        .history-list {{
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 8px;
            overflow: hidden;
        }}

        .history-row {{
            display: flex;
            align-items: center;
            padding: 11px 18px;
            border-bottom: 1px solid var(--border-subtle);
            transition: background 0.1s ease;
            gap: 14px;
        }}

        .history-row:last-child {{
            border-bottom: none;
        }}

        .history-row:hover {{
            background: var(--bg-hover);
        }}

        .time-col {{
            font-size: 12px;
            color: var(--text-muted);
            width: 75px;
            flex-shrink: 0;
            white-space: nowrap;
        }}

        .icon-col {{
            flex-shrink: 0;
            display: flex;
            align-items: center;
        }}

        .info-col {{
            flex: 1;
            min-width: 0;
        }}

        .title-text {{
            font-size: 13.5px;
            font-weight: 500;
            color: var(--text-primary);
            cursor: pointer;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            transition: color 0.1s ease;
        }}

        .title-text:hover {{
            color: var(--accent);
            text-decoration: underline;
        }}

        .meta-row {{
            display: flex;
            align-items: center;
            gap: 8px;
            margin-top: 2px;
        }}

        .domain-text {{
            font-size: 12px;
            color: var(--text-secondary);
        }}

        .visit-count {{
            font-size: 11px;
            color: var(--text-muted);
            background: rgba(255, 255, 255, 0.04);
            padding: 1px 5px;
            border-radius: 3px;
        }}

        .actions-col {{
            display: flex;
            align-items: center;
            gap: 6px;
            flex-shrink: 0;
        }}

        .btn-action {{
            background: transparent;
            border: 1px solid var(--border);
            color: var(--text-secondary);
            padding: 4px 10px;
            border-radius: 4px;
            font-size: 12px;
            cursor: pointer;
            transition: background 0.1s ease, color 0.1s ease;
        }}

        .btn-action:hover {{
            background: rgba(255, 255, 255, 0.06);
            color: var(--text-primary);
        }}

        .btn-icon {{
            background: transparent;
            border: none;
            color: var(--text-muted);
            padding: 5px;
            border-radius: 4px;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: color 0.1s ease;
        }}

        .btn-icon:hover {{
            color: var(--danger);
        }}

        .empty-state {{
            padding: 60px 20px;
            text-align: center;
        }}

        .empty-state h3 {{
            font-size: 15px;
            font-weight: 500;
            color: var(--text-primary);
            margin: 12px 0 6px 0;
        }}

        .empty-state p {{
            font-size: 13px;
            color: var(--text-secondary);
        }}

        #toast {{
            position: fixed;
            bottom: 24px;
            right: 24px;
            background: #2B2B2B;
            border: 1px solid var(--border);
            color: #FFFFFF;
            padding: 10px 16px;
            border-radius: 6px;
            font-size: 13px;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
            display: none;
            z-index: 1000;
        }}
    </style>
</head>
<body>
    <div class="top-bar">
        <div class="brand-section">
            {logo_svg}
            <h1>History</h1>
            <span class="count-pill">{total_count} records</span>
        </div>

        <div class="search-box">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
            <input type="text" id="history-search" placeholder="Search history" oninput="filterHistory(this.value)" />
        </div>

        <div class="header-actions">
            <button class="btn btn-danger" onclick="clearAllHistory()">Clear All History</button>
        </div>
    </div>

    <div class="content-container">
        <div class="history-list" id="history-container">
            {entry_cards}
        </div>
    </div>

    <div id="toast"></div>

    <script>
        function showToast(msg) {{
            const toast = document.getElementById('toast');
            toast.textContent = msg;
            toast.style.display = 'block';
            setTimeout(() => {{
                toast.style.display = 'none';
            }}, 2000);
        }}

        function openHistoryUrl(url) {{
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage(JSON.stringify({{
                    action: "navigate",
                    url: url
                }}));
            }} else {{
                window.location.href = url;
            }}
        }}

        function deleteHistoryEntry(id) {{
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage(JSON.stringify({{
                    action: "delete_history_entry",
                    id: id
                }}));
            }}
            const row = document.querySelector(`.history-row[data-id="${{id}}"]`);
            if (row) {{
                row.remove();
            }}
            showToast("Entry removed");
        }}

        function clearAllHistory() {{
            if (confirm("Are you sure you want to clear your complete browsing history?")) {{
                if (window.chrome && window.chrome.webview) {{
                    window.chrome.webview.postMessage(JSON.stringify({{
                        action: "clear_history"
                    }}));
                }}
                const container = document.getElementById('history-container');
                container.innerHTML = `
                    <div class="empty-state">
                        <h3>Browsing history cleared</h3>
                        <p>Visited webpages will appear here as you browse.</p>
                    </div>
                `;
                showToast("History cleared");
            }}
        }}

        function filterHistory(query) {{
            const q = query.toLowerCase().trim();
            const rows = document.querySelectorAll('.history-row');
            rows.forEach(r => {{
                const text = r.getAttribute('data-search') || '';
                if (!q || text.includes(q)) {{
                    r.style.display = 'flex';
                }} else {{
                    r.style.display = 'none';
                }}
            }});
        }}
    </script>
</body>
</html>"###,
            total_count = total_count,
            logo_svg = logo_svg,
            entry_cards = entry_cards,
        )
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn extract_domain(url: &str) -> String {
    if let Some(pos) = url.find("://") {
        let rest = &url[pos + 3..];
        let domain = match rest.find('/') {
            Some(end) => &rest[..end],
            None => rest,
        };
        domain.to_string()
    } else {
        url.to_string()
    }
}

fn format_timestamp(ts: i64) -> String {
    if ts == 0 {
        return "Unknown".to_string();
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let diff = now - ts;
    if diff < 60 {
        "Just now".to_string()
    } else if diff < 3600 {
        format!("{} min ago", diff / 60)
    } else if diff < 86400 {
        format!("{} hr ago", diff / 3600)
    } else {
        format!("{} days ago", diff / 86400)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_history_html() {
        let entries = vec![
            HistoryEntry {
                id: 1,
                url: "https://www.youtube.com".to_string(),
                title: "YouTube".to_string(),
                visit_time: 1700000000,
                visit_count: 5,
                typed_count: 2,
            },
        ];

        let html = HistoryUi::render_html(&entries);
        assert!(html.contains("YouTube"));
        assert!(html.contains("5 visits"));
        assert!(html.contains("openHistoryUrl"));
        assert!(html.contains("deleteHistoryEntry"));
        assert!(html.contains("clearAllHistory"));
    }
}
