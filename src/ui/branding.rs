// Nexora Visual Identity & Branding Engine
// Product Philosophy: "Maximum performance. Minimum overhead. Complete browsing freedom."

pub const PRODUCT_NAME: &str = "Nexora";
pub const PRODUCT_SLOGAN: &str = "Maximum performance. Minimum overhead. Complete browsing freedom.";
pub const PRODUCT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Clean, scalable SVG Vector Logo with minimalist geometric typography - The Quantum Prism
pub const NEXORA_VECTOR_LOGO_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 440 100" width="100%" height="100%" fill="none">
    <defs>
        <linearGradient id="logo-prism1" x1="16" y1="16" x2="72" y2="96" gradientUnits="userSpaceOnUse">
            <stop offset="0%" stop-color="#FF385C" />
            <stop offset="100%" stop-color="#E11D48" />
        </linearGradient>
        <linearGradient id="logo-prism2" x1="112" y1="112" x2="56" y2="32" gradientUnits="userSpaceOnUse">
            <stop offset="0%" stop-color="#FB7185" />
            <stop offset="100%" stop-color="#9F1239" />
        </linearGradient>
    </defs>
    <!-- Quantum Prism Monogram -->
    <g transform="translate(15, 6) scale(0.68)">
        <path d="M22 20 L58 20 L38 88 L18 88 Z" fill="url(#logo-prism1)" />
        <path d="M58 20 L78 68 L58 68 L38 20 Z" fill="#FDA4AF" opacity="0.85" />
        <path d="M106 108 L70 108 L90 40 L110 40 Z" fill="url(#logo-prism2)" />
        <path d="M70 108 L50 60 L70 60 L90 108 Z" fill="#F43F5E" opacity="0.85" />
        <polygon points="50,60 78,68 64,104" fill="#9F1239" opacity="0.6" />
        <circle cx="64" cy="64" r="3.5" fill="#FFFFFF" />
    </g>
    <!-- Brand Typography -->
    <text x="120" y="62" font-family="-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif" font-size="52" font-weight="700" letter-spacing="1.5" fill="#F7F0F1">
        NEX<tspan fill="#E11D48">O</tspan>RA
    </text>
    <text x="122" y="84" font-family="-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif" font-size="12" font-weight="500" letter-spacing="3.5" fill="#997F83">
        STREAMING-FIRST BROWSER
    </text>
</svg>"##;

/// Scalable Application Icon SVG - The Quantum Prism Monogram
pub const NEXORA_APP_ICON_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" fill="none">
    <defs>
        <linearGradient id="icon-prism1" x1="32" y1="32" x2="144" y2="192" gradientUnits="userSpaceOnUse">
            <stop offset="0%" stop-color="#FF385C" />
            <stop offset="100%" stop-color="#E11D48" />
        </linearGradient>
        <linearGradient id="icon-prism2" x1="224" y1="224" x2="112" y2="64" gradientUnits="userSpaceOnUse">
            <stop offset="0%" stop-color="#FB7185" />
            <stop offset="100%" stop-color="#9F1239" />
        </linearGradient>
        <linearGradient id="icon-inner" x1="90" y1="90" x2="160" y2="160" gradientUnits="userSpaceOnUse">
            <stop offset="0%" stop-color="#E11D48" />
            <stop offset="100%" stop-color="#4C0519" />
        </linearGradient>
    </defs>
    <!-- Pure AMOLED Black Base with Carbon Border -->
    <rect x="8" y="8" width="240" height="240" rx="54" fill="#000000" stroke="#291518" stroke-width="4" />
    
    <!-- The Quantum Prism Monogram 'N' -->
    <g transform="translate(32, 32) scale(1.5)">
        <path d="M22 20 L58 20 L38 88 L18 88 Z" fill="url(#icon-prism1)" />
        <path d="M58 20 L78 68 L58 68 L38 20 Z" fill="#FDA4AF" opacity="0.85" />
        <path d="M106 108 L70 108 L90 40 L110 40 Z" fill="url(#icon-prism2)" />
        <path d="M70 108 L50 60 L70 60 L90 108 Z" fill="#F43F5E" opacity="0.85" />
        <polygon points="50,60 78,68 64,104" fill="url(#icon-inner)" opacity="0.6" />
        <circle cx="64" cy="64" r="3.5" fill="#FFFFFF" />
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
