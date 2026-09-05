// Embeds the branded icon and VERSIONINFO block into vtt.exe on Windows
// (TASK-VTT108) — without this, Explorer/Alt-Tab/the taskbar show a generic
// executable with no icon and no version metadata. A no-op on Linux/macOS:
// winresource is only pulled in as a build-dependency under the Windows cfg
// in Cargo.toml, so this file has nothing to do off Windows.
fn main() {
    #[cfg(target_os = "windows")]
    {
        let version = env!("CARGO_PKG_VERSION");
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/vtt.ico");
        res.set("ProductName", "Voice to Text");
        res.set(
            "FileDescription",
            "Voice to Text — offline push-to-talk dictation",
        );
        res.set("ProductVersion", version);
        res.set("FileVersion", version);
        res.set(
            "LegalCopyright",
            "Copyright Emmanuel Powell-Clark. Apache-2.0 licensed.",
        );
        if let Err(e) = res.compile() {
            // A missing resource compiler (rc.exe) shows up here rather than as
            // a build failure — surfacing it as a cargo warning keeps a broken
            // icon from silently shipping unnoticed.
            println!("cargo:warning=winresource failed to embed icon/version info: {e}");
        }
    }
}
