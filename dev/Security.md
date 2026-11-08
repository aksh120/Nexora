<p align="center">
  <img src="../assets/branding/logo.svg" width="380" alt="Nexora Logo"/>
</p>

# NEXORA: Security Architecture & Threat Model
## Ultra-Lightweight, Streaming-First Desktop Browser

---

### 1. Security Philosophy & Inviolable Principles

In NEXORA, security and user privacy are foundational requirements that are never subordinated to performance or convenience.

#### Inviolable Rules
1. **Never Disable TLS Verification**: All HTTPS certificate validation chains, revocation checks (OCSP/CRL), and cipher suite policies are enforced without exception. No `--ignore-certificate-errors` flags are permitted under any build profile.
2. **Never Compromise Sandbox Boundaries**: Renderer, GPU, and network subprocesses run in restricted Windows integrity levels (Low Integrity Token). Process isolation and origin isolation policies remain fully active.
3. **Never Bypass DRM or Subscription Protections**: NEXORA implements only legitimate, authorized Content Decryption Modules (Widevine and Microsoft PlayReady via standard EME). No reverse engineering, key extraction, or DRM stripping is implemented.
4. **Never Execute Untrusted Native Code**: Web pages cannot trigger arbitrary local process execution, shell commands, or unvetted protocol handlers.
5. **Never Transmit User Telemetry**: Browsing history, omnibox queries, cookies, and hardware profiles remain strictly local on the client machine. Zero telemetry pings, background pings, or analytics endpoints exist in the application core.

---

### 2. Threat Model & Attack Vectors

```
                                [ THE INTERNET ]
                                        |
                            (Untrusted Web Content)
                                        |
                                        v
                 +----------------------------------------------+
                 |          NETWORK PERIMETER (TLS)             |
                 |  - Strict TLS 1.2 / 1.3 Ciphers              |
                 |  - Root Store Certificate Validation         |
                 |  - Built-in Ad/Malware Domain Blocker        |
                 +----------------------------------------------+
                                        |
                                        v
                 +----------------------------------------------+
                 |            SANDBOX BOUNDARY (COM)            |
                 |  - Low Integrity Renderer Subprocesses       |
                 |  - Out-of-Process Direct3D Compositing       |
                 |  - Site Isolation (Per-Origin Process)       |
                 +----------------------------------------------+
                                        |
                           [ Validated Native IPC ]
                                        |
                                        v
+==============================================================================+
|                            NEXORA HOST (MEDIUM IL)                           |
|  - Safe Rust Core (Memory Safety, Data Race Freedom)                         |
|  - SQLite Persistence Protected by User ACLs (%APPDATA%)                     |
|  - Input Validation on all Omnibox & URL Protocols                           |
+==============================================================================+
```

#### Identified Threat Vectors & Mitigations:

| Threat Vector | Attack Mechanism | Nexora Architectural Mitigation |
| :--- | :--- | :--- |
| **Malicious Web Scripts** | Heap spray, buffer overflow, RCE targeting browser engine | Sandboxed Chromium renderers in Low Integrity tokens; Rust host isolates memory spaces |
| **Drive-By File Downloads** | Automatic background downloading of executable malware | Direct download confirmation required; unsafe protocol execution blocked |
| **Omnibox Spoofing** | Unicode homograph attacks, fake URL display | Punycode conversion for internationalized domain names; high-visibility protocol badge |
| **Tracking & Fingerprinting** | Cross-site third-party cookies, canvas fingerprinting | Built-in ad/tracker network blocker; third-party cookie blocking policy |
| **Malicious Extensions** | Rogue browser extension exfiltrating credentials | **Zero Extension Architecture** in initial scope eliminates this entire attack surface |
| **Local Database Tampering** | Local privilege escalation reading browser history/cookies | Files stored in `%APPDATA%\Nexora` with user-specific Windows DACLs; encryption support |
| **Corrupted Session Checkpoint**| Malformed session file triggering host memory crash | Strongly typed Rust JSON/bincode parsing with defensive schema validation |

---

### 3. IPC & Sandboxing Security

1. **Host-to-Engine COM Boundary**:
   - Communication between `Nexora.exe` and `msedgewebview2.exe` occurs over authenticated Windows COM IPC channels.
   - Script injection via `ExecuteScript` is strictly confined to read-only state gathering (e.g., scroll coordinates, video paused state).
2. **Web Content-to-Host Boundary**:
   - Content scripts communicate with the host exclusively via `window.chrome.webview.postMessage`.
   - The Rust host treats all incoming web messages as completely untrusted:
     - Strict JSON schema verification is enforced.
     - Origin checking (`ICoreWebView2WebMessageReceivedEventArgs::get_Source`) confirms that privileged messages are accepted only from internal `nexora://` schemes.

---

### 4. Data Protection & Cryptography

1. **Local Storage Security**:
   - SQLite databases (`nexora.db`) and user preferences are placed in `%APPDATA%\Nexora\Data`.
   - Windows Data Protection API (`CryptProtectData` / DPAPI) is scheduled for sensitive credentials and cookie encryption.
2. **Network Protocol Hardening**:
   - Automatic HTTP-to-HTTPS upgrade attempted for omnibox navigations.
   - Mixed-content blocking active: HTTP subresources on HTTPS pages are blocked by default.

---

### 5. Update Security & Verification

1. **Engine Updates**:
   - Handled seamlessly and securely through the official Microsoft Edge / Windows Update servicing pipeline. Nexora avoids bundling stale, vulnerable Chromium builds.
2. **Application Updates**:
   - Any future binary update mechanism must mandate cryptographic signature verification using Authenticode and Ed25519 digital signatures before execution.

---

### 6. Known Security Limitations & Scope Boundaries

- **Kepler GPU Driver Legacy**: The NVIDIA GeForce GT 730 driver branch (R470 / 474.xx) is in legacy security maintenance. While DXVA2 video acceleration remains functional, NEXORA monitors for driver stability and provides instant software rendering fallback if driver anomalies occur.
- **Physical Machine Access**: A compromised local user account with physical access can read files in `%APPDATA%`. Users requiring multi-user separation must rely on Windows user account isolation.

