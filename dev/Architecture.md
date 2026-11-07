# NEXORA: Technical Architecture & System Specification
## Ultra-Lightweight, Streaming-First Desktop Browser

---

### 1. Architectural Philosophy

Modern commercial desktop browsers are built on monolithic multi-gigabyte codebases laden with legacy features, heavy JavaScript-based UI layers, extensive advertising and telemetry frameworks, and aggressive memory-caching strategies designed to favor page switching speed over system memory availability.

**NEXORA** rejects this paradigm. Its architectural philosophy is governed by three non-negotiable principles:

1. **Native Orchestration, Minimal Overhead**: The host browser shell, UI layout, tab management state machine, persistence, and network filtering run in 100% native Rust, compiled directly to optimized x86_64 machine code with zero virtual machine or garbage collector overhead.
2. **Streaming-First Resource Allocation**: The active media tab possesses first-class priority. System threads responsible for video frame delivery and audio synchronization are pinned to high priority; inactive background tabs are aggressively trimmed or suspended.
3. **Decoupled Engine Subsystem**: The web rendering engine (Microsoft WebView2 Evergreen) is treated as a sandboxed rendering worker, governed by strict native controllers, preventing engine subprocesses from holding idle RAM indefinitely.

---

### 2. High-Level System Architecture

NEXORA divides its responsibilities across well-defined architectural boundaries:

```
+===================================================================================+
|                                    NEXORA CORE                                    |
|                      (Pure Native Rust Application Layer)                         |
+===================================================================================+
|                                                                                   |
|  +-------------------------+  +--------------------------+  +-------------------+ |
|  |     Win32 Host Shell    |  |       Tab Manager        |  |  Smart Suspension | |
|  |  - Custom Tab Strip     |  |  - Tab Collection State  |  |  - Inactivity     | |
|  |  - Omnibox / Autocomp   |  |  - Active Tab Router     |  |    Detection      | |
|  |  - Navigation Toolbar   |  |  - Lazy Tab Restoration  |  |  - Working Set    | |
|  |  - High-DPI & Dark Mode |  |  - Close / Reorder       |  |    Trimming       | |
|  +-------------------------+  +--------------------------+  +-------------------+ |
|               |                             |                         |           |
|  +-------------------------+  +--------------------------+  +-------------------+ |
|  |  Network Filter Engine  |  |     Media Controller     |  | Persistence Layer | |
|  |  - Native Ad Blocker    |  |  - D3D11 / DXVA2 Hooks   |  |  - SQLite 3 (WAL) | |
|  |  - Tracker Trie Matching|  |  - Picture-in-Picture    |  |  - FTS5 History   | |
|  |  - WebResource Filters  |  |  - Audio State Exemption |  |  - Bookmarks &    | |
|  |  - Zero-Telemetry Guard |  |  - Fullscreen Handlers   |  |    Permissions    | |
|  +-------------------------+  +--------------------------+  +-------------------+ |
|               |                             |                         |           |
+===================================================================================+
                                         |
                       [ Low-Overhead Native COM IPC ]
                                         |
+===================================================================================+
|                          MICROSOFT WEBVIEW2 RUNTIME                               |
|                     (Evergreen Chromium-Based Subprocesses)                       |
+===================================================================================+
|                                                                                   |
|  +--------------------------------+       +------------------------------------+  |
|  |        GPU Subprocess          |       |         Render Subprocess          |  |
|  |  - Direct3D 11 Compositor      |       |  - Blink HTML5 / CSS3 Engine       |  |
|  |  - DXVA2 Hardware Video Decoder|       |  - V8 JavaScript Engine            |  |
|  |  - Kepler / GT 730 D3D Pipes   |       |  - Encrypted Media Extensions (EME)|  |
|  +--------------------------------+       |  - Widevine / PlayReady CDM        |  |
|                                           +------------------------------------+  |
+===================================================================================+
```

---

### 3. Component & Module Responsibilities

The codebase is organized into discrete, single-responsibility modules:

```
Nexora/
├── src/
│   ├── main.rs                 # WinMain entry point, command-line arguments, mutex single-instance check
│   ├── app/
│   │   ├── application.rs      # Main application state orchestrator
│   │   └── config.rs           # Global configuration and user preferences
│   ├── browser/
│   │   ├── host_window.rs      # Win32 host window, WndProc message router, DPI scaling
│   │   └── webview_host.rs     # Controller-to-window binding, coordinate transformation
│   ├── engine/
│   │   ├── webview.rs          # Microsoft WebView2 COM initialization, environment options
│   │   ├── flags.rs            # GPU, media, and cache command-line switches
│   │   └── events.rs           # COM event sink implementations (Navigation, DOM, Process)
│   ├── tabs/
│   │   ├── tab.rs              # Tab data structure, state (Active, Inactive, Suspended, Playing)
│   │   └── tab_manager.rs      # Multi-tab collection, selection logic, tab bar synchronization
│   ├── navigation/
│   │   ├── omnibox.rs          # URL input parsing, domain extraction, search query differentiation
│   │   └── search_engines.rs   # Configurable search providers (DuckDuckGo, Google, Bing, Brave)
│   ├── performance/
│   │   ├── suspension.rs       # Inactivity timers, memory-pressure-triggered tab suspension
│   │   ├── memory_trimmer.rs   # Win32 SetProcessWorkingSetSize and heap compaction
│   │   └── monitor.rs          # System RAM / CPU usage poller using GlobalMemoryStatusEx
│   ├── media/
│   │   ├── pip.rs              # Floating Picture-in-Picture window management
│   │   ├── drm.rs              # Widevine / PlayReady verification and EME policy
│   │   └── acceleration.rs     # DXVA2 / Direct3D 11 capability detection
│   ├── privacy/
│   │   ├── filter_engine.rs    # High-speed ad/tracker filter using Trie / Aho-Corasick
│   │   ├── easy_list.rs        # EasyList and uBlock rule compilation and serialization
│   │   └── permissions.rs      # Per-site permission grant and restriction engine
│   ├── storage/
│   │   ├── db.rs               # SQLite connection pool, WAL mode, transaction helpers
│   │   ├── migrations.rs       # Schema versioning and automated table creation
│   │   ├── history.rs          # Chronological and FTS5 searchable history service
│   │   └── bookmarks.rs        # Tree-structured bookmark service
│   ├── ui/
│   │   ├── tab_bar.rs          # Custom native tab strip painting and hit testing
│   │   ├── toolbar.rs          # Navigation button controls (Back, Forward, Refresh, Home)
│   │   ├── theme.rs            # Dark and light palette tokens and brush caching
│   │   └── dialogs.rs          # Lightweight native modal dialogs (crash recovery, alerts)
│   ├── settings/
│   │   └── settings_ui.rs      # Native settings viewport for General, Performance, Privacy
│   └── diagnostics/
│       └── diagnostics_ui.rs   # Real-time resource overlay (RAM, CPU, GPU, tab counts)
```

---

### 4. Engine Integration Details

#### WebView2 Environment Configuration
NEXORA instantiates the WebView2 environment via `CreateCoreWebView2EnvironmentWithOptions`:
1. **User Data Directory**: Directed to `%LOCALAPPDATA%\Nexora\EngineData` to guarantee separation from system Edge or other WebView2 host applications.
2. **Engine Switches Passed via `AdditionalBrowserArguments`**:
   - `--enable-features=VaapiVideoDecoder,D3D11VideoDecoder`: Maximizes GPU offload for H.264 / VP8 / VP9 video streams.
   - `--disable-features=Translate,OptimizationHints,MediaRouter`: Eliminates background cloud translation and Google Cast discovery telemetry.
   - `--disk-cache-size=268435456`: Caps disk cache at 256 MB (configurable).
   - `--media-cache-size=134217728`: Caps media cache at 128 MB.
   - `--enable-gpu-rasterization`: Forces Direct3D GPU rasterization for smooth scrolling.
   - `--disable-background-networking`: Disables background update checks and network chatter within the Chromium instance.

#### COM Event Interception
Key event interfaces implemented in Rust via `webview2-com` / `windows-rs`:
- `ICoreWebView2NavigationStartingEventHandler`: Inspects destination URI, validates HTTPS, triggers omnibox security padlock.
- `ICoreWebView2WebResourceRequestedEventHandler`: Intercepts outgoing HTTP/HTTPS resource requests (scripts, images, iframes) and blocks ad/tracking URLs based on the Rust filter trie.
- `ICoreWebView2ContainsFullScreenElementChangedEventHandler`: Intercepts HTML5 full-screen requests and toggles host window borderless fullscreen.
- `ICoreWebView2ProcessFailedEventHandler`: Detects renderer or GPU process crashes, logs failure diagnostic, and cleanly recovers tab state without crashing the host browser.

---

### 5. Multi-Tab Process & Controller Architecture

In NEXORA, each active tab has an associated `ICoreWebView2Controller` and `ICoreWebView2` instance:
1. **Active Tab**: Its controller window handle (`HWND`) is visible and scaled to fill the main client area below the navigation toolbar.
2. **Inactive Background Tab (Awake)**: Its controller window is hidden (`put_IsVisible(FALSE)`). In this state, Chromium throttles timers to 1 Hz, reducing CPU consumption to < 0.1%.
3. **Suspended Background Tab**: When the inactivity threshold is exceeded, the controller captures the current URL, page title, favicon, and scroll offset via injected script, navigates the webview to `about:blank`, or releases the controller entirely. The tab remains in the tab bar with a `[Suspended]` visual indicator. Clicking the tab immediately reloads the URL and restores scroll position.

---

### 6. Inter-Process Communication (IPC)

1. **Rust Core to Web Content**:
   - Executed via `ICoreWebView2::ExecuteScript(javascript, handler)` for non-blocking asynchronous script evaluation (e.g., retrieving video playback state, scroll position).
   - Executed via `ICoreWebView2::PostWebMessageAsJson(json)` for structured bi-directional communication.
2. **Web Content to Rust Core**:
   - Web pages register message handlers communicating via `window.chrome.webview.postMessage(message)`.
   - The Rust host receives these messages via `ICoreWebView2WebMessageReceivedEventHandler`, permitting safe, sandboxed interaction for internal browser pages (`nexora://settings`, `nexora://diagnostics`).

---

### 7. Data Flow & Persistence Architecture

```
User Action (Navigate) ---> Omnibox Parsing ---> WebResourceRequested (Filter Check)
                                                      |
                                          +-----------+-----------+
                                          |                       |
                                       [Match]                [No Match]
                                          |                       |
                                     Block Request          Permit Request
                                     (Ad/Tracker)                 |
                                                                  v
                                                        WebView2 Network Engine
                                                                  |
                                                                  v
                                                      Navigation Completed
                                                                  |
                                                      Record in SQLite DB
                                                      - Chronological History
                                                      - FTS5 Full-Text Index
                                                      - Update Session State
```

All persistent operations on the SQLite database use explicit transactions wrapped in the WAL journal, ensuring that sudden system power outages or hard terminations do not corrupt bookmarks or session history.
