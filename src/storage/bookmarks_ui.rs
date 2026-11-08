use crate::storage::bookmarks::BookmarkEntry;

pub struct BookmarksUi;

impl BookmarksUi {
    pub fn render_html(bookmarks: &[BookmarkEntry]) -> String {
        let mut bookmark_cards = String::new();
        let total_count = bookmarks.len();

        if bookmarks.is_empty() {
            bookmark_cards.push_str(r###"
                <div class="empty-state">
                    <svg viewBox="0 0 24 24" width="40" height="40" fill="none" stroke="#70757A" stroke-width="1.75">
                        <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
                    </svg>
                    <h3>No Bookmarks Saved Yet</h3>
                    <p>Click the Star icon on the address bar or use the Add button above to bookmark your favorite sites.</p>
                </div>
            "###);
        } else {
            for bm in bookmarks {
                let escaped_title = escape_html(&bm.title);
                let escaped_url = escape_html(&bm.url);
                let domain = extract_domain(&bm.url);
                let escaped_domain = escape_html(&domain);

                bookmark_cards.push_str(&format!(
                    r###"
                    <div class="bookmark-row" data-id="{id}" data-search="{search_term}">
                        <div class="icon-col">
                            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="#9AA0A6" stroke-width="2">
                                <circle cx="12" cy="12" r="10"/>
                                <line x1="2" y1="12" x2="22" y2="12"/>
                                <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
                            </svg>
                        </div>
                        <div class="info-col">
                            <div class="title-text" onclick="openBookmark('{escaped_url}')" title="{escaped_title}">{escaped_title}</div>
                            <div class="url-text">{escaped_domain}</div>
                        </div>
                        <div class="actions-col">
                            <button class="btn-action" onclick="openBookmark('{escaped_url}')" title="Open in active tab">
                                Open
                            </button>
                            <button class="btn-icon" onclick="deleteBookmark({id})" title="Delete bookmark">
                                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                                    <polyline points="3 6 5 6 21 6"/>
                                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
                                </svg>
                            </button>
                        </div>
                    </div>
                    "###,
                    id = bm.id,
                    escaped_title = escaped_title,
                    escaped_url = escaped_url,
                    escaped_domain = escaped_domain,
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
    <title>Bookmarks - Nexora</title>
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
            --accent-hover: #F43F5E;
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

        .btn-primary {{
            background: var(--accent);
            border-color: var(--accent);
            color: #FFFFFF;
        }}

        .btn-primary:hover {{
            background: var(--accent-hover);
        }}

        .content-container {{
            flex: 1;
            padding: 28px 32px 60px 32px;
            max-width: 900px;
            width: 100%;
            margin: 0 auto;
        }}

        /* Inline Add Card */
        .add-card {{
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 8px;
            padding: 16px 20px;
            margin-bottom: 20px;
            display: none;
        }}

        .add-card.active {{
            display: block;
        }}

        .add-card h3 {{
            font-size: 14px;
            font-weight: 600;
            margin-bottom: 12px;
            color: var(--text-primary);
        }}

        .form-row {{
            display: flex;
            gap: 12px;
            margin-bottom: 12px;
        }}

        .form-row input {{
            flex: 1;
            background: var(--bg-base);
            border: 1px solid var(--border);
            border-radius: 5px;
            padding: 7px 10px;
            color: var(--text-primary);
            font-size: 13px;
            outline: none;
        }}

        .form-row input:focus {{
            border-color: var(--border-focus);
        }}

        .bookmarks-list {{
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 8px;
            overflow: hidden;
        }}

        .bookmark-row {{
            display: flex;
            align-items: center;
            padding: 11px 18px;
            border-bottom: 1px solid var(--border-subtle);
            transition: background 0.1s ease;
            gap: 14px;
        }}

        .bookmark-row:last-child {{
            border-bottom: none;
        }}

        .bookmark-row:hover {{
            background: var(--bg-hover);
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

        .url-text {{
            font-size: 12px;
            color: var(--text-secondary);
            margin-top: 1px;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
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
            <svg viewBox="0 0 100 100" width="22" height="22" fill="none">
                <defs>
                    <linearGradient id="prism-bm-logo" x1="0" y1="0" x2="100" y2="100">
                        <stop offset="0%" stop-color="#FF385C"/>
                        <stop offset="100%" stop-color="#E11D48"/>
                    </linearGradient>
                </defs>
                <polygon points="20,15 55,15 35,85 15,85" fill="url(#prism-bm-logo)"/>
                <polygon points="55,15 75,65 55,65 35,15" fill="#FDA4AF" opacity="0.85"/>
                <polygon points="85,85 50,85 70,15 90,15" fill="url(#prism-bm-logo)"/>
                <circle cx="50" cy="50" r="4" fill="#FFFFFF"/>
            </svg>
            <h1>Bookmarks</h1>
            <span class="count-pill">{total_count} Bookmarks</span>
        </div>

        <div class="search-box">
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
            <input type="text" id="bm-search" placeholder="Search bookmarks" oninput="filterBookmarks(this.value)" />
        </div>

        <div class="header-actions">
            <button class="btn btn-primary" onclick="toggleAddForm()">Add Bookmark</button>
        </div>
    </div>

    <div class="content-container">
        <!-- Add Bookmark Form -->
        <div class="add-card" id="add-form">
            <h3>Add New Bookmark</h3>
            <div class="form-row">
                <input type="text" id="new-title" placeholder="Name" />
                <input type="text" id="new-url" placeholder="URL (https://...)" />
            </div>
            <div style="display: flex; gap: 8px; justify-content: flex-end;">
                <button class="btn" onclick="toggleAddForm()">Cancel</button>
                <button class="btn btn-primary" onclick="submitBookmark()">Save</button>
            </div>
        </div>

        <div class="bookmarks-list" id="bookmarks-container">
            {bookmark_cards}
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

        function openBookmark(url) {{
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage(JSON.stringify({{
                    action: "navigate",
                    url: url
                }}));
            }} else {{
                window.location.href = url;
            }}
        }}

        function deleteBookmark(id) {{
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage(JSON.stringify({{
                    action: "delete_bookmark",
                    id: id
                }}));
            }}
            const row = document.querySelector(`.bookmark-row[data-id="${{id}}"]`);
            if (row) {{
                row.remove();
            }}
            showToast("Bookmark deleted");
        }}

        function toggleAddForm() {{
            const form = document.getElementById('add-form');
            form.classList.toggle('active');
            if (form.classList.contains('active')) {{
                document.getElementById('new-title').focus();
            }}
        }}

        function submitBookmark() {{
            const title = document.getElementById('new-title').value.trim();
            let url = document.getElementById('new-url').value.trim();

            if (!title || !url) {{
                alert("Please provide both a title and a valid URL.");
                return;
            }}

            if (!url.startsWith('http://') && !url.startsWith('https://') && !url.startsWith('nexora://')) {{
                url = 'https://' + url;
            }}

            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage(JSON.stringify({{
                    action: "add_bookmark",
                    title: title,
                    url: url
                }}));
            }}

            showToast("Bookmark added");
            setTimeout(() => {{
                window.location.reload();
            }}, 400);
        }}

        function filterBookmarks(query) {{
            const q = query.toLowerCase().trim();
            const rows = document.querySelectorAll('.bookmark-row');
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
            bookmark_cards = bookmark_cards,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_bookmarks_html() {
        let items = vec![
            BookmarkEntry {
                id: 1,
                parent_id: None,
                title: "YouTube".to_string(),
                url: "https://www.youtube.com".to_string(),
                is_folder: false,
                position: 0,
                created_at: 1700000000,
            },
            BookmarkEntry {
                id: 2,
                parent_id: None,
                title: "GitHub".to_string(),
                url: "https://github.com".to_string(),
                is_folder: false,
                position: 1,
                created_at: 1700000001,
            },
        ];

        let html = BookmarksUi::render_html(&items);
        assert!(html.contains("YouTube"));
        assert!(html.contains("GitHub"));
        assert!(html.contains("2 Bookmarks"));
        assert!(html.contains("openBookmark"));
        assert!(html.contains("deleteBookmark"));
    }
}
