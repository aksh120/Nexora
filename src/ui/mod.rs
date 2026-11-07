// Native UI Components and Theming
#![allow(unused_imports)]

pub mod chrome;
pub mod layout;
pub mod theme;
pub mod branding;

pub use chrome::{BrowserChrome, ChromeHitResult, IDC_OMNIBOX};
pub use layout::*;
pub use theme::{rgb, ThemeMode, ThemePalette};
pub use branding::{BrandIdentity, PRODUCT_NAME, PRODUCT_SLOGAN, PRODUCT_VERSION};
