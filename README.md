# NEXORA
### Ultra-Lightweight, Streaming-First Desktop Browser

> *"Maximum performance. Minimum overhead. Complete browsing freedom."*

---

NEXORA is a high-performance, streaming-first desktop browser engineered for Windows 10 and Windows 11. Built with pure Rust orchestration and Microsoft WebView2 Evergreen integration, NEXORA strips away background telemetry, news feeds, and bloatware to deliver smooth video streaming, minimal memory usage, and native responsiveness.

## Key Features

- **Streaming-First Engine**: Direct3D 11 & DXVA2 hardware video decoding offload for YouTube (up to 4K60), Netflix, Prime Video, JioHotstar, Sony LIV, ZEE5, Twitch, and Spotify Web.
- **Legitimate DRM**: Built-in Widevine CDM and Microsoft PlayReady EME compliance.
- **Smart Tab Suspension**: Suspends inactive background tabs to reclaim memory while strictly protecting media-playing tabs.
- **Ultra-Low Memory Footprint**: Targets < 120 MB for single-tab browsing and < 350 MB for 10 tabs under smart suspension.
- **Advanced Omnibox**: Unified address and search bar with autocomplete, local history search, and zero search tracking.
- **Built-in Ad & Tracker Blocker**: Native request-level ad and tracking domain filtering.
- **Privacy-First Architecture**: 100% local-first. Zero telemetry, zero analytics, zero user accounts.
- **Persistent Local Storage**: SQLite 3 with WAL mode for bookmarks, full-text history, and session crash recovery.

## Development Documentation

Detailed engineering specifications and architecture decision records are located in [`/dev`](/dev/):

- [`dev/Plan.md`](/dev/Plan.md) - Master product plan and functional/technical requirements
- [`dev/Phases.md`](/dev/Phases.md) - Complete phase execution model (Phases 0 - 11)
- [`dev/Tasks.md`](/dev/Tasks.md) - Active task tracker and completion logs
- [`dev/Architecture.md`](/dev/Architecture.md) - Component and process architecture
- [`dev/Performance.md`](/dev/Performance.md) - Performance benchmarks and optimization specifications
- [`dev/Security.md`](/dev/Security.md) - Security threat model and boundary defenses
- [`dev/Testing.md`](/dev/Testing.md) - Quality assurance matrix and test specifications
- [`dev/Decisions.md`](/dev/Decisions.md) - Architecture Decision Records (ADRs)

## System Requirements

- **Operating System**: Windows 10 (Build 1809+) or Windows 11 (x64)
- **Runtime**: Microsoft WebView2 Evergreen Runtime (pre-installed on modern Windows)
- **Build Toolchain**: Rust 1.80+ (`x86_64-pc-windows-gnu` or `x86_64-pc-windows-msvc`)

## License

This project is licensed under the [MIT License](LICENSE).
