<p align="center">
  <img src="assets/branding/logo.svg" width="380" alt="Nexora Logo"/>
</p>

# NEXORA Engineering Roadmap

This document outlines the strategic roadmap and future engineering milestones for the **NEXORA** desktop browser.

---

## 🧭 Milestone Overview

```
v0.1.0 (Current) ─────► v0.2.0 ─────► v0.3.0 ─────► v1.0.0
 Foundation &          Power User        Performance        LTS & Ecosystem
 Streaming Core        Features          & Customization    Maturity
```

---

## 📍 Released: v0.1.0 — Initial Production Release

- [x] Pure Rust Win32 host orchestration (`windows-rs` 0.58).
- [x] WebView2 Evergreen COM integration with user data directory isolation.
- [x] Direct3D 11 & DXVA2 hardware video decoding offload (Kepler GT 730 baseline verified).
- [x] Widevine CDM & Microsoft PlayReady EME DRM streaming compliance (Netflix, Prime, Hotstar, Spotify).
- [x] Smart Tab Suspension Engine with audio/video/download exemptions.
- [x] Native Working Set Trimmer utilizing Windows API `SetProcessWorkingSetSize`.
- [x] Lazy Session Restoration with sub-400ms startup times.
- [x] Sub-15µs Native Ad & Tracker Blocker with 79 pre-seeded rules.
- [x] Dynamic OS SQLite binding to Windows `winsqlite3.dll` with WAL mode.
- [x] Unified Omnibox with local history autocomplete and search engine fallbacks.
- [x] Minimalist native settings (`nexora://settings`) and diagnostics (`nexora://diagnostics`).
- [x] Standalone portable package (310 KB ZIP) and Inno Setup installer.

---

## 🚀 In Progress: v0.2.0 — Power User & Media Enhancements

- [ ] **Custom Adblock Filter Subscription**:
  - Ability to import external Adblock Plus / uBlock Origin compatible filter lists (`EasyList`, `EasyPrivacy`, `Peter Lowe's List`).
  - Native trie / Aho-Corasick rule compiler in Rust for high-throughput multi-pattern matching.
- [ ] **Advanced Picture-in-Picture (PiP) Controls**:
  - Playback speed stepping (`0.5x` to `3.0x`).
  - Seek bar and volume slider embedded directly in the borderless PiP overlay.
  - Subtitle / closed-caption rendering within the PiP frame.
- [ ] **Dual-Pane / Split-Screen Browsing**:
  - Side-by-side view for simultaneous video streaming and document reading in a single window.
  - Synchronized and independent scroll modes.
- [ ] **Portable Profile Mode**:
  - Command-line switch (`--portable`) storing all databases and cookies relative to `Nexora.exe` on USB drives.

---

## 🔮 Planned: v0.3.0 — Advanced Performance & Scripting

- [ ] **User-Script & Custom CSS Engine**:
  - Lightweight, sandboxed user-script injector (`GM_xmlhttpRequest`, custom styling overrides) without heavy WebExtension overhead.
- [ ] **Smart Network Bandwidth Throttling**:
  - Per-tab bandwidth allocation policy giving maximum download priority to active video streams while background tabs are bandwidth-capped.
- [ ] **Tab Groups & Workspaces**:
  - Color-coded tab groups with one-click group suspension and session export.
- [ ] **Dark Reader Mode**:
  - Distraction-free article view stripping ads, scripts, and clutter with custom typography and reading time estimators.

---

## 🌟 Future Horizon: v1.0.0 — Long-Term Support

- [ ] **Zero-Overhead Tab Audio Equalizer**:
  - Native WebAudio / Windows Core Audio API hook for bass boost, vocal clarity, and volume leveling.
- [ ] **Enterprise / Kiosk Policy Deployment**:
  - Windows Group Policy (`.admx`) templates for deploying Nexora in institutional and streaming-kiosk environments.
- [ ] **Multi-Monitor Video Detachment**:
  - Dedicated multi-display streaming mode launching video players directly to secondary monitors in fullscreen borderless mode with zero desktop taskbar flicker.

