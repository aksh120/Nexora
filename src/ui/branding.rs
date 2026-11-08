// Nexora Visual Identity & Branding Engine
// Product Philosophy: "Maximum performance. Minimum overhead. Complete browsing freedom."

pub const PRODUCT_NAME: &str = "Nexora";
pub const PRODUCT_SLOGAN: &str = "Maximum performance. Minimum overhead. Complete browsing freedom.";
pub const PRODUCT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Clean, scalable SVG Vector Logo with official Orbital Ribbon N and modern typography
pub const NEXORA_VECTOR_LOGO_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 440 100" width="100%" height="100%" fill="none">
    <defs>
        <linearGradient id="orbit-silver-full" x1="0" y1="0" x2="100" y2="100">
            <stop offset="0%" stop-color="#FFFFFF"/>
            <stop offset="50%" stop-color="#D4D4D8"/>
            <stop offset="100%" stop-color="#71717A"/>
        </linearGradient>
        <linearGradient id="ribbon-n1-full" x1="20" y1="20" x2="60" y2="85">
            <stop offset="0%" stop-color="#FF385C"/>
            <stop offset="100%" stop-color="#E11D48"/>
        </linearGradient>
        <linearGradient id="ribbon-n2-full" x1="45" y1="20" x2="75" y2="85">
            <stop offset="0%" stop-color="#E11D48"/>
            <stop offset="100%" stop-color="#9F1239"/>
        </linearGradient>
    </defs>
    <!-- Official Orbital Ribbon Monogram -->
    <g transform="translate(15, 5) scale(0.9)">
        <!-- Orbital Ring Back Arc -->
        <path d="M 48 10 C 20 18 8 45 14 68 C 17 80 25 89 36 93 C 33 89 30 83 29 76 C 26 58 35 34 56 22 Z" fill="url(#orbit-silver-full)" opacity="0.85"/>
        <!-- Ribbon N Left Leg -->
        <path d="M 32 25 L 48 25 L 28 82 L 20 82 C 18 82 17 80 18 78 Z" fill="url(#ribbon-n1-full)"/>
        <!-- Fold Crease -->
        <path d="M 32 25 L 48 25 L 42 38 L 30 38 Z" fill="#FDA4AF" opacity="0.9"/>
        <!-- Main Diagonal -->
        <path d="M 32 25 L 48 25 L 68 82 L 52 82 Z" fill="url(#ribbon-n2-full)"/>
        <!-- Right Leg -->
        <path d="M 62 25 L 78 25 C 80 25 81 27 80 29 L 66 82 L 52 82 Z" fill="url(#ribbon-n1-full)"/>
        <!-- Orbital Ring Front Arc -->
        <path d="M 52 90 C 80 82 92 55 86 32 C 83 20 75 11 64 7 C 67 11 70 17 71 24 C 74 42 65 66 44 78 Z" fill="url(#orbit-silver-full)"/>
    </g>
    <!-- Brand Typography -->
    <text x="120" y="62" font-family="-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif" font-size="50" font-weight="700" letter-spacing="1.5" fill="#F4F4F5">
        NEX<tspan fill="#E11D48">O</tspan>RA
    </text>
    <text x="122" y="84" font-family="-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif" font-size="11" font-weight="500" letter-spacing="3.5" fill="#A1A1AA">
        STREAMING-FIRST BROWSER
    </text>
</svg>"##;

/// Scalable Application Icon SVG - Official Orbital Ribbon N Monogram
pub const NEXORA_APP_ICON_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256" fill="none">
    <defs>
        <linearGradient id="orbit-silver-icon" x1="0" y1="0" x2="256" y2="256">
            <stop offset="0%" stop-color="#FFFFFF"/>
            <stop offset="50%" stop-color="#D4D4D8"/>
            <stop offset="100%" stop-color="#71717A"/>
        </linearGradient>
        <linearGradient id="ribbon-n1-icon" x1="50" y1="50" x2="160" y2="220">
            <stop offset="0%" stop-color="#FF385C"/>
            <stop offset="100%" stop-color="#E11D48"/>
        </linearGradient>
        <linearGradient id="ribbon-n2-icon" x1="110" y1="50" x2="190" y2="220">
            <stop offset="0%" stop-color="#E11D48"/>
            <stop offset="100%" stop-color="#9F1239"/>
        </linearGradient>
    </defs>
    <!-- Pure Dark Charcoal Base with Subtle Border -->
    <rect x="8" y="8" width="240" height="240" rx="54" fill="#121214" stroke="rgba(255, 255, 255, 0.08)" stroke-width="3" />
    
    <!-- The Official Orbital Ribbon Monogram 'N' (Scaled to 256x256) -->
    <g transform="translate(38, 38) scale(1.8)">
        <!-- Orbital Ring Back Arc -->
        <path d="M 48 10 C 20 18 8 45 14 68 C 17 80 25 89 36 93 C 33 89 30 83 29 76 C 26 58 35 34 56 22 Z" fill="url(#orbit-silver-icon)" opacity="0.85"/>
        <!-- Ribbon N Left Leg -->
        <path d="M 32 25 L 48 25 L 28 82 L 20 82 C 18 82 17 80 18 78 Z" fill="url(#ribbon-n1-icon)"/>
        <!-- Fold Crease -->
        <path d="M 32 25 L 48 25 L 42 38 L 30 38 Z" fill="#FDA4AF" opacity="0.9"/>
        <!-- Main Diagonal -->
        <path d="M 32 25 L 48 25 L 68 82 L 52 82 Z" fill="url(#ribbon-n2-icon)"/>
        <!-- Right Leg -->
        <path d="M 62 25 L 78 25 C 80 25 81 27 80 29 L 66 82 L 52 82 Z" fill="url(#ribbon-n1-icon)"/>
        <!-- Orbital Ring Front Arc -->
        <path d="M 52 90 C 80 82 92 55 86 32 C 83 20 75 11 64 7 C 67 11 70 17 71 24 C 74 42 65 66 44 78 Z" fill="url(#orbit-silver-icon)"/>
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

    /// Renders the official Orbital Ribbon N monogram at any requested pixel size
    pub fn monogram_svg(size: u32) -> String {
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" width="{size}" height="{size}" fill="none">
    <defs>
        <linearGradient id="orbit-silver-mono" x1="0" y1="0" x2="100" y2="100">
            <stop offset="0%" stop-color="#FFFFFF"/>
            <stop offset="50%" stop-color="#D4D4D8"/>
            <stop offset="100%" stop-color="#71717A"/>
        </linearGradient>
        <linearGradient id="ribbon-n1-mono" x1="20" y1="20" x2="60" y2="85">
            <stop offset="0%" stop-color="#FF385C"/>
            <stop offset="100%" stop-color="#E11D48"/>
        </linearGradient>
        <linearGradient id="ribbon-n2-mono" x1="45" y1="20" x2="75" y2="85">
            <stop offset="0%" stop-color="#E11D48"/>
            <stop offset="100%" stop-color="#9F1239"/>
        </linearGradient>
    </defs>
    <!-- Orbital Ring Back Arc -->
    <path d="M 48 10 C 20 18 8 45 14 68 C 17 80 25 89 36 93 C 33 89 30 83 29 76 C 26 58 35 34 56 22 Z" fill="url(#orbit-silver-mono)" opacity="0.85"/>
    <!-- Ribbon N Left Leg -->
    <path d="M 32 25 L 48 25 L 28 82 L 20 82 C 18 82 17 80 18 78 Z" fill="url(#ribbon-n1-mono)"/>
    <!-- Fold Crease -->
    <path d="M 32 25 L 48 25 L 42 38 L 30 38 Z" fill="#FDA4AF" opacity="0.9"/>
    <!-- Main Diagonal -->
    <path d="M 32 25 L 48 25 L 68 82 L 52 82 Z" fill="url(#ribbon-n2-mono)"/>
    <!-- Right Leg -->
    <path d="M 62 25 L 78 25 C 80 25 81 27 80 29 L 66 82 L 52 82 Z" fill="url(#ribbon-n1-mono)"/>
    <!-- Orbital Ring Front Arc -->
    <path d="M 52 90 C 80 82 92 55 86 32 C 83 20 75 11 64 7 C 67 11 70 17 71 24 C 74 42 65 66 44 78 Z" fill="url(#orbit-silver-mono)"/>
</svg>"##,
            size = size
        )
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

        let mono = BrandIdentity::monogram_svg(28);
        assert!(mono.contains("<svg"));
        assert!(mono.contains("width=\"28\""));
    }
}
