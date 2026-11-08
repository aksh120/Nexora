# Nexora Information Page Redesign Specification

## 1. Overview & Objective
This document outlines the architectural and visual redesign of the Nexora Information Page (`info-page/`).
The redesign transitions the page from an AI-generated SaaS landing-page template into an authentic, developer-first open-source software project website with high technical density, understated typography, intentional whitespace, and consistent Lucide iconography.

## 2. Design Principles
- **Understated & Confident**: Deep charcoal/zinc canvas (`#0a0a0c`, `#141418`), restrained neutral borders (`#23232a`), off-white typography (`#ededf0`), with Nexora Crimson (`#e11d48`) reserved strictly for primary actions, active tabs, and key states.
- **Document Flow over Card Clutter**: Group related features and specifications into logical sections, definition tables, and flow diagrams rather than uniform card grids.
- **Readable Typography**: System sans-serif font stack with balanced line heights (1.6) and comfortable font sizes (14px–16px body, 20px–36px headings). Monospace typography strictly reserved for code, keyboard shortcuts, version tags, and performance metrics.
- **Strict Lucide Iconography**: Every icon conforms to official Lucide specs (stroke width 1.75–2px, visual alignment, accessible labelling).
- **Technical Rigor & Transparency**: Every performance claim, memory measurement, and architecture flow is grounded in verified benchmarks and codebase reality.

## 3. Structural Sections
1. **Header & Global Navigation**: Logo, section jump links (Overview, Architecture, Benchmarks, Capabilities, Internal Pages, Shortcuts, Install), GitHub repo link, release pill.
2. **Hero Section**:
   - Headline: "Nexora. A browser built around performance."
   - Subtitle: "A lightweight, streaming-focused desktop browser for Windows, built with native Rust, Win32 and WebView2."
   - Primary actions: Download Windows Installer (`Download`), View Source (`Github`).
   - Horizontal Statistics Matrix: 1.46 MB binary, 4.8–18 MB host RAM, 263 MB YouTube 1080p, 14 ms tab switch latency, with documented test environment.
3. **Process Architecture**:
   - High-contrast directional architecture flowchart comparing Host (`Nexora.exe`) vs WebView2 Subprocesses (`msedgewebview2.exe`).
   - Technical analysis of W3C standards breadth, Widevine CDM hardware decryption, zero-copy D3D11 frame compositing, and YouTube V8/buffer memory realities.
4. **Benchmark Matrix & Visualizer**:
   - Standardized 12-scenario table (SC-01 to SC-12) with category filters (All, Media, Tabs, Latency).
   - Side-by-side memory visualizer comparing Nexora Total against Google Chrome 124 and Microsoft Edge.
   - Documented test conditions, hardware specifications, and measurement methodology.
5. **Engineered Capabilities**:
   - Structured into 4 domains: Browser Fundamentals, Media & Streaming, Privacy & Security, Performance & Persistence.
   - Clear technical descriptions, shortcuts, and Lucide icons.
6. **Internal Browser Pages Documentation**:
   - Visual mockups and technical function descriptions for `nexora://home`, `nexora://settings`, `nexora://diagnostics`, `nexora://bookmarks`, and `nexora://history`.
7. **Keyboard Shortcuts Directory**:
   - Categorized reference table with live search input and readable `<kbd>` elements.
8. **Installation & Source Build Guide**:
   - Standalone Installer (`.exe`), Portable Zip (`.zip`), and Rust build instructions with functional copy buttons.
9. **Technical Specification Table**:
   - Comprehensive matrix covering framework, rendering engine, binary size, OS requirements, test coverage (72/72 tests passing), and MIT license.
10. **Footer**:
    - Authentic open-source footer with GitHub links, releases, license, and author attribution (Akshat Apoorv).
