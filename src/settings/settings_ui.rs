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
        let vertical_tabs_checked = if config.vertical_tabs { "checked" } else { "" };
        let mem_gov_checked = if config.memory_governor.enabled { "checked" } else { "" };
        let trim_ws_checked = if config.memory_governor.auto_trim_working_set { "checked" } else { "" };
        let pressure_thresh = config.memory_governor.pressure_threshold_percent;
        let susp_timeout = config.suspension_timeout_minutes;
        let disk_cache = config.disk_cache_limit_mb;
        let media_cache = config.media_cache_limit_mb;
        let version = BrandIdentity::version();

        let custom_rules_content = crate::app::application::ApplicationPaths::resolve()
            .ok()
            .and_then(|p| std::fs::read_to_string(p.app_data_dir.join("custom_rules.txt")).ok())
            .unwrap_or_default();
        let custom_rules_escaped = html_escape(&custom_rules_content);

        let general_icon_svg = LucideIcon::Sliders.to_svg(16, 2.0, "currentColor");
        let perf_icon_svg = LucideIcon::Cpu.to_svg(16, 2.0, "currentColor");
        let privacy_icon_svg = LucideIcon::ShieldCheck.to_svg(16, 2.0, "currentColor");
        let streaming_icon_svg = LucideIcon::Layers.to_svg(16, 2.0, "currentColor");
        let video_icon_svg = LucideIcon::Volume2.to_svg(16, 2.0, "currentColor");
        let storage_icon_svg = LucideIcon::HardDrive.to_svg(16, 2.0, "currentColor");
        let about_icon_svg = LucideIcon::Sparkles.to_svg(16, 2.0, "currentColor");
        let search_icon_svg = LucideIcon::Search.to_svg(14, 2.0, "currentColor");
        let refresh_icon_svg = LucideIcon::RefreshCw.to_svg(20, 2.0, "currentColor");
        let logo_svg = BrandIdentity::monogram_svg(24);
        let shared_css = DesignTokens::shared_css(true);

        let streaming_mode = config.streaming_engine_mode;
        let std_selected = streaming_mode == crate::app::config::StreamingEngineMode::Standard;
        let ultra_selected = streaming_mode == crate::app::config::StreamingEngineMode::Ultralight;
        let servo_selected = streaming_mode == crate::app::config::StreamingEngineMode::Servo;

        let std_card_cls = if std_selected { "selected" } else { "" };
        let std_radio_cls = if std_selected { "selected" } else { "" };
        let ultra_card_cls = if ultra_selected { "selected" } else { "" };
        let ultra_radio_cls = if ultra_selected { "selected" } else { "" };
        let servo_card_cls = if servo_selected { "selected" } else { "" };
        let servo_radio_cls = if servo_selected { "selected" } else { "" };

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

        /* Restart Modal Popup */
        .modal-overlay {{
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.72);
            backdrop-filter: blur(8px);
            display: none;
            align-items: center;
            justify-content: center;
            z-index: 2000;
        }}

        .modal-overlay.active {{
            display: flex;
        }}

        .modal-card {{
            background: #121216;
            border: 1px solid rgba(225, 29, 72, 0.35);
            box-shadow: 0 20px 50px rgba(0, 0, 0, 0.75), 0 0 24px rgba(225, 29, 72, 0.12);
            border-radius: 12px;
            width: 90%;
            max-width: 480px;
            padding: 24px;
            color: var(--text-primary);
        }}

        .modal-header {{
            display: flex;
            align-items: center;
            gap: 14px;
            margin-bottom: 16px;
        }}

        .modal-icon-wrap {{
            width: 40px;
            height: 40px;
            border-radius: 10px;
            background: rgba(225, 29, 72, 0.15);
            border: 1px solid rgba(225, 29, 72, 0.35);
            color: #f43f5e;
            display: flex;
            align-items: center;
            justify-content: center;
            flex-shrink: 0;
        }}

        .modal-title-wrap {{
            flex: 1;
        }}

        .modal-title {{
            font-size: 16px;
            font-weight: 700;
            color: var(--text-primary);
        }}

        .modal-subtitle {{
            font-size: 12px;
            color: var(--text-secondary);
            margin-top: 2px;
        }}

        .modal-close-btn {{
            background: transparent;
            border: none;
            color: var(--text-muted);
            font-size: 22px;
            cursor: pointer;
            padding: 4px;
            line-height: 1;
            transition: color 0.15s ease;
        }}

        .modal-close-btn:hover {{
            color: var(--text-primary);
        }}

        .modal-body {{
            font-size: 13px;
            line-height: 1.6;
            color: var(--text-secondary);
            margin-bottom: 22px;
        }}

        .modal-body strong {{
            color: var(--text-primary);
        }}

        .modal-info-box {{
            margin-top: 12px;
            background: rgba(255, 255, 255, 0.03);
            border: 1px solid #22222a;
            border-radius: 8px;
            padding: 12px 14px;
            font-size: 12px;
            color: var(--text-muted);
        }}

        .modal-footer {{
            display: flex;
            justify-content: flex-end;
            gap: 10px;
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
                                <div class="setting-title">Vertical Tabs Bar</div>
                                <div class="setting-desc">Switch from top horizontal tab bar to an Arc/Edge style vertical sidebar</div>
                            </div>
                            <label class="toggle">
                                <input type="checkbox" {vertical_tabs_checked} onchange="saveSetting('vertical_tabs', this.checked); openRestartModal('Vertical Tabs Layout');" />
                                <span class="toggle-slider"></span>
                            </label>
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
                                <input type="checkbox" {hw_accel_checked} onchange="saveSetting('hardware_acceleration', this.checked); openRestartModal('Hardware Acceleration');" />
                                <span class="toggle-slider"></span>
                            </label>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Automated Memory Governor</div>
                                <div class="setting-desc">Proactively hibernates idle background tabs when system free RAM drops below threshold</div>
                            </div>
                            <label class="toggle">
                                <input type="checkbox" {mem_gov_checked} onchange="saveSetting('memory_governor_enabled', this.checked)" />
                                <span class="toggle-slider"></span>
                            </label>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Free RAM Headroom Threshold</div>
                                <div class="setting-desc">Minimum system free RAM percentage before tab hibernation activates</div>
                            </div>
                            <select class="select-input" onchange="saveSetting('pressure_threshold_percent', parseInt(this.value))">
                                <option value="10" {pt10_sel}>10% Free RAM</option>
                                <option value="15" {pt15_sel}>15% Free RAM (Recommended)</option>
                                <option value="20" {pt20_sel}>20% Free RAM</option>
                                <option value="25" {pt25_sel}>25% Free RAM</option>
                            </select>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Auto Working-Set Trim</div>
                                <div class="setting-desc">Flush unused physical RAM pages to Windows Memory Manager after tab hibernation</div>
                            </div>
                            <label class="toggle">
                                <input type="checkbox" {trim_ws_checked} onchange="saveSetting('auto_trim_working_set', this.checked)" />
                                <span class="toggle-slider"></span>
                            </label>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">ANGLE Graphics Backend</div>
                                <div class="setting-desc">DirectX/Vulkan translation layer for hardware accelerated rasterization</div>
                            </div>
                            <select class="select-input" onchange="saveSetting('angle_backend', this.value); openRestartModal('ANGLE Backend: ' + this.value);">
                                <option value="default" {ang_def_sel}>Default (Auto-Detect)</option>
                                <option value="d3d11" {ang_d3d11_sel}>Direct3D 11 (Recommended)</option>
                                <option value="d3d11on12" {ang_d3d12_sel}>Direct3D 11on12</option>
                                <option value="d3d9" {ang_d3d9_sel}>Direct3D 9 (Legacy)</option>
                                <option value="vulkan" {ang_vulkan_sel}>Vulkan</option>
                                <option value="warp" {ang_warp_sel}>WARP (Software Fallback)</option>
                            </select>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Rasterization Threads</div>
                                <div class="setting-desc">Number of parallel worker threads allocated for GPU tile rasterization</div>
                            </div>
                            <select class="select-input" onchange="saveSetting('raster_threads', parseInt(this.value)); openRestartModal('Raster Threads: ' + this.value);">
                                <option value="0" {rt0_sel}>Auto (Default)</option>
                                <option value="1" {rt1_sel}>1 Thread (Low Overhead)</option>
                                <option value="2" {rt2_sel}>2 Threads</option>
                                <option value="4" {rt4_sel}>4 Threads</option>
                                <option value="8" {rt8_sel}>8 Threads (High Performance)</option>
                            </select>
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

                        <div class="setting-row" style="flex-direction: column; align-items: stretch; gap: 8px;">
                            <div class="setting-info">
                                <div class="setting-title">Custom AdBlock Filter Rules</div>
                                <div class="setting-desc">Import custom filter rules using ABP/uBlock syntax (one per line: <code>||domain^</code>, <code>@@||whitelist^</code>, <code>/path/*</code>)</div>
                            </div>
                            <textarea id="custom-adblock-rules" class="input-text" style="width: 100%; height: 110px; font-family: Consolas, monospace; font-size: 12px; resize: vertical;" placeholder="||ads.example.com^&#10;@@||safe-site.com^&#10;/tracking/pixel/*">{custom_rules_escaped}</textarea>
                            <div style="display: flex; justify-content: flex-end; gap: 8px;">
                                <button class="btn btn-primary btn-sm" onclick="saveCustomAdblockRules()">Apply Custom Rules</button>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- 4. STREAMING CORE ARCHITECTURE -->
                <div id="streaming_core" class="category-pane">
                    <h2 class="category-title">Streaming Core Architecture</h2>
                    <p class="category-desc">Configure your preferred browser engine architecture. Choose between full web compatibility, lightweight WebKit, or pure Rust rendering.</p>

                    <!-- Option 0: Standard Multi-Controller (Default) -->
                    <div id="card-engine-standard" class="engine-card {std_card_cls}" onclick="selectEngine('standard')">
                        <div class="engine-header">
                            <div class="engine-title-wrap">
                                <div class="radio-circle {std_radio_cls}"></div>
                                <div>
                                    <div class="engine-name">Standard Multi-Controller Engine <span class="badge badge-success">Default</span></div>
                                    <div class="engine-sub">Direct3D 11 &amp; NVDEC GPU decode offload via WebView2 Evergreen</div>
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
                            <span class="spec-tag">Full Web Comments &amp; Live Chat</span>
                            <span class="spec-tag">Multi-Controller Architecture</span>
                        </div>
                    </div>

                    <!-- Option 1: Ultralight (Lightweight WebKit) -->
                    <div id="card-engine-ultralight" class="engine-card {ultra_card_cls}" onclick="selectEngine('ultralight')">
                        <div class="engine-header">
                            <div class="engine-title-wrap">
                                <div class="radio-circle {ultra_radio_cls}"></div>
                                <div>
                                    <div class="engine-name">Option 1: Ultralight (Lightweight WebKit) <span class="badge-accent">~15 – 35 MB RAM</span></div>
                                    <div class="engine-sub">+35 – 50 MB binary • ~65% Compatibility • Basic HTML5/CSS • Game UI Engine</div>
                                </div>
                            </div>
                            <div class="engine-stat">~15 – 35 MB RAM</div>
                        </div>
                        <p class="engine-desc">
                            Built for ultra-lightweight game user interfaces (EA, Unreal Engine). Minimal RAM footprint and rapid initialization for low-spec systems. Renders basic HTML5/CSS and article reading, but lacks DRM support (cannot play protected YouTube/Netflix video) and heavy SPAs (Google Docs, Discord) may glitch or crash.
                        </p>
                        <div class="engine-specs">
                            <span class="spec-tag">WebKit C++ Core</span>
                            <span class="spec-tag">~15 – 35 MB Host RAM</span>
                            <span class="spec-tag">+35 – 50 MB Binary Size</span>
                            <span class="spec-tag">~65% Web Compatibility</span>
                            <span class="spec-tag">Basic HTML5 &amp; CSS</span>
                            <span class="spec-tag">Zero DRM Support</span>
                        </div>
                    </div>

                    <!-- Option 2: Servo (Pure Rust Web Engine) -->
                    <div id="card-engine-servo" class="engine-card {servo_card_cls}" onclick="selectEngine('servo')">
                        <div class="engine-header">
                            <div class="engine-title-wrap">
                                <div class="radio-circle {servo_radio_cls}"></div>
                                <div>
                                    <div class="engine-name">Option 2: Servo (Pure Rust Web Engine) <span class="badge badge-success">~25 – 45 MB RAM</span></div>
                                    <div class="engine-sub">+40 – 60 MB binary • ~45% Compatibility • WebRender GPU Compositor • Pure Rust</div>
                                </div>
                            </div>
                            <div class="engine-stat">~25 – 45 MB RAM</div>
                        </div>
                        <p class="engine-desc">
                            Written completely in Rust with parallel WebRender GPU compositor and memory-safe DOM execution. Highly optimized memory footprint without Chromium baggage. Experimental CSS and DOM engine under active development; not yet daily-driver ready (cannot authenticate Google accounts or render modern dynamic web apps reliably).
                        </p>
                        <div class="engine-specs">
                            <span class="spec-tag">Pure Rust Core</span>
                            <span class="spec-tag">WebRender Pipeline</span>
                            <span class="spec-tag">~25 – 45 MB Host RAM</span>
                            <span class="spec-tag">+40 – 60 MB Binary Size</span>
                            <span class="spec-tag">~45% Web Compatibility</span>
                            <span class="spec-tag">Experimental CSS / DOM</span>
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
                            <span class="badge">v{version}</span>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Rendering Engine Core</div>
                                <div class="setting-desc">Multi-Controller Architecture with Direct3D 11 NVDEC &amp; Modular Engine Support</div>
                            </div>
                            <span class="badge badge-success">Hardware Accelerated</span>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Software Updates</div>
                                <div class="setting-desc">Check GitHub releases for the latest official build</div>
                            </div>
                            <div style="display: flex; align-items: center; gap: 10px;">
                                <span id="update-status-badge" class="badge">v{version}</span>
                                <button class="btn btn-primary btn-sm" id="btn-check-updates" onclick="checkForUpdates()">Check for Updates</button>
                            </div>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Official GitHub Repository</div>
                                <div class="setting-desc">Source code, release notes, bug tracker, and roadmap</div>
                            </div>
                            <a href="https://github.com/aksh120/Nexora" target="_blank" class="btn btn-ghost btn-sm" style="color:var(--text-main); text-decoration:none; border:1px solid var(--border-color); display:inline-flex; align-items:center; gap:6px;">
                                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path></svg>
                                github.com/aksh120/Nexora
                            </a>
                        </div>

                        <div class="setting-row">
                            <div class="setting-info">
                                <div class="setting-title">Created &amp; Maintained By</div>
                                <div class="setting-desc">Akshat Apoorv • Lead Systems &amp; Browser Architecture</div>
                            </div>
                            <a href="https://github.com/aksh120" target="_blank" class="btn btn-ghost btn-sm" style="color:var(--accent-color); text-decoration:none; border:1px solid var(--border-color); display:inline-flex; align-items:center; gap:6px;">
                                github.com/aksh120
                            </a>
                        </div>

                        <div class="setting-row" style="flex-direction: column; align-items: flex-start; gap: 10px;">
                            <div class="setting-info">
                                <div class="setting-title">Support &amp; Assistance Inquiries</div>
                                <div class="setting-desc">For technical support, reporting issues, or any inquiries, contact directly:</div>
                            </div>
                            <div style="display: flex; flex-wrap: wrap; gap: 8px; width: 100%;">
                                <a href="mailto:hi@akshatcodes.xyz" class="badge" style="text-decoration:none; color:var(--text-main); font-family:var(--font-mono); font-size:12px; padding:6px 12px; background:var(--bg-tertiary); border:1px solid var(--border-color);">
                                    ✉ hi@akshatcodes.xyz
                                </a>
                                <a href="mailto:hi@akimbolabs.site" class="badge" style="text-decoration:none; color:var(--text-main); font-family:var(--font-mono); font-size:12px; padding:6px 12px; background:var(--bg-tertiary); border:1px solid var(--border-color);">
                                    ✉ hi@akimbolabs.site
                                </a>
                                <a href="mailto:main.akimbo@gmail.com" class="badge" style="text-decoration:none; color:var(--text-main); font-family:var(--font-mono); font-size:12px; padding:6px 12px; background:var(--bg-tertiary); border:1px solid var(--border-color);">
                                    ✉ main.akimbo@gmail.com
                                </a>
                            </div>
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

    <!-- Restart Required Modal Popup -->
    <div id="restart-modal" class="modal-overlay">
        <div class="modal-card">
            <div class="modal-header">
                <div class="modal-icon-wrap">
                    {refresh_icon_svg}
                </div>
                <div class="modal-title-wrap">
                    <div class="modal-title">Restart Required</div>
                    <div class="modal-subtitle">Configuration Change</div>
                </div>
                <button class="modal-close-btn" onclick="closeRestartModal()">&times;</button>
            </div>
            <div class="modal-body">
                <p id="restart-modal-msg">Setting updated: <strong>Native D3D11 Video Player Engine</strong>.</p>
                <div class="modal-info-box">
                    Please restart Nexora for hardware video swapchains, GPU raster thread pools, or UI layouts to take full effect.
                </div>
            </div>
            <div class="modal-footer">
                <button class="btn btn-ghost btn-sm" onclick="closeRestartModal()">Restart Later</button>
                <button class="btn btn-primary btn-sm" onclick="restartBrowser()">Restart Now</button>
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

        function openRestartModal(label) {{
            const modal = document.getElementById('restart-modal');
            const msg = document.getElementById('restart-modal-msg');
            if (msg) {{
                msg.innerHTML = "Setting updated: <strong>" + label + "</strong>.";
            }}
            if (modal) {{
                modal.classList.add('active');
            }}
        }}

        function checkForUpdates() {{
            const btn = document.getElementById('btn-check-updates');
            const badge = document.getElementById('update-status-badge');
            if (btn) btn.textContent = "Checking...";
            if (badge) badge.textContent = "Checking GitHub...";
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage(JSON.stringify({{
                    action: "check_for_updates"
                }}));
            }}
            setTimeout(() => {{
                if (btn) btn.textContent = "Check for Updates";
                if (badge) badge.textContent = "v{version}";
            }}, 4000);
        }}

        function closeRestartModal() {{
            const modal = document.getElementById('restart-modal');
            if (modal) {{
                modal.classList.remove('active');
            }}
        }}

        function restartBrowser() {{
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage(JSON.stringify({{
                    action: "restart_browser"
                }}));
            }} else {{
                window.location.reload();
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
            if (mode === 'ultralight' || mode === 'ultra') label = "Option 1: Ultralight - Lightweight WebKit (~15 – 35 MB)";
            if (mode === 'servo') label = "Option 2: Servo - Pure Rust Web Engine (~25 – 45 MB)";
            showToast("Streaming Core switched to: " + label);
            openRestartModal(label);
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

        function saveCustomAdblockRules() {{
            const text = document.getElementById('custom-adblock-rules').value;
            if (window.chrome && window.chrome.webview) {{
                window.chrome.webview.postMessage(JSON.stringify({{
                    action: "save_custom_adblock_rules",
                    value: text
                }}));
            }}
            showToast("Custom AdBlock rules applied");
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
            refresh_icon_svg = refresh_icon_svg,
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
            vertical_tabs_checked = vertical_tabs_checked,
            mem_gov_checked = mem_gov_checked,
            trim_ws_checked = trim_ws_checked,
            pt10_sel = if pressure_thresh == 10 { "selected" } else { "" },
            pt15_sel = if pressure_thresh == 15 { "selected" } else { "" },
            pt20_sel = if pressure_thresh == 20 { "selected" } else { "" },
            pt25_sel = if pressure_thresh == 25 { "selected" } else { "" },
            ang_def_sel = if config.angle_backend == crate::app::config::AngleBackend::Default { "selected" } else { "" },
            ang_d3d11_sel = if config.angle_backend == crate::app::config::AngleBackend::D3D11 { "selected" } else { "" },
            ang_d3d12_sel = if config.angle_backend == crate::app::config::AngleBackend::D3D11on12 { "selected" } else { "" },
            ang_d3d9_sel = if config.angle_backend == crate::app::config::AngleBackend::D3D9 { "selected" } else { "" },
            ang_vulkan_sel = if config.angle_backend == crate::app::config::AngleBackend::Vulkan { "selected" } else { "" },
            ang_warp_sel = if config.angle_backend == crate::app::config::AngleBackend::Warp { "selected" } else { "" },
            rt0_sel = if config.raster_threads == 0 { "selected" } else { "" },
            rt1_sel = if config.raster_threads == 1 { "selected" } else { "" },
            rt2_sel = if config.raster_threads == 2 { "selected" } else { "" },
            rt4_sel = if config.raster_threads == 4 { "selected" } else { "" },
            rt8_sel = if config.raster_threads == 8 { "selected" } else { "" },
            custom_rules_escaped = custom_rules_escaped,
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
        assert!(html.contains("Ultralight (Lightweight WebKit)"));
        assert!(html.contains("Servo (Pure Rust Web Engine)"));
        assert!(html.contains("https://github.com/aksh120/Nexora"));
        assert!(html.contains("Akshat Apoorv"));
        assert!(html.contains("hi@akshatcodes.xyz"));
        assert!(html.contains("hi@akimbolabs.site"));
        assert!(html.contains("main.akimbo@gmail.com"));
        assert!(html.contains("search-bar-wrap"));
        assert!(html.contains("restart-modal"));
    }
}
