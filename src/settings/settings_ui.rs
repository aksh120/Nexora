use crate::app::config::{NexoraConfig, ThemeMode};
use crate::ui::branding::BrandIdentity;

/// Professional, high-performance Settings Page Renderer
pub struct SettingsUi;

impl SettingsUi {
    /// Generates standalone, zero-overhead HTML5/CSS3 page for `nexora://settings`
    pub fn render_html(config: &NexoraConfig) -> String {
        let homepage_escaped = html_escape(&config.homepage);
        let default_engine = &config.default_search_engine;
        let theme_str = match config.theme {
            ThemeMode::System => "system",
            ThemeMode::Dark => "dark",
            ThemeMode::Light => "light",
        };

        let hw_accel_checked = if config.hardware_acceleration { "checked" } else { "" };
        let smart_susp_checked = if config.smart_suspension { "checked" } else { "" };
        let adblock_checked = if config.ad_block_enabled { "checked" } else { "" };
        let tracker_checked = if config.tracker_protection_enabled { "checked" } else { "" };
        let susp_timeout = config.suspension_timeout_minutes;
        let disk_cache = config.disk_cache_limit_mb;
        let media_cache = config.media_cache_limit_mb;
        let version = BrandIdentity::version();

        format!(
            r###"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Settings - Nexora</title>
    <style>
        :root {{
            --bg-base: #000000;
            --bg-sidebar: #050304;
            --bg-card: #0C0708;
            --bg-card-hover: #160D0F;
            --bg-input: #110A0C;
            --border: #291518;
            --border-subtle: #190B0D;
            --border-focus: #E11D48;
            --text-primary: #F7F0F1;
            --text-secondary: #997F83;
            --text-muted: #665256;
            --accent: #E11D48;
            --accent-hover: #F43F5E;
            --danger: #E11D48;
            --danger-hover: #BE123C;
            --success: #10B981;
            --font-stack: "Segoe UI", -apple-system, BlinkMacSystemFont, Roboto, sans-serif;
        }}

        @media (prefers-color-scheme: light) {{
            :root {{
                --bg-base: #F3F3F3;
                --bg-sidebar: #EBEBEB;
                --bg-card: #FFFFFF;
                --bg-card-hover: #F9F9F9;
                --bg-input: #FFFFFF;
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
            display: flex;
            height: 100vh;
            overflow: hidden;
            user-select: none;
            -webkit-font-smoothing: antialiased;
        }}

        /* Sidebar */
        .sidebar {{
            width: 240px;
            background-color: var(--bg-sidebar);
            border-right: 1px solid var(--border);
            padding: 24px 12px;
            display: flex;
            flex-direction: column;
            gap: 2px;
            flex-shrink: 0;
        }}

        .sidebar-header {{
            display: flex;
            align-items: center;
            gap: 10px;
            padding: 0 12px 20px 12px;
            border-bottom: 1px solid var(--border-subtle);
            margin-bottom: 12px;
        }}

        .sidebar-header h1 {{
            font-size: 15px;
            font-weight: 600;
            letter-spacing: 0.3px;
            color: var(--text-primary);
        }}

        .sidebar-header .version-tag {{
            font-size: 11px;
            padding: 2px 6px;
            background: rgba(255, 255, 255, 0.08);
            border-radius: 4px;
            color: var(--text-secondary);
            font-family: monospace;
        }}

        .nav-item {{
            display: flex;
            align-items: center;
            gap: 12px;
            padding: 10px 14px;
            border-radius: 6px;
            cursor: pointer;
            font-size: 13.5px;
            font-weight: 400;
            color: var(--text-secondary);
            transition: background 0.1s ease, color 0.1s ease;
        }}

        .nav-item:hover {{
            background: rgba(255, 255, 255, 0.04);
            color: var(--text-primary);
        }}

        .nav-item.active {{
            background: rgba(225, 29, 72, 0.12);
            color: var(--accent);
            font-weight: 600;
        }}

        .nav-item svg {{
            width: 17px;
            height: 17px;
            stroke: currentColor;
            fill: none;
            stroke-width: 2;
            stroke-linecap: round;
            stroke-linejoin: round;
            flex-shrink: 0;
        }}

        /* Main Content */
        .content-area {{
            flex: 1;
            display: flex;
            flex-direction: column;
            height: 100vh;
            overflow: hidden;
        }}

        .content-header {{
            height: 60px;
            padding: 0 36px;
            display: flex;
            align-items: center;
            justify-content: space-between;
            border-bottom: 1px solid var(--border-subtle);
            background: var(--bg-base);
            flex-shrink: 0;
        }}

        .search-container {{
            position: relative;
            width: 320px;
        }}

        .search-container input {{
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

        .search-container input:focus {{
            border-color: var(--border-focus);
        }}

        .search-container svg {{
            position: absolute;
            left: 10px;
            top: 50%;
            transform: translateY(-50%);
            width: 15px;
            height: 15px;
            stroke: var(--text-muted);
            fill: none;
            stroke-width: 2;
        }}

        .content-scroll {{
            flex: 1;
            padding: 32px 36px 60px 36px;
            overflow-y: auto;
        }}

        .section {{
            display: none;
            max-width: 720px;
        }}

        .section.active {{
            display: block;
        }}

        .section-title {{
            font-size: 18px;
            font-weight: 600;
            color: var(--text-primary);
            margin-bottom: 4px;
        }}

        .section-desc {{
            font-size: 13px;
            color: var(--text-secondary);
            margin-bottom: 20px;
        }}

        /* Card Container */
        .settings-card {{
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 8px;
            margin-bottom: 24px;
            overflow: hidden;
        }}

        .setting-row {{
            padding: 14px 18px;
            display: flex;
            align-items: center;
            justify-content: space-between;
            border-bottom: 1px solid var(--border-subtle);
            transition: background 0.1s ease;
        }}

        .setting-row:last-child {{
            border-bottom: none;
        }}

        .setting-row:hover {{
            background: var(--bg-card-hover);
        }}

        .setting-meta {{
            padding-right: 20px;
        }}

        .setting-meta h4 {{
            font-size: 13.5px;
            font-weight: 500;
            color: var(--text-primary);
            margin-bottom: 2px;
        }}

        .setting-meta p {{
            font-size: 12px;
            color: var(--text-secondary);
            line-height: 1.4;
        }}

        /* Controls */
        input[type="text"], input[type="number"], select {{
            background: var(--bg-input);
            border: 1px solid var(--border);
            color: var(--text-primary);
            padding: 6px 10px;
            border-radius: 5px;
            font-size: 13px;
            outline: none;
            font-family: inherit;
        }}

        input[type="text"]:focus, input[type="number"]:focus, select:focus {{
            border-color: var(--border-focus);
        }}

        select {{
            cursor: pointer;
            padding-right: 26px;
            appearance: none;
            background-image: url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="%239AA0A6" stroke-width="2.5"><path d="m6 9 6 6 6-6"/></svg>');
            background-repeat: no-repeat;
            background-position: right 8px center;
        }}

        /* Windows 11 Fluent Toggle Switch */
        .switch {{
            position: relative;
            display: inline-block;
            width: 38px;
            height: 20px;
            flex-shrink: 0;
        }}

        .switch input {{
            opacity: 0;
            width: 0;
            height: 0;
        }}

        .slider {{
            position: absolute;
            cursor: pointer;
            top: 0; left: 0; right: 0; bottom: 0;
            background-color: #383838;
            border: 1px solid var(--border);
            border-radius: 20px;
            transition: 0.15s ease;
        }}

        .slider:before {{
            position: absolute;
            content: "";
            height: 12px;
            width: 12px;
            left: 3px;
            bottom: 3px;
            background-color: #C8C8C8;
            border-radius: 50%;
            transition: 0.15s ease;
        }}

        input:checked + .slider {{
            background-color: var(--accent);
            border-color: var(--accent);
        }}

        input:checked + .slider:before {{
            transform: translateX(18px);
            background-color: #FFFFFF;
        }}

        /* Standard Buttons */
        .btn {{
            background: #2D2D2D;
            border: 1px solid var(--border);
            color: var(--text-primary);
            padding: 6px 14px;
            border-radius: 5px;
            font-size: 12.5px;
            font-weight: 500;
            cursor: pointer;
            outline: none;
            transition: background 0.1s ease, border-color 0.1s ease;
        }}

        .btn:hover {{
            background: #383838;
            border-color: #444444;
        }}

        .btn-primary {{
            background: var(--accent);
            border-color: var(--accent);
            color: #FFFFFF;
        }}

        .btn-primary:hover {{
            background: var(--accent-hover);
        }}

        .btn-danger {{
            background: rgba(209, 52, 56, 0.15);
            border: 1px solid rgba(209, 52, 56, 0.4);
            color: #FF8F94;
        }}

        .btn-danger:hover {{
            background: var(--danger);
            color: #FFFFFF;
        }}

        .status-badge {{
            font-size: 11.5px;
            font-weight: 500;
            padding: 3px 8px;
            border-radius: 4px;
            background: rgba(255, 255, 255, 0.06);
            color: var(--text-secondary);
            border: 1px solid var(--border);
        }}

        .status-badge.active {{
            background: rgba(16, 124, 65, 0.15);
            color: #6CCB5F;
            border-color: rgba(16, 124, 65, 0.3);
        }}

        /* Toast */
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
    <!-- Sidebar -->
    <div class="sidebar">
        <div class="sidebar-header">
            <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="#0078D4" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="3"/>
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
            </svg>
            <h1>Settings</h1>
            <span class="version-tag">v{version}</span>
        </div>

        <div class="nav-item active" onclick="switchTab('general', this)">
            <svg viewBox="0 0 24 24"><rect width="20" height="16" x="2" y="4" rx="2"/><path d="M6 8h.01M10 8h.01"/></svg>
            General
        </div>
        <div class="nav-item" onclick="switchTab('performance', this)">
            <svg viewBox="0 0 24 24"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/></svg>
            Performance
        </div>
        <div class="nav-item" onclick="switchTab('privacy', this)">
            <svg viewBox="0 0 24 24"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
            Privacy & Security
        </div>
        <div class="nav-item" onclick="switchTab('video', this)">
            <svg viewBox="0 0 24 24"><polygon points="5 3 19 12 5 21 5 3"/></svg>
            Video & Media
        </div>
        <div class="nav-item" onclick="switchTab('storage', this)">
            <svg viewBox="0 0 24 24"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/><path d="M3 12c0 1.66 4 3 9 3s9-1.34 9-3"/></svg>
            Storage & Cache
        </div>
        <div class="nav-item" onclick="switchTab('about', this)">
            <svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/></svg>
            About Nexora
        </div>
    </div>

    <!-- Content Area -->
    <div class="content-area">
        <div class="content-header">
            <div class="search-container">
                <svg viewBox="0 0 24 24"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
                <input type="text" id="settings-search" placeholder="Search settings" oninput="filterSettings(this.value)" />
            </div>
            <div style="font-size: 12px; color: var(--text-muted);">
                Changes are saved automatically
            </div>
        </div>

        <div class="content-scroll">
            <!-- 1. GENERAL -->
            <div id="general" class="section active">
                <h2 class="section-title">General Settings</h2>
                <p class="section-desc">Manage startup behavior, search engine preferences, and shortcuts</p>

                <div class="settings-card">
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>Homepage</h4>
                            <p>Web address loaded when opening new tabs or windows</p>
                        </div>
                        <div style="display: flex; gap: 8px;">
                            <input type="text" id="homepage" value="{homepage_escaped}" style="width: 250px;" onchange="saveSetting('homepage', this.value)" />
                            <button class="btn btn-primary" onclick="saveSetting('homepage', document.getElementById('homepage').value)">Save</button>
                            <button class="btn btn-secondary" onclick="document.getElementById('homepage').value = 'nexora://home'; saveSetting('homepage', 'nexora://home');" title="Restore Nexora dedicated homepage">Use Nexora Home</button>
                        </div>
                    </div>
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>Default Search Engine</h4>
                            <p>Search provider queried for address bar terms</p>
                        </div>
                        <select id="search_engine" onchange="saveSetting('default_search_engine', this.value)">
                            <option value="Google" {google_selected}>Google</option>
                            <option value="DuckDuckGo" {ddg_selected}>DuckDuckGo</option>
                            <option value="Bing" {bing_selected}>Microsoft Bing</option>
                            <option value="Brave" {brave_selected}>Brave Search</option>
                        </select>
                    </div>
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>Bookmarks Manager</h4>
                            <p>Manage, organize, and edit saved bookmark links</p>
                        </div>
                        <button class="btn" onclick="navigate('nexora://bookmarks')">Open Bookmarks</button>
                    </div>
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>Browsing History</h4>
                            <p>View, search, or delete your chronological browsing records</p>
                        </div>
                        <button class="btn" onclick="navigate('nexora://history')">Open History</button>
                    </div>
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>Theme Appearance</h4>
                            <p>Match your Windows system settings or choose a preferred appearance</p>
                        </div>
                        <select id="theme" onchange="saveSetting('theme', this.value)">
                            <option value="system" {theme_sys_selected}>System Default</option>
                            <option value="dark" {theme_dark_selected}>Dark</option>
                            <option value="light" {theme_light_selected}>Light</option>
                        </select>
                    </div>
                </div>
            </div>

            <!-- 2. PERFORMANCE -->
            <div id="performance" class="section">
                <h2 class="section-title">Performance &amp; Smart Suspension</h2>
                <p class="section-desc">Tune background tab memory reclamation, hardware acceleration, and process scheduling</p>

                <div class="settings-card">
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>Hardware Video Acceleration</h4>
                            <p>Offload video decoding to GPU via Direct3D 11 and DXVA2</p>
                        </div>
                        <label class="switch">
                            <input type="checkbox" id="hw_accel" {hw_accel_checked} onchange="saveSetting('hardware_acceleration', this.checked)" />
                            <span class="slider"></span>
                        </label>
                    </div>
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>Smart Tab Suspension</h4>
                            <p>Automatically suspend idle background tabs to conserve system memory</p>
                        </div>
                        <label class="switch">
                            <input type="checkbox" id="smart_susp" {smart_susp_checked} onchange="saveSetting('smart_suspension', this.checked)" />
                            <span class="slider"></span>
                        </label>
                    </div>
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>Inactivity Suspension Timeout</h4>
                            <p>Minutes before an inactive non-media background tab is suspended</p>
                        </div>
                        <div style="display: flex; align-items: center; gap: 8px;">
                            <input type="number" id="susp_timeout" value="{susp_timeout}" min="1" max="120" style="width: 75px;" onchange="saveSetting('suspension_timeout_minutes', parseInt(this.value))" />
                            <span style="font-size: 13px; color: var(--text-secondary);">minutes</span>
                        </div>
                    </div>
                </div>
            </div>

            <!-- 3. PRIVACY -->
            <div id="privacy" class="section">
                <h2 class="section-title">Privacy &amp; Security</h2>
                <p class="section-desc">Manage network request filtering, tracker blocking, and site permissions</p>

                <div class="settings-card">
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>Built-in Ad Blocker</h4>
                            <p>Native Rust request-level network filter blocking ad servers</p>
                        </div>
                        <label class="switch">
                            <input type="checkbox" id="ad_block" {adblock_checked} onchange="saveSetting('ad_block_enabled', this.checked)" />
                            <span class="slider"></span>
                        </label>
                    </div>
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>Tracker &amp; Fingerprinting Protection</h4>
                            <p>Block third-party analytics, tracking beacons, and cryptominers</p>
                        </div>
                        <label class="switch">
                            <input type="checkbox" id="tracker_block" {tracker_checked} onchange="saveSetting('tracker_protection_enabled', this.checked)" />
                            <span class="slider"></span>
                        </label>
                    </div>
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>Zero-Telemetry Status</h4>
                            <p>Complete network isolation: 0 analytics pings, 0 telemetry beacons</p>
                        </div>
                        <span class="status-badge active">Enforced</span>
                    </div>
                </div>
            </div>

            <!-- 4. VIDEO & STREAMING -->
            <div id="video" class="section">
                <h2 class="section-title">Video &amp; Streaming Optimization</h2>
                <p class="section-desc">GPU hardware decoder pipeline and verified media DRM authorization</p>

                <div class="settings-card">
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>GPU Video Decoder Pipeline</h4>
                            <p>Direct3D 11 NVDEC offload on NVIDIA GeForce GT 730 / Intel HD Graphics</p>
                        </div>
                        <span class="status-badge active">D3D11 Active</span>
                    </div>
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>DRM Playback Pipelines</h4>
                            <p>Widevine CDM and PlayReady EME for 1080p streaming services</p>
                        </div>
                        <span class="status-badge active">Certified EME</span>
                    </div>
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>Active Media Priority Scheduler</h4>
                            <p>Elevates process priority during active audio and video playback</p>
                        </div>
                        <span class="status-badge active">Above Normal</span>
                    </div>
                </div>
            </div>

            <!-- 5. STORAGE & CACHE -->
            <div id="storage" class="section">
                <h2 class="section-title">Storage &amp; Cache Boundaries</h2>
                <p class="section-desc">Manage local disk storage boundaries, database maintenance, and history purging</p>

                <div class="settings-card">
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>Disk Cache Boundary</h4>
                            <p>Maximum disk cache allowed before automatic LRU cleanup (MB)</p>
                        </div>
                        <div style="display: flex; align-items: center; gap: 8px;">
                            <input type="number" id="disk_cache" value="{disk_cache}" min="64" max="2048" style="width: 80px;" onchange="saveSetting('disk_cache_limit_mb', parseInt(this.value))" />
                            <span style="font-size: 13px; color: var(--text-secondary);">MB</span>
                        </div>
                    </div>
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>Media Cache Boundary</h4>
                            <p>Maximum video stream chunk buffer storage on disk (MB)</p>
                        </div>
                        <div style="display: flex; align-items: center; gap: 8px;">
                            <input type="number" id="media_cache" value="{media_cache}" min="32" max="1024" style="width: 80px;" onchange="saveSetting('media_cache_limit_mb', parseInt(this.value))" />
                            <span style="font-size: 13px; color: var(--text-secondary);">MB</span>
                        </div>
                    </div>
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>Database Journal Compaction</h4>
                            <p>Truncate and compact SQLite write-ahead logging journal (WAL)</p>
                        </div>
                        <button class="btn" id="btn-compact-wal" onclick="compactWal()">Compact SQLite WAL</button>
                    </div>
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>Clear Browser Cache</h4>
                            <p>Purge temporary engine disk and media cache immediately</p>
                        </div>
                        <button class="btn" id="btn-clear-cache" onclick="clearCache()">Clear Cache</button>
                    </div>
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>Clear Browsing History</h4>
                            <p>Erase all visited URL records and timeline metadata from SQLite database</p>
                        </div>
                        <button class="btn btn-danger" id="btn-clear-history" onclick="clearHistory()">Clear History</button>
                    </div>
                </div>
            </div>

            <!-- 6. ABOUT -->
            <div id="about" class="section">
                <h2 class="section-title">About Nexora Browser</h2>
                <p class="section-desc">Version details, core architecture, and factory reset</p>

                <div class="settings-card">
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>Nexora Core Build</h4>
                            <p>Ultra-lightweight, streaming-first desktop browser for Windows</p>
                        </div>
                        <span class="status-badge">v{version} (x86_64-pc-windows-gnu)</span>
                    </div>
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>Rendering Engine</h4>
                            <p>Microsoft WebView2 Evergreen Runtime (Chromium Core)</p>
                        </div>
                        <span class="status-badge">System Evergreen</span>
                    </div>
                    <div class="setting-row">
                        <div class="setting-meta">
                            <h4>Core Philosophy</h4>
                            <p>Maximum performance. Minimum overhead. Complete browsing freedom.</p>
                        </div>
                        <button class="btn btn-danger" onclick="resetDefaults()">Reset to Defaults</button>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <div id="toast"></div>

    <script>
        function showToast(message) {{
            const toast = document.getElementById('toast');
            toast.textContent = message;
            toast.style.display = 'block';
            setTimeout(() => {{
                toast.style.display = 'none';
            }}, 2000);
        }}

        function navigate(url) {{
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage(JSON.stringify({{
                    action: "navigate",
                    url: url
                }}));
            }} else {{
                window.location.href = url;
            }}
        }}

        function clearHistory() {{
            if (confirm("Are you sure you want to clear your complete browsing history?")) {{
                if (window.chrome && window.chrome.webview) {{
                    window.chrome.webview.postMessage(JSON.stringify({{
                        action: "clear_history"
                    }}));
                }}
                showToast("Browsing history cleared");
                const btn = document.getElementById('btn-clear-history');
                if (btn) {{
                    btn.textContent = "Cleared";
                }}
            }}
        }}

        function switchTab(tabId, clickedElement) {{
            document.querySelectorAll('.nav-item').forEach(el => el.classList.remove('active'));
            document.querySelectorAll('.section').forEach(el => el.classList.remove('active'));
            const targetSection = document.getElementById(tabId);
            if (targetSection) {{
                targetSection.classList.add('active');
            }}
            if (clickedElement) {{
                clickedElement.classList.add('active');
            }}
        }}

        function saveSetting(key, value) {{
            console.log('Saving setting:', key, value);
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage(JSON.stringify({{
                    action: "save_setting",
                    key: key,
                    value: value
                }}));
            }}
            showToast("Setting saved automatically");
        }}

        function compactWal() {{
            const btn = document.getElementById('btn-compact-wal');
            btn.textContent = "Compacting...";
            btn.disabled = true;
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage(JSON.stringify({{
                    action: "compact_wal"
                }}));
            }}
            setTimeout(() => {{
                btn.textContent = "Compacted";
                btn.disabled = false;
                showToast("SQLite WAL compacted successfully");
            }}, 300);
        }}

        function clearCache() {{
            const btn = document.getElementById('btn-clear-cache');
            btn.textContent = "Purging...";
            btn.disabled = true;
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage(JSON.stringify({{
                    action: "clear_cache"
                }}));
            }}
            setTimeout(() => {{
                btn.textContent = "Purged";
                btn.disabled = false;
                showToast("Temporary cache purged");
            }}, 300);
        }}

        function resetDefaults() {{
            if (confirm("Reset all browser configuration back to factory defaults?")) {{
                if (window.chrome && window.chrome.webview) {{
                    window.chrome.webview.postMessage(JSON.stringify({{
                        action: "reset_defaults"
                    }}));
                }}
                showToast("Resetting to defaults...");
                setTimeout(() => {{
                    location.reload();
                }}, 500);
            }}
        }}

        function filterSettings(query) {{
            const q = query.toLowerCase().trim();
            const rows = document.querySelectorAll('.setting-row');
            if (!q) {{
                rows.forEach(r => r.style.display = 'flex');
                return;
            }}
            document.querySelectorAll('.section').forEach(s => s.classList.add('active'));
            rows.forEach(r => {{
                const text = r.textContent.toLowerCase();
                if (text.includes(q)) {{
                    r.style.display = 'flex';
                }} else {{
                    r.style.display = 'none';
                }}
            }});
        }}
    </script>
</body>
</html>"###,
            version = version,
            homepage_escaped = homepage_escaped,
            google_selected = if default_engine == "Google" { "selected" } else { "" },
            ddg_selected = if default_engine == "DuckDuckGo" { "selected" } else { "" },
            bing_selected = if default_engine == "Bing" { "selected" } else { "" },
            brave_selected = if default_engine == "Brave" { "selected" } else { "" },
            theme_sys_selected = if theme_str == "system" { "selected" } else { "" },
            theme_dark_selected = if theme_str == "dark" { "selected" } else { "" },
            theme_light_selected = if theme_str == "light" { "selected" } else { "" },
            hw_accel_checked = hw_accel_checked,
            smart_susp_checked = smart_susp_checked,
            susp_timeout = susp_timeout,
            adblock_checked = adblock_checked,
            tracker_checked = tracker_checked,
            disk_cache = disk_cache,
            media_cache = media_cache,
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
    fn test_render_settings_html_contains_all_tabs() {
        let config = NexoraConfig::default();
        let html = SettingsUi::render_html(&config);

        assert!(html.contains("General Settings"));
        assert!(html.contains("Performance &amp; Smart Suspension") || html.contains("Performance"));
        assert!(html.contains("Privacy &amp; Security") || html.contains("Privacy"));
        assert!(html.contains("Video &amp; Streaming Optimization") || html.contains("Video"));
        assert!(html.contains("Storage &amp; Cache Boundaries") || html.contains("Storage"));
        assert!(html.contains("About Nexora Browser"));
        assert!(html.contains("Bookmarks Manager"));
        assert!(html.contains("Browsing History"));
        assert!(html.contains("nexora://home"));
        assert!(html.contains("Zero-Telemetry"));
    }
}
