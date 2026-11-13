//! Build script: embeds the app icon (`assets/icon.ico`) into the Windows
//! `.exe` so it shows in Explorer, not just as the window/taskbar icon.
//!
//! Windows-only. On other platforms this is a no-op. If the Windows
//! resource compiler is missing (a local build without the Windows SDK),
//! it warns and keeps building instead of failing — the exe just won't
//! carry the embedded icon. The GitHub Actions Windows runner has the
//! SDK, so release builds get the icon.

#[cfg(windows)]
fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    if let Err(e) = res.compile() {
        println!("cargo:warning=winres failed ({e}); exe icon not embedded");
    }
}

#[cfg(not(windows))]
fn main() {}
