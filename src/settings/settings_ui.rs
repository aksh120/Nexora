use crate::app::config::{NexoraConfig, ThemeMode};
use crate::ui::branding::BrandIdentity;
use crate::ui::design_system::DesignTokens;
use crate::ui::icons::LucideIcon;

pub struct SettingsUi;

impl SettingsUi {
    /// Generates a professional, two-column settings dashboard for `nexora://settings`.
    /// Follows clean desktop browser aesthetics with zero futuristic AI clichés.
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

        let general_icon_svg = LucideIcon::Sliders.to_svg(16, 2.0, "currentColor");
        let perf_icon_svg = LucideIcon::Cpu.to_svg(16, 2.0, "currentColor");
        let privacy_icon_svg = LucideIcon::ShieldCheck.to_svg(16, 2.0, "currentColor");
        let streaming_icon_svg = LucideIcon::Layers.to_svg(16, 2.0, "currentColor");
        let video_icon_svg = LucideIcon::Volume2.to_svg(16, 2.0, "currentColor");
        let storage_icon_svg = LucideIcon::HardDrive.to_svg(16, 2.0, "currentColor");
        let about_icon_svg = LucideIcon::Sparkles.to_svg(16, 2.0, "currentColor");
        let search_icon_svg = LucideIcon::Search.to_svg(14, 2.0, "currentColor");
        let logo_svg = BrandIdentity::monogram_svg(24);
        let shared_css = DesignTokens::shared_css(true);

        let streaming_mode = config.streaming_engine_mode;
        let std_selected = streaming_mode == crate::app::config::StreamingEngineMode::Standard;
        let native_selected = streaming_mode == crate::app::config::StreamingEngineMode::NativeD3D11;
        let hybrid_selected = streaming_mode == crate::app::config::StreamingEngineMode::HybridStealth;

        let std_card_cls = if std_selected { "selected" } else { "" };
        let std_radio_cls = if std_selected { "selected" } else { "" };
        let native_card_cls = if native_selected { "selected" } else { "" };
        let native_radio_cls = if native_selected { "selected" } else { "" };
        let hybrid_card_cls = if hybrid_selected { "selected" } else { "" };
        let hybrid_radio_cls = if hybrid_selected { "selected" } else { "" };

        format!(
            r###"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Settings - Nexora</title>
    <style>
        {shared_css}

        body {{
            display: flex;
            height: 100vh;
            overflow: hidden;
            background-color: var(--bg-base);
        }}

        /* Two-Column Layout: Sidebar */
        .sidebar {{
            width: 230px;
            background: var(--bg-surface);
            border-right: 1px solid var(--border);
            display: flex;
            flex-direction: column;
            padding: 24px 12px;
            gap: 4px;
            flex-shrink: 0;
        }}

        .sidebar-brand {{
            display: flex;
            align-items: center;
            gap: 10px;
            padding: 0 10px 20px 10px;
            border-bottom: 1px solid var(--border-subtle);
            margin-bottom: 12px;
        }}

        .sidebar-brand h1 {{
            font-size: 15px;
            font-weight: 600;
            color: var(--text-primary);
        }}

        .nav-item {{
            display: flex;
            align-items: center;
            gap: 10px;
            padding: 8px 12px;
            font-size: 13px;
            font-weight: 500;
            color: var(--text-secondary);
            border-radius: var(--radius-sm);
            cursor: pointer;
            transition: all 0.12s ease;
        }}

        .nav-item:hover {{
            color: var(--text-primary);
            background: var(--hover-bg);
        }}

        .nav-item.active {{
            color: var(--text-primary);
            background: var(--bg-card);
            border: 1px solid var(--border);
        }}

        .nav-item.active svg {{
            stroke: var(--accent);
        }}

        /* Streaming Core Engine Cards */
        .engine-card {{
            background: var(--bg-surface);
            border: 1px solid var(--border);
            border-radius: var(--radius-md);
            padding: 20px;
            margin-bottom: 16px;
            cursor: pointer;
            transition: all 0.18s cubic-bezier(0.16, 1, 0.3, 1);
            position: relative;
        }}

        .engine-card:hover {{
            border-color: rgba(225, 29, 72, 0.4);
            background: var(--bg-card);
            transform: translateY(-1px);
        }}

        .engine-card.selected {{
            border-color: var(--accent);
            background: var(--bg-card);
            box-shadow: 0 0 0 1px var(--accent);
        }}

        .engine-header {{
            display: flex;
            align-items: flex-start;
            justify-content: space-between;
            margin-bottom: 12px;
            gap: 12px;
        }}

        .engine-title-wrap {{
            display: flex;
            align-items: center;
            gap: 12px;
        }}

        .radio-circle {{
            width: 18px;
            height: 18px;
            border-radius: 50%;
            border: 2px solid var(--border);
            display: flex;
            align-items: center;
            justify-content: center;
            flex-shrink: 0;
            transition: all 0.15s ease;
        }}

        .radio-circle.selected {{
            border-color: var(--accent);
        }}

        .radio-circle.selected::after {{
            content: '';
            width: 8px;
            height: 8px;
            border-radius: 50%;
            background: var(--accent);
        }}

        .engine-name {{
            font-size: 15px;
            font-weight: 600;
            color: var(--text-primary);
            display: flex;
            align-items: center;
            gap: 8px;
            flex-wrap: wrap;
        }}

        .engine-sub {{
            font-size: 12px;
            color: var(--text-secondary);
            margin-top: 2px;
        }}

        .engine-stat {{
            font-family: Consolas, monospace;
            font-size: 12px;
            font-weight: 600;
            color: #10b981;
            background: rgba(16, 185, 129, 0.1);
            border: 1px solid rgba(16, 185, 129, 0.25);
            padding: 4px 8px;
            border-radius: 4px;
            white-space: nowrap;
        }}

        .engine-desc {{
            font-size: 13px;
            line-height: 1.55;
            color: var(--text-secondary);
            margin-bottom: 14px;
        }}

        .engine-specs {{
            display: flex;
            flex-wrap: wrap;
            gap: 6px;
        }}

        .spec-tag {{
            font-size: 11px;
            font-weight: 500;
            background: #141418;
            border: 1px solid #22222a;
            color: var(--text-muted);
            padding: 2px 8px;
            border-radius: 4px;
        }}

        .badge-accent {{
            background: rgba(225, 29, 72, 0.15);
            color: #f43f5e;
            border: 1px solid rgba(225, 29, 72, 0.3);
            font-size: 11px;
            padding: 2px 7px;
            border-radius: 4px;
            font-weight: 600;
        }}

        /* Two-Column Layout: Content Area */
        .main-content {{
            flex: 1;
            display: flex;
            flex-direction: column;
            height: 100vh;
            overflow: hidden;
        }}

        .content-header {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 20px 36px;
            border-bottom: 1px solid var(--border-subtle);
            background: var(--bg-base);
        }}

        .search-bar-wrap {{
            position: relative;
            width: 260px;
        }}

        .search-bar-wrap svg {{
            position: absolute;
            left: 10px;
            top: 50%;
            transform: translateY(-50%);
            color: var(--text-muted);
            pointer-events: none;
        }}

        .search-bar-wrap input {{
            width: 100%;
            padding: 7px 10px 7px 32px;
            font-size: 13px;
            background: var(--bg-surface);
            border: 1px solid var(--border);
            border-radius: var(--radius-sm);
            color: var(--text-primary);
            outline: none;
        }}

        .search-bar-wrap input:focus {{
            border-color: var(--accent);
        }}

        .content-scroll {{
            flex: 1;
            overflow-y: auto;
            padding: 28px 36px 64px 36px;
        }}

        .settings-container {{
            max-width: 740px;
        }}

        .category-pane {{
            display: none;
        }}

        .category-pane.active {{
            display: block;
        }}

        .category-title {{
            font-size: 18px;
            font-weight: 600;
            color: var(--text-primary);
            margin-bottom: 4px;
        }}

        .category-desc {{
            font-size: 13px;
            color: var(--text-secondary);
            margin-bottom: 20px;
        }}

        /* Toast notification */
        .toast {{
            position: fixed;
            bottom: 24px;
            right: 24px;
            background: var(--bg-card);
            border: 1px solid var(--accent);
            color: var(--text-primary);
            padding: 10px 18px;
            border-radius: var(--radius-sm);
            font-size: 13px;
            font-weight: 500;
            display: none;
            box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
            z-index: 1000;
        }}
    </style>
</head>
<body>
    <!-- Sidebar -->
    <div class="sidebar">
        <div class="sidebar-brand">
            {logo_svg}
            <h1>Settings</h1>
            <span class="badge" style="margin-left: auto;">v{version}</span>
        </div>

        <div class="nav-item active" onclick="switchCategory('general', this)">
            {general_icon_svg} General
        </div>
        <div class="nav-item" onclick="switchCategory('performance', this)">
            {perf_icon_svg} Performance
        </div>
        <div class="nav-item" onclick="switchCategory('privacy', this)">
            {privacy_icon_svg} Privacy & Security
        </div>
        <div class="nav-item" onclick="switchCategory('streaming_core', this)">
            {streaming_icon_svg} Streaming Core
        </div>
        <div class="nav-item" onclick="switchCategory('video', this)">
            {video_icon_svg} Video & Media
        </div>
        <div class="nav-item" onclick="switchCategory('storage', this)">
            {storage_icon_svg} Storage & Cache
        </div>
        <div class="nav-item" onclick="switchCategory('about', this)">
            {about_icon_svg} About Nexora
        </div>
    </div>

    <!-- Main Content Area -->
    <div class="main-content">
        <div class="content-header">
            <div class="search-bar-wrap">
                {search_icon_svg}
                <input type="text" id="settings-filter" placeholder="Search settings..." oninput="filterSettings(this.value)" />
            </div>
            <div style="font-size: 12px; color: var(--text-muted);">
                Changes saved automatically
            </div>
        </div>

        <div class="content-scroll">
            <div class="settings-container">
                <!-- 1. GENERAL -->
                <div id="general" class="category-pane active">
                    <h2 class="category-title">General Settings</h2>
                    <p class="category-desc">Manage startup behavior, search provider, and default appearance.</p>

                    <div class="card">
                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Homepage URL</div>
                                <div class="setting-desc">Web address loaded when opening new tabs or launch</div>
                            </div>
                            <div style="display: flex; gap: 8px;">
                                <input type="text" id="homepage-input" class="input-text" style="width: 240px;" value="{homepage_escaped}" onchange="saveSetting('homepage', this.value)" />
                                <button class="btn btn-primary btn-sm" onclick="saveSetting('homepage', document.getElementById('homepage-input').value)">Save</button>
                                <button class="btn btn-ghost btn-sm" onclick="document.getElementById('homepage-input').value = 'nexora://home'; saveSetting('homepage', 'nexora://home');">Nexora Home</button>
                            </div>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Default Search Engine</div>
                                <div class="setting-desc">Search provider used for queries entered in the address bar</div>
                            </div>
                            <select class="select-input" onchange="saveSetting('default_search_engine', this.value)">
                                <option value="Google" {google_sel}>Google</option>
                                <option value="DuckDuckGo" {ddg_sel}>DuckDuckGo</option>
                                <option value="Bing" {bing_sel}>Microsoft Bing</option>
                                <option value="Brave" {brave_sel}>Brave Search</option>
                            </select>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Theme Appearance</div>
                                <div class="setting-desc">Choose between dark, light, or Windows system default</div>
                            </div>
                            <select class="select-input" onchange="saveSetting('theme', this.value)">
                                <option value="dark" {theme_dark_sel}>Dark (Default)</option>
                                <option value="light" {theme_light_sel}>Light</option>
                                <option value="system" {theme_sys_sel}>System Default</option>
                            </select>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Bookmarks Manager</div>
                                <div class="setting-desc">Organize and manage your saved bookmarks</div>
                            </div>
                            <button class="btn btn-ghost btn-sm" onclick="navigate('nexora://bookmarks')">Open Bookmarks</button>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Browsing History</div>
                                <div class="setting-desc">View and search your chronological browsing records</div>
                            </div>
                            <button class="btn btn-ghost btn-sm" onclick="navigate('nexora://history')">Open History</button>
                        </div>
                    </div>
                </div>

                <!-- 2. PERFORMANCE -->
                <div id="performance" class="category-pane">
                    <h2 class="category-title">Performance Settings</h2>
                    <p class="category-desc">Configure hardware acceleration, memory pressure limits, and tab suspension.</p>

                    <div class="card">
                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Smart Tab Suspension</div>
                                <div class="setting-desc">Suspend inactive background tabs under memory pressure to keep RAM usage low</div>
                            </div>
                            <label class="toggle">
                                <input type="checkbox" {smart_susp_checked} onchange="saveSetting('smart_suspension', this.checked)" />
                                <span class="toggle-slider"></span>
                            </label>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Inactivity Timeout</div>
                                <div class="setting-desc">Minutes of inactivity before background tabs become eligible for suspension</div>
                            </div>
                            <select class="select-input" onchange="saveSetting('suspension_timeout_minutes', parseInt(this.value))">
                                <option value="5" {t5_sel}>5 Minutes</option>
                                <option value="10" {t10_sel}>10 Minutes (Recommended)</option>
                                <option value="15" {t15_sel}>15 Minutes</option>
                                <option value="30" {t30_sel}>30 Minutes</option>
                                <option value="60" {t60_sel}>60 Minutes</option>
                            </select>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Direct3D 11 & NVDEC Acceleration</div>
                                <div class="setting-desc">Offload high-resolution video decoding directly to GPU to reduce host CPU load</div>
                            </div>
                            <label class="toggle">
                                <input type="checkbox" {hw_accel_checked} onchange="saveSetting('hardware_acceleration', this.checked)" />
                                <span class="toggle-slider"></span>
                            </label>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">System Diagnostics</div>
                                <div class="setting-desc">Inspect live memory, GPU decode status, and tab architecture metrics</div>
                            </div>
                            <button class="btn btn-ghost btn-sm" onclick="navigate('nexora://diagnostics')">Open Diagnostics</button>
                        </div>
                    </div>
                </div>

                <!-- 3. PRIVACY & SECURITY -->
                <div id="privacy" class="category-pane">
                    <h2 class="category-title">Privacy & Security</h2>
                    <p class="category-desc">Manage built-in ad blocking, tracker protection, and browsing data.</p>

                    <div class="card">
                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">High-Speed Ad Blocker</div>
                                <div class="setting-desc">Block intrusive advertising network requests while preserving video streaming CDNs</div>
                            </div>
                            <label class="toggle">
                                <input type="checkbox" {adblock_checked} onchange="saveSetting('ad_block_enabled', this.checked)" />
                                <span class="toggle-slider"></span>
                            </label>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Tracker Protection</div>
                                <div class="setting-desc">Block cross-site tracking beacons and profiling scripts</div>
                            </div>
                            <label class="toggle">
                                <input type="checkbox" {tracker_checked} onchange="saveSetting('tracker_protection_enabled', this.checked)" />
                                <span class="toggle-slider"></span>
                            </label>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Zero-Telemetry Hardening</div>
                                <div class="setting-desc">7 OS-level kill switches active: background pings, reporting, and cloud sync disabled</div>
                            </div>
                            <span class="badge badge-success">Enforced</span>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Clear Browsing History</div>
                                <div class="setting-desc">Delete all visit records and search queries from local database</div>
                            </div>
                            <button class="btn btn-ghost btn-sm" id="btn-clear-history" onclick="clearHistory()">Clear History</button>
                        </div>
                    </div>
                </div>

                <!-- 4. STREAMING CORE ARCHITECTURE -->
                <div id="streaming_core" class="category-pane">
                    <h2 class="category-title">Streaming Core Architecture</h2>
                    <p class="category-desc">Configure your preferred video playback engine. Choose between full web compatibility, dedicated hardware player, or hybrid dual-engine mode.</p>

                    <!-- Option 0: Standard Multi-Controller (Default) -->
                    <div id="card-engine-standard" class="engine-card {std_card_cls}" onclick="selectEngine('standard')">
                        <div class="engine-header">
                            <div class="engine-title-wrap">
                                <div class="radio-circle {std_radio_cls}"></div>
                                <div>
                                    <div class="engine-name">Standard Multi-Controller Engine <span class="badge badge-success">Default</span></div>
                                    <div class="engine-sub">Direct3D 11 & NVDEC GPU decode offload via WebView2 Evergreen</div>
                                </div>
                            </div>
                            <div class="engine-stat">~120 – 220 MB RAM</div>
                        </div>
                        <p class="engine-desc">
                            Integrates the full Chromium runtime with hardware Direct3D 11 GPU video decoding, NVDEC zero-copy surfaces, and Widevine DRM. Delivers complete website functionality including comments, live chat, recommendations, and Google account login.
                        </p>
                        <div class="engine-specs">
                            <span class="spec-tag">Direct3D 11 NVDEC</span>
                            <span class="spec-tag">Widevine CDM Certified</span>
                            <span class="spec-tag">Full Web Comments & Live Chat</span>
                            <span class="spec-tag">Multi-Controller Architecture</span>
                        </div>
                    </div>

                    <!-- Option 1: Native D3D11/WMF Video Player Engine -->
                    <div id="card-engine-native_d3d11" class="engine-card {native_card_cls}" onclick="selectEngine('native_d3d11')">
                        <div class="engine-header">
                            <div class="engine-title-wrap">
                                <div class="radio-circle {native_radio_cls}"></div>
                                <div>
                                    <div class="engine-name">Option 1: Native D3D11 / WMF Video Player Engine <span class="badge-accent">&lt; 60 MB RAM</span></div>
                                    <div class="engine-sub">Direct stream extraction to Windows Media Foundation & NVDEC hardware decoder</div>
                                </div>
                            </div>
                            <div class="engine-stat">~35 – 55 MB RAM</div>
                        </div>
                        <p class="engine-desc">
                            Bypasses the bloated website DOM and JavaScript execution engine entirely. Queries YouTube streaming endpoints to feed raw video/audio DASH streams directly to Windows Media Foundation (WMF) and your GPU's Direct3D 11 NVDEC hardware decoder. Zero web ads, instant playback launch, and zero frame drops.
                        </p>
                        <div class="engine-specs">
                            <span class="spec-tag">Zero Chromium DOM</span>
                            <span class="spec-tag">Zero V8 JS Overhead</span>
                            <span class="spec-tag">~1 – 3% CPU @ 1080p60</span>
                            <span class="spec-tag">Clean Native Player UI</span>
                            <span class="spec-tag">Zero Web Ads</span>
                        </div>
                    </div>

                    <!-- Option 2: Hybrid Stealth Core -->
                    <div id="card-engine-hybrid_stealth" class="engine-card {hybrid_card_cls}" onclick="selectEngine('hybrid_stealth')">
                        <div class="engine-header">
                            <div class="engine-title-wrap">
                                <div class="radio-circle {hybrid_radio_cls}"></div>
                                <div>
                                    <div class="engine-name">Option 2: Hybrid Stealth Core <span class="badge badge-success">Smart Dual Mode</span></div>
                                    <div class="engine-sub">Intelligent coordinator: web browsing in WebView2, cinema playback in Native D3D11</div>
                                </div>
                            </div>
                            <div class="engine-stat">~50 – 65 MB RAM</div>
                        </div>
                        <p class="engine-desc">
                            Operates in dual coordinated modes: general browsing and YouTube channel browsing stay in the fast multi-controller WebView2 engine. When you navigate to a video, Nexora intercepts the stream, hands off playback to the Native D3D11 hardware player, and places the heavy webview tab into dormant suspension.
                        </p>
                        <div class="engine-specs">
                            <span class="spec-tag">Dual-Engine Coordinator</span>
                            <span class="spec-tag">Dormant Tab Suspension</span>
                            <span class="spec-tag">Browse YouTube Normally</span>
                            <span class="spec-tag">One-Click Return to Web</span>
                            <span class="spec-tag">Drops RAM by ~80%</span>
                        </div>
                    </div>
                </div>

                <!-- 4. VIDEO & MEDIA -->
                <div id="video" class="category-pane">
                    <h2 class="category-title">Video & Media Engine</h2>
                    <p class="category-desc">Manage DRM pipelines, media priority boost, and picture-in-picture.</p>

                    <div class="card">
                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Legitimate DRM Pipelines</div>
                                <div class="setting-desc">Widevine CDM and PlayReady EME supported natively via Windows OS runtime</div>
                            </div>
                            <span class="badge badge-success">Active & Certified</span>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Active Media Priority Boost</div>
                                <div class="setting-desc">Boost process priority during active audio/video playback to eliminate frame drops</div>
                            </div>
                            <span class="badge badge-success">Enabled</span>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Picture-in-Picture (PiP)</div>
                                <div class="setting-desc">Seamless floating video window with 16:9 aspect ratio and always-on-top positioning</div>
                            </div>
                            <span class="badge">Ready (Ctrl+P)</span>
                        </div>
                    </div>
                </div>

                <!-- 5. STORAGE & CACHE -->
                <div id="storage" class="category-pane">
                    <h2 class="category-title">Storage & Cache Limits</h2>
                    <p class="category-desc">Manage local disk footprints and SQLite database integrity.</p>

                    <div class="card">
                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Disk Cache Limit</div>
                                <div class="setting-desc">Maximum disk space allocated for web resource caching</div>
                            </div>
                            <select class="select-input" onchange="saveSetting('disk_cache_limit_mb', parseInt(this.value))">
                                <option value="64" {dc64_sel}>64 MB</option>
                                <option value="128" {dc128_sel}>128 MB</option>
                                <option value="256" {dc256_sel}>256 MB (Default)</option>
                                <option value="512" {dc512_sel}>512 MB</option>
                            </select>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Media Cache Limit</div>
                                <div class="setting-desc">Maximum disk space allocated for streaming video buffer chunks</div>
                            </div>
                            <select class="select-input" onchange="saveSetting('media_cache_limit_mb', parseInt(this.value))">
                                <option value="32" {mc32_sel}>32 MB</option>
                                <option value="64" {mc64_sel}>64 MB (Default)</option>
                                <option value="128" {mc128_sel}>128 MB</option>
                                <option value="256" {mc256_sel}>256 MB</option>
                            </select>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Compact SQLite Database WAL</div>
                                <div class="setting-desc">Flush write-ahead logs and truncate database file to reclaim disk space</div>
                            </div>
                            <button class="btn btn-ghost btn-sm" id="btn-compact-wal" onclick="compactWal()">Compact Database</button>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Clear Engine Cache</div>
                                <div class="setting-desc">Delete temporary web assets and cookies from disk cache</div>
                            </div>
                            <button class="btn btn-ghost btn-sm" id="btn-clear-cache" onclick="clearCache()">Clear Cache</button>
                        </div>
                    </div>
                </div>

                <!-- 6. ABOUT NEXORA -->
                <div id="about" class="category-pane">
                    <h2 class="category-title">About Nexora</h2>
                    <p class="category-desc">High-performance desktop browser engineered in Rust for Windows.</p>

                    <div class="card">
                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Nexora Desktop Browser</div>
                                <div class="setting-desc">Version {version} • Windows x86_64</div>
                            </div>
                            <span class="badge">Up to date</span>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Rendering Engine</div>
                                <div class="setting-desc">Microsoft WebView2 Evergreen with Direct3D 11 NVDEC Acceleration</div>
                            </div>
                            <span class="badge">Hardware Accelerated</span>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Reset to Defaults</div>
                                <div class="setting-desc">Restore all preferences and configurations to initial state</div>
                            </div>
                            <button class="btn btn-ghost btn-sm" style="color:var(--error);" onclick="resetDefaults()">Reset All Settings</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Notification Toast -->
    <div class="toast" id="toast"></div>

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

        function switchCategory(catId, clickedItem) {{
            document.querySelectorAll('.nav-item').forEach(el => el.classList.remove('active'));
            document.querySelectorAll('.category-pane').forEach(el => el.classList.remove('active'));
            const target = document.getElementById(catId);
            if (target) {{
                target.classList.add('active');
            }}
            if (clickedItem) {{
                clickedItem.classList.add('active');
            }}
        }}

        function saveSetting(key, value, suppressToast) {{
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage(JSON.stringify({{
                    action: "save_setting",
                    key: key,
                    value: value
                }}));
            }}
            if (!suppressToast) {{
                showToast("Setting saved automatically");
            }}
        }}

        function selectEngine(mode) {{
            saveSetting('streaming_engine_mode', mode, true);
            document.querySelectorAll('.engine-card').forEach(el => el.classList.remove('selected'));
            document.querySelectorAll('.radio-circle').forEach(el => el.classList.remove('selected'));
            const target = document.getElementById('card-engine-' + mode);
            if (target) {{
                target.classList.add('selected');
                const radio = target.querySelector('.radio-circle');
                if (radio) radio.classList.add('selected');
            }}
            let label = "Standard Multi-Controller (Default)";
            if (mode === 'native_d3d11' || mode === 'native') label = "Native D3D11 Video Player Engine (< 60 MB)";
            if (mode === 'hybrid_stealth' || mode === 'hybrid') label = "Hybrid Stealth Core (Dual Engine)";
            showToast("Streaming Core switched to: " + label);
        }}

        function clearHistory() {{
            if (confirm("Clear complete local browsing history?")) {{
                if (window.chrome && window.chrome.webview) {{
                    window.chrome.webview.postMessage(JSON.stringify({{
                        action: "clear_history"
                    }}));
                }}
                showToast("Browsing history cleared");
                const btn = document.getElementById('btn-clear-history');
                if (btn) btn.textContent = "Cleared";
            }}
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
                showToast("SQLite database compacted successfully");
            }}, 350);
        }}

        function clearCache() {{
            const btn = document.getElementById('btn-clear-cache');
            btn.textContent = "Clearing...";
            btn.disabled = true;
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage(JSON.stringify({{
                    action: "clear_cache"
                }}));
            }}
            setTimeout(() => {{
                btn.textContent = "Cleared";
                btn.disabled = false;
                showToast("Cache cleared successfully");
            }}, 350);
        }}

        function resetDefaults() {{
            if (confirm("Reset all browser settings back to factory defaults?")) {{
                if (window.chrome && window.chrome.webview) {{
                    window.chrome.webview.postMessage(JSON.stringify({{
                        action: "reset_defaults"
                    }}));
                }}
                showToast("Resetting to defaults...");
                setTimeout(() => location.reload(), 400);
            }}
        }}

        function filterSettings(query) {{
            const q = query.toLowerCase().trim();
            const rows = document.querySelectorAll('.setting-row');
            if (!q) {{
                rows.forEach(r => r.style.display = 'flex');
                return;
            }}
            rows.forEach(r => {{
                const text = r.textContent.toLowerCase();
                r.style.display = text.includes(q) ? 'flex' : 'none';
            }});
        }}
    </script>
</body>
</html>
            "###,
            shared_css = shared_css,
            version = version,
            general_icon_svg = general_icon_svg,
            perf_icon_svg = perf_icon_svg,
            privacy_icon_svg = privacy_icon_svg,
            video_icon_svg = video_icon_svg,
            storage_icon_svg = storage_icon_svg,
            about_icon_svg = about_icon_svg,
            search_icon_svg = search_icon_svg,
            logo_svg = logo_svg,
            homepage_escaped = homepage_escaped,
            google_sel = if default_engine == "Google" { "selected" } else { "" },
            ddg_sel = if default_engine == "DuckDuckGo" { "selected" } else { "" },
            bing_sel = if default_engine == "Bing" { "selected" } else { "" },
            brave_sel = if default_engine == "Brave" { "selected" } else { "" },
            theme_dark_sel = if theme_str == "dark" { "selected" } else { "" },
            theme_light_sel = if theme_str == "light" { "selected" } else { "" },
            theme_sys_sel = if theme_str == "system" { "selected" } else { "" },
            smart_susp_checked = smart_susp_checked,
            hw_accel_checked = hw_accel_checked,
            adblock_checked = adblock_checked,
            tracker_checked = tracker_checked,
            t5_sel = if susp_timeout == 5 { "selected" } else { "" },
            t10_sel = if susp_timeout == 10 { "selected" } else { "" },
            t15_sel = if susp_timeout == 15 { "selected" } else { "" },
            t30_sel = if susp_timeout == 30 { "selected" } else { "" },
            t60_sel = if susp_timeout == 60 { "selected" } else { "" },
            dc64_sel = if disk_cache == 64 { "selected" } else { "" },
            dc128_sel = if disk_cache == 128 { "selected" } else { "" },
            dc256_sel = if disk_cache == 256 { "selected" } else { "" },
            dc512_sel = if disk_cache == 512 { "selected" } else { "" },
            mc32_sel = if media_cache == 32 { "selected" } else { "" },
            mc64_sel = if media_cache == 64 { "selected" } else { "" },
            mc128_sel = if media_cache == 128 { "selected" } else { "" },
            mc256_sel = if media_cache == 256 { "selected" } else { "" },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_settings_html_contains_all_tabs() {
        let config = NexoraConfig::default();
        let html = SettingsUi::render_html(&config);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("General Settings"));
        assert!(html.contains("Performance Settings"));
        assert!(html.contains("Privacy & Security"));
        assert!(html.contains("Video & Media Engine"));
        assert!(html.contains("Storage & Cache Limits"));
        assert!(html.contains("Streaming Core"));
        assert!(html.contains("About Nexora"));
        assert!(html.contains("search-bar-wrap"));
    }
}
