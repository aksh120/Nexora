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
| `TASK-601` | Smart Tab Suspension Engine | P0 | **Not Started** | `TASK-504` | Automatic background tab suspension based on idle time and system RAM. |
| `TASK-602` | Working Set Trimming via Windows Native APIs | P1 | **Not Started** | `TASK-601` | Call `SetProcessWorkingSetSize` to release inactive physical memory. |
| `TASK-603` | Lazy Session Restoration Pipeline | P0 | **Not Started** | `TASK-401` | Cold-start tab restoration using lightweight placeholder stubs. |
| `TASK-604` | System Memory Pressure Monitor | P1 | **Not Started** | `TASK-601` | Poll `GlobalMemoryStatusEx` and trigger aggressive cleanup under low memory. |

---

### Phase 7: Privacy & Security Hardening

| Task ID | Description | Priority | Status | Dependencies | Completion Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TASK-701` | Built-in High-Speed Ad & Tracker Blocker | P0 | **Not Started** | `TASK-204` | Intercept web resource requests via fast domain trie matching. |
| `TASK-702` | Per-Site Permissions Manager | P1 | **Not Started** | `TASK-401` | Granular permission prompts and storage (Camera, Mic, Geolocation, Autoplay). |
| `TASK-703` | Zero-Telemetry Network Audit & Verification | P0 | **Not Started** | `TASK-701` | Wireshark/Fiddler inspection confirming 0 background telemetry outbound. |

---

### Phase 8: Reliability & Crash Recovery

| Task ID | Description | Priority | Status | Dependencies | Completion Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TASK-801` | Session Crash Checkpointing & Auto-Recovery | P0 | **Not Started** | `TASK-401` | Periodic tab state persistence for instant crash session restore. |
| `TASK-802` | SQLite Database Integrity & Recovery Routine | P1 | **Not Started** | `TASK-401` | Database corruption detection, automated WAL cleanup, and backup restore. |
| `TASK-803` | Renderer Process Abort & Silent Reload Handler | P1 | **Not Started** | `TASK-205` | Auto-respawn crashed webview tabs without dropping user navigation state. |

---

### Phase 9: UI Polish, Branding & Settings

| Task ID | Description | Priority | Status | Dependencies | Completion Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TASK-901` | Custom Branding & Visual Identity Implementation | P1 | **Not Started** | `TASK-306` | Minimalist Nexora vector logo, application icon, and styling. |
| `TASK-902` | Minimal Native Settings Interface | P1 | **Not Started** | `TASK-401` | Tabbed settings dialog (General, Performance, Privacy, Video, Storage, About). |
| `TASK-903` | Real-Time Resource Diagnostics View | P1 | **Not Started** | `TASK-604` | Lightweight overlay displaying active RAM, CPU, GPU decoder, and tab count. |

---

### Phase 10: Comparative Benchmarks

| Task ID | Description | Priority | Status | Dependencies | Completion Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TASK-1001` | Automated Performance Benchmark Suite | P0 | **Not Started** | `TASK-903` | PowerShell script gathering exact process-tree memory and CPU metrics. |
| `TASK-1002` | Execute 11 Standardized Benchmark Scenarios | P0 | **Not Started** | `TASK-1001` | Comparative runs against Google Chrome and Microsoft Edge on GT 730/i7-3770. |
| `TASK-1003` | Benchmark Report Synthesis in `Performance.md` | P0 | **Not Started** | `TASK-1002` | Record full telemetry-free comparative analysis. |

---

### Phase 11: Packaging & Release Engineering

| Task ID | Description | Priority | Status | Dependencies | Completion Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TASK-1101` | Production Release Build Configuration | P0 | **Not Started** | `TASK-1002` | Highly optimized Cargo release flags, strip debug symbols, LTO enabled. |
| `TASK-1102` | Lightweight Installer & Uninstaller Script | P0 | **Not Started** | `TASK-1101` | Clean standalone setup executable deploying Nexora and desktop/start shortcuts. |
| `TASK-1103` | Windows 10 & 11 Deployment Verification | P0 | **Not Started** | `TASK-1102` | Verify clean install, upgrade, and uninstall sequences on test environments. |
