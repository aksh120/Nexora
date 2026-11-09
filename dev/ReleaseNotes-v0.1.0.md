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
| `Nexora-Setup-v0.1.0.exe` | 3.07 MB | `9815B542663AB5EBC8F16B74EACA1F8E01968610464DB293184BE54FC752A7A1` |
| `Nexora-v0.1.0-windows-x64-portable.zip` | 1.36 MB | `1103A9F91A1632D86ECFD9E7A679B08D0DEEF81AE77EED162F4C99595B3A3626` |
| `Nexora.exe` (Core Binary) | 1.56 MB | `790846600AA1E766EA8ABEC2C5BCB7BE04D1276A27DACAF51E003617E4A41BEB` |

## Installation

### Windows Installer (`.exe`)
Download and run `Nexora-Setup-v0.1.0.exe`. It installs to `%LOCALAPPDATA%\Programs\Nexora`, creates Start Menu and Desktop shortcuts, and registers an uninstaller in Windows Settings. Silent installation is supported with `/S`.

### Portable Archive (`.zip`)
Download and extract `Nexora-v0.1.0-windows-x64-portable.zip`. Run `Nexora.exe` directly without installation or administrator privileges.

