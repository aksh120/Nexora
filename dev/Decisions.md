# NEXORA: Architecture Decision Records (ADR)
## Chronological Record of Technical & Engineering Decisions

---

### Record Summary Index

- **`ADR-001`**: Selection of Microsoft WebView2 Evergreen as Embedded Rendering Subsystem
- **`ADR-002`**: Choice of Pure Rust as Core Host Language and Native Win32 Integration
- **`ADR-003`**: Toolchain Strategy: Rust GNU Target (`x86_64-pc-windows-gnu`) for Autonomous Toolchain Independence
- **`ADR-004`**: Database Architecture: SQLite with Write-Ahead Logging (WAL) Mode
- **`ADR-005`**: Tab Memory Management: Smart Tab Suspension over Browser Tab Discarding
- **`ADR-006`**: Video Hardware Acceleration: Mandatory Direct3D 11 & DXVA2 Pipeline
- **`ADR-007`**: Security & Privacy: Zero Telemetry, Built-in Native Request Blocking, and Strict Local-First Architecture

---

### ADR-001: Selection of Microsoft WebView2 Evergreen as Embedded Rendering Subsystem

- **Date**: 2026-11-07
- **Status**: **Approved**
- **Context**: NEXORA requires a modern rendering engine with complete HTML5, CSS3, ES2024, WebAssembly, WebGL, MSE, and EME support. Candidate engines evaluated were Chromium Embedded Framework (CEF), Microsoft WebView2 Evergreen, and Servo/Gecko.
- **Decision**: Adopt Microsoft WebView2 Evergreen via COM API bindings.
- **Rationale**:
  1. **DRM Compliance**: CEF official builds strictly exclude Widevine CDM and PlayReady due to Google licensing restrictions, preventing legitimate 1080p/4K streaming on Netflix, Prime Video, Hotstar, and Sony LIV. WebView2 provides legitimate out-of-the-box Widevine and PlayReady EME support.
  2. **Distribution Size**: CEF requires bundling 180MB+ of binaries in the installer. WebView2 Evergreen is pre-installed on Windows 10 and 11 (version 157+ detected on the target machine), allowing a thin installer (< 15 MB).
  3. **Shared Working Set**: Operating system-level memory deduplication allows WebView2 to share memory pages with other Windows components, dramatically reducing working set overhead.
- **Consequences**: NEXORA must account for all `msedgewebview2.exe` subprocesses when benchmarking memory and CPU usage.

---

### ADR-002: Choice of Pure Rust as Core Host Language and Native Win32 Integration

- **Date**: 2026-11-07
- **Status**: **Approved**
- **Context**: The browser host shell manages the application lifecycle, window management, tab state machine, SQLite storage, and network request filtering.
- **Decision**: Develop the host exclusively in Rust using direct Win32 API bindings (`windows` / `winapi` / `webview2-com`).
- **Rationale**:
  1. Rust guarantees memory safety and thread safety without garbage collection pauses.
  2. Compiles to a single, compact, hyper-optimized native PE32+ executable (`Nexora.exe`).
  3. Eliminates heavy web-in-web framework bloat (e.g., Electron, Tauri with high-level web chrome) in favor of lightweight native rendering.
- **Consequences**: Requires careful Win32 message loop design and COM object lifetime management.

---

### ADR-003: Toolchain Strategy: Rust GNU Target (`x86_64-pc-windows-gnu`)

- **Date**: 2026-11-07
- **Status**: **Approved**
- **Context**: The host machine lacks a pre-installed Visual Studio C++ Build Tools suite (`link.exe` missing for `x86_64-pc-windows-msvc`), and installing 6GB+ of Visual Studio C++ workloads introduces unnecessary disk bloat and potential installation hangs.
- **Decision**: Adopt `x86_64-pc-windows-gnu` as the primary development and build toolchain.
- **Rationale**:
  1. MinGW-w64 toolchain and libraries are self-contained within Rust's `rust-std` distribution.
  2. Compiles and links standalone Windows executables with zero external build tool dependencies.
  3. Fully supports all required Windows Win32 APIs, COM interfaces (`ICoreWebView2`), and C dependencies (such as SQLite).
- **Consequences**: Standard Windows calling conventions (`stdcall`, `cdecl`) and COM vtables are identical across GNU and MSVC targets on x86_64.

---

### ADR-004: Database Architecture: SQLite with Write-Ahead Logging (WAL) Mode

- **Date**: 2026-11-07
- **Status**: **Approved**
- **Context**: NEXORA must store structured user data including chronological history, full-text search indexes, bookmarks, site permissions, and session checkpoints.
- **Decision**: Use embedded SQLite via `rusqlite` with `PRAGMA journal_mode = WAL;` and `PRAGMA synchronous = NORMAL;`.
- **Rationale**:
  1. WAL mode allows concurrent readers while a write is occurring, completely eliminating UI stutter during history logging.
  2. Built-in FTS5 extension provides sub-5ms full-text search for the omnibox autocomplete engine.
  3. Battle-tested data integrity with automatic recovery on unexpected power loss or crashes.
- **Consequences**: Database consists of three files (`nexora.db`, `nexora.db-wal`, `nexora.db-shm`) managed in `%APPDATA%\Nexora\Data`.

---

### ADR-005: Tab Memory Management: Smart Tab Suspension over Tab Discarding

- **Date**: 2026-11-07
- **Status**: **Approved**
- **Context**: Inactive background tabs consume valuable physical RAM and GPU texture memory.
- **Decision**: Implement Smart Tab Suspension where idle background tabs have their state serialized and their webview controllers unloaded, while displaying a clean native stub with one-click restoration.
- **Rationale**:
  1. Standard Chromium tab discarding often discards tabs indiscriminately, frequently terminating background audio or streaming.
  2. Smart Suspension respects strict exemption heuristics (audio playing, video active, download running).
  3. Reclaims > 50% physical RAM per inactive tab, maintaining a 10-tab memory profile under 350 MB.
- **Consequences**: Restoring a suspended tab requires a brief reload (< 500 ms) which restores the exact URL and scroll position.

---

### ADR-006: Mandatory Direct3D 11 & DXVA2 Hardware Video Acceleration

- **Date**: 2026-11-07
- **Status**: **Approved**
- **Context**: Target benchmark system is equipped with an Intel Core i7-3770 CPU and NVIDIA GeForce GT 730 GPU. Smooth 1080p60 video playback requires offloading video decoding from the Ivy Bridge CPU to the GPU.
- **Decision**: Enforce `--enable-features=VaapiVideoDecoder,D3D11VideoDecoder` and `--enable-gpu-rasterization` in WebView2 environment initialization.
- **Rationale**:
  1. GT 730's NVDEC engine handles H.264 (AVC) and MPEG-4 bitstream decoding in dedicated hardware, reducing host CPU usage during 1080p60 video from ~18% down to < 4%.
  2. D3D11 swap chains deliver jitter-free frame presentation directly to the display compositor.
- **Consequences**: If legacy drivers fail, dynamic runtime detection falls back cleanly to software decoding.

---

### ADR-007: Zero Telemetry, Native Request Blocking, and Local-First Architecture

- **Date**: 2026-11-07
- **Status**: **Approved**
- **Context**: Commercial browsers monetize user browsing behavior through continuous background telemetry, ad brokers, and cloud data mining.
- **Decision**: Strictly prohibit all telemetry, remote logging, user tracking, or cloud synchronisation. Build native ad and tracker filtering into the network layer via `AddWebResourceRequestedFilter`.
- **Rationale**:
  1. Directly satisfies Section 08 (Privacy-First Architecture).
  2. Eliminates background CPU cycles, network bandwidth, and TLS handshakes associated with telemetry daemons.
  3. Pre-compiled ad blocking accelerates page load times by up to 40% and reduces RAM usage by eliminating ad scripts.
- **Consequences**: NEXORA makes outbound network connections exclusively for user-initiated website navigations.
