<p align="center">
  <img src="../assets/branding/logo.svg" width="380" alt="Nexora - The Quantum Prism Logo"/>
</p>

# NEXORA: Development Lifecycle & Phase Execution Model
## Ultra-Lightweight, Streaming-First Desktop Browser

---

### Phase Overview Matrix

| Phase | Title | Core Focus | Prerequisite | Verification Gate |
| :--- | :--- | :--- | :--- | :--- |
| **Phase 0** | Research & Feasibility | Engine selection, DRM & hardware acceleration viability, baseline benchmarks | None | Architecture sign-off & feasibility report |
| **Phase 1** | Foundation & Project Setup | Rust toolchain, repo scaffold, logging, error handling, CI/build scripts | Phase 0 | Clean build & zero-dependency baseline |
| **Phase 2** | Browser Engine Integration | WebView2 COM embedding, navigation controller, lifetime handling | Phase 1 | Basic web navigation in native window |
| **Phase 3** | Main Browser Interface | Custom tab bar, omnibox/URL bar, navigation controls, dark/light theme | Phase 2 | Full interactive UI with tab switching |
| **Phase 4** | Browser Data & Persistence | SQLite storage (history, bookmarks), cookie persistence, cache caps | Phase 3 | Survives restart; FTS5 history search |
| **Phase 5** | Video Streaming & Media Engine | YouTube 4K/60, DRM (Netflix, Prime, Hotstar), PiP, DXVA video offload | Phase 4 | Zero dropped frames; DRM verification |
| **Phase 6** | Performance Engineering | Smart Tab Suspension, memory trimming, CPU throttling, lazy load | Phase 5 | 10 tabs < 350 MB RAM; instant resume |
| **Phase 7** | Privacy & Security Hardening | Built-in ad/tracker filter, permission manager, zero telemetry audit | Phase 6 | Adblock test pass; network audit clean |
| **Phase 8** | Reliability & Crash Recovery | Session crash recovery, database repair, network reconnection | Phase 7 | Process kill & instant tab recovery |
| **Phase 9** | UI Polish & Branding | Original iconography, smooth transitions, settings & diagnostics panel | Phase 8 | Complete design system compliance |
| **Phase 10** | Comparative Benchmarks | Rigorous performance testing vs Chrome & Edge on baseline hardware | Phase 9 | Documented memory, CPU & GPU wins |
| **Phase 11** | Packaging & Release Engineering | Thin installer, runtime bootstrapper, clean uninstall, release notes | Phase 10 | Validated installer on Win 10 & 11 |

---

### Phase 0: Research and Feasibility

#### Objectives
- Exhaustively research candidate browser engines (Microsoft WebView2, CEF, Wry, Servo/Gecko).
- Evaluate hardware video decoding (Direct3D 11 / DXVA2) on Kepler / Intel Ivy Bridge target hardware (GT 730 / i7-3770).
- Validate legitimate DRM support (Widevine CDM, Microsoft PlayReady, EME) across major streaming platforms.
- Establish baseline resource consumption (idle, active tab, YouTube 1080p60).
- Produce complete architectural and technical specification documentation.

#### Features & Deliverables
- Detailed engine trade-off matrix (`/dev/Architecture.md`).
- Hardware decoding capability assessment (`/dev/Performance.md`).
- DRM compliance strategy (`/dev/Security.md`).
- Complete `/dev` documentation suite (`Plan.md`, `Phases.md`, `Tasks.md`, `Architecture.md`, `Performance.md`, `Security.md`, `Testing.md`, `Decisions.md`).

#### Dependencies
- Local hardware: Intel Core i7-3770, NVIDIA GeForce GT 730, Windows 11 Pro 64-bit.
- Installed WebView2 Evergreen Runtime (v157+).

#### Testing Requirements
- Direct DRM test suite via EME APIs.
- Video playback smoothness check on YouTube VP9 / AVC streams.
- Process-tree RAM inspection.

#### Completion Criteria
- Formal sign-off on engine selection (WebView2 COM) with verified DRM and hardware video acceleration paths.

---

### Phase 1: Foundation & Project Setup

#### Objectives
- Initialize Rust workspace and directory structure following clean modular patterns.
- Configure build profiles (debug vs. high-performance optimized release with LTO and strip).
- Implement structured logging (with file and debug output sink) and robust error handling.
- Configure Windows-specific subsystem flags (`#![windows_subsystem = "windows"]` for production).

#### Features & Deliverables
- `Cargo.toml` with minimal, carefully audited dependencies.
- `src/main.rs`, `src/app/`, `src/core/error.rs`, `src/core/logging.rs`.
- Build automation scripts (`build.ps1`).

#### Dependencies
- Phase 0 sign-off; Rust stable toolchain.

#### Testing Requirements
- Unit tests for error conversion and logging formatting.
- Clean build verification on Windows 10/11.

#### Completion Criteria
- Native executable compiles without warnings and launches clean logging subsystem.

---

### Phase 2: Browser Engine Integration

#### Objectives
- Embed Microsoft WebView2 via direct COM bindings into native Win32 window.
- Implement window sizing, message dispatching, and DPI awareness.
- Establish navigation pipeline (`Navigate`, `GoBack`, `GoForward`, `Reload`, `Stop`).
- Handle core engine events: `NavigationStarting`, `NavigationCompleted`, `SourceChanged`, `ProcessFailed`.

#### Features & Deliverables
- `src/engine/webview.rs` encapsulating `ICoreWebView2Environment`, `ICoreWebView2Controller`, and `ICoreWebView2`.
- Custom User Data Directory (UDF) configured at `%LOCALAPPDATA%\Nexora\EngineData`.
- Runtime detection and graceful failure reporting.

#### Dependencies
- Phase 1 foundation.

#### Testing Requirements
- Navigation to standard HTTP and HTTPS websites.
- Verification of child process tree creation (`msedgewebview2.exe`).

#### Completion Criteria
- Smooth web page rendering and reliable navigation control within a native Win32 host window.

---

### Phase 3: Main Browser Interface

#### Objectives
- Create a modern, compact, native tab bar supporting multi-tab management.
- Develop the Advanced URL Bar (Omnibox) with smart search fallback, autocomplete, and security indicators.
- Implement native navigation controls (Back, Forward, Refresh, Home).
- Implement native dark and light themes.

#### Features & Deliverables
- `src/tabs/tab_manager.rs`, `src/tabs/tab.rs`.
- `src/navigation/omnibox.rs`, `src/navigation/search.rs`.
- `src/ui/theme.rs`, `src/ui/layout.rs`.

#### Dependencies
- Phase 2 engine integration.

#### Testing Requirements
- Opening, switching, and closing 20 tabs without layout corruption.
- Omnibox URL detection vs query parsing accuracy tests.

#### Completion Criteria
- Fully functional tabbed browser shell with responsive omnibox and theme toggling.

---

### Phase 4: Browser Data & Persistence

#### Objectives
- Integrate SQLite with WAL mode for all structured application data.
- Implement persistent cookie management surviving restarts.
- Create full-text searchable history with configurable retention limits.
- Implement hierarchical bookmarks system.
- Implement disk and memory cache size enforcement.

#### Features & Deliverables
- `src/storage/db.rs`, `src/storage/migrations.rs`.
- `src/history/history_service.rs` (FTS5 search index).
- `src/bookmarks/bookmark_service.rs`.
- `src/cache/cache_manager.rs`.

#### Dependencies
- Phase 3 interface.

#### Testing Requirements
- Restart browser and verify cookies/logins persist.
- Insert 10,000 history records and verify omnibox search completes in < 5 ms.

#### Completion Criteria
- Zero data loss across simulated browser restarts; cache stays strictly within configured boundaries.

---

### Phase 5: Video Streaming & Media Engine

#### Objectives
- Optimize video streaming pipeline for YouTube (720p, 1080p, 4K60), Netflix, Prime Video, Hotstar, Twitch.
- Verify hardware-accelerated video decoding via Direct3D 11 / DXVA2.
- Integrate native Picture-in-Picture (PiP) floating window.
- Implement fullscreen mode (`F11` and HTML5 Fullscreen API).
- Prioritize active video tab resources in Windows thread scheduler.

#### Features & Deliverables
- `src/media/pip.rs`, `src/media/drm_manager.rs`.
- WebView2 environment flags for media: `--enable-features=VaapiVideoDecoder,D3D11VideoDecoder`.
- Active media detection via `ICoreWebView2::ContainsFullScreenElement` and audio state.

#### Dependencies
- Phase 4 persistence.

#### Testing Requirements
- 1080p60 YouTube playback with DXVA2 GPU video engine verification in Task Manager.
- DRM handshake verification on authorized streaming services.
- PiP drag, resize, and always-on-top validation.

#### Completion Criteria
- Continuous smooth playback with < 0.1% dropped frames and verified hardware offload.

---

### Phase 6: Performance Engineering & Smart Tab Suspension

#### Objectives
- Implement the Smart Tab Suspension state machine.
- Automatically suspend background tabs based on idle time and RAM threshold.
- Trim working set of idle processes via Windows API (`SetProcessWorkingSetSize`).
- Lazy session restoration on application launch.

#### Features & Deliverables
- `src/performance/suspension.rs`, `src/performance/memory_trimmer.rs`.
- `src/performance/system_monitor.rs` (tracks RAM, CPU, power status).
- Suspended tab placeholder page with one-click restore.

#### Dependencies
- Phase 5 media engine (must respect media exemptions).

#### Testing Requirements
- Open 10 complex tabs (including 1 video tab); verify background tabs suspend while video plays uninterrupted.
- Measure memory reduction before and after suspension (> 50% RAM reclaimed).

#### Completion Criteria
- Memory profile under 350 MB for 10 tabs; zero interference with active media.

---

### Phase 7: Privacy & Security Hardening

#### Objectives
- Implement high-speed native ad and tracker blocker using network request interception.
- Support EasyList / uBlock filter syntax via Rust prefix tree/Aho-Corasick matching.
- Implement granular per-site permissions (camera, microphone, location, autoplay).
- Conduct strict zero-telemetry network audit.

#### Features & Deliverables
- `src/privacy/adblock.rs`, `src/privacy/rules_engine.rs`.
- `src/privacy/permissions.rs`.
- Blocking statistics counters.

#### Dependencies
- Phase 6 performance engine.

#### Testing Requirements
- Test against adblock validation suites (e.g., d3ward, AdBlock Tester).
- Network packet capture verifying zero external pings outside user-initiated browsing.

#### Completion Criteria
- > 90% ad/tracker blocking score with zero regressions on major video streaming platforms.

---

### Phase 8: Reliability & Crash Recovery

#### Objectives
- Implement automated session checkpointing.
- Detect abnormal shutdowns and present a clean "Restore Session" dialog on next launch.
- SQLite database integrity check (`PRAGMA integrity_check`) with automatic backup restoration.
- WebView2 renderer process crash recovery (`ProcessFailed` handler).

#### Features & Deliverables
- `src/sessions/recovery.rs`.
- `src/storage/backup.rs`.

#### Dependencies
- Phase 7 security.

#### Testing Requirements
- Force-kill `Nexora.exe` and `msedgewebview2.exe` via `taskkill /F`; verify clean session restoration.
- Intentionally corrupt SQLite database header and verify auto-repair fallback.

#### Completion Criteria
- Seamless tab state recovery without orphaned processes or permanent data loss.

---

### Phase 9: UI Polish, Branding & Settings

#### Objectives
- Implement custom minimalist branding, typography, and crisp Windows icons.
- Build the distraction-free Settings interface (General, Performance, Privacy, Video, Storage, About).
- Build the real-time Resource Diagnostics panel.

#### Features & Deliverables
- `src/settings/settings_ui.rs`.
- `src/diagnostics/diagnostics_ui.rs`.
- High-resolution SVG / ICO assets (`assets/icons/nexora.ico`).

#### Dependencies
- Phase 8 reliability.

#### Testing Requirements
- Diagnostics memory/CPU values validated against Windows Performance Monitor / Task Manager.
- High-DPI scaling checks (100%, 125%, 150%, 200%).

#### Completion Criteria
- Visually stunning, ultra-responsive native interface adhering to Windows 10/11 design standards.

---

### Phase 10: Comparative Benchmarks

#### Objectives
- Execute rigorous, repeatable benchmarks against Google Chrome and Microsoft Edge on the benchmark hardware.
- Test 11 standardized scenarios: Empty browser, 1 tab, YouTube 1080p, YouTube 4K, 5 tabs, 10 tabs, 10 tabs with suspension, DRM streaming, long-duration browsing, cold startup, warm startup.
- Document every process in the process tree.

#### Features & Deliverables
- `dev/Performance.md` benchmark reports with exact memory, CPU, and frame drop numbers.
- Automated benchmark measurement script (`tests/benchmark.ps1`).

#### Dependencies
- Phase 9 UI and branding.

#### Testing Requirements
- Direct comparison across all 11 scenarios using PowerShell performance counter queries.

#### Completion Criteria
- Measurable and documented resource consumption superiority over standard Chrome and Edge.

---

### Phase 11: Packaging & Release Engineering

#### Objectives
- Build clean, standalone installer (`Nexora-Setup.exe`) and uninstaller.
- Package optimized release binaries with stripped symbols and LTO.
- Verify WebView2 bootstrapper fallback on pristine Windows 10 machines.
- Prepare complete release documentation and user guide.

#### Features & Deliverables
- Inno Setup / NSIS / WiX script in `installer/`.
- `README.md`, `CHANGELOG.md`, `LICENSE`.
- GitHub Release artifacts.

#### Dependencies
- Phase 10 benchmarking.

#### Testing Requirements
- Clean install and uninstall tests on Windows 10 (1809) and Windows 11 (22H2+).
- Registry and file cleanup verification upon uninstall.

#### Completion Criteria
- Verified, single-click installable production distribution ready for end-user deployment.

---

### Phase 12: UI/UX Refinement & Instant Tab Performance

#### Objectives
- Eliminate all AI-generated sci-fi dashboard artifacts (excessive red glows, decorative badges, futuristic terminology).
- Establish the unified Nexora Design System (`src/ui/design_system.rs`) with charcoal base, soft white text, and restrained crimson accents.
- Integrate official Lucide vector icons (`src/ui/icons.rs`) consistently across top toolbar and all internal schemes.
- Transition from single-controller navigation to persistent multi-controller browser views (`tab_views: RefCell<HashMap<usize, TabView>>`).
- Achieve instant tab switching (< 15–50 ms) with zero page reloads, zero DOM teardown, and preserved background video playback.
- Redesign `nexora://home`, `nexora://settings`, and `nexora://diagnostics` into professional desktop browser interfaces.
- Protect omnibox and tab title synchronization against `about:blank` and data URIs.

#### Features & Deliverables
- `src/ui/design_system.rs`: Shared design tokens and CSS components.
- `src/ui/icons.rs`: Official Lucide SVG vector icon renderer (31 icons).
- `src/engine/webview.rs`: Multi-controller persistent views, visibility toggles, and omnibox URI guards.
- `src/home/home_ui.rs`: Modern start page with search engine switcher, shortcuts modal, and bookmarks.
- `src/settings/settings_ui.rs`: Two-column settings dashboard with 100% working IPC config persistence.
- `src/diagnostics/diagnostics_ui.rs`: Truthful system monitor displaying real host/tree RAM via `get_tree_memory()`, rounded values, Copy JSON, and Export Report.
- `tests/benchmark.ps1`: SC-12 automated tab switching latency benchmark (< 25 ms target).

#### Testing Requirements
- Unit test suite: 65 passing tests including Lucide icon generation, settings rendering, and diagnostics.
- Benchmarking: SC-12 instant tab-switching validation.
- Video continuity: YouTube 1080p playback preservation across tab switches.

#### Completion Criteria
- Instant tab switching (< 20 ms resident switch).
- 100% passing tests and complete elimination of AI-generated dashboard artifacts.

