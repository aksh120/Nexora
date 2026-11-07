// Nexora Visual Identity & Branding Engine
// Product Philosophy: "Maximum performance. Minimum overhead. Complete browsing freedom."

pub const PRODUCT_NAME: &str = "Nexora";
pub const PRODUCT_SLOGAN: &str = "Maximum performance. Minimum overhead. Complete browsing freedom.";
pub const PRODUCT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Clean, scalable SVG Vector Logo with minimalist geometric typography
pub const NEXORA_VECTOR_LOGO_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 420 100" width="100%" height="100%">
    <defs>
        <linearGradient id="nexoraBlue" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stop-color="#3b82f6" />
            <stop offset="100%" stop-color="#1d4ed8" />
        </linearGradient>
    </defs>
    <!-- Geometric Apex Monogram 'N' -->
    <g transform="translate(15, 10)">
        <polygon points="10,80 10,0 28,0 56,52 56,0 74,0 74,80 56,80 28,28 28,80" fill="url(#nexoraBlue)" />
        <polygon points="56,0 74,0 74,28" fill="#60a5fa" />
        <circle cx="19" cy="14" r="5" fill="#93c5fd" />
    </g>
    <!-- Brand Typography -->
    <text x="110" y="62" font-family="-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif" font-size="52" font-weight="700" letter-spacing="1.5" fill="#ffffff">
        NEX<tspan fill="#3b82f6">O</tspan>RA
    </text>
    <text x="112" y="84" font-family="-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif" font-size="12" font-weight="500" letter-spacing="3.5" fill="#8b949e">
        STREAMING-FIRST BROWSER
    </text>
</svg>"##;

/// Scalable Application Icon SVG
pub const NEXORA_APP_ICON_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256">
    <defs>
        <linearGradient id="iconBg" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stop-color="#181e29" />
            <stop offset="100%" stop-color="#0b0e14" />
        </linearGradient>
        <linearGradient id="iconBlue" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stop-color="#60a5fa" />
            <stop offset="50%" stop-color="#3b82f6" />
            <stop offset="100%" stop-color="#1d4ed8" />
        </linearGradient>
    </defs>
    <!-- Rounded Squircle Base -->
    <rect x="8" y="8" width="240" height="240" rx="54" fill="url(#iconBg)" stroke="#2a3443" stroke-width="4" />
    <!-- Sharp Geometric Apex 'N' -->
    <g transform="translate(64, 52)">
        <polygon points="12,152 12,0 42,0 86,96 86,0 116,0 116,152 86,152 42,56 42,152" fill="url(#iconBlue)" />
        <polygon points="86,0 116,0 116,48" fill="#93c5fd" />
        <polygon points="12,104 12,152 42,152" fill="#1e40af" />
        <!-- Aerodynamic Energy Node -->
        <circle cx="27" cy="22" r="8" fill="#dbeafe" />
    </g>
</svg>"##;

pub struct BrandIdentity;

impl BrandIdentity {
    pub fn name() -> &'static str {
        PRODUCT_NAME
    }

    pub fn slogan() -> &'static str {
        PRODUCT_SLOGAN
    }

    pub fn version() -> &'static str {
        PRODUCT_VERSION
    }

    pub fn vector_logo() -> &'static str {
        NEXORA_VECTOR_LOGO_SVG
    }

    pub fn app_icon() -> &'static str {
        NEXORA_APP_ICON_SVG
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brand_metadata() {
        assert_eq!(BrandIdentity::name(), "Nexora");
        assert!(BrandIdentity::slogan().contains("Maximum performance"));
        assert!(BrandIdentity::slogan().contains("Minimum overhead"));
        assert!(BrandIdentity::slogan().contains("Complete browsing freedom"));
        assert!(!BrandIdentity::version().is_empty());
    }

    #[test]
    fn test_svg_validity() {
        assert!(BrandIdentity::vector_logo().contains("<svg"));
        assert!(BrandIdentity::vector_logo().contains("</svg>"));
        assert!(BrandIdentity::vector_logo().contains("NEX"));

        assert!(BrandIdentity::app_icon().contains("<svg"));
        assert!(BrandIdentity::app_icon().contains("</svg>"));
    }
}
