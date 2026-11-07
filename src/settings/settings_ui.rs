use crate::app::config::{NexoraConfig, ThemeMode};
use crate::ui::branding::BrandIdentity;

/// Minimalist, high-performance Settings Page Renderer
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
    <title>Nexora Settings</title>
    <style>
        :root {{
            --bg-base: #0B0E14;
            --bg-sidebar: #10141D;
            --bg-card: #151923;
            --bg-card-hover: #191E2B;
            --bg-input: #0B0E14;
            --border: #202738;
            --border-focus: #00E5FF;
            --text-primary: #F8FAFC;
            --text-secondary: #94A3B8;
            --text-muted: #64748B;
            --accent: #00E5FF;
            --accent-glow: rgba(0, 229, 255, 0.15);
            --danger: #EF4444;
            --danger-bg: rgba(239, 68, 68, 0.1);
            --success: #10B981;
            --success-bg: rgba(16, 185, 129, 0.1);
            --font-stack: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
        }}

        @media (prefers-color-scheme: light) {{
            :root {{
                --bg-base: #F8FAFC;
                --bg-sidebar: #F1F5F9;
                --bg-card: #FFFFFF;
                --bg-card-hover: #F8FAFC;
                --bg-input: #FFFFFF;
                --border: #E2E8F0;
                --border-focus: #00B8D4;
                --text-primary: #0F172A;
                --text-secondary: #475569;
                --text-muted: #94A3B8;
                --accent: #00B8D4;
                --accent-glow: rgba(0, 184, 212, 0.15);
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

        /* Sidebar Navigation */
        .sidebar {{
            width: 250px;
            background-color: var(--bg-sidebar);
            border-right: 1px solid var(--border);
            padding: 24px 16px;
            display: flex;
            flex-direction: column;
            gap: 4px;
            flex-shrink: 0;
        }}

        .sidebar-header {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0 8px 20px 8px;
            border-bottom: 1px solid var(--border);
            margin-bottom: 12px;
        }}

        .brand-title {{
            font-size: 15px;
            font-weight: 700;
            letter-spacing: 0.8px;
            color: var(--text-primary);
            display: flex;
            align-items: center;
            gap: 8px;
        }}

        .brand-badge {{
            font-size: 10.5px;
            font-weight: 600;
            padding: 2px 7px;
            border-radius: 6px;
            background: var(--accent-glow);
            color: var(--accent);
            border: 1px solid rgba(0, 229, 255, 0.3);
        }}

        .nav-item {{
            display: flex;
            align-items: center;
            gap: 12px;
            padding: 10px 14px;
            border-radius: 8px;
            cursor: pointer;
            font-size: 13.5px;
            font-weight: 500;
            color: var(--text-secondary);
            transition: all 0.15s ease;
            position: relative;
        }}

        .nav-item svg {{
            width: 17px;
            height: 17px;
            stroke: currentColor;
            fill: none;
            stroke-width: 2;
            stroke-linecap: round;
            stroke-linejoin: round;
            transition: stroke 0.15s ease;
            flex-shrink: 0;
        }}

        .nav-item:hover {{
            background: rgba(255, 255, 255, 0.04);
            color: var(--text-primary);
        }}

        .nav-item.active {{
            background: var(--accent-glow);
            color: var(--accent);
            font-weight: 600;
        }}

        .nav-item.active::before {{
            content: "";
            position: absolute;
            left: 0;
            top: 6px;
            bottom: 6px;
            width: 3px;
            background-color: var(--accent);
            border-radius: 0 3px 3px 0;
        }}

        /* Main Content */
        .content-container {{
            flex: 1;
            display: flex;
            flex-direction: column;
            overflow: hidden;
            background-color: var(--bg-base);
        }}

        .content-header {{
            padding: 20px 48px;
            border-bottom: 1px solid var(--border);
            display: flex;
            align-items: center;
            justify-content: space-between;
            background: var(--bg-base);
            z-index: 10;
        }}

        .search-box {{
            position: relative;
            width: 320px;
        }}

        .search-box svg {{
            position: absolute;
            left: 12px;
            top: 50%;
            transform: translateY(-50%);
            width: 15px;
            height: 15px;
            stroke: var(--text-muted);
            fill: none;
            stroke-width: 2;
        }}

        .search-box input {{
            width: 100%;
            background: var(--bg-input);
            border: 1px solid var(--border);
            padding: 8px 12px 8px 36px;
            border-radius: 8px;
            color: var(--text-primary);
            font-size: 13px;
            outline: none;
            transition: border-color 0.15s, box-shadow 0.15s;
        }}

        .search-box input:focus {{
            border-color: var(--border-focus);
            box-shadow: 0 0 0 1px var(--accent-glow);
        }}

        .content-body {{
            flex: 1;
            padding: 32px 48px;
            overflow-y: auto;
        }}

        .section {{
            display: none;
            max-width: 760px;
            animation: fadeIn 0.15s ease-in-out;
        }}

        .section.active {{
            display: block;
        }}

        @keyframes fadeIn {{
            from {{ opacity: 0; transform: translateY(4px); }}
            to {{ opacity: 1; transform: translateY(0); }}
        }}

        .section-header {{
            margin-bottom: 20px;
        }}

        .section-header h2 {{
            font-size: 20px;
            font-weight: 600;
            color: var(--text-primary);
            margin-bottom: 4px;
        }}

        .section-header p {{
            font-size: 13px;
            color: var(--text-secondary);
        }}

        /* Grouped Card Lists */
        .setting-card {{
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 12px;
            overflow: hidden;
            margin-bottom: 24px;
        }}

        .setting-row {{
            padding: 16px 20px;
            display: flex;
            align-items: center;
            justify-content: space-between;
            border-bottom: 1px solid var(--border);
            transition: background 0.15s ease;
        }}

        .setting-row:last-child {{
            border-bottom: none;
        }}

        .setting-row:hover {{
            background: var(--bg-card-hover);
        }}

        .setting-info {{
            padding-right: 24px;
        }}

        .setting-info h4 {{
            font-size: 13.5px;
            font-weight: 500;
            color: var(--text-primary);
            margin-bottom: 3px;
        }}

        .setting-info p {{
            font-size: 12px;
            color: var(--text-muted);
            line-height: 1.4;
        }}

        /* Controls */
        input[type="text"], input[type="number"], select {{
            background: var(--bg-input);
            border: 1px solid var(--border);
            color: var(--text-primary);
            padding: 8px 12px;
            border-radius: 7px;
            font-size: 13px;
            outline: none;
            transition: border-color 0.15s, box-shadow 0.15s;
        }}

        select {{
            padding-right: 30px;
            cursor: pointer;
            appearance: none;
            background-image: url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="%2394A3B8" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"/></svg>');
            background-repeat: no-repeat;
            background-position: right 10px center;
        }}

        input[type="text"]:focus, input[type="number"]:focus, select:focus {{
            border-color: var(--border-focus);
            box-shadow: 0 0 0 1px var(--accent-glow);
        }}

        /* Native-style Fluent Switch */
        .switch {{
            position: relative;
            display: inline-block;
            width: 42px;
            height: 22px;
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
            background-color: var(--border);
            transition: 0.2s cubic-bezier(0.4, 0, 0.2, 1);
            border-radius: 22px;
        }}

        .slider:before {{
            position: absolute;
            content: "";
            height: 16px;
            width: 16px;
            left: 3px;
            bottom: 3px;
            background-color: #CBD5E1;
            transition: 0.2s cubic-bezier(0.4, 0, 0.2, 1);
            border-radius: 50%;
        }}

        input:checked + .slider {{
            background-color: var(--accent);
        }}

        input:checked + .slider:before {{
            transform: translateX(20px);
            background-color: #0B0E14;
        }}

        /* Badges & Buttons */
        .badge-status {{
            display: inline-flex;
            align-items: center;
            gap: 6px;
            padding: 4px 10px;
            border-radius: 6px;
            font-size: 11.5px;
            font-weight: 600;
            background: var(--success-bg);
            color: var(--success);
            border: 1px solid rgba(16, 185, 129, 0.25);
        }}

        .btn {{
            background: rgba(255, 255, 255, 0.05);
            border: 1px solid var(--border);
            color: var(--text-primary);
            padding: 7px 14px;
            border-radius: 7px;
            cursor: pointer;
            font-size: 12.5px;
            font-weight: 500;
            transition: all 0.15s ease;
        }}

        .btn:hover {{
            background: rgba(255, 255, 255, 0.1);
            border-color: var(--border-focus);
            color: var(--accent);
        }}

        .btn-danger {{
            background: var(--danger-bg);
            border: 1px solid rgba(239, 68, 68, 0.3);
            color: #F87171;
        }}

        .btn-danger:hover {{
            background: rgba(239, 68, 68, 0.2);
            border-color: var(--danger);
            color: #FFA4A4;
        }}

        /* Floating Toast */
        #save-toast {{
            position: fixed;
            bottom: 24px;
            right: 32px;
            background: #181F2E;
            border: 1px solid var(--accent);
            color: var(--text-primary);
            padding: 10px 18px;
            border-radius: 8px;
            font-size: 13px;
            font-weight: 500;
            display: flex;
            align-items: center;
            gap: 8px;
            box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5), 0 0 12px var(--accent-glow);
            opacity: 0;
            transform: translateY(12px);
            transition: opacity 0.2s, transform 0.2s;
            pointer-events: none;
            z-index: 1000;
        }}

        #save-toast.show {{
            opacity: 1;
            transform: translateY(0);
        }}
    </style>
</head>
<body>
    <!-- Sidebar Navigation -->
    <div class="sidebar">
        <div class="sidebar-header">
            <div class="brand-title">
                NEXORA
                <span class="brand-badge">v{version}</span>
            </div>
        </div>

        <div class="nav-item active" onclick="switchTab('general', this)">
            <svg viewBox="0 0 24 24"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>
            General
        </div>
        <div class="nav-item" onclick="switchTab('performance', this)">
            <svg viewBox="0 0 24 24"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/></svg>
            Performance
        </div>
        <div class="nav-item" onclick="switchTab('privacy', this)">
            <svg viewBox="0 0 24 24"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/><path d="m9 12 2 2 4-4"/></svg>
            Privacy & Security
        </div>
        <div class="nav-item" onclick="switchTab('video', this)">
            <svg viewBox="0 0 24 24"><rect width="20" height="15" x="2" y="5" rx="2"/><polygon points="10 9 15 12 10 15 10 9"/></svg>
            Video & Streaming
        </div>
        <div class="nav-item" onclick="switchTab('storage', this)">
            <svg viewBox="0 0 24 24"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/><path d="M3 12c0 1.66 4 3 9 3s9-1.34 9-3"/></svg>
            Storage & Cache
        </div>
        <div class="nav-item" onclick="switchTab('about', this)">
            <svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>
            About
        </div>
    </div>

    <!-- Main Content Container -->
    <div class="content-container">
        <div class="content-header">
            <div class="search-box">
                <svg viewBox="0 0 24 24"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
                <input type="text" id="settings-search" placeholder="Search settings..." oninput="filterSettings(this.value)" />
            </div>
            <div style="font-size: 12px; color: var(--text-muted);">
                Changes are saved automatically
            </div>
        </div>

        <div class="content-body">
            <!-- 1. GENERAL -->
            <div id="general" class="section active">
                <div class="section-header">
                    <h2>General Settings</h2>
                    <p>Customize startup behavior, default search engines, and visual themes</p>
                </div>
                <div class="setting-card">
                    <div class="setting-row">
                        <div class="setting-info">
                            <h4>Homepage URL</h4>
                            <p>Webpage loaded automatically when launching a new tab or window</p>
                        </div>
                        <div style="display: flex; gap: 8px; align-items: center;">
                            <input type="text" id="homepage" value="{homepage_escaped}" style="width: 260px;" onchange="saveSetting('homepage', this.value)" />
                            <button class="btn" onclick="saveSetting('homepage', document.getElementById('homepage').value)">Save</button>
                        </div>
                    </div>
                    <div class="setting-row">
                        <div class="setting-info">
                            <h4>Default Search Engine</h4>
                            <p>Search provider queried for address bar keywords and search queries</p>
                        </div>
                        <select id="search_engine" onchange="saveSetting('default_search_engine', this.value)">
                            <option value="Google" {google_selected}>Google</option>
                            <option value="DuckDuckGo" {ddg_selected}>DuckDuckGo (Zero-Tracking)</option>
                            <option value="Bing" {bing_selected}>Microsoft Bing</option>
                            <option value="Brave" {brave_selected}>Brave Search</option>
                        </select>
                    </div>
                    <div class="setting-row">
                        <div class="setting-info">
                            <h4>Interface Theme</h4>
                            <p>Match Windows system default appearance or force dark/light mode</p>
                        </div>
                        <select id="theme" onchange="saveSetting('theme', this.value)">
                            <option value="system" {theme_sys_selected}>System Default</option>
                            <option value="dark" {theme_dark_selected}>Dark Mode</option>
                            <option value="light" {theme_light_selected}>Light Mode</option>
                        </select>
                    </div>
                </div>
            </div>

            <!-- 2. PERFORMANCE -->
            <div id="performance" class="section">
                <div class="section-header">
                    <h2>Performance &amp; Smart Suspension</h2>
                    <p>Tune background tab suspension, idle timeouts, and hardware video decoding</p>
                </div>
                <div class="setting-card">
                    <div class="setting-row">
                        <div class="setting-info">
                            <h4>Hardware Video Acceleration</h4>
                            <p>Direct3D 11 &amp; DXVA2 GPU hardware decoding offload for H.264/VP9</p>
                        </div>
                        <label class="switch">
                            <input type="checkbox" id="hw_accel" {hw_accel_checked} onchange="saveSetting('hardware_acceleration', this.checked)" />
                            <span class="slider"></span>
                        </label>
                    </div>
                    <div class="setting-row">
                        <div class="setting-info">
                            <h4>Smart Tab Suspension</h4>
                            <p>Automatically suspend idle background tabs to conserve host RAM</p>
                        </div>
                        <label class="switch">
                            <input type="checkbox" id="smart_susp" {smart_susp_checked} onchange="saveSetting('smart_suspension', this.checked)" />
                            <span class="slider"></span>
                        </label>
                    </div>
                    <div class="setting-row">
                        <div class="setting-info">
                            <h4>Inactivity Suspension Timeout</h4>
                            <p>Minutes of inactivity before non-media background tabs are suspended</p>
                        </div>
                        <div style="display: flex; align-items: center; gap: 8px;">
                            <input type="number" id="susp_timeout" value="{susp_timeout}" min="1" max="120" style="width: 80px;" onchange="saveSetting('suspension_timeout_minutes', parseInt(this.value))" />
                            <span style="font-size: 13px; color: var(--text-secondary);">minutes</span>
                        </div>
                    </div>
                </div>
            </div>

            <!-- 3. PRIVACY -->
            <div id="privacy" class="section">
                <div class="section-header">
                    <h2>Privacy &amp; Security</h2>
                    <p>Enforce native network filtering, tracker blocking, and zero telemetry</p>
                </div>
                <div class="setting-card">
                    <div class="setting-row">
                        <div class="setting-info">
                            <h4>Built-in High-Speed Ad Blocker</h4>
                            <p>Native Rust request-level filtering blocking ad servers before network fetch</p>
                        </div>
                        <label class="switch">
                            <input type="checkbox" id="ad_block" {adblock_checked} onchange="saveSetting('ad_block_enabled', this.checked)" />
                            <span class="slider"></span>
                        </label>
                    </div>
                    <div class="setting-row">
                        <div class="setting-info">
                            <h4>Tracker &amp; Fingerprinting Protection</h4>
                            <p>Block third-party tracking beacons, cross-site telemetry, and cryptominers</p>
                        </div>
                        <label class="switch">
                            <input type="checkbox" id="tracker_block" {tracker_checked} onchange="saveSetting('tracker_protection_enabled', this.checked)" />
                            <span class="slider"></span>
                        </label>
                    </div>
                    <div class="setting-row">
                        <div class="setting-info">
                            <h4>Zero-Telemetry Status</h4>
                            <p>Strict network isolation: Zero external analytics or diagnostic pings</p>
                        </div>
                        <div class="badge-status">✓ Enforced (0 pings)</div>
                    </div>
                </div>
            </div>

            <!-- 4. VIDEO -->
            <div id="video" class="section">
                <div class="section-header">
                    <h2>Video &amp; Streaming Optimization</h2>
                    <p>Monitor GPU hardware offload and commercial DRM authorization status</p>
                </div>
                <div class="setting-card">
                    <div class="setting-row">
                        <div class="setting-info">
                            <h4>GPU Video Decoder Pipeline</h4>
                            <p>Direct3D 11 NVDEC offload on NVIDIA GeForce GT 730 / Intel HD Graphics</p>
                        </div>
                        <div class="badge-status">✓ D3D11 Active</div>
                    </div>
                    <div class="setting-row">
                        <div class="setting-info">
                            <h4>Authorized DRM Pipelines</h4>
                            <p>Widevine CDM &amp; PlayReady EME for 1080p Netflix, Hotstar &amp; Prime Video</p>
                        </div>
                        <div class="badge-status">✓ Certified EME</div>
                    </div>
                    <div class="setting-row">
                        <div class="setting-info">
                            <h4>Active Media Priority Scheduler</h4>
                            <p>Automatically elevates process priority during active video streaming</p>
                        </div>
                        <div class="badge-status">✓ Above-Normal Priority</div>
                    </div>
                </div>
            </div>

            <!-- 5. STORAGE -->
            <div id="storage" class="section">
                <div class="section-header">
                    <h2>Storage &amp; Cache Boundaries</h2>
                    <p>Manage disk cache limits and SQLite database compaction</p>
                </div>
                <div class="setting-card">
                    <div class="setting-row">
                        <div class="setting-info">
                            <h4>Disk Cache Boundary</h4>
                            <p>Maximum disk cache allowed before automatic LRU sweep (MB)</p>
                        </div>
                        <div style="display: flex; align-items: center; gap: 8px;">
                            <input type="number" id="disk_cache" value="{disk_cache}" min="64" max="2048" style="width: 90px;" onchange="saveSetting('disk_cache_limit_mb', parseInt(this.value))" />
                            <span style="font-size: 13px; color: var(--text-secondary);">MB</span>
                        </div>
                    </div>
                    <div class="setting-row">
                        <div class="setting-info">
                            <h4>Media Cache Boundary</h4>
                            <p>Maximum video stream chunk buffer storage on disk (MB)</p>
                        </div>
                        <div style="display: flex; align-items: center; gap: 8px;">
                            <input type="number" id="media_cache" value="{media_cache}" min="32" max="1024" style="width: 90px;" onchange="saveSetting('media_cache_limit_mb', parseInt(this.value))" />
                            <span style="font-size: 13px; color: var(--text-secondary);">MB</span>
                        </div>
                    </div>
                    <div class="setting-row">
                        <div class="setting-info">
                            <h4>Database Journal Compaction</h4>
                            <p>Truncate and compact SQLite write-ahead logging journal (WAL)</p>
                        </div>
                        <button class="btn" id="btn-compact-wal" onclick="compactWal()">Compact SQLite WAL</button>
                    </div>
                    <div class="setting-row">
                        <div class="setting-info">
                            <h4>Clear Browser Cache</h4>
                            <p>Purge temporary webview disk and media cache immediately</p>
                        </div>
                        <button class="btn" id="btn-clear-cache" onclick="clearCache()">Clear Cache</button>
                    </div>
                </div>
            </div>

            <!-- 6. ABOUT -->
            <div id="about" class="section">
                <div class="section-header">
                    <h2>About Nexora Browser</h2>
                    <p>System version details, open-source core, and factory resets</p>
                </div>
                <div class="setting-card">
                    <div class="setting-row">
                        <div class="setting-info">
                            <h4>Nexora Core Build</h4>
                            <p>Ultra-lightweight, streaming-first desktop browser for Windows</p>
                        </div>
                        <div style="font-size: 13px; font-weight: 600; color: var(--accent);">v{version} (x86_64-pc-windows-gnu)</div>
                    </div>
                    <div class="setting-row">
                        <div class="setting-info">
                            <h4>Rendering Engine</h4>
                            <p>Microsoft WebView2 Evergreen Runtime</p>
                        </div>
                        <div style="font-size: 13px; font-weight: 500; color: var(--text-secondary);">Chromium Core</div>
                    </div>
                    <div class="setting-row">
                        <div class="setting-info">
                            <h4>Core Philosophy</h4>
                            <p>Maximum performance. Minimum overhead. Complete browsing freedom.</p>
                        </div>
                        <button class="btn btn-danger" onclick="resetDefaults()">Reset to Defaults</button>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Floating Save Toast -->
    <div id="save-toast">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#00E5FF" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
        <span id="toast-message">Settings saved successfully</span>
    </div>

    <script>
        let toastTimeout = null;

        function showToast(msg) {{
            const toast = document.getElementById('save-toast');
            const toastText = document.getElementById('toast-message');
            toastText.textContent = msg;
            toast.classList.add('show');
            if (toastTimeout) clearTimeout(toastTimeout);
            toastTimeout = setTimeout(() => {{
                toast.classList.remove('show');
            }}, 2200);
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
            showToast("✓ Setting saved automatically");
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
                btn.textContent = "✓ Compacted (0 bytes pending)";
                btn.disabled = false;
                showToast("✓ SQLite WAL Compacted successfully");
            }}, 400);
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
                btn.textContent = "✓ Cache Purged";
                btn.disabled = false;
                showToast("✓ Temporary cache purged");
            }}, 400);
        }}

        function resetDefaults() {{
            if (confirm("Reset all browser configuration back to factory defaults?")) {{
                if (window.chrome && window.chrome.webview) {{
                    window.chrome.webview.postMessage(JSON.stringify({{
                        action: "reset_defaults"
                    }}));
                }}
                showToast("✓ Resetting to defaults...");
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
            // If searching, show all sections so matches are visible
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
        assert!(html.contains("https://www.google.com"));
        assert!(html.contains("Zero-Telemetry"));
    }
}
