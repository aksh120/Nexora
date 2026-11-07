use crate::storage::history::HistoryEntry;

pub struct HistoryUi;

impl HistoryUi {
    pub fn render_html(history_entries: &[HistoryEntry]) -> String {
        let mut entry_cards = String::new();

        if history_entries.is_empty() {
            entry_cards.push_str(r###"
                <div class="empty-state">
                    <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
                        <circle cx="12" cy="12" r="10"/>
                        <polyline points="12 6 12 12 16 14"/>
                    </svg>
                    <h3>No Browsing History Found</h3>
                    <p>Webpages you visit will appear here chronologically for instant access and search.</p>
                </div>
            "###);
        } else {
            for entry in history_entries {
                let escaped_title = escape_html(&entry.title);
                let escaped_url = escape_html(&entry.url);
                let domain = extract_domain(&entry.url);
                let escaped_domain = escape_html(&domain);
                let time_str = format_timestamp(entry.visit_time);

                entry_cards.push_str(&format!(
                    r###"
                    <div class="history-item" data-id="{id}" data-search="{search_term}">
                        <div class="hi-icon">
                            <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="#00E5FF" stroke-width="2">
                                <circle cx="12" cy="12" r="10"/>
                                <polyline points="12 6 12 12 16 14"/>
                            </svg>
                        </div>
                        <div class="hi-info">
                            <div class="hi-title-row">
                                <span class="hi-title" title="{escaped_title}">{escaped_title}</span>
                                <span class="hi-badge">{visits} visit{plural}</span>
                            </div>
                            <div class="hi-meta">
                                <span class="hi-url" title="{escaped_url}">{escaped_domain}</span>
                                <span class="hi-sep">&bull;</span>
                                <span class="hi-time">{time_str}</span>
                            </div>
                        </div>
                        <div class="hi-actions">
                            <button class="btn-open" onclick="openHistoryUrl('{escaped_url}')" title="Open page in active tab">
                                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                                    <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
                                    <polyline points="15 3 21 3 21 9"/>
                                    <line x1="10" y1="14" x2="21" y2="3"/>
                                </svg>
                                Open
                            </button>
                            <button class="btn-delete" onclick="deleteHistoryEntry({id})" title="Delete entry">
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
                    plural = if entry.visit_count == 1 { "" } else { "s" },
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
            --bg-base: #0B0E14;
            --bg-sidebar: #10141D;
            --bg-card: #151923;
            --bg-card-hover: #1A202C;
            --bg-input: #121620;
            --border-subtle: #232936;
            --border-focus: #00E5FF;
            --text-main: #F0F4F8;
            --text-sub: #8B9BB0;
            --text-dim: #546478;
            --accent-cyan: #00E5FF;
            --accent-glow: rgba(0, 229, 255, 0.15);
            --danger-red: #FF4D4D;
        }}

        * {{
            box-sizing: border-box;
            margin: 0;
            padding: 0;
            font-family: 'Segoe UI Variable Display', 'Segoe UI', system-ui, -apple-system, sans-serif;
            -webkit-font-smoothing: antialiased;
        }}

        body {{
            background-color: var(--bg-base);
            color: var(--text-main);
            min-height: 100vh;
            display: flex;
            flex-direction: column;
        }}

        .header {{
            background-color: var(--bg-sidebar);
            border-bottom: 1px solid var(--border-subtle);
            padding: 24px 36px;
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 20px;
            position: sticky;
            top: 0;
            z-index: 100;
        }}

        .brand {{
            display: flex;
            align-items: center;
            gap: 12px;
        }}

        .brand-icon {{
            width: 32px;
            height: 32px;
            background: linear-gradient(135deg, #00E5FF 0%, #0077FE 100%);
            border-radius: 8px;
            display: flex;
            align-items: center;
            justify-content: center;
            color: #000;
        }}

        .brand h1 {{
            font-size: 1.3rem;
            font-weight: 600;
            letter-spacing: -0.3px;
        }}

        .badge-count {{
            font-size: 0.75rem;
            color: var(--text-dim);
            background: var(--bg-card);
            border: 1px solid var(--border-subtle);
            padding: 3px 10px;
            border-radius: 12px;
            font-weight: 500;
        }}

        .controls {{
            display: flex;
            align-items: center;
            gap: 12px;
            flex: 1;
            max-width: 640px;
        }}

        .search-wrapper {{
            position: relative;
            flex: 1;
        }}

        .search-icon {{
            position: absolute;
            left: 12px;
            top: 50%;
            transform: translateY(-50%);
            stroke: var(--text-dim);
            pointer-events: none;
        }}

        .search-input {{
            width: 100%;
            background: var(--bg-input);
            border: 1px solid var(--border-subtle);
            border-radius: 8px;
            padding: 9px 12px 9px 36px;
            color: var(--text-main);
            font-size: 0.85rem;
            outline: none;
            transition: all 0.2s ease;
        }}

        .search-input:focus {{
            border-color: var(--border-focus);
            box-shadow: 0 0 0 1px var(--border-focus), 0 0 12px var(--accent-glow);
        }}

        .btn-clear-all {{
            display: flex;
            align-items: center;
            gap: 8px;
            background: rgba(255, 77, 77, 0.08);
            border: 1px solid rgba(255, 77, 77, 0.25);
            color: var(--danger-red);
            border-radius: 8px;
            padding: 9px 16px;
            font-size: 0.85rem;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.2s;
            white-space: nowrap;
        }}

        .btn-clear-all:hover {{
            background: rgba(255, 77, 77, 0.2);
            border-color: var(--danger-red);
            transform: translateY(-1px);
        }}

        .content {{
            flex: 1;
            padding: 32px 36px;
            max-width: 1100px;
            width: 100%;
            margin: 0 auto;
        }}

        .history-list {{
            display: flex;
            flex-direction: column;
            gap: 10px;
        }}

        .history-item {{
            background: var(--bg-card);
            border: 1px solid var(--border-subtle);
            border-radius: 8px;
            padding: 14px 18px;
            display: flex;
            align-items: center;
            gap: 16px;
            transition: all 0.2s ease;
        }}

        .history-item:hover {{
            background: var(--bg-card-hover);
            border-color: #333D50;
        }}

        .hi-icon {{
            width: 36px;
            height: 36px;
            border-radius: 8px;
            background: rgba(0, 229, 255, 0.06);
            border: 1px solid rgba(0, 229, 255, 0.15);
            display: flex;
            align-items: center;
            justify-content: center;
            flex-shrink: 0;
        }}

        .hi-info {{
            flex: 1;
            min-width: 0;
        }}

        .hi-title-row {{
            display: flex;
            align-items: center;
            gap: 10px;
            margin-bottom: 4px;
        }}

        .hi-title {{
            font-size: 0.9rem;
            font-weight: 600;
            color: var(--text-main);
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }}

        .hi-badge {{
            font-size: 0.7rem;
            color: var(--accent-cyan);
            background: rgba(0, 229, 255, 0.08);
            border: 1px solid rgba(0, 229, 255, 0.2);
            padding: 1px 7px;
            border-radius: 10px;
            font-weight: 500;
            flex-shrink: 0;
        }}

        .hi-meta {{
            display: flex;
            align-items: center;
            gap: 8px;
            font-size: 0.75rem;
            color: var(--text-dim);
        }}

        .hi-url {{
            color: var(--text-sub);
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            max-width: 450px;
        }}

        .hi-sep {{
            color: var(--border-subtle);
        }}

        .hi-actions {{
            display: flex;
            align-items: center;
            gap: 6px;
        }}

        .btn-open {{
            display: flex;
            align-items: center;
            gap: 5px;
            background: rgba(255, 255, 255, 0.05);
            border: 1px solid var(--border-subtle);
            color: var(--text-sub);
            padding: 6px 10px;
            border-radius: 6px;
            font-size: 0.75rem;
            font-weight: 500;
            cursor: pointer;
            transition: all 0.2s;
        }}

        .btn-open:hover {{
            background: var(--accent-cyan);
            border-color: var(--accent-cyan);
            color: #000;
        }}

        .btn-delete {{
            background: transparent;
            border: 1px solid transparent;
            color: var(--text-dim);
            padding: 6px;
            border-radius: 6px;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: all 0.2s;
        }}

        .btn-delete:hover {{
            color: var(--danger-red);
            background: rgba(255, 77, 77, 0.1);
            border-color: rgba(255, 77, 77, 0.2);
        }}

        .empty-state {{
            text-align: center;
            padding: 60px 20px;
            color: var(--text-dim);
        }}

        .empty-state svg {{
            stroke: var(--text-dim);
            margin-bottom: 16px;
        }}

        .empty-state h3 {{
            color: var(--text-sub);
            font-size: 1.1rem;
            margin-bottom: 8px;
        }}

        .empty-state p {{
            font-size: 0.85rem;
            max-width: 400px;
            margin: 0 auto;
            line-height: 1.5;
        }}

        .toast {{
            position: fixed;
            bottom: 24px;
            right: 24px;
            background: var(--bg-card);
            border: 1px solid var(--border-focus);
            padding: 10px 18px;
            border-radius: 8px;
            box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4), 0 0 12px var(--accent-glow);
            color: var(--text-main);
            font-size: 0.85rem;
            font-weight: 500;
            display: none;
            animation: toastPop 0.3s ease;
            z-index: 200;
        }}

        @keyframes toastPop {{
            from {{ transform: translateY(12px); opacity: 0; }}
            to {{ transform: translateY(0); opacity: 1; }}
        }}
    </style>
</head>
<body>
    <div class="header">
        <div class="brand">
            <div class="brand-icon">
                <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2.5">
                    <circle cx="12" cy="12" r="10"/>
                    <polyline points="12 6 12 12 16 14"/>
                </svg>
            </div>
            <h1>Browsing History</h1>
            <span class="badge-count" id="count-badge">{total_count} Visits</span>
        </div>
        <div class="controls">
            <div class="search-wrapper">
                <svg class="search-icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke-width="2">
                    <circle cx="11" cy="11" r="8"/>
                    <line x1="21" y1="21" x2="16.65" y2="16.65"/>
                </svg>
                <input type="text" class="search-input" id="search-input" placeholder="Search browsing history by title or domain...">
            </div>
            <button class="btn-clear-all" onclick="clearAllHistory()">
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="3 6 5 6 21 6"/>
                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
                </svg>
                Clear History
            </button>
        </div>
    </div>

    <div class="content">
        <div class="history-list" id="list">
            {entry_cards}
        </div>
    </div>

    <div class="toast" id="toast">History Updated</div>

    <script>
        function showToast(msg) {{
            const t = document.getElementById('toast');
            t.innerText = msg;
            t.style.display = 'block';
            setTimeout(() => {{ t.style.display = 'none'; }}, 2500);
        }}

        function openHistoryUrl(url) {{
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage({{ action: 'navigate', url: url }});
            }} else {{
                window.location.href = url;
            }}
        }}

        function deleteHistoryEntry(id) {{
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage({{ action: 'delete_history_entry', id: id }});
            }}
            const card = document.querySelector(`.history-item[data-id="${{id}}"]`);
            if (card) {{
                card.style.opacity = '0';
                card.style.transform = 'translateX(20px)';
                setTimeout(() => {{
                    card.remove();
                    updateCount();
                }}, 200);
            }}
            showToast('History entry removed');
        }}

        function clearAllHistory() {{
            if (confirm('Are you sure you want to clear your entire browsing history?')) {{
                if (window.chrome && window.chrome.webview) {{
                    window.chrome.webview.postMessage({{ action: 'clear_history' }});
                }}
                document.getElementById('list').innerHTML = `
                    <div class="empty-state">
                        <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
                            <circle cx="12" cy="12" r="10"/>
                            <polyline points="12 6 12 12 16 14"/>
                        </svg>
                        <h3>Browsing History Cleared</h3>
                        <p>All recorded history entries have been deleted.</p>
                    </div>
                `;
                document.getElementById('count-badge').innerText = '0 Visits';
                showToast('Browsing history cleared');
            }}
        }}

        function updateCount() {{
            const items = document.querySelectorAll('.history-item');
            document.getElementById('count-badge').innerText = items.length + ' Visits';
        }}

        document.getElementById('search-input').addEventListener('input', function(e) {{
            const q = e.target.value.toLowerCase().trim();
            const items = document.querySelectorAll('.history-item');
            items.forEach(item => {{
                const term = item.getAttribute('data-search') || '';
                if (term.includes(q)) {{
                    item.style.display = 'flex';
                }} else {{
                    item.style.display = 'none';
                }}
            }});
        }});
    </script>
</body>
</html>"###,
            total_count = history_entries.len(),
            entry_cards = entry_cards,
        )
    }
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn extract_domain(url: &str) -> String {
    if let Some(stripped) = url.strip_prefix("https://") {
        stripped.split('/').next().unwrap_or(stripped).to_string()
    } else if let Some(stripped) = url.strip_prefix("http://") {
        stripped.split('/').next().unwrap_or(stripped).to_string()
    } else {
        url.to_string()
    }
}

fn format_timestamp(ts: i64) -> String {
    if ts <= 0 {
        return "Recent".to_string();
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

