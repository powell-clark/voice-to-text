//! Start-at-login registration (TASK-VTT094, parity with the Linux systemd
//! `--user` service, FEAT-VTT015).
//!
//! Windows: a per-user `HKCU\...\CurrentVersion\Run` value — no admin required,
//! launches the tray app at login. Linux (systemd `--user`) and macOS
//! (LaunchAgent) equivalents are stubs for now so the shared tray code compiles;
//! they are tracked in TASK-VTT094.

#[cfg(target_os = "windows")]
mod imp {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
    use winreg::RegKey;

    const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const VALUE_NAME: &str = "VoiceToText";

    fn exe_path() -> anyhow::Result<String> {
        Ok(std::env::current_exe()?.to_string_lossy().into_owned())
    }

    pub fn is_enabled() -> bool {
        RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(RUN_KEY)
            .and_then(|run| run.get_value::<String, _>(VALUE_NAME))
            .is_ok()
    }

    pub fn enable() -> anyhow::Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (run, _) = hkcu.create_subkey(RUN_KEY)?;
        // Quote the path so spaces in "Program Files" survive a login launch.
        run.set_value(VALUE_NAME, &format!("\"{}\"", exe_path()?))?;
        crate::vtt_log!("Autostart enabled (HKCU Run)");
        Ok(())
    }

    pub fn disable() -> anyhow::Result<()> {
        if let Ok(run) =
            RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE)
        {
            let _ = run.delete_value(VALUE_NAME); // ignore "not found"
        }
        crate::vtt_log!("Autostart disabled");
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    // Linux (systemd --user) + macOS (LaunchAgent) autostart are tracked in
    // TASK-VTT094; no-ops for now so the portable tray compiles everywhere.
    pub fn is_enabled() -> bool {
        false
    }
    pub fn enable() -> anyhow::Result<()> {
        Ok(())
    }
    pub fn disable() -> anyhow::Result<()> {
        Ok(())
    }
}

pub use imp::is_enabled;

// `enable` is called directly by the Windows first-run default (TASK-VTT109) in
// main.rs; re-export it on Windows so the public `autostart::enable()` path
// resolves. On macOS the stub is reached only via `toggle()`, so no re-export is
// needed there (and re-exporting it unused would trip dead-code on a -D warnings
// build).
#[cfg(target_os = "windows")]
pub use imp::enable;

/// Flip the autostart registration; returns the new enabled state.
pub fn toggle() -> anyhow::Result<bool> {
    if is_enabled() {
        imp::disable()?;
        Ok(false)
    } else {
        imp::enable()?;
        Ok(true)
    }
}

/// True on platforms where the toggle actually persists (Windows today).
pub const SUPPORTED: bool = cfg!(target_os = "windows");
