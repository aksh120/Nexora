<div align="center">

<img src="assets/branding/app_icon.png" width="128" height="128" alt="Nexora Logo"/>

# Nexora

**A lightweight, streaming-focused desktop browser for Windows.**  
Built with native Rust, Win32, and Direct3D 11 hardware decoding.

[![Release](https://img.shields.io/badge/release-v0.1.0-crimson.svg)](https://github.com/Nexora/Nexora/releases)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%20%7C%2011%20x64-blue.svg)](https://www.microsoft.com/windows)
[![Binary Size](https://img.shields.io/badge/binary%20size-1.35%20MB-emerald.svg)](https://github.com/Nexora/Nexora/releases)
[![Tests](https://img.shields.io/badge/tests-65%2F65%20passing-brightgreen.svg)](tests/)
[![License](https://img.shields.io/badge/license-MIT-gray.svg)](LICENSE)

[**Download Portable (v0.1.0)**](https://github.com/Nexora/Nexora/releases) • [**Benchmarks**](BENCHMARKS.md) • [**Architecture**](#architecture--technical-design) • [**Build Instructions**](#building-from-source)

</div>

---

## Overview

Nexora is a lightweight Windows desktop browser built for users who want fast video streaming, minimal background resource usage, and strict privacy without the bloat of modern commercial browsers.

Standard browsers often consume 1.5 to 3 GB of RAM with a handful of tabs open, run dozens of background helper tasks, and bundle news feeds, shopping trackers, and telemetry services. Nexora decouples browser interface management from web rendering: a compiled 1.35 MB Rust executable handles windowing, tab lifecycle, memory reclamation, and storage, while the Windows-native WebView2 runtime handles web rendering and hardware-accelerated video decoding.

### Key Characteristics

- **Lightweight Native Core**: The browser chrome, tab manager, database, and window frame are compiled into a single 1.35 MB native Rust binary.
- **Hardware-Accelerated Streaming**: Direct3D 11 and NVDEC zero-copy video pipelines decode 1080p60 and 4K video directly on the GPU, keeping host CPU usage under 3.5%.
- **Smart Tab Suspension**: Inactive background tabs unload their webview controllers automatically, reducing multi-tab memory consumption by up to 75%.
- **Zero Telemetry**: All background analytics, search reporting, and telemetry endpoints are permanently blocked at the network layer.
- **Native Win32 Interface**: Clean, restrained dark-mode interface with custom DWM drop-shadows, 2X Retina Lucide icons, and zero dashboard noise.

---

## System Usage & Benchmarks

Nexora uses a two-tier process model:
1. **Host Process (`Nexora.exe`)**: The native Rust process managing Win32 windowing, custom chrome rendering, tabs, and local SQLite storage. It typically consumes **4.8 MB to 18 MB** of RAM and 0% CPU at idle.
2. **Web Engine Subprocesses (`msedgewebview2.exe`)**: Spawned on demand to parse HTML5/CSS, execute JavaScript, and decode streaming media.

### Resource Usage Breakdown

The table below shows measured system resource usage across common workloads, tested on Windows 11 (x64) with an Intel Core i7-3770, 16 GB DDR3, and an NVIDIA GeForce GT 730:

| Workload | Host Process (`Nexora.exe`) | WebView2 Runtime (`msedgewebview2.exe`) | Total Nexora Footprint | Google Chrome (Reference) | Difference |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **Cold Startup (`nexora://home`)** | **~6.5 MB** | **~48.0 MB** | **~54.5 MB** | ~380 MB | **-85.6%** |
| **1 Static Tab (Wikipedia)** | **~9.2 MB** | **~70.7 MB** | **~79.9 MB** | ~230 MB | **-65.2%** |
| **YouTube 1080p60 (Active Playback)** | **~14.1 MB** | **~249.1 MB** | **~263.2 MB** | ~420 MB | **-37.3%** |
| **YouTube 4K30 VP9 (Active Playback)** | **~16.8 MB** | **~406.2 MB** | **~423.0 MB** | ~680 MB | **-37.8%** |
| **5 Open Tabs (News, Docs, Wiki)** | **~13.5 MB** | **~287.3 MB** | **~300.8 MB** | ~780 MB | **-61.4%** |
| **10 Open Tabs (Active / Mixed)** | **~18.2 MB** | **~527.0 MB** | **~545.2 MB** | ~1,450 MB | **-62.4%** |
| **10 Open Tabs (Suspended)** | **~14.6 MB** | **~72.0 MB** | **~86.6 MB** | ~1,100 MB | **-92.1%** |
| **Idle CPU (Background Tab)** | **0.0%** | **0.0% – 0.1%** | **0.0% – 0.1%** | 0.8% – 2.5% | **Zero idle drain** |
| **Video CPU (1080p60 AVC)** | **0.1%** | **3.2%** (GPU Decode) | **3.3%** | 7.2% | **-54.2%** |
| **On-Disk Executable Size** | **1.35 MB** | **0 MB** (System component) | **1.35 MB** | ~180 MB | **-99.2%** |

*For complete test methodology and detailed logs, see [BENCHMARKS.md](BENCHMARKS.md).*

---

## Architecture & Technical Design

### Why WebView2?

Building a web browser presents three potential approaches:

1. **Custom Engine from Scratch (e.g. Servo or toy engines)**: While attractive for theoretical purity, modern web standards (HTML5, CSS Flexbox/Grid, WebAssembly, WebRTC, modern ECMAScript) comprise over 35 million lines of specifications. A custom engine cannot reliably render complex modern web applications like Google Docs, Figma, Discord, or YouTube. Furthermore, custom engines lack Widevine DRM certification, which means services like Netflix, Spotify, and YouTube Premium cannot play protected content.
2. **Bundling a Full Chromium Fork (e.g. Brave, Vivaldi)**: Forking Chromium requires bundling 150–250 MB of binary dependencies, consumes 350+ MB of RAM just to initialize the browser shell, and requires continuous maintenance to keep up with bi-weekly security CVEs.
3. **The Nexora Hybrid Model**: Nexora couples a native Rust application shell with the Evergreen WebView2 runtime already built into Windows 10 and 11. This yields:
   - Full compatibility with the modern web.
   - Certified Widevine CDM hardware decryption for commercial streaming.
   - Direct3D 11 hardware-accelerated video decoding (NVDEC / QuickSync / AMF).
   - Bi-weekly zero-day security patches delivered by the OS.
   - A download size of only 1.35 MB, rather than 200 MB.

### How Nexora Manages WebView2

Nexora does not run as an Edge web shortcut or an Electron wrapper. The architecture is strictly decoupled:

```
┌─────────────────────────────────────────────────────────────────┐
│                     NEXORA NATIVE HOST (Rust)                   │
│                                                                 │
│  • Custom Win32 message loop with DWM 1px borderless margins    │
│  • 2X Retina Lucide icon chrome renderer (double-buffered GDI)  │
│  • Independent TabManager tracking tab state & history          │
│  • SmartSuspensionEngine & Working Set memory trimmer           │
│  • Embedded SQLite storage (WAL mode) & crash checkpointer      │
│  • Request-level AdBlock & Zero-Telemetry filter                │
└───────────────┬─────────────────────────────────┬───────────────┘
                │ COM Interface                   │ COM Interface
                ▼                                 ▼
┌───────────────────────────────┐ ┌───────────────────────────────┐
│     ACTIVE TAB CONTROLLER     │ │   SUSPENDED TAB CONTROLLER    │
│    (ICoreWebView2Controller)  │ │   (Memory Completely Freed)   │
│                               │ │                               │
│ • State: SetIsVisible(true)   │ │ • State: Closed / Destroyed   │
│ • Bounds: Synchronized to UI  │ │ • Placeholder: nexora://page  │
│ • Hardware Decoding: Enabled  │ │ • Memory Overhead: ~1.5 KB    │
│ • Audio Monitored (♪)         │ │ • Restores on tab focus       │
└───────────────────────────────┘ └───────────────────────────────┘
```

- **Per-Tab Controllers**: Each resident tab maintains its own `ICoreWebView2Controller`. Switching tabs uses native Win32 visibility toggles (`SetIsVisible(false)` on background tabs, `SetIsVisible(true)` on the target tab), ensuring tab states do not bleed into each other.
- **Smart Suspension**: Background tabs inactive for longer than the configured timeout (default: 10 minutes) have their controllers unmapped from memory, freeing 150–300 MB of RAM per tab. Active media tabs (playing audio or video) are automatically exempted.
- **Bidirectional IPC**: Internal pages (`nexora://home`, `nexora://settings`) communicate with the Rust host via JSON WebMessages (`window.chrome.webview.postMessage`), keeping UI state synced without running local HTTP servers.

---

## Features

### Interface & Navigation
- **Borderless Native Window**: Eliminates standard Windows title bars while preserving OS window snapping, drop shadows, and resize borders via `WM_NCCALCSIZE` and `WM_NCHITTEST`.
- **High-Resolution Vector Icons**: 48×48px supersampled vector icon atlas based on Lucide icons, ensuring crisp toolbar controls on high-DPI and scaled displays.
- **Smart Omnibox**: Automatically detects domain names, IP addresses, search terms, and internal `nexora://` schemes. Includes a live connection security indicator (HTTPS / Internal / HTTP).
- **Tab Audio Indicator**: Visual audio glyph (`♪`) displayed on tabs currently outputting sound.

### Media & Streaming
- **Direct3D 11 / NVDEC Offload**: Direct GPU video frame decoding for H.264, VP9, and AV1 video streams.
- **Widevine DRM Support**: Hardware-backed decryption for Netflix, Spotify, YouTube Premium, Disney+, and Prime Video.
- **Picture-in-Picture (PiP)**: Detachable floating video player that stays on top while switching tabs or applications.
- **Borderless Fullscreen**: True fullscreen mode via <kbd>F11</kbd> that expands across the entire monitor canvas.

### Privacy & Storage
- **Built-in Ad & Tracker Blocker**: Intercepts network requests before transmission using an in-memory Bloom filter and domain matching, blocking ads, tracking pixels, and miners without browser extension overhead.
- **Zero Telemetry**: All Microsoft, Google, and third-party analytical endpoints are permanently blocked.
- **Embedded SQLite (WAL Mode)**: High-speed local database for bookmarks and browsing history, supporting concurrent reads and atomic writes.
- **Session Crash Recovery**: Checkpoints session state atomically to disk on every navigation, offering instant tab restoration in the event of an abnormal shutdown.

---

## Internal Browser Pages

Nexora includes dedicated internal pages accessible via the address bar:

| Page | URL | Description |
| :--- | :--- | :--- |
| **Home** | `nexora://home` | Minimal start page with web search, customizable quick-access shortcuts, and recent bookmarks. |
| **Settings** | `nexora://settings` | Two-column configuration panel for general preferences, suspension timeouts, ad blocking, and hardware acceleration. |
| **Diagnostics** | `nexora://diagnostics` | Real-time system monitor showing host RAM, WebView2 memory, GPU decoding vendor, and active tab states. |
| **Bookmarks** | `nexora://bookmarks` | Searchable bookmark manager with folder organization and editing tools. |
| **History** | `nexora://history` | Chronological browsing timeline with date filters, search, and deletion tools. |

---

## Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| <kbd>Ctrl</kbd> + <kbd>T</kbd> | Open a new tab |
| <kbd>Ctrl</kbd> + <kbd>W</kbd> | Close active tab |
| <kbd>Ctrl</kbd> + <kbd>Tab</kbd> | Switch to next tab |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>Tab</kbd> | Switch to previous tab |
| <kbd>Ctrl</kbd> + <kbd>L</kbd> / <kbd>Alt</kbd> + <kbd>D</kbd> | Focus address bar |
| <kbd>Ctrl</kbd> + <kbd>R</kbd> / <kbd>F5</kbd> | Reload current page |
| <kbd>Alt</kbd> + <kbd>Left</kbd> | Navigate back |
| <kbd>Alt</kbd> + <kbd>Right</kbd> | Navigate forward |
| <kbd>F11</kbd> | Toggle borderless fullscreen |
| <kbd>Ctrl</kbd> + <kbd>D</kbd> | Bookmark current page |
| <kbd>Ctrl</kbd> + <kbd>B</kbd> | Open bookmarks (`nexora://bookmarks`) |
| <kbd>Ctrl</kbd> + <kbd>H</kbd> | Open history (`nexora://history`) |
| <kbd>Ctrl</kbd> + <kbd>,</kbd> | Open settings (`nexora://settings`) |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>D</kbd> | Open diagnostics (`nexora://diagnostics`) |

---

## Getting Started

### Option 1: Portable Version (Recommended)
1. Download `Nexora-v0.1.0-windows-x64-portable.zip` from [Releases](https://github.com/Nexora/Nexora/releases).
2. Extract the archive to any directory.
3. Launch `Nexora.exe`. No installation or administrator permissions are required.

### Option 2: Windows Installer
1. Download `Nexora-Setup-v0.1.0.exe`.
2. Run the setup wizard to install Nexora, register `nexora://` protocol associations, and create Start Menu and Desktop shortcuts.

### Option 3: Building from Source

#### Prerequisites
- Windows 10 (Build 1809+) or Windows 11 (x64)
- [Rust](https://www.rust-lang.org/tools/install) (stable channel, 1.80+)
- Microsoft Edge WebView2 Runtime (pre-installed on Windows 10/11)

```powershell
# 1. Clone repository
git clone https://github.com/Nexora/Nexora.git
cd Nexora

# 2. Build release executable
cargo build --release

# 3. Embed Windows application icon & metadata
python scripts/embed_resources.py

# 4. Run the browser
.\target\release\Nexora.exe
```

---

## Testing & Verification

All core subsystems (layout, tab management, smart suspension, SQLite persistence, ad blocking, and DRM probes) are verified through automated tests:

```powershell
# Run the test suite (65 tests)
cargo test
```

```text
running 65 tests
test app::config::tests::test_default_config ... ok
test engine::crash_handler::tests::test_crash_controller_silent_reload_and_throttling ... ok
test media::drm::tests::test_drm_key_systems ... ok
test media::gpu::tests::test_gpu_capabilities_switches ... ok
test navigation::omnibox::tests::test_parse_input_direct_urls ... ok
test performance::suspension::tests::test_suspension_engine_rules ... ok
test performance::memory::tests::test_trim_working_set ... ok
test privacy::adblock::tests::test_adblock_engine_blocking_and_stats ... ok
test storage::db::tests::test_sqlite_in_memory_connection_and_crud ... ok
test ui::icons::tests::test_lucide_svg_generation ... ok
...
test result: ok. 65 passed; 0 failed; 0 ignored; 0 measured; finished in 2.05s
```

---

## License

Nexora is open-source software licensed under the [MIT License](LICENSE).
