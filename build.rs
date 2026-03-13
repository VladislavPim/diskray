fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/diskray.ico");
        // Добавляем информацию о версии
        res.set("FileDescription", "DiskRay Disk Space Analyzer");
        res.set("ProductName", "DiskRay");
        res.set("CompanyName", "Vladislav Pimenov");   // ← имя издателя
        res.set("LegalCopyright", "© 2025 Vladislav Pimenov");
        res.set("FileVersion", env!("CARGO_PKG_VERSION"));
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        // Остальной код (manifest и т.д.)
        res.set_manifest(r#"
            <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
                <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
                    <security>
                        <requestedPrivileges>
                            <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
                        </requestedPrivileges>
                    </security>
                </trustInfo>
                <application xmlns="urn:schemas-microsoft-com:asm.v3">
                    <windowsSettings>
                        <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true</dpiAware>
                    </windowsSettings>
                </application>
            </assembly>
        "#);
        if let Err(e) = res.compile() {
            eprintln!("Failed to compile resources: {}", e);
            std::process::exit(1);
        }
    }
    println!("cargo:rerun-if-changed=assets/diskray.ico");
}
