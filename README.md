<div align="center">

<img src="assets/branding/app_icon.png" width="128" height="128" alt="Nexora Logo"/>

# Nexora

**A lightweight, streaming-focused desktop browser for Windows.**  
Built with native Rust, Win32, and Direct3D 11 hardware decoding.

[![Release](https://img.shields.io/badge/release-v0.1.0-crimson.svg)](https://github.com/Nexora/Nexora/releases)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%20%7C%2011%20x64-blue.svg)](https://www.microsoft.com/windows)
[![Binary Size](https://img.shields.io/badge/binary%20size-1.46%20MB-emerald.svg)](https://github.com/Nexora/Nexora/releases)
[![Tests](https://img.shields.io/badge/tests-72%2F72%20passing-brightgreen.svg)](tests/)
[![License](https://img.shields.io/badge/license-MIT-gray.svg)](LICENSE)

[**Download Installer (.exe)**](https://github.com/Nexora/Nexora/releases) • [**Download Portable (.zip)**](https://github.com/Nexora/Nexora/releases) • [**Performance Benchmarks**](BENCHMARKS.md) • [**Architecture**](#architecture--process-model) • [**Building from Source**](#building-from-source)

</div>

---

## Overview

Nexora is a lightweight Windows desktop browser designed for video streaming, low background resource utilization, and privacy.

Mainstream browsers routinely consume 1.5 GB to 3.0 GB of RAM with a standard workload of tabs, bundle telemetry collectors, and execute background tasks for news feeds and shopping assistants. Nexora separates the application frame and browser management from web content rendering:
- A compiled **1.46 MB native Rust executable** handles the Win32 window message loop, custom chrome rendering, tab lifecycles, memory suspension, SQLite persistence, and network request filtering.
- The Windows-native **WebView2 Evergreen runtime** handles HTML5 layout, CSS styling, JavaScript execution (V8), and Direct3D 11 GPU-accelerated video decoding.

The result is a browser that launches in under 700 ms, maintains an idle host working set between 4.8 MB and 18 MB RAM, and plays 1080p60/4K video with certified Widevine DRM without running extraneous background services.

---

## Architecture & Process Model

Nexora uses a two-tier process architecture that isolates native application management from web page rendering:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        NEXORA HOST (Nexora.exe)                        │
│                                                                        │
│  • Pure Rust executable (1.46 MB), native Win32 & double-buffered GDI │
│  • Memory usage: 4.8 MB – 18 MB RAM (0.0% CPU at idle)                 │
│  • Window management: DWM borderless frame, snap, and resize borders   │
│  • High-DPI icon atlas (48x48px supersampled Lucide icons)             │
│  • Tab Manager: drag-and-drop reordering, pinning, hover previews      │
│  • Audio control: per-tab mute toggling via COM IsMuted API            │
│  • Embedded SQLite (WAL mode): history, bookmarks, per-site zoom       │
│  • SmartSuspensionEngine & Working Set memory trimmer                  │
│  • Bloom filter AdBlock & Zero-Telemetry network request interceptor   │
└──────────────────┬─────────────────────────────────┬───────────────────┘
                   │ COM Interface                   │ COM Interface
                   ▼                                 ▼
┌──────────────────────────────────┐ ┌───────────────────────────────────┐
│      ACTIVE TAB CONTROLLER       │ │     SUSPENDED TAB CONTROLLER      │
│       (msedgewebview2.exe)       │ │     (Memory Completely Freed)     │
│                                  │ │                                   │
│ • State: SetIsVisible(true)      │ │ • Controller: Closed / Destroyed  │
│ • HTML5, CSS layout, V8 engine   │ │ • Tab state preserved in Rust     │
│ • Direct3D 11 / NVDEC GPU decode │ │ • Memory held: ~1.5 KB per tab    │
│ • Synchronized bounds to Win32   │ │ • Restored on tab click           │
└──────────────────────────────────┘ └───────────────────────────────────┘
```

### Why WebView2?

Building a production-ready desktop browser involves practical engineering trade-offs:

1. **Web Standards Compatibility**: Modern web specifications (HTML5, CSS Flexbox/Grid/Subgrid, WebGL 2, WebGPU, WebAssembly, WebRTC, and modern ECMAScript) exceed 35 million lines of specifications and code. A scratch engine cannot reliably display complex single-page applications like Google Docs, Figma, Discord, or YouTube.
2. **Commercial DRM (Widevine / PlayReady)**: Protected streaming services (Netflix, Spotify, Disney+, YouTube Premium) require Google-certified Widevine CDM or Microsoft PlayReady hardware decryption. Independent custom engines fail DRM negotiation and cannot stream high-definition content.
3. **Zero-Copy Video Pipelines**: WebView2 interfaces directly with Windows Direct3D 11 and DXVA2 hardware decoders. Video bitstreams decode directly on the GPU without intermediate memory copies across host RAM.
4. **Security Updates**: Web security patches and zero-day vulnerability fixes (CVEs) are delivered directly by the Windows OS component, removing the burden of downloading 200 MB browser updates every fortnight.
5. **Executable Footprint**: By utilizing the runtime already present on Windows 10 and 11, Nexora ships as a **1.46 MB binary** rather than bundling an entire 200 MB Chromium distribution.

---

## System Usage & Benchmarks Summary

The following measurements reflect committed working set (`WorkingSet64`) recorded on a test system running Windows 11 (x64) with an Intel Core i7-3770, 16 GB DDR3, and an NVIDIA GeForce GT 730:

| Workload | Host Process (`Nexora.exe`) | WebView2 Runtime (`msedgewebview2.exe`) | Total Nexora Footprint | Chrome 124 (Full Tree) | Difference |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **Start Page (`nexora://home`)** | **~6.5 MB** | **~48.0 MB** | **~54.5 MB** | ~185.0 MB | **-70.5%** |
| **1 Static Tab (Wikipedia)** | **~9.2 MB** | **~70.7 MB** | **~79.9 MB** | ~230.0 MB | **-65.2%** |
| **YouTube 1080p60 (Active Playback)** | **~14.1 MB** | **~249.1 MB** | **~263.2 MB** | ~420.0 MB | **-37.3%** |
| **YouTube 4K30 VP9 (Active Playback)** | **~16.8 MB** | **~406.2 MB** | **~423.0 MB** | ~680.0 MB | **-37.8%** |
| **5 Open Tabs (News, Docs, Wiki)** | **~13.5 MB** | **~287.3 MB** | **~300.8 MB** | ~780.0 MB | **-61.4%** |
| **10 Open Tabs (Active / Mixed)** | **~18.2 MB** | **~527.0 MB** | **~545.2 MB** | ~1,450.0 MB | **-62.4%** |
| **10 Open Tabs (Suspended)** | **~14.6 MB** | **~72.0 MB** | **~86.6 MB** | ~1,100.0 MB | **-92.1%** |
| **Idle CPU (Background Tab)** | **0.0%** | **0.0% – 0.1%** | **0.0% – 0.1%** | 0.8% – 2.5% | **Zero idle drain** |
| **Video CPU (1080p60 AVC)** | **0.1%** | **3.2%** (GPU Decode) | **3.3%** | 7.2% | **-54.2%** |
| **On-Disk Executable Size** | **1.46 MB** | **0 MB** (System component) | **1.46 MB** | ~180 MB | **-99.2%** |

*For complete benchmark data, process trees, and reproducibility scripts, see [BENCHMARKS.md](BENCHMARKS.md).*

---

## Features

### Tab Management & Navigation
- **Drag-and-Drop Tab Rearrangement**: Reorder tabs smoothly by dragging left or right. A 5-pixel drag threshold prevents accidental activation, and a vertical insertion indicator shows the drop target. Underlying WebView2 controllers are preserved during reordering to avoid page reloads.
- **Pinned Tabs**: Pin frequently used tabs to the left of the tab bar. Pinned tabs collapse to a compact 36px favicon strip, are protected from tab suspension, and persist across browser restarts.
- **Tab Audio Controls**: Each tab displays its live audio state (`Volume2` playing, `VolumeX` muted, `Volume1` paused). Clicking the audio icon mutes or unmutes the tab directly without switching focus. Context menu provides "Mute Tab", "Unmute Tab", "Mute Other Tabs", and "Unmute All Tabs".
- **Tab Hover Previews**: Hovering over any tab for 450 ms opens a floating preview card showing the page title, URL, rendered thumbnail, and audio/pin/suspension state.
- **Duplicate Tab**: Quickly clone any tab through the tab context menu, creating an independent tab instance inheriting the original URL and navigation context.
- **Smart Tab Suspension**: Background tabs inactive past the configured timeout (default: 10 minutes) automatically release their WebView2 controllers, reducing memory consumption by up to 75%. Tabs with active audio/video playback, downloads, pinned status, or Picture-in-Picture are automatically exempted.

### In-Page Tools & Controls
- **Find in Page (<kbd>Ctrl</kbd> + <kbd>F</kbd>)**: A floating search bar positioned at the upper right of the web content area. Features non-destructive text matching, live match counter (`current / total`), Enter / Shift+Enter navigation, and Escape to dismiss.
- **Per-Site Persistent Zoom**: Adjust zoom levels (50% to 200%) using <kbd>Ctrl</kbd> + <kbd>+</kbd>, <kbd>Ctrl</kbd> + <kbd>-</kbd>, <kbd>Ctrl</kbd> + <kbd>0</kbd>, or <kbd>Ctrl</kbd> + mouse wheel. Zoom levels are saved per website origin in SQLite and automatically restored on return visits.
- **Website Permissions Panel**: Shield icon dropdown in the omnibox displaying per-site permissions (Camera, Microphone, Geolocation, Notifications, Autoplay) with instant toggles saved to SQLite.

### Media & Video Streaming
- **Direct3D 11 & NVDEC Offloading**: Hardware-accelerated video decoding for H.264, VP9, and AV1 video streams via native GPU pipelines.
- **Widevine DRM Support**: Hardware-backed decryption pipeline supporting Netflix, Spotify, YouTube Premium, Disney+, and Amazon Prime Video.
- **Advanced Picture-in-Picture (<kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>P</kbd>)**: Detachable always-on-top floating player window with hover controls, volume toggle, and fullscreen return. Playback continues uninterrupted when switching tabs or minimizing the main browser.
- **Borderless Fullscreen (<kbd>F11</kbd>)**: True full-display presentation without window borders or taskbar intrusion.

### Privacy, Performance & Persistence
- **Built-in Ad & Tracker Blocker**: Intercepts outgoing HTTP requests before dispatch using an in-memory Bloom filter and rules engine. Blocks advertising domains, tracking pixels, and cryptominers with zero extension overhead.
- **Zero Telemetry**: All analytical, tracking, and diagnostic telemetry endpoints (Microsoft, Google, and third-party) are permanently blocked.
- **Embedded SQLite (WAL Mode)**: High-speed local database storing history, bookmarks, per-site permissions, and zoom factors with atomic write operations.
- **Session Checkpointing**: Atomic session state serialization to disk after every navigation, enabling crash recovery upon restart.

---

## Internal Browser Pages

Nexora provides built-in internal utility pages rendered directly by the host:

| Page | URL | Description |
| :--- | :--- | :--- |
| **Home** | `nexora://home` | Minimal start page with web search, customizable quick-launch shortcuts, and bookmarks. |
| **Settings** | `nexora://settings` | Configuration interface for theme, search provider, suspension timeouts, ad blocking, and hardware switches. |
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
| <kbd>Ctrl</kbd> + <kbd>F</kbd> | Open Find in Page toolbar |
| <kbd>Enter</kbd> / <kbd>Shift</kbd> + <kbd>Enter</kbd> | Next / Previous match in Find in Page |
| <kbd>Ctrl</kbd> + <kbd>+</kbd> or <kbd>=</kbd> | Zoom in (+10%) |
| <kbd>Ctrl</kbd> + <kbd>-</kbd> | Zoom out (-10%) |
| <kbd>Ctrl</kbd> + <kbd>0</kbd> | Reset zoom to 100% |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>P</kbd> | Toggle Picture-in-Picture window |
| <kbd>Ctrl</kbd> + <kbd>L</kbd> / <kbd>Alt</kbd> + <kbd>D</kbd> | Focus address bar |
| <kbd>Ctrl</kbd> + <kbd>R</kbd> / <kbd>F5</kbd> | Reload current page |
| <kbd>Alt</kbd> + <kbd>Left</kbd> | Navigate back |
| <kbd>Alt</kbd> + <kbd>Right</kbd> | Navigate forward |
| <kbd>F11</kbd> | Toggle borderless fullscreen |
| <kbd>Ctrl</kbd> + <kbd>D</kbd> | Bookmark current page |
| <kbd>Ctrl</kbd> + <kbd>B</kbd> | Open bookmarks manager (`nexora://bookmarks`) |
| <kbd>Ctrl</kbd> + <kbd>H</kbd> | Open history manager (`nexora://history`) |
| <kbd>Ctrl</kbd> + <kbd>,</kbd> | Open settings (`nexora://settings`) |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>D</kbd> | Open diagnostics (`nexora://diagnostics`) |

---

## Installation & Distribution

### Option 1: Standalone Windows Installer (.exe)
1. Download `Nexora-Setup-v0.1.0.exe` from [Releases](https://github.com/Nexora/Nexora/releases).
2. Run the executable. The installer provides:
   - Installation to `%LOCALAPPDATA%\Programs\Nexora`.
   - Windows Desktop and Start Menu shortcuts.
   - Clean uninstaller (`uninstall.exe`) registered in Windows "Installed apps".
   - Silent installation support via `/S` flag: `Nexora-Setup-v0.1.0.exe /S`.

### Option 2: Portable Archive (.zip)
1. Download `Nexora-v0.1.0-windows-x64-portable.zip`.
2. Extract the archive into any folder.
3. Run `Nexora.exe`. No elevated permissions or registry modifications are required.

### Option 3: Building from Source

#### Prerequisites
- Windows 10 (Build 1809+) or Windows 11 (64-bit)
- [Rust toolchain](https://www.rust-lang.org/tools/install) (stable channel, 1.80+)
- Microsoft Edge WebView2 Runtime (installed by default on Windows 10/11)

```powershell
# 1. Clone the repository
git clone https://github.com/Nexora/Nexora.git
cd Nexora

# 2. Build the release binary
cargo build --release

# 3. Embed Windows application icon & version manifest
python scripts/embed_resources.py

# 4. Build the standalone installer (optional)
cargo build --release --manifest-path installer/Cargo.toml

# 5. Launch Nexora
.\target\release\Nexora.exe
```

---

## Testing & Verification

The codebase includes an automated unit test suite covering window layout, tab management, drag-and-drop mechanics, audio state synchronization, SQLite persistence, Bloom filter ad blocking, per-site zoom storage, and DRM key system probes:

```powershell
cargo test
```

```text
running 72 tests
test app::config::tests::test_default_config ... ok
test engine::crash_handler::tests::test_crash_controller_silent_reload_and_throttling ... ok
test media::drm::tests::test_drm_key_systems ... ok
test media::gpu::tests::test_gpu_capabilities_switches ... ok
test media::pip::tests::test_pip_window_lifecycle ... ok
test navigation::omnibox::tests::test_parse_input_direct_urls ... ok
test performance::suspension::tests::test_suspension_engine_rules ... ok
test performance::memory::tests::test_trim_working_set ... ok
test privacy::adblock::tests::test_adblock_engine_blocking_and_stats ... ok
test privacy::permissions::tests::test_permissions_crud_and_defaults ... ok
test storage::db::tests::test_sqlite_in_memory_connection_and_crud ... ok
test storage::zoom::tests::test_zoom_crud ... ok
test tabs::tab_manager::tests::test_tab_audio_and_muting ... ok
test tabs::tab_manager::tests::test_tab_drag_reordering ... ok
test tabs::tab_manager::tests::test_tab_pinning_and_boundaries ... ok
test tabs::tab_manager::tests::test_tab_duplication ... ok
test ui::icons::tests::test_lucide_svg_generation ... ok
test ui::layout::tests::test_browser_layout_rects ... ok
...
test result: ok. 72 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.80s
```

---

## License

Nexora is open-source software licensed under the [MIT License](LICENSE).
