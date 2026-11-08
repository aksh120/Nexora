<p align="center">
  <img src="assets/branding/logo.svg" width="380" alt="Nexora Logo"/>
</p>

# Security Policy & Threat Model

NEXORA treats user privacy and browser security with uncompromising rigor. Because browsers operate directly on untrusted remote code, NEXORA employs defense-in-depth isolation across its process, network, and storage boundaries.

---

## Supported Versions

Only the latest release of NEXORA receives active security patches and updates:

| Version | Supported | Status |
| :--- | :--- | :--- |
| `0.1.x` | ✅ Yes | Current Active Release |
| `< 0.1.0` | ❌ No | Pre-release / Deprecated |

---

## Reporting a Vulnerability

We appreciate responsible disclosure of security vulnerabilities:

1. **Do not** report security vulnerabilities via public GitHub issues.
2. Submit your report privately via **GitHub Security Advisories** on the Nexora repository (or email `security@nexorabrowser.org`).
3. Include detailed reproduction steps, target Windows version, impact assessment, and proof-of-concept (PoC) code or scripts if applicable.
4. We aim to acknowledge receipt within **48 hours** and provide a patch or mitigation plan within **7 business days**.

---

## Security Architecture & Defense-in-Depth

NEXORA's security posture is enforced across four distinct pillars:

### 1. Process & Sandbox Isolation
- **Renderer Sandboxing**: All web content executes inside Microsoft Chromium's hardened AppContainer sandbox processes with restricted Windows token privileges.
- **Process Boundary**: The host orchestration process (`Nexora.exe`) executes pure, memory-safe Rust with `panic = "abort"` and strict pointer bounds checking. Host APIs are never exposed to JavaScript execution contexts without verified asynchronous message channels.

### 2. Network Boundary & Zero-Telemetry Hardening
- **Prohibited Endpoints**: All background telemetry pings, error reporting (`breakpad`), domain reliability metrics, Google account synchronization, and Chrome component updater network calls are permanently disabled via Chromium command-line switches.
- **Native Ad & Tracker Interceptor**: Intercepts HTTP/HTTPS requests at the WebResource layer before socket handshakes occur, terminating tracking beacons and malware domains with `< 15 µs` lookup latency.
- **Strict Cache Isolation**: Disk caches are partitioned and hard-capped (`--disk-cache-size=268435456`) to prevent cache timing attacks and cross-origin tracking leakage.

### 3. Local Storage Integrity
- **Operating System Dynamic SQLite Binding**: Storage utilizes Windows OS `winsqlite3.dll` with Write-Ahead Logging (`WAL`), strict atomic checkpointing, and `PRAGMA integrity_check` verification on startup.
- **Directory Isolation**: All cookies, history, and cache reside strictly under `%LOCALAPPDATA%\Nexora\`, never in shared or world-readable temporary paths.
- **Auto-Quarantine**: Any detected database corruption triggers immediate atomic quarantine (`.corrupt.<timestamp>`) and auto-recovery from the latest verified `.bak` snapshot.

### 4. Permissions & Device Access
- **Explicit Site Permissions**: High-privilege browser APIs (Camera, Microphone, Geolocation, Clipboard access) require explicit user consent and are stored in a normalized, local-only SQLite table (`site_permissions`).
- **Autoplay Control**: Strict autoplay policies prevent unexpected media playback, acoustic fingerprinting, and resource exhaustion.

