// Nexora Visual Identity & Branding Engine
// Product Philosophy: "Maximum performance. Minimum overhead. Complete browsing freedom."

pub const PRODUCT_NAME: &str = "Nexora";
pub const PRODUCT_SLOGAN: &str = "Maximum performance. Minimum overhead. Complete browsing freedom.";
pub const PRODUCT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const APP_ICON_DATA_URI: &str = include_str!("../../assets/branding/app_icon_b64.txt");
pub const NEXORA_VECTOR_LOGO_SVG: &str = include_str!("../../assets/branding/logo.svg");
pub const NEXORA_APP_ICON_SVG: &str = include_str!("../../assets/branding/icon.svg");

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

    /// Renders the official master logo monogram at any requested pixel size
    pub fn monogram_svg(size: u32) -> String {
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="{size}" height="{size}">
    <image href="{data_uri}" width="512" height="512" />
</svg>"##,
            size = size,
            data_uri = APP_ICON_DATA_URI
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
        assert!(BrandIdentity::app_icon().contains("<svg"));
        assert!(BrandIdentity::monogram_svg(32).contains("<svg"));
        assert!(BrandIdentity::monogram_svg(32).contains("image href="));
    }
}
