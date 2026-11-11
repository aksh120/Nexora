//! Quick Command Palette (Spotlight / Raycast Launcher)
//!
//! Provides instant keyboard-driven search (`Ctrl+K` / `Ctrl+Shift+P`) across:
//! - Active open tabs
//! - Bookmarks and reading lists
//! - Browsing history
//! - Browser system commands (`> Toggle Dark Mode`, `> Vertical Tabs`, `> Suspend Inactive Tabs`, etc.)

use crate::storage::bookmarks::BookmarkEntry;
use crate::storage::history::HistoryEntry;
use crate::tabs::Tab;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteCategory {
    Command,
    Tab,
    Bookmark,
    History,
}

impl PaletteCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Command => "COMMAND",
            Self::Tab => "TAB",
            Self::Bookmark => "BOOKMARK",
            Self::History => "HISTORY",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteAction {
    SwitchTab(usize),
    Navigate(String),
    ExecuteCommand(String),
}

#[derive(Debug, Clone)]
pub struct PaletteItem {
    pub title: String,
    pub subtitle: String,
    pub category: PaletteCategory,
    pub action: PaletteAction,
    pub score: i32,
}

pub struct CommandPalette;

impl CommandPalette {
    pub fn get_system_commands() -> Vec<PaletteItem> {
        vec![
            PaletteItem {
                title: "Toggle Dark / Light Theme".into(),
                subtitle: "Switch between dark and light color palettes".into(),
                category: PaletteCategory::Command,
                action: PaletteAction::ExecuteCommand("toggle_theme".into()),
                score: 10,
            },
            PaletteItem {
                title: "Toggle Vertical Tabs".into(),
                subtitle: "Switch between horizontal top strip and left sidebar tabs".into(),
                category: PaletteCategory::Command,
                action: PaletteAction::ExecuteCommand("toggle_vertical_tabs".into()),
                score: 10,
            },
            PaletteItem {
                title: "Suspend Inactive Background Tabs".into(),
                subtitle: "Reclaim memory by hibernating all idle background tabs".into(),
                category: PaletteCategory::Command,
                action: PaletteAction::ExecuteCommand("suspend_inactive_tabs".into()),
                score: 10,
            },
            PaletteItem {
                title: "Trim Process Working Set".into(),
                subtitle: "Flush unreferenced memory pages directly back to Windows".into(),
                category: PaletteCategory::Command,
                action: PaletteAction::ExecuteCommand("trim_working_set".into()),
                score: 10,
            },
            PaletteItem {
                title: "Open Settings".into(),
                subtitle: "Configure browser preferences, GPU, and streaming engine".into(),
                category: PaletteCategory::Command,
                action: PaletteAction::Navigate("nexora://settings".into()),
                score: 10,
            },
            PaletteItem {
                title: "Open Downloads".into(),
                subtitle: "View downloaded files, progress, and SHA-256 hashes".into(),
                category: PaletteCategory::Command,
                action: PaletteAction::Navigate("nexora://downloads".into()),
                score: 10,
            },
            PaletteItem {
                title: "Open Bookmarks".into(),
                subtitle: "Manage saved sites, folders, and bookmarks".into(),
                category: PaletteCategory::Command,
                action: PaletteAction::Navigate("nexora://bookmarks".into()),
                score: 10,
            },
            PaletteItem {
                title: "Open History".into(),
                subtitle: "Search and inspect browsing chronology".into(),
                category: PaletteCategory::Command,
                action: PaletteAction::Navigate("nexora://history".into()),
                score: 10,
            },
            PaletteItem {
                title: "Open System Diagnostics".into(),
                subtitle: "Inspect hardware decode, GPU capabilities, and memory load".into(),
                category: PaletteCategory::Command,
                action: PaletteAction::Navigate("nexora://diagnostics".into()),
                score: 10,
            },
            PaletteItem {
                title: "New Tab".into(),
                subtitle: "Open a fresh tab".into(),
                category: PaletteCategory::Command,
                action: PaletteAction::ExecuteCommand("new_tab".into()),
                score: 10,
            },
        ]
    }

    /// Generates injected spotlight overlay modal script for active web page
    pub fn script_open() -> &'static str {
        r#"(function() {
    const MODAL_ID = '__nexora_command_palette__';
    let existing = document.getElementById(MODAL_ID);
    if (existing) {
        existing.remove();
        return;
    }

    const backdrop = document.createElement('div');
    backdrop.id = MODAL_ID;
    backdrop.style.cssText = `
        position: fixed;
        top: 0; left: 0; right: 0; bottom: 0;
        z-index: 2147483647;
        background: rgba(0, 0, 0, 0.65);
        backdrop-filter: blur(8px);
        display: flex;
        align-items: flex-start;
        justify-content: center;
        padding-top: 15vh;
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    `;

    const container = document.createElement('div');
    container.style.cssText = `
        width: 580px;
        background: #18181b;
        border: 1px solid #27272a;
        border-radius: 12px;
        box-shadow: 0 16px 48px rgba(0,0,0,0.6);
        overflow: hidden;
        display: flex;
        flex-direction: column;
    `;

    const input = document.createElement('input');
    input.type = 'text';
    input.placeholder = "Type a command or search... (e.g. '> toggle theme', 'settings')";
    input.style.cssText = `
        width: 100%;
        padding: 16px 20px;
        background: transparent;
        border: none;
        border-bottom: 1px solid #27272a;
        outline: none;
        font-size: 15px;
        color: #f4f4f5;
        box-sizing: border-box;
    `;

    const list = document.createElement('div');
    list.style.cssText = `
        max-height: 340px;
        overflow-y: auto;
        padding: 6px;
    `;

    const commands = [
        { title: "> Toggle Dark / Light Theme", sub: "Switch browser theme", action: { type: "command", cmd: "toggle_theme" } },
        { title: "> Toggle Vertical Tabs", sub: "Switch between top and sidebar tab layout", action: { type: "command", cmd: "toggle_vertical_tabs" } },
        { title: "> Suspend Inactive Tabs", sub: "Reclaim memory by sleeping background tabs", action: { type: "command", cmd: "suspend_inactive_tabs" } },
        { title: "> Trim Working Set (Reclaim RAM)", sub: "Flush discarded pages to Windows OS", action: { type: "command", cmd: "trim_working_set" } },
        { title: "Open Settings", sub: "nexora://settings", action: { type: "nav", url: "nexora://settings" } },
        { title: "Open Downloads", sub: "nexora://downloads", action: { type: "nav", url: "nexora://downloads" } },
        { title: "Open Bookmarks", sub: "nexora://bookmarks", action: { type: "nav", url: "nexora://bookmarks" } },
        { title: "Open History", sub: "nexora://history", action: { type: "nav", url: "nexora://history" } },
        { title: "Open System Diagnostics", sub: "nexora://diagnostics", action: { type: "nav", url: "nexora://diagnostics" } },
        { title: "New Tab", sub: "Open a fresh browsing tab", action: { type: "command", cmd: "new_tab" } },
    ];

    let selectedIdx = 0;

    function renderItems(filter) {
        list.innerHTML = '';
        const f = filter.toLowerCase().trim();
        const filtered = commands.filter(c => !f || c.title.toLowerCase().includes(f) || c.sub.toLowerCase().includes(f));
        selectedIdx = Math.max(0, Math.min(selectedIdx, filtered.length - 1));

        filtered.forEach((item, idx) => {
            const el = document.createElement('div');
            const isSelected = idx === selectedIdx;
            el.style.cssText = `
                padding: 10px 14px;
                border-radius: 6px;
                cursor: pointer;
                background: ${isSelected ? '#27272a' : 'transparent'};
                display: flex;
                flex-direction: column;
                gap: 2px;
            `;
            el.innerHTML = `
                <div style="font-size:13px;font-weight:500;color:${isSelected ? '#f43f5e' : '#f4f4f5'};">${item.title}</div>
                <div style="font-size:11px;color:#a1a1aa;">${item.sub}</div>
            `;
            el.onclick = () => execute(item);
            list.appendChild(el);
        });

        if (filtered.length === 0) {
            list.innerHTML = '<div style="padding:16px;text-align:center;color:#71717a;font-size:13px;">No matching commands</div>';
        }
    }

    function execute(item) {
        backdrop.remove();
        if (window.chrome && window.chrome.webview) {
            if (item.action.type === 'command') {
                window.chrome.webview.postMessage(JSON.stringify({ action: "command_palette_action", command: item.action.cmd }));
            } else if (item.action.type === 'nav') {
                window.chrome.webview.postMessage(JSON.stringify({ action: "navigate", url: item.action.url }));
            }
        }
    }

    input.oninput = () => { selectedIdx = 0; renderItems(input.value); };
    input.onkeydown = (e) => {
        const items = list.children;
        if (e.key === 'ArrowDown') {
            e.preventDefault();
            selectedIdx = Math.min(selectedIdx + 1, items.length - 1);
            renderItems(input.value);
        } else if (e.key === 'ArrowUp') {
            e.preventDefault();
            selectedIdx = Math.max(selectedIdx - 1, 0);
            renderItems(input.value);
        } else if (e.key === 'Enter') {
            e.preventDefault();
            const f = input.value.toLowerCase().trim();
            const filtered = commands.filter(c => !f || c.title.toLowerCase().includes(f) || c.sub.toLowerCase().includes(f));
            if (filtered[selectedIdx]) execute(filtered[selectedIdx]);
        } else if (e.key === 'Escape') {
            e.preventDefault();
            backdrop.remove();
        }
    };

    backdrop.onclick = (e) => { if (e.target === backdrop) backdrop.remove(); };

    container.appendChild(input);
    container.appendChild(list);
    backdrop.appendChild(container);
    document.body.appendChild(backdrop);
    input.focus();
    renderItems('');
})();"#
    }

    pub fn search(
        query: &str,
        tabs: &[Tab],
        bookmarks: &[BookmarkEntry],
        history: &[HistoryEntry],
    ) -> Vec<PaletteItem> {
        let q = query.trim().to_lowercase();
        let mut results = Vec::new();

        // 1. System Commands
        for cmd in Self::get_system_commands() {
            if q.is_empty() || cmd.title.to_lowercase().contains(&q) || cmd.subtitle.to_lowercase().contains(&q) {
                let score = if cmd.title.to_lowercase().starts_with(&q) { 100 } else { 50 };
                let mut item = cmd;
                item.score = score;
                results.push(item);
            }
        }

        // 2. Open Tabs
        for tab in tabs {
            let title_lower = tab.title.to_lowercase();
            let url_lower = tab.url.to_lowercase();
            if q.is_empty() || title_lower.contains(&q) || url_lower.contains(&q) {
                let score = if title_lower.starts_with(&q) { 90 } else { 40 };
                results.push(PaletteItem {
                    title: tab.title.clone(),
                    subtitle: format!("Tab #{} • {}", tab.id, tab.url),
                    category: PaletteCategory::Tab,
                    action: PaletteAction::SwitchTab(tab.id),
                    score,
                });
            }
        }

        // 3. Bookmarks
        for bm in bookmarks {
            let title_lower = bm.title.to_lowercase();
            let url_lower = bm.url.to_lowercase();
            if !q.is_empty() && (title_lower.contains(&q) || url_lower.contains(&q)) {
                results.push(PaletteItem {
                    title: bm.title.clone(),
                    subtitle: bm.url.clone(),
                    category: PaletteCategory::Bookmark,
                    action: PaletteAction::Navigate(bm.url.clone()),
                    score: 30,
                });
            }
        }

        // 4. History
        for h in history.iter().take(20) {
            let title_lower = h.title.to_lowercase();
            let url_lower = h.url.to_lowercase();
            if !q.is_empty() && (title_lower.contains(&q) || url_lower.contains(&q)) {
                results.push(PaletteItem {
                    title: h.title.clone(),
                    subtitle: h.url.clone(),
                    category: PaletteCategory::History,
                    action: PaletteAction::Navigate(h.url.clone()),
                    score: 20,
                });
            }
        }

        // Sort descending by score
        results.sort_by(|a, b| b.score.cmp(&a.score));
        results.truncate(15);
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_palette_commands() {
        let commands = CommandPalette::get_system_commands();
        assert!(commands.len() >= 8);
        assert!(commands.iter().any(|c| c.title.contains("Vertical Tabs")));
    }

    #[test]
    fn test_command_palette_search() {
        let tab = Tab::new(1, "https://github.com/rust-lang", Some("Rust GitHub"));
        let results = CommandPalette::search("rust", &[tab], &[], &[]);
        assert!(!results.is_empty());
        assert_eq!(results[0].category, PaletteCategory::Tab);
    }
}

