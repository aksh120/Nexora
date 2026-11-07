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
| `TASK-201` | Win32 Host Window Creation & Message Loop | P0 | **Not Started** | `TASK-104` | High-performance Win32 event loop with minimal CPU polling. |
| `TASK-202` | Microsoft WebView2 COM Initialization | P0 | **Not Started** | `TASK-201` | Create environment with custom User Data Folder and media acceleration switches. |
| `TASK-203` | Implement WebView2 Controller & Sizing Pipeline | P0 | **Not Started** | `TASK-202` | Attach WebView2 controller to host window client area with DPI scaling. |
| `TASK-204` | Core Navigation Commands (URL, Back, Forward, Reload) | P0 | **Not Started** | `TASK-203` | Implement navigation state machine and event hooks. |
| `TASK-205` | Process Lifecycle & Failure Recovery Hooks | P1 | **Not Started** | `TASK-204` | Handle `ProcessFailed` events and renderer crashes gracefully. |

---

### Phase 3: Main Browser Interface

| Task ID | Description | Priority | Status | Dependencies | Completion Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TASK-301` | Native Multi-Tab Manager State Machine | P0 | **Not Started** | `TASK-204` | Tab collection, active tab switching, tab closure, and creation. |
| `TASK-302` | Compact Tab Bar UI Rendering | P1 | **Not Started** | `TASK-301` | Low-profile tab strip with titles, audio indicators, and close buttons. |
| `TASK-303` | Advanced Omnibox (URL / Search Bar) | P0 | **Not Started** | `TASK-301` | URL validation, search query differentiation, keyboard navigation. |
| `TASK-304` | Omnibox Autocomplete & Suggestions Engine | P1 | **Not Started** | `TASK-303` | Instant local history and bookmark suggestions matching prefix. |
| `TASK-305` | Navigation Toolbar Controls (Back/Forward/Reload/Home) | P1 | **Not Started** | `TASK-303` | Render responsive, minimal native control buttons. |
| `TASK-306` | Dark and Light Theme Engine | P2 | **Not Started** | `TASK-302` | Native Win32 dark mode theme support conforming to Windows system theme. |

---

### Phase 4: Browser Data & Persistence

| Task ID | Description | Priority | Status | Dependencies | Completion Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TASK-401` | SQLite Persistence Architecture & Migrations | P0 | **Not Started** | `TASK-101` | Implement `rusqlite` database layer with WAL mode and schema versioning. |
| `TASK-402` | Cookie Persistence & Session Management | P0 | **Not Started** | `TASK-401` | Utilize `ICoreWebView2CookieManager` for cross-session cookie persistence. |
| `TASK-403` | Full-Text Searchable History Subsystem | P1 | **Not Started** | `TASK-401` | SQLite FTS5 table for lightning-fast chronological history queries. |
| `TASK-404` | Bookmarks Management (Add, Edit, Delete, Folders) | P1 | **Not Started** | `TASK-401` | Hierarchical bookmark tree storage and fast retrieval. |
| `TASK-405` | Smart Cache Policy & Disk Boundary Enforcement | P1 | **Not Started** | `TASK-401` | WebView2 `--disk-cache-size` enforcement and cache sweeping. |

---

### Phase 5: Video Streaming & Media Engine

| Task ID | Description | Priority | Status | Dependencies | Completion Notes |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `TASK-501` | Direct3D 11 & DXVA2 Hardware Acceleration Configuration | P0 | **Not Started** | `TASK-202` | Verify GPU hardware video decoding flags and runtime capabilities. |
| `TASK-502` | Legitimate Widevine & PlayReady DRM Pipeline | P0 | **Not Started** | `TASK-202` | Verify Encrypted Media Extensions (EME) handshake on Netflix/Prime/Hotstar. |
| `TASK-503` | Picture-in-Picture (PiP) Window Architecture | P1 | **Not Started** | `TASK-301` | Always-on-top draggable floating video window with playback controls. |
| `TASK-504` | Active Media Detection & Priority Scheduling | P0 | **Not Started** | `TASK-301` | Detect `IsDocumentPlayingAudio` / fullscreen video; exempt from suspension. |
| `TASK-505` | Fullscreen Video Transition & HTML5 Fullscreen | P1 | **Not Started** | `TASK-301` | Handle `ContainsFullScreenElementChanged` with seamless host window toggle. |

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
