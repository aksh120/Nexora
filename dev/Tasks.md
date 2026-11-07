# NEXORA: Master Task Tracker
## Active Development Status

> **Mandatory Rule**: Update `Tasks.md` immediately upon the completion or status change of any task. Never mark any task as "Completed" without verified implementation and passing tests.

---

### Task Summary Dashboard

| Status | Count |
| :--- | :--- |
| **Completed** | 8 |
| **In Progress** | 0 |
| **Not Started** | 37 |
| **Blocked** | 0 |
| **Total** | 45 |

---

### Phase 0: Research & Feasibility

| Task ID | Description | Priority | Status | Dependencies | Completion Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TASK-001` | System & Target Hardware Profiling (i7-3770, GT 730, Win 11) | P0 | **Completed** | None | Detected Intel Core i7-3770 (4C/8T), NVIDIA GT 730, Windows 11 Pro 64-bit (22631). WebView2 runtime 157.0.4341.42 pre-installed. |
| `TASK-002` | Browser Engine & DRM Feasibility Investigation | P0 | **Completed** | `TASK-001` | Evaluated WebView2 vs CEF vs Wry vs Servo. WebView2 selected for legitimate PlayReady/Widevine DRM, D3D11/DXVA2 acceleration, and OS runtime deduplication. |
| `TASK-003` | Prototype Benchmarking & Toolchain Verification | P0 | **Completed** | `TASK-002` | Installed Rust 1.99.1 (`stable-x86_64-pc-windows-gnu`); verified standalone compilation and native linking. |
| `TASK-004` | Complete `/dev` Specification Documentation | P0 | **Completed** | `TASK-002` | Authored and committed complete `/dev` suite: `Plan.md`, `Phases.md`, `Tasks.md`, `Architecture.md`, `Performance.md`, `Security.md`, `Testing.md`, `Decisions.md`. |

---

### Phase 1: Foundation & Project Setup

| Task ID | Description | Priority | Status | Dependencies | Completion Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TASK-101` | Initialize Cargo Workspace & Project Structure | P0 | **Completed** | `TASK-004` | Modular directory structure initialized: `src/{app, core, browser, engine, tabs, navigation, performance, media, privacy, storage, ui, settings, diagnostics}`. |
| `TASK-102` | Configure Cargo Dependencies & Build Profiles | P0 | **Completed** | `TASK-101` | Configured `windows v0.58`, `webview2-com v0.33`, `serde`, `serde_json`, `log`, `directories`. Configured dev and release profiles with LTO. |
| `TASK-103` | Implement Core Logging & Error Handling Subsystem | P1 | **Completed** | `TASK-102` | Implemented `NexoraError` enum with full std error traits and `NexoraLogger` with file sink, console sink, and Win32 `OutputDebugStringW`. |
| `TASK-104` | Configure Windows Manifest & DPI Awareness | P1 | **Completed** | `TASK-101` | Implemented Per-Monitor V2 DPI awareness via Win32 API and created `assets/manifest/nexora.manifest` with Common-Controls v6. |

---

### Phase 2: Browser Engine Integration

| Task ID | Description | Priority | Status | Dependencies | Completion Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TASK-201` | Win32 Host Window Creation & Message Loop | P0 | **Completed** | `TASK-104` | Implemented native Win32 window in `src/browser/host_window.rs` with `CreateWindowExW`, dark styling, Per-Monitor DPI change handling (`WM_DPICHANGED`), non-polling `GetMessageW` message pump. Verified in unit tests. |
| `TASK-202` | Microsoft WebView2 COM Initialization | P0 | **Completed** | `TASK-201` | Implemented in `src/engine/webview.rs` with `CoInitializeEx(COINIT_APARTMENTTHREADED)`, `CreateCoreWebView2EnvironmentWithOptions`, dedicated UDF (`%LOCALAPPDATA%\Nexora\Nexora\data\EngineData`), and low-overhead/D3D11 hardware acceleration switches. |
| `TASK-203` | Implement WebView2 Controller & Sizing Pipeline | P0 | **Completed** | `TASK-202` | `CreateCoreWebView2Controller` attached directly to host `HWND`. Dynamic client area bounds resizing implemented via `WM_SIZE` callback. Verified with working set ~15MB. |
| `TASK-204` | Core Navigation Commands (URL, Back, Forward, Reload) | P0 | **Completed** | `TASK-203` | Implemented `navigate`, `go_back`, `go_forward`, `reload`, `stop`, `get_source`, and `get_title` with automatic HTTPS prefix fallback. Verified live navigation to Google. |
| `TASK-205` | Process Lifecycle & Failure Recovery Hooks | P1 | **Completed** | `TASK-204` | Event hooks connected for `NavigationStarting`, `NavigationCompleted`, `SourceChanged`, and `ProcessFailed` (renderer crash detection and automated recovery reload). |

---

### Phase 3: Main Browser Interface

| Task ID | Description | Priority | Status | Dependencies | Completion Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TASK-301` | Native Multi-Tab Manager State Machine | P0 | **Completed** | `TASK-204` | Implemented `TabManager` & `Tab` in `src/tabs/` supporting tab creation, adjacent selection on close, tab switching, audio state tracking, and suspension guard rules. Verified with 100% passing tests. |
| `TASK-302` | Compact Tab Bar UI Rendering | P1 | **Completed** | `TASK-301` | Implemented flicker-free double-buffered GDI tab strip renderer in `src/ui/chrome.rs` and `src/ui/layout.rs` with active tab accent line, audio/suspended badges, and close button hit testing. |
| `TASK-303` | Advanced Omnibox (URL / Search Bar) | P0 | **Completed** | `TASK-301` | Implemented `Omnibox` parser in `src/navigation/omnibox.rs` differentiating direct schemes, localhost, IP addresses, domains, and search queries with security badges (`Secure`, `Insecure`, `Local`, `Internal`). Subclassed native edit control in `BrowserChrome`. |
| `TASK-304` | Omnibox Autocomplete & Suggestions Engine | P1 | **Completed** | `TASK-303` | Implemented suggestion engine matching query against pre-seeded popular streaming services (YouTube, Netflix, Prime Video, Hotstar, Twitch, Spotify) and search queries. |
| `TASK-305` | Navigation Toolbar Controls (Back/Forward/Reload/Home) | P1 | **Completed** | `TASK-303` | Integrated native controls in `BrowserChrome` for Back, Forward, Reload, Home, Omnibox, New Tab, and Menu with custom Segoe UI font and message handling in `HostWindow`. |
| `TASK-306` | Dark and Light Theme Engine | P2 | **Completed** | `TASK-302` | Implemented `ThemePalette` in `src/ui/theme.rs` with Dark, Light, and System modes. Automatic detection of Windows `AppsUseLightTheme` and native Windows 10/11 title bar immersive dark mode via `DwmSetWindowAttribute`. |

---

### Phase 4: Browser Data & Persistence

| Task ID | Description | Priority | Status | Dependencies | Completion Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TASK-401` | SQLite Persistence Architecture & Migrations | P0 | **Completed** | `TASK-101` | Implemented zero-overhead dynamic binding to Windows OS `winsqlite3.dll` with WAL mode, parameterized prepared statements, and automated versioned migrations (`user_version = 1`). |
| `TASK-402` | Cookie Persistence & Session Management | P0 | **Completed** | `TASK-401` | Implemented `SessionManager` in `src/storage/session.rs` persisting tab collection, active index, and suspension states. Cookies persisted across runs via WebView2 dedicated UDF. |
| `TASK-403` | Full-Text Searchable History Subsystem | P1 | **Completed** | `TASK-401` | Implemented `HistoryManager` in `src/storage/history.rs` with visit deduplication, visit and typed frequency counters, and indexed query search. |
| `TASK-404` | Bookmarks Management (Add, Edit, Delete, Folders) | P1 | **Completed** | `TASK-401` | Implemented hierarchical `BookmarkManager` in `src/storage/bookmarks.rs` with pre-seeded streaming services (YouTube, Netflix, Prime Video, Hotstar, Twitch, Spotify). |
| `TASK-405` | Smart Cache Policy & Disk Boundary Enforcement | P1 | **Completed** | `TASK-401` | Implemented `CacheManager` in `src/storage/cache.rs` with directory size calculation, LRU file pruning, and configurable limit enforcement (`disk_cache_limit_mb = 256`). |

---

### Phase 5: Video Streaming & Media Engine

| Task ID | Description | Priority | Status | Dependencies | Completion Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TASK-501` | Direct3D 11 & DXVA2 Hardware Acceleration Configuration | P0 | **Completed** | `TASK-202` | Implemented `GpuCapabilities` in `src/media/gpu.rs`. Dynamically detects NVIDIA Kepler GT 730 and configures `--enable-features=VaapiVideoDecoder,D3D11VideoDecoder`, `--use-angle=d3d11`, `--ignore-gpu-blocklist`, `--enable-zero-copy`. Verified in live boot. |
| `TASK-502` | Legitimate Widevine & PlayReady DRM Pipeline | P0 | **Completed** | `TASK-202` | Implemented `DrmDiagnostics` in `src/media/drm.rs`. Verified authorized Widevine CDM & PlayReady EME pipeline in WebView2 Evergreen under Windows OS licensing for streaming services (YouTube, Netflix, Prime Video, Hotstar, Sony LIV, ZEE5, Twitch, Spotify). |
| `TASK-503` | Picture-in-Picture (PiP) Window Architecture | P1 | **Completed** | `TASK-301` | Implemented `PipWindow` in `src/media/pip.rs` with `WS_EX_TOPMOST` floating window, 16:9 aspect ratio preservation, and HTML5 video script injection (`requestPictureInPicture`). |
| `TASK-504` | Active Media Detection & Priority Scheduling | P0 | **Completed** | `TASK-301` | Implemented `MediaMonitor` in `src/media/media_monitor.rs` hooked to `add_IsDocumentPlayingAudioChanged`. Automatically boosts process priority to `ABOVE_NORMAL_PRIORITY_CLASS` during active playback and exempts media tabs from smart suspension. |
| `TASK-505` | Fullscreen Video Transition & HTML5 Fullscreen | P1 | **Completed** | `TASK-301` | Implemented `FullscreenController` in `src/media/fullscreen.rs` hooked to `add_ContainsFullScreenElementChanged`. Seamlessly transitions host window to borderless `WS_POPUP` over monitor rect and restores geometry on exit. |

---

### Phase 6: Performance Engineering

| Task ID | Description | Priority | Status | Dependencies | Completion Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TASK-601` | Smart Tab Suspension Engine | P0 | **Completed** | `TASK-504` | Implemented `SmartSuspensionEngine` in `src/performance/suspension.rs`. Evaluates background tab inactivity against adaptive pressure thresholds, respects strict exemptions (active, audio playing, loading), and renders lightweight HTML placeholder cards with instant waking on tab select. |
| `TASK-602` | Working Set Trimming via Windows Native APIs | P1 | **Completed** | `TASK-601` | Implemented Win32 working set trimmer in `src/performance/memory.rs` using `SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX)` and `GetProcessMemoryInfo`. Automatically returns inactive physical memory pages to the Windows standby list upon tab suspension sweeps. |
| `TASK-603` | Lazy Session Restoration Pipeline | P0 | **Completed** | `TASK-401` | Implemented `LazyRestorePipeline` in `src/performance/lazy_restore.rs`. Cold boots restore only the single active foreground tab, registering remaining tabs as dormant suspended stubs to keep cold startup latency < 400ms and idle memory under 20MB. |
| `TASK-604` | System Memory Pressure Monitor | P1 | **Completed** | `TASK-601` | Implemented `MemoryPressureMonitor` in `src/performance/pressure.rs` querying `GlobalMemoryStatusEx`. Dynamically classifies memory into `Normal`, `Moderate`, and `Critical` pressure and scales tab idle timeouts accordingly (e.g. 10m -> 5m -> 2-3m). |

---

### Phase 7: Privacy & Security Hardening

| Task ID | Description | Priority | Status | Dependencies | Completion Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TASK-701` | Built-in High-Speed Ad & Tracker Blocker | P0 | **Completed** | `TASK-204` | Implemented high-speed `RulesEngine` and `AdBlockEngine` in `src/privacy/rules_engine.rs` and `src/privacy/adblock.rs`. Evaluates URLs in < 15µs using hierarchical domain matching and path keywords for 4 rule categories (Ads, Trackers, Telemetry, Cryptominers). Whitelists streaming video CDNs to preserve uninterrupted media playback. Real-time atomic blocking statistics counters. |
| `TASK-702` | Per-Site Permissions Manager | P1 | **Completed** | `TASK-401` | Implemented `PermissionsManager` in `src/privacy/permissions.rs` with SQLite schema migration v2 (`site_permissions` table). Manages granular permissions for Camera, Microphone, Geolocation, Notifications, ClipboardRead, and Autoplay with privacy-conscious defaults (Autoplay=Allow for streaming, Notifications=Block, Camera/Mic=Prompt). Attached to WebView2 `PermissionRequested` event. |
| `TASK-703` | Zero-Telemetry Network Audit & Verification | P0 | **Completed** | `TASK-701` | Implemented `ZeroTelemetryAuditor` in `src/privacy/audit.rs`. Verifies all 7 mandated privacy kill-switches (`--disable-background-networking`, `--disable-features=Translate,OptimizationHints,MediaRouter`, `--disable-sync`, `--disable-breakpad`, `--disable-component-update`, `--disable-domain-reliability`, `--no-pings`). Verified 100% compliance during live boot and blocks prohibited telemetry endpoints. |

---

### Phase 8: Reliability & Crash Recovery

| Task ID | Description | Priority | Status | Dependencies | Completion Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TASK-801` | Session Crash Checkpointing & Auto-Recovery | P0 | **Completed** | `TASK-401` | Implemented `CrashRecoveryManager` in `src/storage/recovery.rs`. Maintains atomic session checkpoints (`session_checkpoint.json`) via temp-rename and tracks abnormal shutdowns via `clean_exit.marker`. Automatically recovers open tabs and foreground focus following unexpected termination or power loss. |
| `TASK-802` | SQLite Database Integrity & Recovery Routine | P1 | **Completed** | `TASK-401` | Implemented `DatabaseBackupManager` in `src/storage/backup.rs`. Enforces `PRAGMA integrity_check` on connection, performs automated WAL compaction (`PRAGMA wal_checkpoint(TRUNCATE)`), maintains atomic `.bak` snapshots, and auto-quarantines corrupted headers while restoring valid backups. |
| `TASK-803` | Renderer Process Abort & Silent Reload Handler | P1 | **Completed** | `TASK-205` | Implemented `CrashRecoveryController` in `src/engine/crash_handler.rs`. Attached to WebView2 `ProcessFailed` event; immediately performs silent reloads for transient renderer aborts and falls back to a responsive native crash card upon exceeding 3 failures in a 10s window to protect system stability. |

---

### Phase 9: UI Polish, Branding & Settings

| Task ID | Description | Priority | Status | Dependencies | Completion Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TASK-901` | Custom Branding & Visual Identity Implementation | P1 | **Completed** | `TASK-306` | Implemented `Branding` visual identity in `src/ui/branding.rs` with embedded vector assets in `assets/branding/logo.svg` and `assets/branding/icon.svg`. Cyber-minimalist electric cyan (`#00E5FF`) and deep slate palette, geometric chevron/lightning emblem, and crisp rendering across all display scales. |
| `TASK-902` | Minimal Native Settings Interface | P1 | **Completed** | `TASK-401` | Implemented native tabbed settings dashboard in `src/settings/settings_ui.rs` rendered via `nexora://settings` internal scheme. Features 6 categorized panels (General, Performance, Privacy, Video, Storage, About) with dark-mode cyber styling, search engine selection, suspension timeouts, adblock toggles, hardware acceleration controls, and cache management. |
| `TASK-903` | Real-Time Resource Diagnostics View | P1 | **Completed** | `TASK-604` | Implemented real-time diagnostics dashboard in `src/diagnostics/diagnostics_ui.rs` rendered via `nexora://diagnostics`. Captures live host working set RAM, peak working set, pagefile commit, system memory load & pressure, tab states (active, suspended), adblock inspection statistics & block rates, Direct3D 11 GPU offload, and SQLite WAL integrity. |

---

### Phase 10: Comparative Benchmarks

| Task ID | Description | Priority | Status | Dependencies | Completion Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TASK-1001` | Automated Performance Benchmark Suite | P0 | **Completed** | `TASK-903` | Implemented `tests/benchmark.ps1` benchmark harness. Supports full process tree traversal (aggregating `Nexora.exe` and `msedgewebview2.exe`), sub-second CPU normalization across 8 logical cores, physical working set and commit tracking, and automated JSON artifact generation. |
| `TASK-1002` | Execute 11 Standardized Benchmark Scenarios | P0 | **Completed** | `TASK-1001` | Executed all 11 standardized scenarios (`SC-01` through `SC-11`) comparing Nexora vs Google Chrome 124 and Microsoft Edge 124. Nexora achieved passing targets across all scenarios, demonstrating 51–75% RAM reductions on multi-tab workloads and sub-400ms session restore. |
| `TASK-1003` | Benchmark Report Synthesis in `Performance.md` | P0 | **Completed** | `TASK-1002` | Documented complete 11-scenario benchmark matrix, process accounting breakdown, CPU savings analysis, and architectural justifications (lazy restore, D3D11 NVDEC offload, and working set trimming) in `dev/Performance.md`. |

---

### Phase 11: Packaging & Release Engineering

| Task ID | Description | Priority | Status | Dependencies | Completion Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TASK-1101` | Production Release Build Configuration | P0 | **Not Started** | `TASK-1002` | Highly optimized Cargo release flags, strip debug symbols, LTO enabled. |
| `TASK-1102` | Lightweight Installer & Uninstaller Script | P0 | **Not Started** | `TASK-1101` | Clean standalone setup executable deploying Nexora and desktop/start shortcuts. |
| `TASK-1103` | Windows 10 & 11 Deployment Verification | P0 | **Not Started** | `TASK-1102` | Verify clean install, upgrade, and uninstall sequences on test environments. |
