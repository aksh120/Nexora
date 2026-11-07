use crate::storage::bookmarks::BookmarkEntry;

pub struct BookmarksUi;

impl BookmarksUi {
    pub fn render_html(bookmarks: &[BookmarkEntry]) -> String {
        let mut bookmark_cards = String::new();

        if bookmarks.is_empty() {
            bookmark_cards.push_str(r###"
                <div class="empty-state">
                    <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
                        <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
                    </svg>
                    <h3>No Bookmarks Saved Yet</h3>
                    <p>Click the Star icon on the toolbar or use the form above to bookmark your favorite sites.</p>
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
                    <div class="bookmark-card" data-id="{id}" data-search="{search_term}">
                        <div class="bm-icon">
                            <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="#00E5FF" stroke-width="2">
                                <circle cx="12" cy="12" r="10"/>
                                <line x1="2" y1="12" x2="22" y2="12"/>
                                <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
                            </svg>
                        </div>
                        <div class="bm-info">
                            <div class="bm-title" title="{escaped_title}">{escaped_title}</div>
                            <div class="bm-url" title="{escaped_url}">{escaped_domain}</div>
                        </div>
                        <div class="bm-actions">
                            <button class="btn-open" onclick="openBookmark('{escaped_url}')" title="Open in active tab">
                                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                                    <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
                                    <polyline points="15 3 21 3 21 9"/>
                                    <line x1="10" y1="14" x2="21" y2="3"/>
                                </svg>
                                Open
                            </button>
                            <button class="btn-delete" onclick="deleteBookmark({id})" title="Delete bookmark">
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
            max-width: 600px;
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

        .btn-add {{
            display: flex;
            align-items: center;
            gap: 8px;
            background: linear-gradient(135deg, #00E5FF 0%, #0077FE 100%);
            color: #0B0E14;
            border: none;
            border-radius: 8px;
            padding: 9px 16px;
            font-size: 0.85rem;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.2s;
            white-space: nowrap;
        }}

        .btn-add:hover {{
            opacity: 0.95;
            box-shadow: 0 0 12px rgba(0, 229, 255, 0.4);
            transform: translateY(-1px);
        }}

        /* Add Bookmark Bar */
        .add-bar {{
            display: none;
            background: var(--bg-sidebar);
            border-bottom: 1px solid var(--border-subtle);
            padding: 16px 36px;
            gap: 12px;
            align-items: center;
            animation: fadeIn 0.2s ease;
        }}

        .add-bar.active {{
            display: flex;
        }}

        .add-input {{
            flex: 1;
            background: var(--bg-input);
            border: 1px solid var(--border-subtle);
            border-radius: 6px;
            padding: 8px 12px;
            color: var(--text-main);
            font-size: 0.85rem;
            outline: none;
        }}

        .add-input:focus {{
            border-color: var(--border-focus);
        }}

        .btn-save {{
            background: var(--accent-cyan);
            color: #000;
            border: none;
            padding: 8px 16px;
            border-radius: 6px;
            font-weight: 600;
            font-size: 0.85rem;
            cursor: pointer;
        }}

        .btn-cancel {{
            background: transparent;
            color: var(--text-sub);
            border: 1px solid var(--border-subtle);
            padding: 8px 14px;
            border-radius: 6px;
            font-size: 0.85rem;
            cursor: pointer;
        }}

        .content {{
            flex: 1;
            padding: 32px 36px;
            max-width: 1400px;
            width: 100%;
            margin: 0 auto;
        }}

        .bookmarks-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
            gap: 16px;
        }}

        .bookmark-card {{
            background: var(--bg-card);
            border: 1px solid var(--border-subtle);
            border-radius: 10px;
            padding: 16px;
            display: flex;
            align-items: center;
            gap: 14px;
            transition: all 0.2s ease;
        }}

        .bookmark-card:hover {{
            background: var(--bg-card-hover);
            border-color: #333D50;
            transform: translateY(-2px);
            box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
        }}

        .bm-icon {{
            width: 40px;
            height: 40px;
            border-radius: 8px;
            background: rgba(0, 229, 255, 0.08);
            border: 1px solid rgba(0, 229, 255, 0.2);
            display: flex;
            align-items: center;
            justify-content: center;
            flex-shrink: 0;
        }}

        .bm-info {{
            flex: 1;
            min-width: 0;
        }}

        .bm-title {{
            font-size: 0.9rem;
            font-weight: 600;
            color: var(--text-main);
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            margin-bottom: 3px;
        }}

        .bm-url {{
            font-size: 0.75rem;
            color: var(--text-dim);
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }}

        .bm-actions {{
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
            grid-column: 1 / -1;
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

        @keyframes fadeIn {{
            from {{ opacity: 0; }}
            to {{ opacity: 1; }}
        }}
    </style>
</head>
<body>
    <div class="header">
        <div class="brand">
            <div class="brand-icon">
                <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                    <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
                </svg>
            </div>
            <h1>Bookmarks</h1>
            <span class="badge-count" id="count-badge">{total_count} Bookmarks</span>
        </div>
        <div class="controls">
            <div class="search-wrapper">
                <svg class="search-icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke-width="2">
                    <circle cx="11" cy="11" r="8"/>
                    <line x1="21" y1="21" x2="16.65" y2="16.65"/>
                </svg>
                <input type="text" class="search-input" id="search-input" placeholder="Search bookmarks by title or domain...">
            </div>
            <button class="btn-add" onclick="toggleAddBar()">
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5">
                    <line x1="12" y1="5" x2="12" y2="19"/>
                    <line x1="5" y1="12" x2="19" y2="12"/>
                </svg>
                Add Bookmark
            </button>
        </div>
    </div>

    <div class="add-bar" id="add-bar">
        <input type="text" class="add-input" id="add-title" placeholder="Bookmark Title (e.g., GitHub)">
        <input type="url" class="add-input" id="add-url" placeholder="URL (e.g., https://github.com)">
        <button class="btn-save" onclick="saveNewBookmark()">Save</button>
        <button class="btn-cancel" onclick="toggleAddBar()">Cancel</button>
    </div>

    <div class="content">
        <div class="bookmarks-grid" id="grid">
            {bookmark_cards}
        </div>
    </div>

    <div class="toast" id="toast">Bookmark Saved</div>

    <script>
        function toggleAddBar() {{
            const bar = document.getElementById('add-bar');
            bar.classList.toggle('active');
            if (bar.classList.contains('active')) {{
                document.getElementById('add-title').focus();
            }}
        }}

        function showToast(msg) {{
            const t = document.getElementById('toast');
            t.innerText = msg;
            t.style.display = 'block';
            setTimeout(() => {{ t.style.display = 'none'; }}, 2500);
        }}

        function openBookmark(url) {{
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage({{ action: 'navigate', url: url }});
            }} else {{
                window.location.href = url;
            }}
        }}

        function deleteBookmark(id) {{
            if (confirm('Delete this bookmark?')) {{
                if (window.chrome && window.chrome.webview) {{
                    window.chrome.webview.postMessage({{ action: 'delete_bookmark', id: id }});
                }}
                const card = document.querySelector(`.bookmark-card[data-id="${{id}}"]`);
                if (card) {{
                    card.style.opacity = '0';
                    card.style.transform = 'scale(0.9)';
                    setTimeout(() => {{
                        card.remove();
                        updateCount();
                    }}, 200);
                }}
                showToast('Bookmark deleted');
            }}
        }}

        function saveNewBookmark() {{
            const titleInput = document.getElementById('add-title');
            const urlInput = document.getElementById('add-url');
            const title = titleInput.value.trim();
            let url = urlInput.value.trim();

            if (!url) {{
                alert('Please enter a valid URL');
                return;
            }}
            if (!url.startsWith('http://') && !url.startsWith('https://')) {{
                url = 'https://' + url;
            }}

            const finalTitle = title || url;
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage({{ action: 'add_bookmark', title: finalTitle, url: url }});
            }}

            showToast('Bookmark added');
            toggleAddBar();
            titleInput.value = '';
            urlInput.value = '';

            setTimeout(() => {{
                if (window.chrome && window.chrome.webview) {{
                    window.chrome.webview.postMessage({{ action: 'navigate', url: 'nexora://bookmarks' }});
                }} else {{
                    window.location.reload();
                }}
            }}, 300);
        }}

        function updateCount() {{
            const cards = document.querySelectorAll('.bookmark-card');
            document.getElementById('count-badge').innerText = cards.length + ' Bookmarks';
        }}

        document.getElementById('search-input').addEventListener('input', function(e) {{
            const q = e.target.value.toLowerCase().trim();
            const cards = document.querySelectorAll('.bookmark-card');
            cards.forEach(card => {{
                const term = card.getAttribute('data-search') || '';
                if (term.includes(q)) {{
                    card.style.display = 'flex';
                }} else {{
                    card.style.display = 'none';
                }}
            }});
        }});
    </script>
</body>
</html>"###,
            total_count = bookmarks.len(),
            bookmark_cards = bookmark_cards,
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

