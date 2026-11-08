//! Nexora Unified Design System
//!
//! Provides design tokens, theme palettes, and shared CSS rules across all internal pages
//! (Home, Settings, Diagnostics, Bookmarks, History). Follows modern desktop browser aesthetics
//! inspired by Arc, Linear, and Microsoft Edge.

pub struct DesignTokens;

impl DesignTokens {
    /// Returns the core CSS rules and custom properties for the shared design system.
    pub fn shared_css(is_dark: bool) -> String {
        let (
            bg_base,
            bg_surface,
            bg_card,
            bg_card_hover,
            border,
            border_subtle,
            text_primary,
            text_secondary,
            text_muted,
            accent,
            accent_hover,
            accent_dim,
            input_bg,
            hover_bg,
        ) = if is_dark {
            (
                "#121214", // Main background: deep neutral charcoal
                "#18181B", // Secondary background
                "#202024", // Surface / card
                "#27272D", // Card hover
                "rgba(255, 255, 255, 0.08)", // Low-contrast border
                "rgba(255, 255, 255, 0.04)",
                "#F4F4F5", // Soft white text
                "#A1A1AA", // Muted grey
                "#71717A", // Deeper muted
                "#E11D48", // Restrained crimson accent
                "#F43F5E",
                "rgba(225, 29, 72, 0.15)",
                "#18181B",
                "rgba(255, 255, 255, 0.05)",
            )
        } else {
            (
                "#F8F9FA",
                "#FFFFFF",
                "#FFFFFF",
                "#F1F3F5",
                "rgba(0, 0, 0, 0.10)",
                "rgba(0, 0, 0, 0.05)",
                "#18181B",
                "#52525B",
                "#71717A",
                "#E11D48",
                "#BE123C",
                "rgba(225, 29, 72, 0.10)",
                "#FFFFFF",
                "rgba(0, 0, 0, 0.04)",
            )
        };

        format!(
            r###"
        :root {{
            --bg-base: {bg_base};
            --bg-surface: {bg_surface};
            --bg-card: {bg_card};
            --bg-card-hover: {bg_card_hover};
            --border: {border};
            --border-subtle: {border_subtle};
            --text-primary: {text_primary};
            --text-secondary: {text_secondary};
            --text-muted: {text_muted};
            --accent: {accent};
            --accent-hover: {accent_hover};
            --accent-dim: {accent_dim};
            --input-bg: {input_bg};
            --hover-bg: {hover_bg};
            --success: #10B981;
            --warning: #F59E0B;
            --error: #EF4444;
            --radius-sm: 6px;
            --radius-md: 8px;
            --radius-lg: 12px;
            --font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", sans-serif;
            --font-mono: "Cascadia Code", Consolas, "JetBrains Mono", monospace;
        }}

        * {{
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }}

        body {{
            background-color: var(--bg-base);
            color: var(--text-primary);
            font-family: var(--font-sans);
            font-size: 13px;
            line-height: 1.5;
            -webkit-font-smoothing: antialiased;
            text-rendering: optimizeLegibility;
            user-select: none;
        }}

        /* Buttons */
        .btn {{
            display: inline-flex;
            align-items: center;
            justify-content: center;
            gap: 6px;
            padding: 7px 14px;
            font-size: 13px;
            font-weight: 500;
            font-family: var(--font-sans);
            border-radius: var(--radius-sm);
            border: 1px solid var(--border);
            background: var(--bg-card);
            color: var(--text-primary);
            cursor: pointer;
            transition: all 0.12s ease;
            outline: none;
            text-decoration: none;
            white-space: nowrap;
        }}

        .btn:hover {{
            background: var(--bg-card-hover);
            border-color: rgba(255, 255, 255, 0.15);
        }}

        .btn:active {{
            transform: scale(0.98);
        }}

        .btn-primary {{
            background: var(--accent);
            border-color: var(--accent);
            color: #FFFFFF;
        }}

        .btn-primary:hover {{
            background: var(--accent-hover);
            border-color: var(--accent-hover);
        }}

        .btn-ghost {{
            background: transparent;
            border-color: transparent;
            color: var(--text-secondary);
        }}

        .btn-ghost:hover {{
            background: var(--hover-bg);
            color: var(--text-primary);
            border-color: transparent;
        }}

        .btn-sm {{
            padding: 4px 9px;
            font-size: 12px;
            border-radius: 4px;
        }}

        .btn-icon {{
            padding: 6px;
            width: 32px;
            height: 32px;
            border-radius: var(--radius-sm);
        }}

        /* Text Inputs & Search */
        .input-text {{
            width: 100%;
            padding: 8px 12px;
            font-size: 13px;
            font-family: var(--font-sans);
            background: var(--input-bg);
            color: var(--text-primary);
            border: 1px solid var(--border);
            border-radius: var(--radius-sm);
            outline: none;
            transition: border-color 0.12s ease;
        }}

        .input-text:focus {{
            border-color: var(--accent);
        }}

        .input-text::placeholder {{
            color: var(--text-muted);
        }}

        /* Dropdown Select */
        .select-input {{
            padding: 7px 28px 7px 12px;
            font-size: 13px;
            font-family: var(--font-sans);
            background: var(--input-bg);
            color: var(--text-primary);
            border: 1px solid var(--border);
            border-radius: var(--radius-sm);
            outline: none;
            cursor: pointer;
            appearance: none;
            background-image: url("data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='%23A1A1AA' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3e%3cpolyline points='6 9 12 15 18 9'%3e%3c/polyline%3e%3c/svg%3e");
            background-repeat: no-repeat;
            background-position: right 8px center;
            background-size: 14px;
        }}

        .select-input:focus {{
            border-color: var(--accent);
        }}

        /* Modern Toggle Switch */
        .toggle {{
            position: relative;
            display: inline-block;
            width: 38px;
            height: 20px;
            cursor: pointer;
            flex-shrink: 0;
        }}

        .toggle input {{
            opacity: 0;
            width: 0;
            height: 0;
        }}

        .toggle-slider {{
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background-color: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 20px;
            transition: all 0.2s ease;
        }}

        .toggle-slider:before {{
            position: absolute;
            content: "";
            height: 14px;
            width: 14px;
            left: 2px;
            bottom: 2px;
            background-color: var(--text-secondary);
            border-radius: 50%;
            transition: all 0.2s ease;
        }}

        .toggle input:checked + .toggle-slider {{
            background-color: var(--accent);
            border-color: var(--accent);
        }}

        .toggle input:checked + .toggle-slider:before {{
            transform: translateX(18px);
            background-color: #FFFFFF;
        }}

        /* Subtle Cards */
        .card {{
            background: var(--bg-surface);
            border: 1px solid var(--border);
            border-radius: var(--radius-md);
            padding: 16px 20px;
        }}

        /* Settings Row Layout */
        .setting-row {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 12px 0;
            border-bottom: 1px solid var(--border-subtle);
            gap: 20px;
        }}

        .setting-row:last-child {{
            border-bottom: none;
            padding-bottom: 0;
        }}

        .setting-info {{
            display: flex;
            flex-direction: column;
            gap: 2px;
            flex: 1;
        }}

        .setting-title {{
            font-size: 13px;
            font-weight: 500;
            color: var(--text-primary);
        }}

        .setting-desc {{
            font-size: 12px;
            color: var(--text-muted);
            line-height: 1.4;
        }}

        /* Badges */
        .badge {{
            display: inline-flex;
            align-items: center;
            gap: 4px;
            font-size: 11px;
            font-weight: 500;
            padding: 2px 8px;
            border-radius: 4px;
            background: var(--hover-bg);
            color: var(--text-secondary);
            border: 1px solid var(--border);
        }}

        .badge-success {{
            color: var(--success);
            background: rgba(16, 185, 129, 0.1);
            border-color: rgba(16, 185, 129, 0.2);
        }}
            "###,
            bg_base = bg_base,
            bg_surface = bg_surface,
            bg_card = bg_card,
            bg_card_hover = bg_card_hover,
            border = border,
            border_subtle = border_subtle,
            text_primary = text_primary,
            text_secondary = text_secondary,
            text_muted = text_muted,
            accent = accent,
            accent_hover = accent_hover,
            accent_dim = accent_dim,
            input_bg = input_bg,
            hover_bg = hover_bg,
        )
    }
}

