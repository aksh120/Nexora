<p align="center">
  <img src="../assets/branding/logo.svg" width="380" alt="Nexora - The Quantum Prism Logo"/>
</p>

# NEXORA: Master Engineering & Product Plan
## Ultra-Lightweight, Streaming-First Desktop Browser

---

### 1. Executive Summary & Product Vision

**NEXORA** is a next-generation, ultra-lightweight desktop browser engineered specifically for Windows 10 and Windows 11. Designed from the ground up with a **streaming-first philosophy**, NEXORA redefines the desktop web experience by aggressively stripping away the bloated background processes, ad-tech telemetry, news feeds, shopping sidebars, and unnecessary abstractions that plague modern commercial browsers (such as Google Chrome and Microsoft Edge).

#### Product Philosophy
> *"Maximum performance. Minimum overhead. Complete browsing freedom."*

NEXORA treats media playback smoothness and system responsiveness as primary constraints. In an era where opening 10 tabs in a standard browser routinely consumes 2 to 4 GB of RAM and 15–25% of background CPU cycles, NEXORA maintains a razor-sharp memory and CPU profile through intelligent process management, native Rust orchestration, and fine-grained smart tab suspension, without breaking website compatibility or degrading video fidelity.

---

### 2. Functional Requirements

1. **Tab & Session Management**:
   - Compact, low-profile tab bar with drag-to-reorder, instant close, and new tab controls.
   - **Smart Tab Suspension**: Suspends background tabs based on inactivity duration, system memory pressure, and tab state (media playing, downloads active, forms filled).
   - **Intelligent Lazy Session Restoration**: On browser startup, tabs from the prior session are restored into a lightweight stub state; only the active foreground tab is loaded immediately into memory.

2. **Advanced URL / Omnibox Navigation**:
   - Unified input field distinguishing between direct URLs, search queries, IP addresses, and localhost ports.
   - Fast autocomplete and instant search suggestions powered by local history, bookmarks, and user-configurable search engines.
   - Real-time HTTPS and TLS security indicators.
   - Zero search telemetry or keystroke logging transmitted to third-party ad brokers.

3. **Streaming & Media Engine**:
   - First-class support for YouTube (up to 4K 60fps / HDR), Netflix, Amazon Prime Video, JioHotstar, Sony LIV, ZEE5, Twitch, and Spotify Web.
   - Hardware-accelerated video decoding via Direct3D 11 and DXVA2 (DirectX Video Acceleration).
   - Legitimate Widevine & PlayReady Digital Rights Management (DRM) support via standard Encrypted Media Extensions (EME).
   - Picture-in-Picture (PiP) floating window with always-on-top, playback controls, and seamless suspension exemption.
   - Active playback prioritization: Media playback tabs are locked in the highest memory and scheduling priority tier.

4. **Privacy & Network Filtering**:
   - Built-in, native Rust ad and tracker blocker using network interception hooks (`AddWebResourceRequestedFilter`) and pre-compiled EasyList/uBlock rules.
   - Third-party tracking cookie blocking and fingerprinting protection.
   - Per-site permissions management (Camera, Microphone, Geolocation, Autoplay, Notifications).
   - Per-site and global blocking statistics.

5. **Local Data & State Persistence**:
   - Fully persistent cookies surviving browser restarts.
   - Chronological, full-text searchable history with customizable retention limits.
   - Hierarchical bookmarks system (folders, toolbar, tagging, fast search).
   - Smart HTTP disk and memory cache with strict configurable size caps (e.g., 250MB - 1GB) preventing runaway cache inflation.

6. **Diagnostics & User Configuration**:
   - Minimalist, distraction-free native settings interface.
   - Real-time diagnostics dashboard tracking process-tree RAM usage, CPU load, GPU decoding engine status, and active vs. suspended tab metrics.
   - Native dark, light, and system-adaptive visual themes.

---

### 3. Non-Functional & Technical Requirements

| Dimension | Specification |
| :--- | :--- |
| **Operating System** | Windows 10 (Build 1809+ / x64) and Windows 11 (x64) |
| **Primary Language** | Rust (Edition 2021/2024), maximizing safety, zero-cost abstractions, and deterministic memory management |
| **Rendering Subsystem** | Microsoft WebView2 Evergreen COM API (Chromium-based rendering engine) |
| **Local Storage Engine** | SQLite 3 (WAL mode) with structured schema migrations |
| **Hardware Baseline** | Intel Core i7-3770 @ 3.40 GHz (4C/8T), 16 GB DDR3, NVIDIA GeForce GT 730 (Kepler / 4 GB VRAM) |
| **Idle Memory Target** | < 120 MB total working set with 1 tab open; < 350 MB with 10 tabs under Smart Suspension |
| **Video Decoding Target** | Direct hardware offload via DXVA2/D3D11 for H.264 / VP8 / VP9; zero dropped frames at 1080p60 |
| **Startup Latency** | Cold start < 800 ms; Warm start < 350 ms |
| **Telemetry & Network Exfiltration** | Strict Zero: 0 tracking pings, 0 user analytics, 0 cloud sync backdoors |

---

### 4. Browser Engine Selection Analysis

Building an independent web rendering engine (HTML parser, CSS layout engine, JavaScript runtime, WebAssembly, WebGL, WebGPU, WebRTC, Media Foundation stack) from scratch is practically intractable for a production browser requiring full modern web and streaming compatibility. Therefore, an embedded browser engine must be utilized.

#### Evaluation of Candidate Engines:

```
+-----------------------------------+--------------------+--------------------+--------------------+
| Criteria                          | Microsoft WebView2 | CEF (Chromium EF)  | Wry / Tao Wrapper  |
+-----------------------------------+--------------------+--------------------+--------------------+
| Binary / Installer Overhead       | < 15 MB (Thin)     | 180 - 250 MB       | ~15 MB             |
| System Memory Footprint           | Minimal (Shared)   | High (Dedicated)   | Minimal (Shared)   |
| Hardware Video Decoding (DXVA)    | Native Direct3D 11 | Yes                | Yes (via WV2)      |
| Legitimate DRM (Widevine/PlayReady)| Yes (Native EME)  | No (License Block) | Limited exposure   |
| Streaming Services Compatibility  | 100% (Netflix/Hot) | Poor / Broken DRM  | 100% (Netflix/Hot) |
| Maintenance & Security Updates    | Auto (Evergreen OS)| Manual Recompiles  | Upstream Dependent |
| Direct Low-Level COM Interop      | Full Native C/Rust | C++ API            | Restricted wrapper |
+-----------------------------------+--------------------+--------------------+--------------------+
```

#### Final Selection: **Microsoft WebView2 Evergreen (via Rust COM Bindings)**
1. **DRM Compliance**: WebView2 natively supports PlayReady and Widevine CDM through the OS-installed Edge runtime under legitimate Microsoft licensing, granting instant, legally compliant access to Netflix (1080p/4K), Prime Video, JioHotstar, Sony LIV, and ZEE5.
2. **Resource Efficiency**: By utilizing the OS-serviced Evergreen runtime, WebView2 shares DLL working sets and runtime memory structures with Windows services.
3. **Rust Interoperability**: Through `windows-rs` and `webview2-com`, the Rust core maintains direct low-level control over `ICoreWebView2_20`, `ICoreWebView2Controller4`, `ICoreWebView2Profile`, and `ICoreWebView2CookieManager` without high-overhead wrappers.

---

### 5. Architecture & Subsystem Blueprint

NEXORA utilizes a **dual-layer process architecture**:
1. **The Native Orchestrator (Nexora.exe)**: Pure Rust executable hosting the Win32 message loop, tab management state machine, smart suspension engine, SQLite persistence layer, native UI widgets, and network request filter rules.
2. **The Rendering Subsystem**: Isolated Chromium rendering processes spawned per-tab or pooled by domain, running in restricted Windows sandboxes with hardware-accelerated Direct3D swap chains.

```
                      +---------------------------------------+
                      |             NEXORA.EXE                |
                      |   (Pure Rust Native Application)      |
                      +---------------------------------------+
                                          |
            +-----------------------------+-----------------------------+
            |                             |                             |
+-----------------------+     +-----------------------+     +-----------------------+
|  Win32 Window Frame   |     | Tab Manager & Smart   |     | SQLite Persistence    |
|  - Custom Tab Bar     |     |   Suspension Engine   |     |   (WAL Mode)          |
|  - Omnibox / URL Bar  |     | - Idle Tab Sweeper    |     | - Bookmarks           |
|  - Ad Filter Intercept|     | - Priority Scheduler  |     | - History (FTS5)      |
|  - Settings & Diag    |     | - Working Set Trimmer |     | - Preferences         |
+-----------------------+     +-----------------------+     +-----------------------+
            |                             |
            +-----------------------------+
                          |
             [ WebView2 COM / Win32 Child ]
                          |
+-------------------------------------------------------------------+
|               Chromium Sandboxed Process Subsystem                |
|  +--------------------+  +--------------------+  +--------------+ |
|  | GPU / DXVA Process |  | Active Tab Renderer|  | Suspended Tab| |
|  | (D3D11 Compositor) |  | (YouTube 1080p60)  |  | (Stub / Dorm)| |
|  +--------------------+  +--------------------+  +--------------+ |
+-------------------------------------------------------------------+
```

---

### 6. Database & Storage Strategy

- **Engine**: SQLite 3 using the `rusqlite` crate.
- **Journal Mode**: `WAL` (Write-Ahead Logging) enabling concurrent readers without blocking writes.
- **Synchronous Flag**: `NORMAL` (safe under WAL mode while minimizing disk flush latency).
- **Storage Location**: `%APPDATA%\Nexora\Data\nexora.db`.
- **Engine Data Isolation**: Engine user data, cache, and cookies are directed to `%LOCALAPPDATA%\Nexora\EngineData` to keep user preferences and structured databases separate from ephemeral browser cache blobs.

---

### 7. Performance Strategy & Smart Tab Suspension

1. **Smart Suspension Decision Matrix**:
   - Background tab idle timer: configurable (default 10 minutes).
   - System memory pressure: monitored via `GlobalMemoryStatusEx`. If available RAM < 15%, idle threshold drops to 3 minutes.
   - Exclusion criteria: Any tab with active audio (`IsDocumentPlayingAudio`), active video element, running download, or recent form submission is strictly immune from suspension.
2. **Suspension Execution**:
   - Page state captured: URL, title, favicon URL, and scroll position via lightweight script injection.
   - WebView2 controller window is hidden and page navigated to `about:blank` or an internal memory-efficient stub, releasing the Chromium renderer working set.
   - Windows API `SetProcessWorkingSetSize` is invoked on idle subprocesses to trim unused virtual pages back to the system pool.
3. **Lazy Restoration**:
   - On browser startup, tabs are instantiated in the UI as tabs with cached title/favicon. The underlying WebView2 instance is only spawned when the user switches to the tab.

---

### 8. Security & Privacy Strategy

1. **Isolation**: Sandboxed renderer processes run with low integrity levels.
2. **Certificate Validation**: All TLS connections enforce strict certificate chain validation without bypass flags.
3. **Zero Telemetry**: No analytical pings, crash telemetry daemons, or remote configuration fetches. Filter list updates and software update checks occur only on explicit user schedule or request via direct HTTPS.
4. **Local Data Security**: SQLite database and configuration files are placed in protected user profile directories (`%APPDATA%`), inheriting Windows access control lists (ACLs).

---

### 9. Deployment Strategy

- **Thin Installer (`Nexora-Setup.exe`)**:
  - Validates Windows 10/11 compatibility and architecture (x64).
  - Checks for the presence of the Microsoft WebView2 Evergreen Runtime. If missing (rare on modern Win10/Win11), bootstraps the official Microsoft silent installer.
  - Deploys `Nexora.exe`, default assets, and registers clean Windows shell associations for HTTP/HTTPS/HTML.
  - Zero bundled bloatware, toolbars, or silent background update services.

