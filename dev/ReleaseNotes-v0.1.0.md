# Nexora v0.1.0

Initial public release of Nexora, a lightweight Windows desktop browser built with native Rust, Win32, and Direct3D 11 hardware decoding.

## Features

- **Native Architecture**: 1.46 MB compiled Rust core handling windowing, custom chrome, tab management, and local SQLite storage with low background resource overhead (4.8 MB – 18 MB idle working set).
- **Hardware-Accelerated Streaming**: Direct3D 11 and NVDEC GPU offload for 1080p60 and 4K video playback with Widevine DRM support.
- **Find in Page (`Ctrl+F`)**: Floating search toolbar with live match highlighting, result counter, previous/next navigation (`Enter` / `Shift+Enter`), and `Escape` dismiss.
- **Tab Audio Controls**: Real-time audio badges (playing, muted, paused) on individual tabs, tab muting via click or context menu, and background playback protection.
- **Tab Management**: Drag-and-drop reordering, pinned tabs, duplicate tab actions, and tab state restoration.
- **Per-Site Zoom**: Zoom levels persisted per-origin in SQLite (`Ctrl+Plus`, `Ctrl+Minus`, `Ctrl+0`, `Ctrl+Wheel`).
- **Smart Tab Suspension**: Automatic controller unloading for inactive background tabs, reducing multi-tab memory consumption while preserving active media.
- **Privacy & Content Filtering**: Embedded network filter blocking known trackers, telemetry beacons, and ads. Zero telemetry reporting.
- **Internal Pages**: Built-in dark-mode pages for Bookmarks (`nexora://bookmarks`), History (`nexora://history`), Settings (`nexora://settings`), and Diagnostics (`nexora://diagnostics`).

## Package Verification

| Asset | Size | SHA-256 Checksum |
| :--- | :--- | :--- |
| `Nexora-Setup-v0.1.0.exe` | 3.00 MB | `E35E6D77FA0CA1785FD3F0E7E79536810DD6847E2DC03689F03D0BE113933068` |
| `Nexora-v0.1.0-windows-x64-portable.zip` | 1.33 MB | `7B79F55995EBE2468B22174538121EAE0802CF3D123C68DDC8B07D9EF1EE1F0C` |
| `Nexora.exe` (Core Binary) | 1.49 MB | `CA751D1F5D9065042DEA301A2B32FE4E05895A8B87515D0BE13696E0D0BC0C7C` |

## Installation

### Windows Installer (`.exe`)
Download and run `Nexora-Setup-v0.1.0.exe`. It installs to `%LOCALAPPDATA%\Programs\Nexora`, creates Start Menu and Desktop shortcuts, and registers an uninstaller in Windows Settings. Silent installation is supported with `/S`.

### Portable Archive (`.zip`)
Download and extract `Nexora-v0.1.0-windows-x64-portable.zip`. Run `Nexora.exe` directly without installation or administrator privileges.

