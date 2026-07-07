/// Embed Windows resources (manifest, icon, version info) into the binary.
///
/// Uses `CARGO_MANIFEST_DIR` for robust path resolution regardless of where
/// cargo is invoked from. Requires `winres` in `[build-dependencies]` and
/// the MSVC `rc.exe` resource compiler from Visual Studio Build Tools.
fn main() {
    #[cfg(target_os = "windows")]
    {
        // CARGO_MANIFEST_DIR is set by Cargo to the directory containing this
        // crate's Cargo.toml (i.e. crates/vidcut-app/).
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let workspace_root = std::path::Path::new(&manifest_dir)
            .parent() // crates/
            .and_then(|p| p.parent()) // workspace root
            .expect("could not resolve workspace root");

        let manifest_path = workspace_root.join("resources").join("vidcut.manifest");
        let icon_path = workspace_root
            .join("resources")
            .join("icons")
            .join("vidcut.ico");

        let mut res = winres::WindowsResource::new();
        res.set_manifest_file(manifest_path.to_str().unwrap());
        res.set_icon(icon_path.to_str().unwrap());
        res.set("ProductName", "VidCut");
        res.set("FileDescription", "VidCut \u{2014} Professional Video Editor");
        res.set("LegalCopyright", "Copyright \u{00A9} 2026 Goriant Studio");
        res.set("CompanyName", "Goriant Studio");
        res.set("FileVersion", "0.1.0.0");
        res.set("ProductVersion", "0.1.0.0");

        if let Err(e) = res.compile() {
            println!(
                "cargo:warning=winres compile failed (no manifest/icon embedded): {e}"
            );
        }
    }
}
