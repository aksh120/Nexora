use crate::app::config::NexoraConfig;
use crate::storage::bookmarks::BookmarkEntry;
use crate::ui::design_system::DesignTokens;
use crate::ui::icons::LucideIcon;

pub struct HomeUi;

impl HomeUi {
    /// Renders a modern, refined start page for `nexora://home` and `nexora://newtab`.
    /// Follows clean desktop browser aesthetics with zero futuristic AI clichés.
    pub fn render_html(config: &NexoraConfig, bookmarks: &[BookmarkEntry]) -> String {
        let version = env!("CARGO_PKG_VERSION");
        let default_engine = &config.default_search_engine;

        let search_icon_svg = LucideIcon::Search.to_svg(18, 2.0, "currentColor");
        let plus_icon_svg = LucideIcon::Plus.to_svg(14, 2.0, "currentColor");
        let bookmark_icon_svg = LucideIcon::Bookmark.to_svg(14, 2.0, "currentColor");
        let settings_icon_svg = LucideIcon::Settings.to_svg(14, 2.0, "currentColor");
        let history_icon_svg = LucideIcon::History.to_svg(14, 2.0, "currentColor");
        let diagnostics_icon_svg = LucideIcon::Cpu.to_svg(14, 2.0, "currentColor");
        let arrow_right_svg = LucideIcon::ArrowRight.to_svg(16, 2.0, "#FFFFFF");
        let shared_css = DesignTokens::shared_css(true);

        // Pre-render up to 8 user bookmarks
        let mut bookmarks_html = String::new();
        if bookmarks.is_empty() {
            bookmarks_html.push_str(r#"
                <div class="empty-state">
                    <div class="empty-icon">
                        <svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16z"/></svg>
                    </div>
                    <div class="empty-text">No bookmarks saved yet</div>
                    <div class="empty-subtext">Click the bookmark icon in the toolbar or press Ctrl+D to bookmark any site.</div>
                </div>
            "#);
        } else {
            bookmarks_html.push_str("<div class=\"bookmarks-grid\">");
            for bm in bookmarks.iter().take(8) {
                let esc_title = html_escape(&bm.title);
                let esc_url = html_escape(&bm.url);
                let domain = extract_domain(&bm.url);
                let esc_domain = html_escape(&domain);
                let initial = domain.chars().next().unwrap_or('B').to_ascii_uppercase();

                bookmarks_html.push_str(&format!(
                    r###"
                    <div class="bookmark-card" onclick="openUrl('{esc_url}')" title="{esc_title}">
                        <div class="site-avatar">{initial}</div>
                        <div class="site-details">
                            <div class="site-title">{esc_title}</div>
                            <div class="site-domain">{esc_domain}</div>
                        </div>
                    </div>
                    "###,
                    esc_url = esc_url,
                    esc_title = esc_title,
                    esc_domain = esc_domain,
                    initial = initial,
                ));
            }
            bookmarks_html.push_str("</div>");
        }

        format!(
            r###"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Nexora Home</title>
    <style>
        {shared_css}

        body {{
            display: flex;
            flex-direction: column;
            align-items: center;
            min-height: 100vh;
            padding: 48px 24px 64px 24px;
        }}

        .home-container {{
            width: 100%;
            max-width: 780px;
            display: flex;
            flex-direction: column;
            gap: 36px;
        }}

        /* Welcome Branding Header */
        .header-brand {{
            display: flex;
            flex-direction: column;
            align-items: center;
            text-align: center;
            gap: 12px;
            margin-top: 24px;
        }}

        .brand-logo-wrap {{
            display: flex;
            align-items: center;
            justify-content: center;
            width: 48px;
            height: 48px;
            background: var(--bg-surface);
            border: 1px solid var(--border);
            border-radius: var(--radius-lg);
        }}

        .greeting-title {{
            font-size: 24px;
            font-weight: 600;
            color: var(--text-primary);
            letter-spacing: -0.3px;
        }}

        .greeting-date {{
            font-size: 13px;
            color: var(--text-secondary);
        }}

        /* Main Search Field */
        .search-wrapper {{
            display: flex;
            flex-direction: column;
            gap: 8px;
        }}

        .search-box {{
            display: flex;
            align-items: center;
            background: var(--bg-surface);
            border: 1px solid var(--border);
            border-radius: var(--radius-lg);
            padding: 4px 6px 4px 16px;
            transition: all 0.15s ease;
            box-shadow: 0 4px 20px rgba(0, 0, 0, 0.2);
        }}

        .search-box:focus-within {{
            border-color: var(--accent);
            box-shadow: 0 4px 24px rgba(225, 29, 72, 0.12);
        }}

        .search-icon {{
            display: flex;
            align-items: center;
            color: var(--text-muted);
            margin-right: 12px;
            flex-shrink: 0;
        }}

        .search-input {{
            flex: 1;
            background: transparent;
            border: none;
            outline: none;
            font-size: 15px;
            font-family: var(--font-sans);
            color: var(--text-primary);
            padding: 10px 0;
        }}

        .search-input::placeholder {{
            color: var(--text-muted);
        }}

        .search-actions {{
            display: flex;
            align-items: center;
            gap: 8px;
        }}

        .engine-select {{
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: var(--radius-sm);
            color: var(--text-secondary);
            font-size: 12px;
            padding: 6px 24px 6px 10px;
            cursor: pointer;
            outline: none;
            appearance: none;
            background-image: url("data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%23A1A1AA' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3e%3cpolyline points='6 9 12 15 18 9'%3e%3c/polyline%3e%3c/svg%3e");
            background-repeat: no-repeat;
            background-position: right 6px center;
            background-size: 12px;
        }}

        .search-submit-btn {{
            display: flex;
            align-items: center;
            justify-content: center;
            width: 36px;
            height: 36px;
            background: var(--accent);
            border: none;
            border-radius: var(--radius-sm);
            cursor: pointer;
            transition: background 0.12s ease;
            flex-shrink: 0;
        }}

        .search-submit-btn:hover {{
            background: var(--accent-hover);
        }}

        /* Section Layout */
        .section-header {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 12px;
        }}

        .section-heading {{
            font-size: 14px;
            font-weight: 600;
            color: var(--text-primary);
            display: flex;
            align-items: center;
            gap: 8px;
        }}

        .section-action {{
            font-size: 12px;
            color: var(--text-secondary);
            cursor: pointer;
            display: flex;
            align-items: center;
            gap: 4px;
            background: transparent;
            border: none;
            padding: 4px 8px;
            border-radius: var(--radius-sm);
            transition: all 0.12s ease;
        }}

        .section-action:hover {{
            color: var(--text-primary);
            background: var(--hover-bg);
        }}

        /* Shortcuts Grid */
        .shortcuts-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(110px, 1fr));
            gap: 12px;
        }}

        .shortcut-tile {{
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            gap: 10px;
            padding: 16px 10px;
            background: var(--bg-surface);
            border: 1px solid var(--border);
            border-radius: var(--radius-md);
            cursor: pointer;
            transition: all 0.12s ease;
            position: relative;
            text-decoration: none;
        }}

        .shortcut-tile:hover {{
            background: var(--bg-card);
            border-color: rgba(255, 255, 255, 0.15);
            transform: translateY(-2px);
        }}

        .shortcut-icon-box {{
            width: 38px;
            height: 38px;
            border-radius: var(--radius-sm);
            background: var(--bg-card);
            border: 1px solid var(--border-subtle);
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 16px;
            font-weight: 600;
            color: var(--text-primary);
        }}

        .shortcut-name {{
            font-size: 12px;
            font-weight: 500;
            color: var(--text-primary);
            text-align: center;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            width: 100%;
        }}

        .shortcut-delete-btn {{
            position: absolute;
            top: 4px;
            right: 4px;
            width: 18px;
            height: 18px;
            border-radius: 50%;
            background: rgba(0, 0, 0, 0.6);
            color: var(--text-muted);
            border: none;
            display: none;
            align-items: center;
            justify-content: center;
            cursor: pointer;
            font-size: 11px;
            line-height: 1;
        }}

        .shortcut-tile:hover .shortcut-delete-btn {{
            display: flex;
        }}

        .shortcut-delete-btn:hover {{
            background: var(--accent);
            color: #FFFFFF;
        }}

        /* Bookmarks Cards */
        .bookmarks-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(230px, 1fr));
            gap: 12px;
        }}

        .bookmark-card {{
            display: flex;
            align-items: center;
            gap: 12px;
            padding: 12px 14px;
            background: var(--bg-surface);
            border: 1px solid var(--border);
            border-radius: var(--radius-md);
            cursor: pointer;
            transition: all 0.12s ease;
        }}

        .bookmark-card:hover {{
            background: var(--bg-card);
            border-color: rgba(255, 255, 255, 0.15);
            transform: translateY(-1px);
        }}

        .site-avatar {{
            width: 32px;
            height: 32px;
            border-radius: var(--radius-sm);
            background: var(--bg-card);
            border: 1px solid var(--border-subtle);
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 13px;
            font-weight: 600;
            color: var(--accent);
            flex-shrink: 0;
        }}

        .site-details {{
            display: flex;
            flex-direction: column;
            overflow: hidden;
        }}

        .site-title {{
            font-size: 13px;
            font-weight: 500;
            color: var(--text-primary);
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }}

        .site-domain {{
            font-size: 11px;
            color: var(--text-muted);
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }}

        .empty-state {{
            display: flex;
            flex-direction: column;
            align-items: center;
            text-align: center;
            padding: 24px;
            background: var(--bg-surface);
            border: 1px dashed var(--border);
            border-radius: var(--radius-md);
            gap: 6px;
        }}

        .empty-icon {{
            color: var(--text-muted);
            margin-bottom: 2px;
        }}

        .empty-text {{
            font-size: 13px;
            font-weight: 500;
            color: var(--text-secondary);
        }}

        .empty-subtext {{
            font-size: 12px;
            color: var(--text-muted);
        }}

        /* Clean Footer */
        .home-footer {{
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 16px;
            margin-top: 20px;
            padding-top: 20px;
            border-top: 1px solid var(--border-subtle);
        }}

        .footer-link {{
            display: inline-flex;
            align-items: center;
            gap: 6px;
            font-size: 12px;
            color: var(--text-secondary);
            text-decoration: none;
            cursor: pointer;
            transition: color 0.12s ease;
        }}

        .footer-link:hover {{
            color: var(--text-primary);
        }}

        .footer-dot {{
            width: 3px;
            height: 3px;
            border-radius: 50%;
            background: var(--border);
        }}

        /* Modal Dialog */
        .modal-overlay {{
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.65);
            backdrop-filter: blur(4px);
            display: none;
            align-items: center;
            justify-content: center;
            z-index: 100;
        }}

        .modal-box {{
            width: 100%;
            max-width: 400px;
            background: var(--bg-surface);
            border: 1px solid var(--border);
            border-radius: var(--radius-lg);
            padding: 24px;
            display: flex;
            flex-direction: column;
            gap: 16px;
            box-shadow: 0 16px 40px rgba(0, 0, 0, 0.4);
        }}

        .modal-header {{
            display: flex;
            align-items: center;
            justify-content: space-between;
        }}

        .modal-title {{
            font-size: 15px;
            font-weight: 600;
            color: var(--text-primary);
        }}

        .modal-footer {{
            display: flex;
            justify-content: flex-end;
            gap: 10px;
            margin-top: 8px;
        }}
    </style>
</head>
<body>
    <div class="home-container">
        <!-- Brand & Greeting Header -->
        <div class="header-brand">
            <div class="brand-logo-wrap">
                <svg viewBox="0 0 100 100" width="28" height="28" fill="none">
                    <defs>
                        <linearGradient id="prism-gradient" x1="0" y1="0" x2="100" y2="100">
                            <stop offset="0%" stop-color="#FF385C"/>
                            <stop offset="100%" stop-color="#E11D48"/>
                        </linearGradient>
                    </defs>
                    <polygon points="20,15 55,15 35,85 15,85" fill="url(#prism-gradient)"/>
                    <polygon points="55,15 75,65 55,65 35,15" fill="#FDA4AF" opacity="0.85"/>
                    <polygon points="85,85 50,85 70,15 90,15" fill="url(#prism-gradient)"/>
                    <circle cx="50" cy="50" r="4" fill="#FFFFFF"/>
                </svg>
            </div>
            <h1 class="greeting-title" id="greeting-text">Welcome to Nexora</h1>
            <div class="greeting-date" id="greeting-date"></div>
        </div>

        <!-- Main Search Field -->
        <div class="search-wrapper">
            <div class="search-box">
                <div class="search-icon">
                    {search_icon_svg}
                </div>
                <input
                    type="text"
                    id="search-input"
                    class="search-input"
                    placeholder="Search the web or type a URL..."
                    autocomplete="off"
                    autofocus
                />
                <div class="search-actions">
                    <select id="engine-select" class="engine-select">
                        <option value="Google" {google_sel}>Google</option>
                        <option value="DuckDuckGo" {ddg_sel}>DuckDuckGo</option>
                        <option value="Bing" {bing_sel}>Bing</option>
                    </select>
                    <button class="search-submit-btn" id="search-btn" title="Search">
                        {arrow_right_svg}
                    </button>
                </div>
            </div>
        </div>

        <!-- Quick Access Shortcuts -->
        <div>
            <div class="section-header">
                <div class="section-heading">Quick Access</div>
                <button class="section-action" onclick="showAddShortcutModal()">
                    {plus_icon_svg} Add Shortcut
                </button>
            </div>
            <div class="shortcuts-grid" id="shortcuts-grid"></div>
        </div>

        <!-- Saved Bookmarks -->
        <div>
            <div class="section-header">
                <div class="section-heading">
                    {bookmark_icon_svg} Bookmarks
                </div>
                <button class="section-action" onclick="openUrl('nexora://bookmarks')">
                    Manage
                </button>
            </div>
            {bookmarks_html}
        </div>

        <!-- Clean Footer Navigation -->
        <div class="home-footer">
            <a class="footer-link" onclick="openUrl('nexora://settings')">
                {settings_icon_svg} Settings
            </a>
            <div class="footer-dot"></div>
            <a class="footer-link" onclick="openUrl('nexora://history')">
                {history_icon_svg} History
            </a>
            <div class="footer-dot"></div>
            <a class="footer-link" onclick="openUrl('nexora://diagnostics')">
                {diagnostics_icon_svg} Diagnostics
            </a>
            <div class="footer-dot"></div>
            <span style="font-size: 11px; color: var(--text-muted);">v{version}</span>
        </div>
    </div>

    <!-- Add Shortcut Modal -->
    <div class="modal-overlay" id="shortcut-modal">
        <div class="modal-box">
            <div class="modal-header">
                <div class="modal-title">Add Quick Shortcut</div>
                <button class="btn btn-ghost btn-sm" onclick="hideAddShortcutModal()">✕</button>
            </div>
            <div style="display:flex; flex-direction:column; gap:12px;">
                <div>
                    <label style="font-size:12px; color:var(--text-secondary); display:block; margin-bottom:4px;">Site Name</label>
                    <input type="text" id="modal-name" class="input-text" placeholder="e.g. YouTube" />
                </div>
                <div>
                    <label style="font-size:12px; color:var(--text-secondary); display:block; margin-bottom:4px;">Web Address (URL)</label>
                    <input type="text" id="modal-url" class="input-text" placeholder="https://..." />
                </div>
            </div>
            <div class="modal-footer">
                <button class="btn btn-ghost" onclick="hideAddShortcutModal()">Cancel</button>
                <button class="btn btn-primary" onclick="saveNewShortcut()">Add</button>
            </div>
        </div>
    </div>

    <script>
        function openUrl(url) {{
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage(JSON.stringify({{
                    action: "navigate",
                    url: url
                }}));
            }} else {{
                window.location.href = url;
            }}
        }}

        // Dynamic Time Greeting
        function updateGreeting() {{
            const now = new Date();
            const hour = now.getHours();
            let greeting = "Good evening";
            if (hour < 12) {{
                greeting = "Good morning";
            }} else if (hour < 18) {{
                greeting = "Good afternoon";
            }}
            document.getElementById('greeting-text').textContent = greeting;

            const options = {{ weekday: 'long', month: 'long', day: 'numeric' }};
            document.getElementById('greeting-date').textContent = now.toLocaleDateString(undefined, options);
        }}
        updateGreeting();

        // Search Execution
        function performSearch() {{
            const input = document.getElementById('search-input');
            const query = input.value.trim();
            if (!query) return;

            // Direct URL detection
            if (query.startsWith('http://') || query.startsWith('https://') || query.startsWith('nexora://')) {{
                openUrl(query);
                return;
            }}
            if (query.includes('.') && !query.includes(' ') && query.length >= 4) {{
                openUrl('https://' + query);
                return;
            }}
            if (query === 'localhost' || query.startsWith('localhost:')) {{
                openUrl('http://' + query);
                return;
            }}

            const engine = document.getElementById('engine-select').value;
            let searchUrl = '';
            if (engine === 'DuckDuckGo') {{
                searchUrl = 'https://duckduckgo.com/?q=' + encodeURIComponent(query);
            }} else if (engine === 'Bing') {{
                searchUrl = 'https://www.bing.com/search?q=' + encodeURIComponent(query);
            }} else {{
                searchUrl = 'https://www.google.com/search?q=' + encodeURIComponent(query);
            }}
            openUrl(searchUrl);
        }}

        document.getElementById('search-btn').addEventListener('click', performSearch);
        document.getElementById('search-input').addEventListener('keydown', (e) => {{
            if (e.key === 'Enter') {{
                performSearch();
            }}
        }});

        // Shortcuts Management (Stored in localStorage with clean pre-seeds)
        const DEFAULT_SHORTCUTS = [
            {{ name: "YouTube", url: "https://www.youtube.com", initial: "YT" }},
            {{ name: "Twitch", url: "https://www.twitch.tv", initial: "TW" }},
            {{ name: "GitHub", url: "https://github.com", initial: "GH" }},
            {{ name: "Reddit", url: "https://www.reddit.com", initial: "RD" }},
            {{ name: "Wikipedia", url: "https://www.wikipedia.org", initial: "W" }},
            {{ name: "X", url: "https://x.com", initial: "X" }},
        ];

        function getShortcuts() {{
            try {{
                const saved = localStorage.getItem('nexora_shortcuts');
                return saved ? JSON.parse(saved) : DEFAULT_SHORTCUTS;
            }} catch (e) {{
                return DEFAULT_SHORTCUTS;
            }}
        }}

        function renderShortcuts() {{
            const grid = document.getElementById('shortcuts-grid');
            grid.innerHTML = '';
            const list = getShortcuts();
            list.forEach((s, idx) => {{
                const tile = document.createElement('div');
                tile.className = 'shortcut-tile';
                tile.title = s.name + ' (' + s.url + ')';
                tile.onclick = (e) => {{
                    if (!e.target.classList.contains('shortcut-delete-btn')) {{
                        openUrl(s.url);
                    }}
                }};

                tile.innerHTML = `
                    <button class="shortcut-delete-btn" title="Remove shortcut" onclick="removeShortcut(${{idx}})">✕</button>
                    <div class="shortcut-icon-box">${{s.initial || s.name[0].toUpperCase()}}</div>
                    <div class="shortcut-name">${{s.name}}</div>
                `;
                grid.appendChild(tile);
            }});
        }}

        function removeShortcut(idx) {{
            const list = getShortcuts();
            list.splice(idx, 1);
            localStorage.setItem('nexora_shortcuts', JSON.stringify(list));
            renderShortcuts();
        }}

        function showAddShortcutModal() {{
            document.getElementById('shortcut-modal').style.display = 'flex';
            document.getElementById('modal-name').focus();
        }}

        function hideAddShortcutModal() {{
            document.getElementById('shortcut-modal').style.display = 'none';
            document.getElementById('modal-name').value = '';
            document.getElementById('modal-url').value = '';
        }}

        function saveNewShortcut() {{
            const name = document.getElementById('modal-name').value.trim();
            let url = document.getElementById('modal-url').value.trim();
            if (!name || !url) return;
            if (!url.startsWith('http://') && !url.startsWith('https://')) {{
                url = 'https://' + url;
            }}
            const initial = name.slice(0, 2).toUpperCase();
            const list = getShortcuts();
            list.push({{ name, url, initial }});
            localStorage.setItem('nexora_shortcuts', JSON.stringify(list));
            hideAddShortcutModal();
            renderShortcuts();
        }}

        renderShortcuts();
    </script>
</body>
</html>
            "###,
            shared_css = shared_css,
            version = version,
            search_icon_svg = search_icon_svg,
            plus_icon_svg = plus_icon_svg,
            bookmark_icon_svg = bookmark_icon_svg,
            settings_icon_svg = settings_icon_svg,
            history_icon_svg = history_icon_svg,
            diagnostics_icon_svg = diagnostics_icon_svg,
            arrow_right_svg = arrow_right_svg,
            google_sel = if default_engine == "Google" { "selected" } else { "" },
            ddg_sel = if default_engine == "DuckDuckGo" { "selected" } else { "" },
            bing_sel = if default_engine == "Bing" { "selected" } else { "" },
            bookmarks_html = bookmarks_html,
        )
    }
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn extract_domain(url: &str) -> String {
    let clean = url.trim();
    if let Some(idx) = clean.find("://") {
        let after_scheme = &clean[idx + 3..];
        let host = after_scheme.split('/').next().unwrap_or(after_scheme);
        host.trim_start_matches("www.").to_string()
    } else {
        clean.split('/').next().unwrap_or(clean).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_home_html() {
        let config = NexoraConfig::default();
        let bookmarks = Vec::new();
        let html = HomeUi::render_html(&config, &bookmarks);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Nexora Home"));
        assert!(html.contains("Search the web or type a URL"));
        assert!(html.contains("Quick Access"));
        assert!(!html.contains("Commander"), "AI clichés like 'Commander' must be removed");
        assert!(!html.contains("D3D11 NVDEC Status"), "Fake status badges must be removed");
    }
}
