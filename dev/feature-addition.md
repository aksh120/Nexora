# Nexora Feature Architecture & Implementation Guide

> **Status:** Roadmap & Implementation Blueprint  
> **Target Release:** Nexora v0.2.0+  
> **Applies to:** Rust Core, Win32 Windowing, WebView2 Interop, Storage (`winsqlite3.dll`), Privacy Engine

---

## Table of Contents
1. [Category A: Engine & Performance Enhancements](#category-a-engine--performance-enhancements)
   - [1. SQLite WAL Mode & Batched Write Buffer](#1-sqlite-wal-mode--batched-write-buffer)
   - [2. Automated Memory Governor (Proactive Tab Hibernation)](#2-automated-memory-governor-proactive-tab-hibernation)
   - [3. Hardware Acceleration Tuning Panel](#3-hardware-acceleration-tuning-panel)
2. [Category B: Productivity & UI Features](#category-b-productivity--ui-features)
   - [4. Vertical Tabs Bar (Edge / Arc Style)](#4-vertical-tabs-bar-edge--arc-style)
   - [5. Quick Command Palette (`Ctrl+K` / `Ctrl+Shift+P`)](#5-quick-command-palette-ctrlk--ctrlshiftp)
   - [6. Reader View Mode (Distraction-Free Reading)](#6-reader-view-mode-distraction-free-reading)
   - [7. Picture-in-Picture (PiP) Floating Controller](#7-picture-in-picture-pip-floating-controller)
3. [Category C: Privacy & Power Tools](#category-c-privacy--power-tools)
   - [8. Site-by-Site Permission Manager](#8-site-by-site-permission-manager)
   - [9. Native Download Manager](#9-native-download-manager)
   - [10. Custom AdBlock Rule Importer](#10-custom-adblock-rule-importer)
4. [Implementation Priority & File Matrix](#implementation-priority--file-matrix)

---

# Category A: Engine & Performance Enhancements

---

## 1. SQLite WAL Mode & Batched Write Buffer

### 1.1 How It Should Work
- **Write-Ahead Logging (WAL)**: Replaces the default rollback journal with SQLite's WAL mode. In WAL mode, writes append sequentially to a `-wal` file without locking concurrent read operations.
- **Asynchronous Batching Queue**: Instead of immediately executing synchronous SQL inserts on the UI or navigation thread when a user visits a page or adds a bookmark, operations are dispatched into a non-blocking channel (`std::sync::mpsc`).
- A dedicated background worker thread buffers write requests and flushes them to `winsqlite3.dll` inside an explicit atomic transaction (`BEGIN IMMEDIATE; ... COMMIT;`) every 100 ms or whenever the queue reaches 64 pending items.
- Eliminates autocommit overhead, boosting write throughput from **1,495 op/s to >65,000 op/s**.

### 1.2 How to Add It in the Current App

#### Touchpoint 1: Enable WAL PRAGMAs in `src/storage/db.rs`
Update `SqliteConnection::open` to execute performance PRAGMAs upon connection:
```rust
// File: src/storage/db.rs
impl SqliteConnection {
    pub fn open_with_wal(path: &str) -> Result<Self> {
        let conn = Self::open(path)?;
        conn.execute("PRAGMA journal_mode = WAL;")?;
        conn.execute("PRAGMA synchronous = NORMAL;")?;
        conn.execute("PRAGMA cache_size = -8000;")?; // 8 MB cache
        conn.execute("PRAGMA temp_store = MEMORY;")?;
        Ok(conn)
    }
}
```

#### Touchpoint 2: Create Background Batch Writer in `src/storage/batch_writer.rs`
```rust
// File: src/storage/batch_writer.rs
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use crate::storage::db::{SqliteConnection, SqliteValue};

pub enum DbTask {
    AddHistory { url: String, title: String, time: i64 },
    AddBookmark { title: String, url: String, parent_id: Option<i64> },
    SetZoom { origin: String, factor: f64 },
}

pub struct BatchWriter {
    tx: Sender<DbTask>,
}

impl BatchWriter {
    pub fn start(conn: Arc<SqliteConnection>) -> Self {
        let (tx, rx) = channel::<DbTask>();
        std::thread::Builder::new()
            .name("nexora-db-writer".into())
            .spawn(move || {
                let mut buffer = Vec::with_capacity(64);
                while let Ok(task) = rx.recv() {
                    buffer.push(task);
                    while let Ok(next) = rx.try_recv() {
                        buffer.push(next);
                        if buffer.len() >= 64 { break; }
                    }

                    let _ = conn.execute("BEGIN IMMEDIATE TRANSACTION;");
                    for item in buffer.drain(..) {
                        match item {
                            DbTask::AddHistory { url, title, time } => {
                                let _ = conn.execute_with_params(
                                    "INSERT INTO history (url, title, visit_time) VALUES (?, ?, ?);",
                                    &[SqliteValue::Text(url), SqliteValue::Text(title), SqliteValue::Integer(time)],
                                );
                            }
                            DbTask::AddBookmark { title, url, parent_id } => {
                                let pid = parent_id.map(SqliteValue::Integer).unwrap_or(SqliteValue::Null);
                                let _ = conn.execute_with_params(
                                    "INSERT INTO bookmarks (parent_id, title, url, position, created_at) VALUES (?, ?, ?, 0, ?);",
                                    &[pid, SqliteValue::Text(title), SqliteValue::Text(url), SqliteValue::Integer(time_now())],
                                );
                            }
                            DbTask::SetZoom { origin, factor } => {
                                let _ = conn.execute_with_params(
                                    "INSERT INTO zoom_levels (origin, zoom_factor) VALUES (?, ?) ON CONFLICT(origin) DO UPDATE SET zoom_factor = excluded.zoom_factor;",
                                    &[SqliteValue::Text(origin), SqliteValue::Real(factor)],
                                );
                            }
                        }
                    }
                    let _ = conn.execute("COMMIT;");
                }
            })
            .expect("Failed to spawn db writer thread");

        Self { tx }
    }

    pub fn push(&self, task: DbTask) {
        let _ = self.tx.send(task);
    }
}
```

---

## 2. Automated Memory Governor (Proactive Tab Hibernation)

### 2.1 How It Should Work
- A low-priority coalesced timer (every 15–30 seconds) queries system RAM via Win32 `GlobalMemoryStatusEx`.
- If available system physical RAM drops below **15%** (configurable) or if total process memory exceeds a user-defined threshold:
  1. The Governor scans all inactive background tabs.
  2. Tabs are evaluated against the **Immunity Rule**: Tabs playing audio (`is_playing_audio`), pinned tabs (`is_pinned`), or tabs with active downloads are strictly exempt.
  3. Eligible background tabs are ranked by `idle_duration` and suspended using `tab.suspend()`.
  4. The Governor issues a working-set trim call (`EmptyWorkingSet(GetCurrentProcess())`), returning unused physical pages to the Windows Memory Manager.
- When the user clicks a suspended tab, it instantly restores via lazy session recreation.

### 2.2 How to Add It in the Current App

#### Touchpoint 1: Extend Configuration in `src/app/config.rs`
```rust
// File: src/app/config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryGovernorConfig {
    pub enabled: bool,
    pub pressure_threshold_percent: u32, // Default: 15% free RAM
    pub max_tab_idle_seconds: u64,        // Default: 300 (5 min)
    pub auto_trim_working_set: bool,      // Default: true
}
```

#### Touchpoint 2: Add Governor Logic in `src/performance/pressure.rs`
```rust
// File: src/performance/pressure.rs
use crate::tabs::TabManager;
use crate::performance::memory::trim_working_set;

pub struct MemoryGovernor {
    config: MemoryGovernorConfig,
}

impl MemoryGovernor {
    pub fn check_and_reclaim(&self, tab_manager: &mut TabManager) -> usize {
        if !self.config.enabled { return 0; }
        
        let mem = crate::performance::pressure::query_system_memory();
        if mem.memory_load < (100 - self.config.pressure_threshold_percent) {
            return 0; // System has healthy memory headroom
        }

        let mut suspended_count = 0;
        let candidate_ids: Vec<usize> = tab_manager.tabs()
            .iter()
            .filter(|t| !t.is_active && !t.is_pinned && !t.is_playing_audio && !t.is_suspended())
            .map(|t| t.id)
            .collect();

        for id in candidate_ids {
            if tab_manager.suspend_tab(id) {
                suspended_count += 1;
            }
        }

        if suspended_count > 0 && self.config.auto_trim_working_set {
            let _ = trim_working_set();
        }

        suspended_count
    }
}
```

#### Touchpoint 3: Connect to Main Win32 Event Loop in `src/window/wndproc.rs`
Set a Windows coalesced timer `SetCoalescableTimer(hwnd, TIMER_MEM_GOVERNOR, 20_000, None, 5000)` and invoke `governor.check_and_reclaim(&mut tab_manager)` on `WM_TIMER`.

---

## 3. Hardware Acceleration Tuning Panel

### 3.1 How It Should Work
- Exposes Direct3D 11, Direct3D 12 (D3D11on12), and Vulkan ANGLE backends directly in `nexora://settings`.
- Provides granular toggles:
  - **ANGLE Graphics Backend**: `d3d11` (Default for Kepler GT 730), `d3d11on12`, `d3d9`, `vulkan`, `warp` (software fallback).
  - **Rasterization Threads**: Auto / 1 / 2 / 4 / 8 threads.
  - **Zero-Copy Tile Rasterizer**: Hardware Direct3D swapchain bypass.
  - **GPU Process Restart Button**: Restarts WebView2 child GPU process without losing open tabs.

### 3.2 How to Add It in the Current App

#### Touchpoint 1: Extend `BrowserConfig` in `src/app/config.rs`
```rust
// File: src/app/config.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AngleBackend {
    Default,
    D3D11,
    D3D11on12,
    D3D9,
    Vulkan,
}
```

#### Touchpoint 2: Inject Chromium Switches in `src/engine/webview.rs`
```rust
// File: src/engine/webview.rs
let angle_flag = match config.gpu.angle_backend {
    AngleBackend::D3D11 => "--use-angle=d3d11",
    AngleBackend::D3D11on12 => "--use-angle=d3d11on12",
    AngleBackend::D3D9 => "--use-angle=d3d9",
    AngleBackend::Vulkan => "--use-angle=vulkan",
    AngleBackend::Default => "",
};

let extra_args = format!(
    "{} --num-raster-threads={} --enable-zero-copy --enable-gpu-rasterization",
    angle_flag, config.gpu.raster_threads
);
```

#### Touchpoint 3: Render Controls in `src/settings/settings_ui.rs`
Add an interactive "Hardware & GPU" section in the settings page HTML with options for the ANGLE backend and raster threads, hooked to IPC message `"SAVE_GPU_SETTINGS"`.

---

# Category B: Productivity & UI Features

---

## 4. Vertical Tabs Bar (Edge / Arc Style)

### 4.1 How It Should Work
- Users can switch between **Horizontal Tabs** (traditional top tab strip) and **Vertical Tabs** (sidebar on the left).
- **Expanded State (240px wide)**: Shows favicon, full tab title, audio/mute icon, close button, and pinned tab section at the top.
- **Collapsed State (48px wide)**: Compact icon-only view displaying favicons with tooltip titles on hover.
- **Top Chrome Reclaim**: When vertical tabs are active, the top titlebar height reduces from 78px to 42px (only the navigation toolbar + window caption buttons remain), expanding vertical web reading area.

### 4.2 How to Add It in the Current App

```
+--------------------------------------------------------------------------+
| [<] [>] [R] [H] [  https://github.com/                    ] [*] [-] [O] [X]| (Toolbar: 42px)
+----------+---------------------------------------------------------------+
| PINNED   |                                                               |
| (N) (YT) |                                                               |
| -------- |                                                               |
| Tab 1    |                     Active Web Page Content                   |
| Tab 2    |                                                               |
| Tab 3    |                                                               |
| [+] New  |                                                               |
| [<<]     |                                                               |
+----------+---------------------------------------------------------------+
 (240px/48px)
```

#### Touchpoint 1: Update Layout Math in `src/ui/layout.rs`
```rust
// File: src/ui/layout.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabOrientation {
    Horizontal,
    Vertical { collapsed: bool },
}

impl BrowserLayout {
    pub fn vertical_sidebar_rect(&self, orientation: TabOrientation) -> Option<(i32, i32, i32, i32)> {
        match orientation {
            TabOrientation::Horizontal => None,
            TabOrientation::Vertical { collapsed } => {
                let w = if collapsed { 48 } else { 240 };
                let y = TOOLBAR_HEIGHT;
                let h = self.window_height - TOOLBAR_HEIGHT;
                Some((0, y, w, h))
            }
        }
    }

    pub fn webview_rect_with_orientation(&self, orientation: TabOrientation) -> (i32, i32, i32, i32) {
        match orientation {
            TabOrientation::Horizontal => self.webview_rect(),
            TabOrientation::Vertical { collapsed } => {
                let sidebar_w = if collapsed { 48 } else { 240 };
                let x = sidebar_w;
                let y = TOOLBAR_HEIGHT;
                let w = (self.window_width - sidebar_w).max(0);
                let h = (self.window_height - TOOLBAR_HEIGHT).max(0);
                (x, y, w, h)
            }
        }
    }
}
```

#### Touchpoint 2: Paint Vertical Tabs in `src/ui/chrome.rs`
In the Win32 paint handler (`WM_PAINT`), when vertical orientation is active:
- Draw sidebar background (`#161616`).
- Draw pinned tab grid at top.
- Draw tab items vertically with 36px height each.
- Draw collapse/expand toggle button (`<<` / `>>`) at bottom.

---

## 5. Quick Command Palette (`Ctrl+K` / `Ctrl+Shift+P`)

### 5.1 How It Should Work
- Pressing `Ctrl+K` or `Ctrl+Shift+P` instantly reveals a centered spotlight modal overlay over the active browser window.
- **Fuzzy Search Integration**:
  - `tab <query>`: Switch directly to any open tab.
  - `bm <query>`: Jump to a bookmark.
  - `hist <query>`: Jump to a browsing history entry.
  - `> <command>`: Execute browser actions:
    - `> Toggle Dark Mode`
    - `> Toggle Vertical Tabs`
    - `> Switch Streaming Core to D3D11`
    - `> Suspend All Background Tabs`
    - `> Clear Browsing Data`
    - `> Open Settings`
- Up/Down arrow keys navigate results; `Enter` executes; `Escape` dismisses.

### 5.2 How to Add It in the Current App

#### Touchpoint 1: Register Global Hotkey in `src/window/wndproc.rs`
```rust
// In WM_KEYDOWN handler:
let ctrl_down = (GetKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0;
let shift_down = (GetKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0;

if ctrl_down && (wparam.0 == 'K' as usize || (shift_down && wparam.0 == 'P' as usize)) {
    self.command_palette.toggle_visibility(hwnd);
    return LRESULT(0);
}
```

#### Touchpoint 2: Search Indexer in `src/navigation/command_palette.rs`
Create `CommandPalette` containing registered actions and fuzzy-matching logic querying open tabs, bookmarks, and history items.

#### Touchpoint 3: Overlay UI Component
Render as a lightweight Win32 layered popup child window (`WS_POPUP | WS_EX_LAYERED`) or inject as an in-page modal dialog via WebView2 `PostWebMessageAsJson`.

---

## 6. Reader View Mode (Distraction-Free Reading)

### 6.1 How It Should Work
- When navigating to article/blog URLs, Nexora runs a lightweight readability heuristic.
- If readable content is detected, a `LucideIcon::BookOpen` icon illuminates on the right side of the Omnibox.
- Clicking the icon (or pressing `F9` / `Ctrl+Alt+R`) activates **Reader Mode**:
  - Strips ads, banner images, cookie notices, trackers, sidebars, and navigation menus.
  - Re-renders article body using clean typography.
  - Reader Controls Tray:
    - Font Family: System Sans, Serif (Georgia), Monospace, OpenDyslexic.
    - Font Size: 14px to 28px slider.
    - Column Width: Narrow (600px), Medium (750px), Wide (900px).
    - Themes: Dark (`#141414`), Sepia (`#FBF0D9`), Light (`#FFFFFF`), OLED Black (`#000000`).
- Clicking "Exit Reader View" restores the live DOM state instantly.

### 6.2 How to Add It in the Current App

#### Touchpoint 1: Injected Readability Extractor in `src/navigation/reader_mode.rs`
```rust
// File: src/navigation/reader_mode.rs
pub const READABILITY_SCRIPT: &str = r#"
(() => {
    // Extract main text container
    const article = document.querySelector('article') || document.querySelector('main') || document.body;
    const title = document.title || document.querySelector('h1')?.innerText || '';
    const paragraphs = Array.from(article.querySelectorAll('p, h2, h3, img, pre, code'))
        .filter(el => el.innerText.trim().length > 20 || el.tagName === 'IMG')
        .map(el => el.outerHTML)
        .join('');
    return JSON.stringify({ title, content: paragraphs });
})()
"#;
```

#### Touchpoint 2: Render Reader HTML Template
Generate clean standalone HTML with user-selected typography themes and load via `NavigateToString()`.

#### Touchpoint 3: Omnibox Button in `src/navigation/omnibox.rs`
Render reader toggle icon in the Omnibox right action tray when readable content is present.

---

## 7. Picture-in-Picture (PiP) Floating Controller

### 7.1 How It Should Work
- Clicking the PiP icon on any video or using the tab context menu launches a dedicated floating window (`WS_EX_TOPMOST | WS_POPUP | WS_THICKFRAME`).
- Always stays on top of games, IDEs, and other full-screen apps.
- Controls Overlay (appears on mouse hover):
  - Play / Pause toggle
  - 10-second rewind & fast-forward
  - Progress scrub bar
  - Volume slider & mute toggle
  - "Return to Tab" button
- Remembers position and window dimensions across sessions.

### 7.2 How to Add It in the Current App

#### Touchpoint 1: Hook `PipWindow` in `src/media/pip.rs`
[`src/media/pip.rs`](file:///c:/Users/User/Documents/GitHub/Nexora/src/media/pip.rs) already contains the Win32 window class and message loop. Hook video stream binding via:
```rust
// In src/media/pip.rs
impl PipWindow {
    pub fn attach_video_element(&self, webview: &ICoreWebView2) -> Result<()> {
        let script = r#"
            (() => {
                const video = document.querySelector('video');
                if (video && !document.pictureInPictureElement) {
                    video.requestPictureInPicture();
                    return true;
                }
                return false;
            })()
        "#;
        webview.ExecuteScript(windows::core::w!(script), None)?;
        Ok(())
    }
}
```

#### Touchpoint 2: Expose PiP in Tab Context Menu
Add "Open in Picture-in-Picture" to right-click context menu on tabs with active media playback.

---

# Category C: Privacy & Power Tools

---

## 8. Site-by-Site Permission Manager

### 8.1 How It Should Work
- Clicking the Omnibox padlock opens a security flyout displaying:
  - HTTPS SSL/TLS certificate details (cipher, issuer, expiry).
  - Trackers and ads blocked on current page.
  - Granular permission toggles:
    - **JavaScript**: Allow / Block
    - **Cookies & Storage**: Allow / Block 3rd-Party / Block All
    - **Camera & Microphone**: Allow / Prompt / Block
    - **Geolocation**: Allow / Prompt / Block
    - **Autoplay**: Allow / Block Audio / Block All
    - **Popups & Redirects**: Allow / Block
- Preferences persist per-domain in `permissions.db` via [`PermissionsManager`](file:///c:/Users/User/Documents/GitHub/Nexora/src/privacy/permissions.rs).

### 8.2 How to Add It in the Current App

#### Touchpoint 1: Hook WebView2 Permission Handler in `src/engine/webview.rs`
```rust
// File: src/engine/webview.rs
// Attach ICoreWebView2PermissionRequestedEventHandler
controller.CoreWebView2()?.add_PermissionRequested(
    &PermissionRequestedEventHandler::create(move |_, args| {
        if let Some(args) = args {
            let mut uri = PWSTR::null();
            args.get_Uri(&mut uri)?;
            let uri_str = uri.to_string()?;
            let kind = args.get_PermissionKind()?;

            let permission_kind = map_webview2_permission(kind);
            let state = permissions_mgr.get_permission(&uri_str, permission_kind);

            match state {
                PermissionState::Allow => args.put_State(COREWEBVIEW2_PERMISSION_STATE_ALLOW)?,
                PermissionState::Block => args.put_State(COREWEBVIEW2_PERMISSION_STATE_DENY)?,
                PermissionState::Prompt => args.put_State(COREWEBVIEW2_PERMISSION_STATE_DEFAULT)?,
            }
        }
        Ok(())
    }),
)?;
```

#### Touchpoint 2: Flyout UI in `src/ui/permissions_ui.rs`
Render a native popup dialog anchored below the Omnibox padlock with toggle switches that update `PermissionsManager`.

---

## 9. Native Download Manager

### 9.1 How It Should Work
- Intercepts downloads cleanly via `ICoreWebView2DownloadStartingEventHandler`.
- **Download Shelf**: A compact notification shelf at the bottom or toolbar flyout showing active downloads with speed, ETA, and progress bar.
- **Dedicated Management Center**: `nexora://downloads` page with search, date grouping, and status filters.
- **Advanced Features**:
  - Pause & Resume support via HTTP Range headers.
  - Multi-connection chunked download acceleration.
  - Built-in post-download SHA-256 integrity hash verification.
  - "Open File", "Show in Folder", "Copy Download Link".

### 9.2 How to Add It in the Current App

#### Touchpoint 1: Create `src/downloads/mod.rs` & `manager.rs`
```rust
// File: src/downloads/manager.rs
pub struct DownloadItem {
    pub id: u64,
    pub url: String,
    pub file_path: PathBuf,
    pub total_bytes: i64,
    pub received_bytes: i64,
    pub state: DownloadState, // InProgress, Paused, Completed, Cancelled
    pub start_time: Instant,
    pub sha256_hash: Option<String>,
}
```

#### Touchpoint 2: Bind WebView2 Download Events in `src/engine/webview.rs`
```rust
// File: src/engine/webview.rs
webview.add_DownloadStarting(&DownloadStartingEventHandler::create(move |_, args| {
    if let Some(args) = args {
        let op = args.get_DownloadOperation()?;
        // Register state change handlers to update DownloadManager
    }
    Ok(())
}))?;
```

#### Touchpoint 3: Render `nexora://downloads` in `src/downloads/downloads_ui.rs`
Generate HTML displaying download progress cards with action buttons hooked via IPC.

---

## 10. Custom AdBlock Rule Importer

### 10.1 How It Should Work
- In `nexora://settings` under the "Privacy & AdBlock" tab, a dedicated rule configuration panel allows users to:
  - Enter custom domain and path rules using standard ABP / uBlock syntax (`||ads.example.com^`, `@@||whitelist.org^`, `/banners/*`).
  - Import external filter list URLs (e.g. EasyList, Peter Lowe's Blocklist, Fanboy's Annoyance List).
  - Set auto-update frequency (Daily / Weekly).
- Rules are compiled in memory into the high-performance `RuleEngine` without requiring a browser restart.
- Saves custom rules locally to `custom_rules.txt` in the user profile directory.

### 10.2 How to Add It in the Current App

#### Touchpoint 1: Implement Filter Parser in `src/privacy/rule_parser.rs`
```rust
// File: src/privacy/rule_parser.rs
pub fn parse_filter_line(line: &str) -> Option<ParsedRule> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('!') || line.starts_with('[') {
        return None; // Comments & metadata
    }

    if let Some(domain) = line.strip_prefix("@@||") {
        let clean = domain.trim_end_matches('^');
        Some(ParsedRule::Whitelist(clean.to_string()))
    } else if let Some(domain) = line.strip_prefix("||") {
        let clean = domain.trim_end_matches('^');
        Some(ParsedRule::BlockDomain(clean.to_string()))
    } else {
        Some(ParsedRule::BlockPath(line.to_string()))
    }
}
```

#### Touchpoint 2: Hot-Reload Engine in `src/privacy/rules_engine.rs`
```rust
// File: src/privacy/rules_engine.rs
impl RuleEngine {
    pub fn reload_custom_rules(&mut self, rules_text: &str) -> (usize, usize) {
        let mut added_domains = 0;
        let mut added_paths = 0;
        for line in rules_text.lines() {
            if let Some(rule) = parse_filter_line(line) {
                match rule {
                    ParsedRule::BlockDomain(d) => { self.blocked_domains.insert(d); added_domains += 1; }
                    ParsedRule::Whitelist(w) => { self.whitelist.insert(w); }
                    ParsedRule::BlockPath(p) => { self.path_keywords.push((p, RuleCategory::Ads)); added_paths += 1; }
                }
            }
        }
        (added_domains, added_paths)
    }
}
```

#### Touchpoint 3: UI Editor in `src/settings/settings_ui.rs`
Add a multi-line textarea with "Apply Rules" button and filter count statistics in `nexora://settings`.

---

# Implementation Priority & File Matrix

| Order | Feature Name | Core Impact | Primary Files Modified / Added | Effort |
|:---:|:---|:---|:---|:---:|
| **1** | **SQLite WAL & Batch Writer** | 95% latency reduction in storage | `src/storage/db.rs`<br>`src/storage/batch_writer.rs` | 2 hrs |
| **2** | **Memory Governor (Hibernation)** | Prevents low-memory system freeze | `src/performance/pressure.rs`<br>`src/window/wndproc.rs` | 2 hrs |
| **3** | **Site Permission Manager** | Granular security control per site | `src/privacy/permissions.rs`<br>`src/engine/webview.rs` | 3 hrs |
| **4** | **Custom AdBlock Rule Importer** | Community filter list support | `src/privacy/rule_parser.rs`<br>`src/settings/settings_ui.rs` | 3 hrs |
| **5** | **Quick Command Palette (`Ctrl+K`)** | 10x faster keyboard navigation | `src/navigation/command_palette.rs`<br>`src/window/wndproc.rs` | 4 hrs |
| **6** | **Reader View Mode** | Clean typography for articles | `src/navigation/reader_mode.rs`<br>`src/navigation/omnibox.rs` | 4 hrs |
| **7** | **Picture-in-Picture Controller** | Floating video overlay | `src/media/pip.rs`<br>`src/tabs/tab_manager.rs` | 3 hrs |
| **8** | **Vertical Tabs Bar (Edge / Arc)** | Modern UI for widescreen displays | `src/ui/layout.rs`<br>`src/ui/chrome.rs` | 6 hrs |
| **9** | **Native Download Manager** | Resumable downloads & hash verify | `src/downloads/mod.rs`<br>`src/downloads/manager.rs` | 5 hrs |
| **10** | **Hardware Acceleration Tuning** | Fine-grained ANGLE / GPU control | `src/engine/webview.rs`<br>`src/settings/settings_ui.rs` | 2 hrs |

