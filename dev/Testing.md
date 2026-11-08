<p align="center">
  <img src="../assets/branding/logo.svg" width="380" alt="Nexora - The Quantum Prism Logo"/>
</p>

# NEXORA: Quality Assurance & Testing Specification
## Ultra-Lightweight, Streaming-First Desktop Browser

---

### 1. Test Strategy & Classification

Testing in NEXORA is organized into rigorous automated and manual validation suites. In adherence to Section 19 of the Master Development Prompt, every test scenario is explicitly tracked and classified under one of four statuses:

- **Passed**: Verified functional requirement with passing automated assertions or validated runtime observation.
- **Failed**: Assertion failure, functional defect, or crash.
- **Not Tested**: Implementation pending or test yet to be executed.
- **Blocked**: Test cannot execute due to external environment or hardware constraint.

---

### 2. Automated Test Matrix

| Suite ID | Category | Description | Verification Target | Status |
| :--- | :--- | :--- | :--- | :--- |
| `TEST-001` | Unit / Omnibox | Parse pure URLs (http, https, ip, localhost) | Strict RFC 3986 validation | **Not Tested** |
| `TEST-002` | Unit / Omnibox | Differentiate search queries from URLs | Keyword query routing to default search engine | **Not Tested** |
| `TEST-003` | Unit / AdBlock | Fast Trie domain matching on EasyList syntax | Block matching domains in < 15 microseconds | **Not Tested** |
| `TEST-004` | Integration / DB | SQLite WAL schema migration & integrity check | Clean bootstrap & `PRAGMA integrity_check` pass | **Not Tested** |
| `TEST-005` | Integration / History | FTS5 full-text search across 5,000 URLs | Query latency < 3 ms | **Not Tested** |
| `TEST-006` | Integration / Cookies | Cookie write, read, and cross-restart survival | Persistent login verification | **Not Tested** |
| `TEST-007` | Integration / Cache | Cache disk quota cap enforcement (256 MB) | Verify disk usage stays <= limit | **Not Tested** |
| `TEST-008` | Unit / Suspension | Suspension exemption rules (audio, download) | Active media tab is never suspended | **Not Tested** |
| `TEST-009` | Integration / State | Crash recovery session serialization | Restore 5 open tabs from simulated crash | **Not Tested** |
| `TEST-010` | Win32 / High-DPI | Window scaling across 100%, 125%, 150%, 200% | Correct WM_DPICHANGED layout repaint | **Not Tested** |

---

### 3. Media & Streaming Compatibility Test Suite

| Platform | Stream Format / Resolution | Hardware Acceleration | DRM Type | Verification Method | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **YouTube** | 1080p60 (AVC1 / H.264) | Direct3D 11 / DXVA2 offload | None (Clear Key) | `edge://media-internals` + Task Manager GPU Video Decode | **Not Tested** |
| **YouTube** | 4K30 (VP9 / WebM) | Hybrid GPU / CPU decode | None | Task Manager CPU & Dropped frames check (< 0.1%) | **Not Tested** |
| **Netflix** | 1080p Full HD (Encrypted) | D3D11 / Protected Media Path | Microsoft PlayReady & Widevine | Authorized playback handshake & secure buffer display | **Not Tested** |
| **Amazon Prime**| 1080p Full HD (Encrypted) | D3D11 Video Pipeline | Widevine L3 / PlayReady | Smooth playback without DRM HDCP renegotiation error | **Not Tested** |
| **JioHotstar** | 1080p Live Stream (HLS/EME) | Hardware D3D11 | Widevine Modular CDM | Live cricket/movie playback without buffer underruns | **Not Tested** |
| **Sony LIV** | 1080p VOD / Live | Hardware D3D11 | Widevine Modular CDM | Stream initialization & token refresh | **Not Tested** |
| **ZEE5** | 1080p VOD | Hardware D3D11 | Widevine Modular CDM | Audio/video synchrony over 30 mins | **Not Tested** |
| **Twitch** | 1080p60 Low Latency Live | Low-latency D3D11 | None (Clear) | Latency to broadcaster < 2.5s | **Not Tested** |
| **Spotify Web** | 320 kbps AAC/Ogg (Encrypted) | Software Audio Pipeline | Widevine Audio CDM | Continuous background audio playback during tab switch | **Not Tested** |

---

### 4. Performance & Resource Benchmarking Tests

| Scenario ID | Test Name | Target Metric | Chrome Comparison | Edge Comparison | Nexora Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `BENCH-01` | Clean Boot Memory | Total Process Tree < 85 MB | ~185 MB | ~160 MB | **Not Tested** |
| `BENCH-02` | Single Tab Memory | Total Working Set < 120 MB | ~230 MB | ~210 MB | **Not Tested** |
| `BENCH-03` | 1080p60 Video CPU | Host CPU < 4.0% | ~7.2% | ~6.5% | **Not Tested** |
| `BENCH-04` | 10 Open Tabs (Active) | Total RAM < 580 MB | ~1,450 MB | ~1,280 MB | **Not Tested** |
| `BENCH-05` | 10 Tabs (Suspended) | Total RAM < 290 MB | ~1,100 MB | ~850 MB | **Not Tested** |
| `BENCH-06` | Cold Launch Latency | Time to First Render < 750 ms | ~1,400 ms | ~1,150 ms | **Not Tested** |
| `BENCH-07` | Warm Launch Latency | Time to Interactive < 400 ms | ~2,100 ms | ~1,750 ms | **Not Tested** |

---

### 5. Windows OS Compatibility Matrix

- **Windows 11 Pro 64-bit (Build 22631)**: Active development and benchmarking host.
- **Windows 10 64-bit (Build 1809 - 22H2)**: Secondary validation target; verified COM API compatibility and WebView2 Evergreen distribution.

---

### 6. Defect & Failure Log

*(No defects recorded. Suite ready for Phase 1 execution.)*

