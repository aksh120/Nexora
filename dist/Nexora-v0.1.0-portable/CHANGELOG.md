# CHANGELOG

All notable changes to the NEXORA desktop browser are documented in this file.
This project adheres to [Semantic Versioning](https://semver.org/).

---

## [0.1.0] - 2026-11-07

### Initial Production Release - Ultra-Lightweight, Streaming-First Desktop Browser

#### Engine & Core Architecture
- **Rust Orchestration**: Native Win32 window management, event message pump, and high-performance COM interop (`windows-rs` v0.58).
- **WebView2 Evergreen Integration**: Embedded Chromium rendering pipeline with custom user data directory isolation (`%LOCALAPPDATA%\Nexora\User Data`).
- **Zero-Telemetry Enforcement**: All Chromium telemetry, component updates, domain reliability reporting, network prediction, crash reporting, and Google sync permanently disabled at the command-line boundary.
- **Dynamic OS SQLite Binding**: Zero-overhead integration with Windows OS `winsqlite3.dll` with WAL mode and memory-mapped I/O.

#### Media & Video Streaming
- **Direct3D 11 & DXVA2 Offload**: Hardware video decoding offload for H.264 (AVC1), VP8/VP9, and HEVC on NVIDIA Kepler (GT 730) and newer GPUs.
- **Commercial DRM Compliance**: Full Widevine CDM and Microsoft PlayReady EME compliance for Netflix, Prime Video, JioHotstar, Sony LIV, ZEE5, Twitch, and Spotify Web.
- **Picture-in-Picture (PiP)**: Native floating, always-on-top video overlay with aspect-ratio preservation and playback controls.
- **Active Media Priority Scheduler**: Automatic thread priority boosting (`ABOVE_NORMAL_PRIORITY_CLASS`) during video playback to prevent frame drops under system load.
- **Borderless Fullscreen**: Seamless HTML5 video fullscreen transition bypassing standard desktop compositing bottlenecks.

#### Performance Engineering
- **Smart Tab Suspension Engine**: Automatic memory reclamation for inactive background tabs while strictly exempting media playback, active downloads, and unsaved form inputs.
- **Native Working Set Trimmer**: Periodic and demand-driven working set trimming using Windows API `SetProcessWorkingSetSize`, maintaining idle host footprint at ~17 MB RAM.
- **Lazy Restore Pipeline**: Sub-400ms startup restoring only active foreground tab; background tabs instantiate on-demand with instant placeholder headers.
- **Dynamic Memory Pressure Monitor**: Dynamic sampling of `GlobalMemoryStatusEx` with adaptive suspension timeout scaling under system stress.

#### Privacy & Security
- **Native Ad & Tracker Blocker**: Sub-15 microsecond URL filtering engine with 79 pre-seeded ad, tracker, analytics, and cryptominer blocking rules.
- **Granular Permissions Engine**: Site-level permission policies (Camera, Microphone, Geolocation, Autoplay, Notifications, Clipboard) persisted in SQLite.
- **Streaming CDN Whitelisting**: Strict whitelisting for DRM key exchange and media chunk delivery across YouTube, Netflix, Disney/Hotstar, Prime, and Spotify.

#### User Interface & Experience
- **Minimalist Cyber Aesthetic**: Dark-mode visual identity with electric cyan accents (`#00E5FF`) and high-DPI vector SVG rendering.
- **Custom Frameless Title Bar**: Native Win32 frameless caption with window dragging, double-click maximize, and integrated custom caption controls.
- **Vector Line Navigation Icons**: High-precision GDI vector line art for Back, Forward, Reload, Home, Bookmark Star, and Menu.
- **Unified Omnibox**: Address and search bar supporting DuckDuckGo, Google, Bing, and Brave search engine fallbacks.
- **Internal Browser Pages**:
  - `nexora://bookmarks`: Visual bookmark manager with real-time search, inline creation, and 1-click launch.
  - `nexora://history`: Chronological browsing history with visit counts, timestamps, search, and clearing.
  - `nexora://settings`: Categorized configuration interface (General, Performance, Privacy, Video, Storage, About).
  - `nexora://diagnostics`: Real-time system diagnostics displaying live host RAM, system memory pressure, tab states, adblock metrics, and D3D11 GPU status.
- **Crash Recovery & Backup**: Atomic session checkpoints (`session_checkpoint.json`) and automated SQLite database integrity checks and backups.
