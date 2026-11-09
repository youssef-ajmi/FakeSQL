fn main() {
    // Re-run the build script (and re-embed the app icon resource) whenever the
    // icon files change; otherwise an old cached .res keeps being linked.
    println!("cargo:rerun-if-changed=icons/icon.ico");
    println!("cargo:rerun-if-changed=icons/128x128.png");
    tauri_build::build()
}
