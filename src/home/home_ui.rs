use crate::app::config::NexoraConfig;
use crate::storage::bookmarks::BookmarkEntry;

pub struct HomeUi;

impl HomeUi {
    /// Renders an ultra-modern, krazy looking, stealth cockpit AMOLED homepage for `nexora://home` and `nexora://newtab`
    pub fn render_html(config: &NexoraConfig, bookmarks: &[BookmarkEntry]) -> String {
        let version = env!("CARGO_PKG_VERSION");
        let default_engine = &config.default_search_engine;

        // Render user bookmarks cards (up to 8 most recent)
        let mut user_bookmarks_html = String::new();
        for bm in bookmarks.iter().take(8) {
            let esc_title = html_escape(&bm.title);
            let esc_url = html_escape(&bm.url);
            let domain = extract_domain(&bm.url);
            let esc_domain = html_escape(&domain);
            let initial = domain.chars().next().unwrap_or('★').to_ascii_uppercase();

            user_bookmarks_html.push_str(&format!(
                r###"
                <div class="dial-card" onclick="openUrl('{esc_url}')" title="{esc_title}">
                    <div class="dial-icon-box custom-initial">
                        <span>{initial}</span>
                    </div>
                    <div class="dial-info">
                        <div class="dial-title">{esc_title}</div>
                        <div class="dial-domain">{esc_domain}</div>
                    </div>
                </div>
                "###,
                esc_url = esc_url,
                esc_title = esc_title,
                esc_domain = esc_domain,
                initial = initial,
            ));
        }

        format!(
            r###"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Nexora Home</title>
    <style>
        :root {{
            --bg-base: #000000;
            --bg-cockpit: #050304;
            --bg-card: #0C0708;
            --bg-card-hover: #140B0E;
            --border: #291518;
            --border-hover: #E11D48;
            --border-subtle: #190B0D;
            --text-primary: #F7F0F1;
            --text-secondary: #997F83;
            --text-muted: #665256;
            --accent: #E11D48;
            --accent-glow: rgba(225, 29, 72, 0.35);
            --accent-hover: #F43F5E;
            --accent-dim: #9F1239;
            --hud-emerald: #10B981;
            --hud-cyan: #06B6D4;
            --hud-amber: #F59E0B;
            --font-main: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", sans-serif;
            --font-mono: "JetBrains Mono", Consolas, "SF Mono", "Courier New", monospace;
        }}

        * {{
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }}

        body {{
            background-color: var(--bg-base);
            background-image: 
                radial-gradient(circle at 50% 0%, rgba(225, 29, 72, 0.09) 0%, transparent 65%),
                radial-gradient(rgba(225, 29, 72, 0.08) 1px, transparent 1px);
            background-size: 100% 100%, 32px 32px;
            color: var(--text-primary);
            font-family: var(--font-main);
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            overflow-x: hidden;
            user-select: none;
            -webkit-font-smoothing: antialiased;
        }}

        /* TOP COCKPIT HUD */
        .cockpit-hud {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 16px 36px;
            border-bottom: 1px solid var(--border-subtle);
            background: linear-gradient(180deg, rgba(12, 7, 8, 0.9) 0%, rgba(0, 0, 0, 0.4) 100%);
            backdrop-filter: blur(12px);
            position: sticky;
            top: 0;
            z-index: 50;
        }}

        .brand-cluster {{
            display: flex;
            align-items: center;
            gap: 14px;
        }}

        .brand-logo {{
            width: 30px;
            height: 30px;
        }}

        .brand-title {{
            font-size: 16px;
            font-weight: 800;
            letter-spacing: 2.5px;
            color: var(--text-primary);
            display: flex;
            align-items: center;
            gap: 8px;
        }}

        .brand-title span {{
            color: var(--accent);
        }}

        .version-badge {{
            font-family: var(--font-mono);
            font-size: 10px;
            font-weight: 600;
            color: var(--accent);
            background: rgba(225, 29, 72, 0.12);
            border: 1px solid rgba(225, 29, 72, 0.25);
            padding: 2px 7px;
            border-radius: 4px;
            letter-spacing: 0.5px;
        }}

        .hud-status-cluster {{
            display: flex;
            align-items: center;
            gap: 20px;
        }}

        .hud-chip {{
            display: flex;
            align-items: center;
            gap: 7px;
            font-family: var(--font-mono);
            font-size: 11px;
            font-weight: 500;
            color: var(--text-secondary);
            background: var(--bg-card);
            border: 1px solid var(--border);
            padding: 6px 12px;
            border-radius: 6px;
        }}

        .pulse-dot {{
            width: 7px;
            height: 7px;
            border-radius: 50%;
            background: var(--hud-emerald);
            box-shadow: 0 0 8px var(--hud-emerald);
            animation: pulse 2.5s infinite;
        }}

        @keyframes pulse {{
            0% {{ transform: scale(0.95); opacity: 0.7; }}
            50% {{ transform: scale(1.15); opacity: 1; box-shadow: 0 0 12px var(--hud-emerald); }}
            100% {{ transform: scale(0.95); opacity: 0.7; }}
        }}

        .hud-clock-group {{
            display: flex;
            flex-direction: column;
            align-items: flex-end;
            font-family: var(--font-mono);
        }}

        .hud-time {{
            font-size: 16px;
            font-weight: 700;
            letter-spacing: 1px;
            color: var(--text-primary);
        }}

        .hud-date {{
            font-size: 10px;
            font-weight: 500;
            color: var(--text-muted);
            letter-spacing: 1.5px;
            text-transform: uppercase;
        }}

        /* HERO & MAIN CONTAINER */
        .home-container {{
            max-width: 1040px;
            margin: 0 auto;
            padding: 44px 24px 60px;
            width: 100%;
            display: flex;
            flex-direction: column;
            align-items: center;
            flex: 1;
        }}

        .hero-banner {{
            text-align: center;
            margin-bottom: 34px;
        }}

        .hero-title {{
            font-size: 40px;
            font-weight: 800;
            letter-spacing: -0.5px;
            color: #FFFFFF;
            margin-bottom: 8px;
            text-shadow: 0 2px 20px rgba(225, 29, 72, 0.2);
        }}

        .hero-subtitle {{
            font-family: var(--font-mono);
            font-size: 12px;
            font-weight: 500;
            color: var(--text-secondary);
            letter-spacing: 2px;
            text-transform: uppercase;
        }}

        .hero-subtitle span {{
            color: var(--accent);
        }}

        /* CRAZY COMMAND & SEARCH BAR */
        .search-cockpit {{
            width: 100%;
            max-width: 760px;
            margin-bottom: 48px;
            display: flex;
            flex-direction: column;
            gap: 12px;
        }}

        .engine-tabs {{
            display: flex;
            align-items: center;
            gap: 6px;
            justify-content: center;
        }}

        .engine-btn {{
            background: var(--bg-card);
            border: 1px solid var(--border);
            color: var(--text-secondary);
            font-family: var(--font-mono);
            font-size: 11px;
            font-weight: 600;
            padding: 6px 14px;
            border-radius: 20px;
            cursor: pointer;
            display: flex;
            align-items: center;
            gap: 6px;
            transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
        }}

        .engine-btn:hover {{
            background: var(--bg-card-hover);
            color: var(--text-primary);
            border-color: var(--accent-dim);
        }}

        .engine-btn.active {{
            background: var(--accent);
            border-color: var(--accent);
            color: #FFFFFF;
            box-shadow: 0 0 16px var(--accent-glow);
        }}

        .engine-btn svg {{
            width: 13px;
            height: 13px;
        }}

        .search-bar-shell {{
            position: relative;
            width: 100%;
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 12px;
            display: flex;
            align-items: center;
            padding: 4px 6px 4px 18px;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
            transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
        }}

        .search-bar-shell:focus-within {{
            border-color: var(--accent);
            box-shadow: 0 0 24px var(--accent-glow), 0 8px 32px rgba(0, 0, 0, 0.8);
            background: #0F080A;
        }}

        .search-icon-badge {{
            display: flex;
            align-items: center;
            justify-content: center;
            color: var(--accent);
            margin-right: 14px;
        }}

        .search-icon-badge svg {{
            width: 19px;
            height: 19px;
        }}

        .search-input {{
            flex: 1;
            background: transparent;
            border: none;
            outline: none;
            color: #FFFFFF;
            font-size: 16px;
            font-family: var(--font-main);
            padding: 12px 0;
        }}

        .search-input::placeholder {{
            color: var(--text-muted);
            font-size: 15px;
        }}

        .search-submit-btn {{
            background: var(--accent);
            border: none;
            outline: none;
            width: 42px;
            height: 42px;
            border-radius: 8px;
            display: flex;
            align-items: center;
            justify-content: center;
            color: #FFFFFF;
            cursor: pointer;
            transition: all 0.2s;
        }}

        .search-submit-btn:hover {{
            background: var(--accent-hover);
            transform: scale(1.04);
            box-shadow: 0 0 14px var(--accent-glow);
        }}

        .search-submit-btn svg {{
            width: 18px;
            height: 18px;
        }}

        .search-hotkey-hint {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0 6px;
            font-family: var(--font-mono);
            font-size: 11px;
            color: var(--text-muted);
        }}

        kbd {{
            background: #140B0E;
            border: 1px solid var(--border);
            padding: 2px 6px;
            border-radius: 4px;
            font-size: 10px;
            color: var(--text-secondary);
        }}

        /* SECTION TITLES */
        .section-header {{
            width: 100%;
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 16px;
            padding-bottom: 8px;
            border-bottom: 1px solid var(--border-subtle);
        }}

        .section-label {{
            font-family: var(--font-mono);
            font-size: 11px;
            font-weight: 700;
            letter-spacing: 2px;
            color: var(--text-secondary);
            text-transform: uppercase;
            display: flex;
            align-items: center;
            gap: 8px;
        }}

        .section-label::before {{
            content: "";
            width: 6px;
            height: 6px;
            background: var(--accent);
            border-radius: 1px;
        }}

        .section-action {{
            font-family: var(--font-mono);
            font-size: 11px;
            color: var(--accent);
            cursor: pointer;
            text-decoration: none;
            transition: color 0.2s;
        }}

        .section-action:hover {{
            color: var(--accent-hover);
            text-decoration: underline;
        }}

        /* SPEED DIALS GRID */
        .dials-grid {{
            width: 100%;
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
            gap: 14px;
            margin-bottom: 40px;
        }}

        .dial-card {{
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 10px;
            padding: 14px 16px;
            display: flex;
            align-items: center;
            gap: 14px;
            cursor: pointer;
            transition: all 0.22s cubic-bezier(0.16, 1, 0.3, 1);
            position: relative;
            overflow: hidden;
        }}

        .dial-card::before {{
            content: "";
            position: absolute;
            top: 0;
            left: 0;
            width: 3px;
            height: 100%;
            background: transparent;
            transition: background 0.2s;
        }}

        .dial-card:hover {{
            background: var(--bg-card-hover);
            border-color: var(--border-hover);
            transform: translateY(-2px);
            box-shadow: 0 6px 20px rgba(225, 29, 72, 0.15);
        }}

        .dial-card:hover::before {{
            background: var(--accent);
        }}

        .dial-icon-box {{
            width: 38px;
            height: 38px;
            border-radius: 8px;
            background: #140A0D;
            border: 1px solid var(--border);
            display: flex;
            align-items: center;
            justify-content: center;
            flex-shrink: 0;
            transition: all 0.2s;
        }}

        .dial-card:hover .dial-icon-box {{
            border-color: var(--accent);
            background: #1D0D12;
        }}

        .dial-icon-box svg {{
            width: 20px;
            height: 20px;
        }}

        .dial-icon-box.custom-initial {{
            font-family: var(--font-mono);
            font-weight: 700;
            font-size: 15px;
            color: var(--accent);
        }}

        .dial-info {{
            display: flex;
            flex-direction: column;
            overflow: hidden;
            flex: 1;
        }}

        .dial-title {{
            font-size: 13px;
            font-weight: 600;
            color: var(--text-primary);
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }}

        .dial-domain {{
            font-family: var(--font-mono);
            font-size: 10px;
            color: var(--text-muted);
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            margin-top: 2px;
        }}

        .dial-badge {{
            font-family: var(--font-mono);
            font-size: 9px;
            color: var(--hud-emerald);
            background: rgba(16, 185, 129, 0.1);
            border: 1px solid rgba(16, 185, 129, 0.2);
            padding: 1px 4px;
            border-radius: 3px;
            margin-top: 3px;
            width: fit-content;
        }}

        .add-shortcut-card {{
            border-style: dashed;
            border-color: var(--border);
            background: transparent;
            justify-content: center;
        }}

        .add-shortcut-card:hover {{
            border-color: var(--accent);
            background: rgba(225, 29, 72, 0.04);
        }}

        .add-shortcut-content {{
            display: flex;
            align-items: center;
            gap: 10px;
            color: var(--text-secondary);
            font-size: 13px;
            font-weight: 500;
        }}

        .add-shortcut-card:hover .add-shortcut-content {{
            color: var(--text-primary);
        }}

        /* STEALTH TELEMETRY CARDS (BOTTOM) */
        .telemetry-row {{
            width: 100%;
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 16px;
            margin-top: 10px;
            margin-bottom: 30px;
        }}

        .telemetry-card {{
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 10px;
            padding: 16px 18px;
            display: flex;
            flex-direction: column;
            gap: 8px;
            position: relative;
        }}

        .telemetry-header {{
            display: flex;
            align-items: center;
            justify-content: space-between;
        }}

        .telemetry-tag {{
            font-family: var(--font-mono);
            font-size: 10px;
            font-weight: 700;
            letter-spacing: 1.5px;
            color: var(--text-secondary);
            text-transform: uppercase;
        }}

        .telemetry-badge {{
            font-family: var(--font-mono);
            font-size: 9px;
            font-weight: 700;
            padding: 2px 6px;
            border-radius: 4px;
        }}

        .badge-green {{
            background: rgba(16, 185, 129, 0.12);
            color: var(--hud-emerald);
            border: 1px solid rgba(16, 185, 129, 0.3);
        }}

        .badge-crimson {{
            background: rgba(225, 29, 72, 0.12);
            color: var(--accent);
            border: 1px solid rgba(225, 29, 72, 0.3);
        }}

        .badge-cyan {{
            background: rgba(6, 182, 212, 0.12);
            color: var(--hud-cyan);
            border: 1px solid rgba(6, 182, 212, 0.3);
        }}

        .telemetry-value {{
            font-size: 15px;
            font-weight: 700;
            color: var(--text-primary);
        }}

        .telemetry-detail {{
            font-size: 11px;
            color: var(--text-muted);
            line-height: 1.4;
        }}

        /* FOOTER SHORTCUTS HUD */
        .footer-hud {{
            border-top: 1px solid var(--border-subtle);
            padding: 14px 36px;
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 20px;
            font-family: var(--font-mono);
            font-size: 11px;
            color: var(--text-muted);
            background: #030202;
        }}

        .shortcut-item {{
            display: flex;
            align-items: center;
            gap: 6px;
        }}

        /* MODAL */
        .modal-overlay {{
            position: fixed;
            top: 0;
            left: 0;
            width: 100vw;
            height: 100vh;
            background: rgba(0, 0, 0, 0.75);
            backdrop-filter: blur(8px);
            display: none;
            align-items: center;
            justify-content: center;
            z-index: 1000;
        }}

        .modal-shell {{
            background: var(--bg-card);
            border: 1px solid var(--border-hover);
            border-radius: 12px;
            padding: 24px;
            width: 100%;
            max-width: 420px;
            box-shadow: 0 16px 40px rgba(0, 0, 0, 0.8), 0 0 24px var(--accent-glow);
        }}

        .modal-title {{
            font-size: 17px;
            font-weight: 700;
            color: #FFFFFF;
            margin-bottom: 16px;
        }}

        .modal-field {{
            margin-bottom: 14px;
        }}

        .modal-field label {{
            display: block;
            font-family: var(--font-mono);
            font-size: 11px;
            color: var(--text-secondary);
            margin-bottom: 6px;
            text-transform: uppercase;
        }}

        .modal-field input {{
            width: 100%;
            background: #140B0E;
            border: 1px solid var(--border);
            border-radius: 6px;
            padding: 10px 12px;
            color: #FFFFFF;
            font-size: 13px;
            outline: none;
            transition: border-color 0.2s;
        }}

        .modal-field input:focus {{
            border-color: var(--accent);
        }}

        .modal-actions {{
            display: flex;
            justify-content: flex-end;
            gap: 10px;
            margin-top: 20px;
        }}

        .btn-modal-cancel {{
            background: transparent;
            border: 1px solid var(--border);
            color: var(--text-secondary);
            padding: 8px 16px;
            border-radius: 6px;
            cursor: pointer;
            font-size: 12px;
            font-weight: 600;
        }}

        .btn-modal-cancel:hover {{
            color: var(--text-primary);
            border-color: var(--text-secondary);
        }}

        .btn-modal-submit {{
            background: var(--accent);
            border: none;
            color: #FFFFFF;
            padding: 8px 18px;
            border-radius: 6px;
            cursor: pointer;
            font-size: 12px;
            font-weight: 600;
            box-shadow: 0 0 12px var(--accent-glow);
        }}

        .btn-modal-submit:hover {{
            background: var(--accent-hover);
        }}

        @media (max-width: 800px) {{
            .telemetry-row {{
                grid-template-columns: 1fr;
            }}
            .cockpit-hud {{
                padding: 12px 18px;
            }}
            .hud-status-cluster {{
                display: none;
            }}
            .dials-grid {{
                grid-template-columns: repeat(2, 1fr);
            }}
        }}
    </style>
</head>
<body>

    <!-- TOP COCKPIT HUD -->
    <header class="cockpit-hud">
        <div class="brand-cluster">
            <svg class="brand-logo" viewBox="0 0 100 100" fill="none">
                <defs>
                    <linearGradient id="prism-hud" x1="0" y1="0" x2="100" y2="100">
                        <stop offset="0%" stop-color="#FF385C"/>
                        <stop offset="100%" stop-color="#E11D48"/>
                    </linearGradient>
                </defs>
                <polygon points="20,15 55,15 35,85 15,85" fill="url(#prism-hud)"/>
                <polygon points="55,15 75,65 55,65 35,15" fill="#FDA4AF" opacity="0.85"/>
                <polygon points="85,85 50,85 70,15 90,15" fill="url(#prism-hud)"/>
                <circle cx="50" cy="50" r="4" fill="#FFFFFF"/>
            </svg>
            <div class="brand-title">
                NEX<span>O</span>RA
                <span class="version-badge">v{version}</span>
            </div>
        </div>

        <div class="hud-status-cluster">
            <div class="hud-chip">
                <div class="pulse-dot"></div>
                <span>CORE ACTIVE</span>
            </div>
            <div class="hud-chip">
                <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="#E11D48" stroke-width="2">
                    <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
                </svg>
                <span>ZERO TELEMETRY</span>
            </div>
            <div class="hud-chip">
                <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="#10B981" stroke-width="2">
                    <rect x="4" y="4" width="16" height="16" rx="2"/>
                    <rect x="9" y="9" width="6" height="6"/>
                    <line x1="9" y1="1" x2="9" y2="4"/>
                    <line x1="15" y1="1" x2="15" y2="4"/>
                    <line x1="9" y1="20" x2="9" y2="23"/>
                    <line x1="15" y1="20" x2="15" y2="23"/>
                    <line x1="20" y1="9" x2="23" y2="9"/>
                    <line x1="20" y1="14" x2="23" y2="14"/>
                    <line x1="1" y1="9" x2="4" y2="9"/>
                    <line x1="1" y1="14" x2="4" y2="14"/>
                </svg>
                <span>D3D11 NVDEC</span>
            </div>
        </div>

        <div class="hud-clock-group">
            <div class="hud-time" id="hud-clock">00:00:00</div>
            <div class="hud-date" id="hud-date">INITIALIZING...</div>
        </div>
    </header>

    <!-- MAIN DASHBOARD CONTENT -->
    <main class="home-container">
        <!-- Hero Greetings -->
        <div class="hero-banner">
            <h1 class="hero-title" id="greeting-text">Welcome, Commander</h1>
            <div class="hero-subtitle">
                PRECISION STREAMING CORE <span>•</span> HARDWARE ACCELERATED <span>•</span> ZERO OVERHEAD
            </div>
        </div>

        <!-- Krazy Modern Command Search Bar -->
        <section class="search-cockpit">
            <div class="engine-tabs">
                <button class="engine-btn {google_active}" onclick="setEngine('Google')" id="engine-Google">
                    <svg viewBox="0 0 24 24" fill="currentColor">
                        <path d="M12.48 10.92v3.28h7.84c-.24 1.84-.853 3.187-1.787 4.133-1.147 1.147-2.933 2.4-6.053 2.4-4.827 0-8.6-3.893-8.6-8.72s3.773-8.72 8.6-8.72c2.6 0 4.507 1.027 5.907 2.347l2.307-2.307C18.747 1.44 16.133 0 12.48 0 5.867 0 .307 5.387.307 12s5.56 12 12.173 12c3.573 0 6.267-1.173 8.373-3.36 2.16-2.16 2.84-5.213 2.84-7.667 0-.76-.053-1.467-.173-2.053H12.48z"/>
                    </svg>
                    Google
                </button>
                <button class="engine-btn {youtube_active}" onclick="setEngine('YouTube')" id="engine-YouTube">
                    <svg viewBox="0 0 24 24" fill="currentColor">
                        <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/>
                    </svg>
                    YouTube
                </button>
                <button class="engine-btn {ddg_active}" onclick="setEngine('DuckDuckGo')" id="engine-DuckDuckGo">
                    <svg viewBox="0 0 24 24" fill="currentColor">
                        <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm-1-13h2v6h-2zm0 8h2v2h-2z"/>
                    </svg>
                    DuckDuckGo
                </button>
                <button class="engine-btn {brave_active}" onclick="setEngine('Brave')" id="engine-Brave">
                    <svg viewBox="0 0 24 24" fill="currentColor">
                        <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z"/>
                    </svg>
                    Brave
                </button>
                <button class="engine-btn {bing_active}" onclick="setEngine('Bing')" id="engine-Bing">
                    <svg viewBox="0 0 24 24" fill="currentColor">
                        <path d="M5 3v18l6-3.5 6 3.5V3H5zm10 11.75l-4-2.33-4 2.33V5h8v9.75z"/>
                    </svg>
                    Bing
                </button>
            </div>

            <div class="search-bar-shell">
                <div class="search-icon-badge">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                        <circle cx="11" cy="11" r="8"/>
                        <line x1="21" y1="21" x2="16.65" y2="16.65"/>
                    </svg>
                </div>
                <input 
                    type="text" 
                    class="search-input" 
                    id="search-input" 
                    placeholder="Search the web or paste any URL..." 
                    autocomplete="off" 
                    autofocus
                />
                <button class="search-submit-btn" onclick="executeSearch()" title="Search or Navigate">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="9 18 15 12 9 6"/>
                    </svg>
                </button>
            </div>

            <div class="search-hotkey-hint">
                <span>Press <kbd>Enter</kbd> to launch query or direct URL</span>
                <span>Press <kbd>/</kbd> anywhere to focus search</span>
            </div>
        </section>

        <!-- STREAMING & CORE SPEED DIALS -->
        <div class="section-header">
            <div class="section-label">High-Performance Media Launchpad</div>
            <a href="javascript:void(0)" class="section-action" onclick="openUrl('https://www.youtube.com')">Launch YouTube &rarr;</a>
        </div>

        <div class="dials-grid">
            <!-- YouTube -->
            <div class="dial-card" onclick="openUrl('https://www.youtube.com')">
                <div class="dial-icon-box" style="color: #FF0000;">
                    <svg viewBox="0 0 24 24" fill="currentColor">
                        <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/>
                    </svg>
                </div>
                <div class="dial-info">
                    <div class="dial-title">YouTube</div>
                    <div class="dial-domain">youtube.com</div>
                    <div class="dial-badge">4K NVDEC</div>
                </div>
            </div>

            <!-- Twitch -->
            <div class="dial-card" onclick="openUrl('https://www.twitch.tv')">
                <div class="dial-icon-box" style="color: #A970FF;">
                    <svg viewBox="0 0 24 24" fill="currentColor">
                        <path d="M11.571 4.714h1.715v5.143H11.57zm4.715 0H18v5.143h-1.714zM6 0L1.714 4.286v15.428h5.143V24l4.286-4.286h3.428L22.286 12V0zm14.571 11.143l-3.428 3.428h-3.429l-3 3v-3H6.857V1.714h13.714Z"/>
                    </svg>
                </div>
                <div class="dial-info">
                    <div class="dial-title">Twitch</div>
                    <div class="dial-domain">twitch.tv</div>
                    <div class="dial-badge">Low Latency</div>
                </div>
            </div>

            <!-- GitHub -->
            <div class="dial-card" onclick="openUrl('https://github.com')">
                <div class="dial-icon-box" style="color: #F7F0F1;">
                    <svg viewBox="0 0 24 24" fill="currentColor">
                        <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"/>
                    </svg>
                </div>
                <div class="dial-info">
                    <div class="dial-title">GitHub</div>
                    <div class="dial-domain">github.com</div>
                </div>
            </div>

            <!-- Reddit -->
            <div class="dial-card" onclick="openUrl('https://www.reddit.com')">
                <div class="dial-icon-box" style="color: #FF4500;">
                    <svg viewBox="0 0 24 24" fill="currentColor">
                        <path d="M12 0A12 12 0 0 0 0 12a12 12 0 0 0 12 12 12 12 0 0 0 12-12A12 12 0 0 0 12 0zm5.01 4.744c.688 0 1.25.561 1.25 1.249a1.25 1.25 0 0 1-2.498.056l-2.597-.547-.8 3.747c1.824.07 3.48.632 4.674 1.488.308-.309.73-.491 1.207-.491.968 0 1.754.786 1.754 1.754 0 .716-.435 1.333-1.01 1.614a3.111 3.111 0 0 1 .042.52c0 2.694-3.13 4.87-7.004 4.87-3.874 0-7.004-2.176-7.004-4.87 0-.183.015-.366.043-.534A1.748 1.748 0 0 1 4.028 12c0-.968.786-1.754 1.754-1.754.463 0 .898.196 1.207.49 1.207-.883 2.878-1.43 4.744-1.487l.885-4.182a.342.342 0 0 1 .14-.197.35.35 0 0 1 .238-.042l2.906.617a1.214 1.214 0 0 1 1.108-.703zM9.25 12C8.561 12 8 12.562 8 13.25c0 .687.561 1.248 1.25 1.248.687 0 1.248-.561 1.248-1.249 0-.688-.561-1.249-1.249-1.249zm5.5 0c-.687 0-1.248.561-1.248 1.25 0 .687.561 1.248 1.249 1.248.688 0 1.249-.561 1.249-1.249 0-.687-.562-1.249-1.25-1.249zm-5.466 3.99a.327.327 0 0 0-.231.094.33.33 0 0 0 0 .463c.842.842 2.484.913 2.961.913.477 0 2.105-.056 2.961-.913a.361.361 0 0 0 .029-.463.33.33 0 0 0-.464 0c-.547.533-1.684.73-2.512.73-.828 0-1.979-.196-2.512-.73a.326.326 0 0 0-.232-.095z"/>
                    </svg>
                </div>
                <div class="dial-info">
                    <div class="dial-title">Reddit</div>
                    <div class="dial-domain">reddit.com</div>
                </div>
            </div>

            <!-- X / Twitter -->
            <div class="dial-card" onclick="openUrl('https://x.com')">
                <div class="dial-icon-box" style="color: #FFFFFF;">
                    <svg viewBox="0 0 24 24" fill="currentColor">
                        <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"/>
                    </svg>
                </div>
                <div class="dial-info">
                    <div class="dial-title">X</div>
                    <div class="dial-domain">x.com</div>
                </div>
            </div>

            <!-- Nexora Settings -->
            <div class="dial-card" onclick="openUrl('nexora://settings')">
                <div class="dial-icon-box" style="color: var(--accent);">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <circle cx="12" cy="12" r="3"/>
                        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
                    </svg>
                </div>
                <div class="dial-info">
                    <div class="dial-title">Settings</div>
                    <div class="dial-domain">nexora://settings</div>
                </div>
            </div>

            <!-- Nexora History -->
            <div class="dial-card" onclick="openUrl('nexora://history')">
                <div class="dial-icon-box" style="color: var(--hud-amber);">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <circle cx="12" cy="12" r="10"/>
                        <polyline points="12 6 12 12 16 14"/>
                    </svg>
                </div>
                <div class="dial-info">
                    <div class="dial-title">History</div>
                    <div class="dial-domain">nexora://history</div>
                </div>
            </div>

            <!-- Nexora Diagnostics -->
            <div class="dial-card" onclick="openUrl('nexora://diagnostics')">
                <div class="dial-icon-box" style="color: var(--hud-cyan);">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <activity/>
                        <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>
                    </svg>
                </div>
                <div class="dial-info">
                    <div class="dial-title">Diagnostics</div>
                    <div class="dial-domain">nexora://diagnostics</div>
                    <div class="dial-badge" style="color: var(--hud-cyan); border-color: rgba(6, 182, 212, 0.2); background: rgba(6, 182, 212, 0.1);">Telemetry</div>
                </div>
            </div>
        </div>

        <!-- USER SAVED SHORTCUTS SECTION -->
        <div class="section-header">
            <div class="section-label">Saved Favorites & Bookmarks</div>
            <a href="javascript:void(0)" class="section-action" onclick="openUrl('nexora://bookmarks')">Manage Bookmarks &rarr;</a>
        </div>

        <div class="dials-grid" id="user-dials-grid">
            {user_bookmarks_html}
            
            <!-- Add Shortcut Card -->
            <div class="dial-card add-shortcut-card" onclick="showAddModal()">
                <div class="add-shortcut-content">
                    <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
                        <line x1="12" y1="5" x2="12" y2="19"/>
                        <line x1="5" y1="12" x2="19" y2="12"/>
                    </svg>
                    <span>Add Shortcut</span>
                </div>
            </div>
        </div>

        <!-- STEALTH HARDWARE & PERFORMANCE TELEMETRY HUD -->
        <div class="section-header">
            <div class="section-label">Active Engine Telemetry</div>
        </div>

        <div class="telemetry-row">
            <!-- Hardware Video -->
            <div class="telemetry-card">
                <div class="telemetry-header">
                    <span class="telemetry-tag">GPU Pipeline</span>
                    <span class="telemetry-badge badge-green">DIRECT3D 11</span>
                </div>
                <div class="telemetry-value">NVDEC Video Zero-Copy</div>
                <div class="telemetry-detail">Hardware-accelerated VP9/AV1 decoding routes video textures directly to GPU swapchain without host memory copies.</div>
            </div>

            <!-- Memory Guard -->
            <div class="telemetry-card">
                <div class="telemetry-header">
                    <span class="telemetry-tag">Memory Shield</span>
                    <span class="telemetry-badge badge-crimson">&lt; 200MB TARGET</span>
                </div>
                <div class="telemetry-value">Chromium V8 Capped at 96MB</div>
                <div class="telemetry-detail">Aggressive renderer limit 1, network and audio services in-process, automatic child process working-set purge every 5s.</div>
            </div>

            <!-- Zero-Telemetry -->
            <div class="telemetry-card">
                <div class="telemetry-header">
                    <span class="telemetry-tag">Privacy Fortress</span>
                    <span class="telemetry-badge badge-cyan">AIRGAPPED</span>
                </div>
                <div class="telemetry-value">100% Zero Pingbacks</div>
                <div class="telemetry-detail">All Google/Edge telemetry switches disabled. Native sub-millisecond Rust ad and tracker filtering enabled.</div>
            </div>
        </div>
    </main>

    <!-- FOOTER SHORTCUTS HUD -->
    <footer class="footer-hud">
        <div class="shortcut-item"><kbd>Ctrl + T</kbd> New Tab</div>
        <div class="shortcut-item"><kbd>Ctrl + W</kbd> Close Tab</div>
        <div class="shortcut-item"><kbd>Ctrl + L</kbd> Focus Omnibox</div>
        <div class="shortcut-item"><kbd>Ctrl + H</kbd> History</div>
        <div class="shortcut-item"><kbd>Ctrl + D</kbd> Bookmark Page</div>
        <div class="shortcut-item"><kbd>/</kbd> Quick Search</div>
    </footer>

    <!-- ADD SHORTCUT MODAL -->
    <div class="modal-overlay" id="add-modal" onclick="onModalOverlayClick(event)">
        <div class="modal-shell">
            <div class="modal-title">Add Speed Dial Shortcut</div>
            <div class="modal-field">
                <label for="bm-title">Shortcut Name</label>
                <input type="text" id="bm-title" placeholder="e.g. YouTube, GitHub" autocomplete="off" />
            </div>
            <div class="modal-field">
                <label for="bm-url">Destination URL</label>
                <input type="text" id="bm-url" placeholder="https://example.com" autocomplete="off" />
            </div>
            <div class="modal-actions">
                <button class="btn-modal-cancel" onclick="closeAddModal()">Cancel</button>
                <button class="btn-modal-submit" onclick="submitAddModal()">Save Shortcut</button>
            </div>
        </div>
    </div>

    <script>
        let currentEngine = "{default_engine}";

        function updateClock() {{
            const now = new Date();
            const hours = String(now.getHours()).padStart(2, '0');
            const minutes = String(now.getMinutes()).padStart(2, '0');
            const seconds = String(now.getSeconds()).padStart(2, '0');
            document.getElementById('hud-clock').textContent = `${{hours}}:${{minutes}}:${{seconds}}`;

            const days = ['SUNDAY', 'MONDAY', 'TUESDAY', 'WEDNESDAY', 'THURSDAY', 'FRIDAY', 'SATURDAY'];
            const months = ['JAN', 'FEB', 'MAR', 'APR', 'MAY', 'JUN', 'JUL', 'AUG', 'SEP', 'OCT', 'NOV', 'DEC'];
            const dayStr = days[now.getDay()];
            const monthStr = months[now.getMonth()];
            const dateStr = now.getDate();
            document.getElementById('hud-date').textContent = `${{dayStr}}, ${{monthStr}} ${{dateStr}}`;

            const hour = now.getHours();
            let greeting = 'Welcome, Commander';
            if (hour >= 5 && hour < 12) {{
                greeting = 'Good Morning, Commander';
            }} else if (hour >= 12 && hour < 17) {{
                greeting = 'Good Afternoon, Commander';
            }} else if (hour >= 17 && hour < 22) {{
                greeting = 'Good Evening, Commander';
            }} else {{
                greeting = 'Stealth Operations Active';
            }}
            document.getElementById('greeting-text').textContent = greeting;
        }}

        setInterval(updateClock, 1000);
        updateClock();

        function setEngine(engine) {{
            currentEngine = engine;
            document.querySelectorAll('.engine-btn').forEach(btn => btn.classList.remove('active'));
            const activeBtn = document.getElementById('engine-' + engine);
            if (activeBtn) {{
                activeBtn.classList.add('active');
            }}
            const input = document.getElementById('search-input');
            input.placeholder = `Search with ${{engine}} or paste any URL...`;
            input.focus();
        }}

        function openUrl(url) {{
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage({{
                    action: 'navigate',
                    url: url
                }});
            }} else {{
                window.location.href = url;
            }}
        }}

        function executeSearch() {{
            const raw = document.getElementById('search-input').value.trim();
            if (!raw) return;

            // Check if input is a direct URL
            if (raw.startsWith('http://') || raw.startsWith('https://') || raw.startsWith('nexora://') || raw.startsWith('file://')) {{
                openUrl(raw);
                return;
            }}

            // Check if input is a domain (no spaces, contains a dot)
            if (!raw.includes(' ') && raw.includes('.')) {{
                openUrl('https://' + raw);
                return;
            }}

            // Otherwise, construct search engine query
            const encoded = encodeURIComponent(raw);
            let searchUrl = `https://www.google.com/search?q=${{encoded}}`;
            switch (currentEngine.toLowerCase()) {{
                case 'youtube':
                    searchUrl = `https://www.youtube.com/results?search_query=${{encoded}}`;
                    break;
                case 'duckduckgo':
                case 'ddg':
                    searchUrl = `https://duckduckgo.com/?q=${{encoded}}`;
                    break;
                case 'brave':
                    searchUrl = `https://search.brave.com/search?q=${{encoded}}`;
                    break;
                case 'bing':
                    searchUrl = `https://www.bing.com/search?q=${{encoded}}`;
                    break;
                default:
                    searchUrl = `https://www.google.com/search?q=${{encoded}}`;
                    break;
            }}
            openUrl(searchUrl);
        }}

        document.getElementById('search-input').addEventListener('keydown', function(e) {{
            if (e.key === 'Enter') {{
                executeSearch();
            }}
        }});

        window.addEventListener('keydown', function(e) {{
            if (e.key === '/' && document.activeElement !== document.getElementById('search-input') && document.activeElement.tagName !== 'INPUT') {{
                e.preventDefault();
                const input = document.getElementById('search-input');
                input.focus();
                input.select();
            }} else if (e.key === 'Escape') {{
                closeAddModal();
                document.getElementById('search-input').blur();
            }}
        }});

        function showAddModal() {{
            document.getElementById('add-modal').style.display = 'flex';
            setTimeout(() => document.getElementById('bm-title').focus(), 50);
        }}

        function closeAddModal() {{
            document.getElementById('add-modal').style.display = 'none';
            document.getElementById('bm-title').value = '';
            document.getElementById('bm-url').value = '';
        }}

        function onModalOverlayClick(e) {{
            if (e.target === document.getElementById('add-modal')) {{
                closeAddModal();
            }}
        }}

        function submitAddModal() {{
            const title = document.getElementById('bm-title').value.trim();
            let url = document.getElementById('bm-url').value.trim();
            if (!url) return;

            if (!url.startsWith('http://') && !url.startsWith('https://') && !url.startsWith('nexora://')) {{
                url = 'https://' + url;
            }}

            const finalTitle = title || url;

            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage({{
                    action: 'add_bookmark',
                    title: finalTitle,
                    url: url
                }});
            }}

            // Reload homepage after a short tick
            setTimeout(() => {{
                openUrl('nexora://home');
            }}, 150);

            closeAddModal();
        }}
    </script>
</body>
</html>"###,
            version = version,
            default_engine = default_engine,
            google_active = if default_engine == "Google" { "active" } else { "" },
            youtube_active = if default_engine == "YouTube" { "active" } else { "" },
            ddg_active = if default_engine == "DuckDuckGo" { "active" } else { "" },
            brave_active = if default_engine == "Brave" { "active" } else { "" },
            bing_active = if default_engine == "Bing" { "active" } else { "" },
            user_bookmarks_html = user_bookmarks_html,
        )
    }
}

fn html_escape(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(c),
        }
    }
    escaped
}

fn extract_domain(url: &str) -> String {
    let trimmed = url.trim();
    let without_scheme = if let Some(stripped) = trimmed.strip_prefix("https://") {
        stripped
    } else if let Some(stripped) = trimmed.strip_prefix("http://") {
        stripped
    } else if let Some(stripped) = trimmed.strip_prefix("nexora://") {
        return format!("nexora://{}", stripped.split('/').next().unwrap_or(""));
    } else {
        trimmed
    };

    without_scheme
        .split('/')
        .next()
        .unwrap_or(without_scheme)
        .trim_start_matches("www.")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::config::NexoraConfig;

    #[test]
    fn test_render_home_html() {
        let config = NexoraConfig::default();
        let bookmarks = vec![
            BookmarkEntry {
                id: 1,
                parent_id: None,
                title: "Test Site".to_string(),
                url: "https://example.com".to_string(),
                is_folder: false,
                position: 0,
                created_at: 0,
            }
        ];

        let html = HomeUi::render_html(&config, &bookmarks);
        assert!(html.contains("Nexora Home"));
        assert!(html.contains("NEX"));
        assert!(html.contains("Test Site"));
        assert!(html.contains("example.com"));
        assert!(html.contains("D3D11 NVDEC"));
        assert!(html.contains("ZERO TELEMETRY"));
    }
}
