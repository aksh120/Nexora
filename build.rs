fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/branding/icon.ico");
        res.set_manifest_file("assets/manifest/nexora.manifest");
        res.set("FileDescription", "Nexora");
        res.set("ProductName", "Nexora");
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));
        res.set("LegalCopyright", "Copyright (c) 2026 Nexora Development Team");
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile windows resource: {}", e);
        }
    }
}

