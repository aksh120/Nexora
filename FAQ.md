<p align="center">
  <img src="assets/branding/logo.svg" width="380" alt="Nexora Logo"/>
</p>

# Frequently Asked Questions (FAQ)

### 1. What is NEXORA and why does it exist?
Modern commercial browsers (such as Google Chrome and Microsoft Edge) have evolved from lightweight user agents into massive operating-system-level runtimes. Opening 10 tabs routinely consumes 2 to 4 GB of RAM, spins up dozens of background helper processes, and quietly runs telemetry pings, news aggregators, shopping price trackers, and sponsored suggestions.

**NEXORA** is engineered for users who want a pure, ultra-fast web and streaming experience without the bloat. Written in pure Rust, NEXORA provides hardware video decoding, legitimate DRM streaming, built-in ad blocking, and smart memory management while consuming a fraction of the system resources.

---

### 2. Why does NEXORA use Microsoft WebView2 Evergreen instead of compiling Chromium from source?
Compiling Chromium from source results in:
- A 150–250 MB installer / binary download.
- Endless maintenance to cherry-pick zero-day security patches.
- Loss of hardware-certified DRM pipelines (PlayReady and hardware Widevine CDM require cryptographic certificates and OS-level platform credentials that Microsoft and Google do not license to independent small builds).
- Redundant duplication of Chromium binary files already present on 99% of Windows machines.

By binding natively to the **WebView2 Evergreen Runtime** via low-level COM interfaces in Rust:
1. **Tiny Binary Size**: `Nexora.exe` compiles to just **662 KB** (less than 1 MB!).
2. **Zero-Day Security**: The underlying Chromium engine is automatically updated by the operating system.
3. **Flawless DRM**: PlayReady and Widevine EME work out of the box with full hardware decryption.
4. **Hardware Video Offload**: Direct access to D3D11 and DXVA2 hardware pipelines on GPUs like NVIDIA GeForce GT 730 and Intel HD Graphics.
5. **Zero Chromium Bloat**: NEXORA boots WebView2 with strict flags that strip out Google sync, component updates, domain reliability pings, and telemetry before the engine initializes.

---

### 3. Does NEXORA collect ANY telemetry or usage data?
**Absolute Zero.**
- 0 analytics SDKs.
- 0 crash reporting pings (no Breakpad / Crashpad network reporting).
- 0 user accounts or cloud synchronization.
- 0 search queries transmitted in the background.
- 100% of your history, bookmarks, and site permissions remain in a local SQLite database on your own machine (`%LOCALAPPDATA%\Nexora\`).

---

### 4. How does Smart Tab Suspension work without interrupting YouTube or Spotify?
NEXORA's [`SmartSuspensionEngine`](src/performance/suspension.rs) monitors user interaction timestamps on background tabs. When a tab has been idle beyond the timeout (default: 10 minutes, or 3 minutes under memory pressure), it qualifies for suspension.

However, the engine evaluates strict **exemption heuristics** before acting:
1. **Audio State**: Queries `ICoreWebView2::get_IsDocumentPlayingAudio`. If audio is playing, the tab is **immune**.
2. **HTML5 Media**: Inspects `<video>` and `<audio>` elements for active non-paused playback.
3. **Active Network Downloads**: Background streaming or file transfers are protected.
4. **Form Inputs**: Detects unsaved `<input>` or `<textarea>` form data.

When a tab is suspended, its controller is unmapped and Windows API `SetProcessWorkingSetSize` trims its working set. Clicking the tab instantly awakens it via the [`LazyRestorePipeline`](src/performance/lazy_restore.rs).

---

### 5. Why does NEXORA bind to `winsqlite3.dll` instead of bundling SQLite?
Every modern Windows 10 and 11 installation already includes Microsoft's official, highly optimized build of SQLite (`winsqlite3.dll`) in `C:\Windows\System32`.

By dynamically loading `winsqlite3.dll` via Win32 `LoadLibraryW`:
- NEXORA eliminates hundreds of kilobytes of redundant binary bloat.
- Takes advantage of OS-level shared memory caching for SQLite.
- Enables Write-Ahead Logging (`WAL`) mode with atomic commits and sub-millisecond query latency.

---

### 6. Are browser extensions (Chrome Web Store) supported?
**Not in v0.1.0.**
Extensions are the #1 cause of browser memory leaks, CPU wakeups, and privacy tracking. To fulfill our commitment to a sub-700 KB binary footprint and maximum streaming performance, third-party WebExtensions are intentionally excluded from the initial release scope.

Future releases (see [ROADMAP.md](ROADMAP.md)) will introduce a lightweight user-script engine (`GM_xmlhttpRequest`, custom CSS styling) without the heavy multi-process overhead of Chrome extensions.

---

### 7. What video streaming platforms are tested and supported?
- **YouTube** (1080p60 AVC/H.264, 4K30/4K60 VP9, HDR)
- **Netflix** (1080p PlayReady & Widevine DRM)
- **Amazon Prime Video** (HD/1080p DRM)
- **JioHotstar** (1080p EME DRM)
- **Sony LIV** (1080p DRM)
- **ZEE5** (1080p DRM)
- **Twitch** (1080p60 live low-latency video)
- **Spotify Web Player** (High-bitrate DRM audio streaming)

---

### 8. Where is my browser data stored and how do I back it up?
All data is stored in your standard Windows user application directory:
```
%LOCALAPPDATA%\Nexora\
  ├── nexora_database.db       (Bookmarks, history, permissions in SQLite)
  ├── session_checkpoint.json  (Crash recovery session checkpoint)
  ├── User Data\               (WebView2 isolated partition)
  └── Backups\                 (Automated database integrity snapshots)
```
To back up your bookmarks and history, simply copy `nexora_database.db`.

