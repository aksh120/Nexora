//! Reader View Mode (Distraction-Free Reading)
//!
//! Strips ads, navbars, banners, sidebars, and tracking widgets from articles,
//! formatting the body in clean typography with customizable reading themes.

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ReaderTheme {
    Dark,
    Sepia,
    Light,
    Oled,
}

impl Default for ReaderTheme {
    fn default() -> Self {
        Self::Dark
    }
}

impl ReaderTheme {
    pub fn colors(&self) -> (&'static str, &'static str, &'static str) {
        // (background, text, border/accent)
        match self {
            Self::Dark => ("#141416", "#e2e8f0", "#26262e"),
            Self::Sepia => ("#fbf0d9", "#3d332a", "#eedbc0"),
            Self::Light => ("#ffffff", "#1e293b", "#e2e8f0"),
            Self::Oled => ("#000000", "#d1d5db", "#1a1a1a"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReaderOptions {
    pub theme: ReaderTheme,
    pub font_size_px: u32,
    pub line_height: f32,
    pub max_width_px: u32,
}

impl Default for ReaderOptions {
    fn default() -> Self {
        Self {
            theme: ReaderTheme::Dark,
            font_size_px: 18,
            line_height: 1.75,
            max_width_px: 720,
        }
    }
}

pub struct ReaderMode;

impl ReaderMode {
    /// Injected script that extracts clean article body from the active DOM
    pub const EXTRACTOR_SCRIPT: &'static str = r#"
        (() => {
            const article = document.querySelector('article') || document.querySelector('main') || document.body;
            const title = document.querySelector('h1')?.innerText || document.title || 'Reader View';
            const paragraphs = Array.from(article.querySelectorAll('p, h2, h3, h4, img, pre, code, blockquote, ul, ol'))
                .filter(el => {
                    if (el.closest('nav, footer, header, aside, .advertisement, #comments, .comments, .sidebar')) return false;
                    return el.innerText.trim().length > 10 || el.tagName === 'IMG' || el.tagName === 'PRE';
                })
                .map(el => el.outerHTML)
                .join('');
            return JSON.stringify({ title, content: paragraphs });
        })()
    "#;

    /// Renders the standalone Reader View HTML document
    pub fn render_html(title: &str, content_html: &str, options: &ReaderOptions) -> String {
        let (bg, text, border) = options.theme.colors();
        let font_size = options.font_size_px;
        let line_height = options.line_height;
        let max_width = options.max_width_px;

        format!(
            r###"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} - Reader View</title>
    <style>
        * {{
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }}
        body {{
            background: {bg};
            color: {text};
            font-family: -apple-system, BlinkMacSystemFont, "Charter", "Georgia", "Cambria", serif;
            font-size: {font_size}px;
            line-height: {line_height};
            padding: 48px 24px;
            transition: background 0.2s, color 0.2s;
        }}
        .reader-container {{
            max-width: {max_width}px;
            margin: 0 auto;
        }}
        .reader-header {{
            margin-bottom: 36px;
            padding-bottom: 24px;
            border-bottom: 1px solid {border};
        }}
        .reader-title {{
            font-size: 2.2em;
            font-weight: 700;
            line-height: 1.25;
            margin-bottom: 12px;
            letter-spacing: -0.02em;
        }}
        .reader-content p {{
            margin-bottom: 1.5em;
        }}
        .reader-content h2, .reader-content h3 {{
            margin-top: 1.8em;
            margin-bottom: 0.8em;
            line-height: 1.3;
        }}
        .reader-content img {{
            max-width: 100%;
            height: auto;
            border-radius: 6px;
            margin: 1.5em 0;
        }}
        .reader-content pre, .reader-content code {{
            font-family: "Consolas", "Monaco", monospace;
            font-size: 0.9em;
            background: rgba(128, 128, 128, 0.12);
            padding: 3px 6px;
            border-radius: 4px;
        }}
        .reader-content pre {{
            padding: 16px;
            overflow-x: auto;
            margin: 1.5em 0;
        }}
        .reader-toolbar {{
            position: fixed;
            bottom: 24px;
            right: 24px;
            display: flex;
            gap: 8px;
            background: {bg};
            border: 1px solid {border};
            padding: 6px 12px;
            border-radius: 20px;
            box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
            font-family: -apple-system, sans-serif;
            font-size: 13px;
        }}
        .reader-btn {{
            background: none;
            border: none;
            color: {text};
            cursor: pointer;
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 13px;
        }}
        .reader-btn:hover {{
            background: rgba(128, 128, 128, 0.15);
        }}
    </style>
</head>
<body>
    <div class="reader-container">
        <header class="reader-header">
            <h1 class="reader-title">{title}</h1>
        </header>
        <main class="reader-content">
            {content_html}
        </main>
    </div>
    <div class="reader-toolbar">
        <button class="reader-btn" onclick="history.back()">Exit Reader</button>
        <button class="reader-btn" onclick="toggleTheme()">Theme</button>
    </div>
    <script>
        function toggleTheme() {{
            const curBg = document.body.style.background;
            if (curBg === 'rgb(251, 240, 217)' || curBg === '#fbf0d9') {{
                document.body.style.background = '#141416';
                document.body.style.color = '#e2e8f0';
            }} else {{
                document.body.style.background = '#fbf0d9';
                document.body.style.color = '#3d332a';
            }}
        }}
    </script>
</body>
</html>"###
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_reader_html() {
        let opts = ReaderOptions::default();
        let html = ReaderMode::render_html("Test Title", "<p>Hello world paragraph</p>", &opts);
        assert!(html.contains("Test Title"));
        assert!(html.contains("Hello world paragraph"));
        assert!(html.contains("reader-container"));
    }
}

