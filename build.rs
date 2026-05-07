fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let version = env!("CARGO_PKG_VERSION");
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "WinFolSize");
        res.set("FileDescription", "Disk Space Visualizer");
        res.set("LegalCopyright", "© 2025 Wictor Wilén");
        res.set("ProductVersion", version);
        res.set("FileVersion", version);
        res.compile().expect("Failed to compile Windows resources");
    }
}
