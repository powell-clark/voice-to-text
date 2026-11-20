//! Start-at-login registration (TASK-VTT094, parity with the Linux systemd
//! `--user` service, FEAT-VTT015).
//!
//! Windows: a per-user `HKCU\...\CurrentVersion\Run` value — no admin required,
//! launches the tray app at login. Linux (systemd `--user`) and macOS
//! (LaunchAgent) equivalents are stubs for now so the shared tray code compiles;
//! they are tracked in TASK-VTT094.

#[cfg(target_os = "windows")]
mod imp {
    use windows_sys::Win32::Foundation::{ERROR_SUCCESS, WIN32_ERROR};
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW,
        RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_ALL_ACCESS, KEY_READ, KEY_SET_VALUE,
        REG_EXPAND_SZ, REG_MULTI_SZ, REG_OPTION_NON_VOLATILE, REG_SAM_FLAGS, REG_SZ,
    };

    const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const VALUE_NAME: &str = "VoiceToText";

    fn exe_path() -> anyhow::Result<String> {
        Ok(std::env::current_exe()?.to_string_lossy().into_owned())
    }

    /// `s` as NUL-terminated UTF-16, the form the registry's `W` calls take.
    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// A registry status as a `Result`, carrying the OS error on failure.
    fn check(status: WIN32_ERROR) -> std::io::Result<()> {
        if status == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(std::io::Error::from_raw_os_error(status as i32))
        }
    }

    /// An open registry key, closed when dropped, on every path.
    struct Key(HKEY);

    impl Drop for Key {
        fn drop(&mut self) {
            // SAFETY: the handle came from a successful open or create and is
            // closed only here, once.
            unsafe { RegCloseKey(self.0) };
        }
    }

    /// The HKCU Run key opened with `access`, or `None` if it cannot be.
    fn open_run_key(access: REG_SAM_FLAGS) -> Option<Key> {
        let subkey = wide(RUN_KEY);
        let mut hkey: HKEY = std::ptr::null_mut();
        // SAFETY: `subkey` is NUL-terminated and outlives the call, and
        // `hkey` is a valid place for the opened handle.
        let status =
            unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, access, &mut hkey) };
        if status == ERROR_SUCCESS {
            Some(Key(hkey))
        } else {
            None
        }
    }

    pub fn is_enabled() -> bool {
        let Some(run) = open_run_key(KEY_READ) else {
            return false;
        };
        let name = wide(VALUE_NAME);
        let mut kind = 0;
        // SAFETY: `name` is NUL-terminated and outlives the call; with no
        // data buffer and no size pointer the call reports only the type.
        let status = unsafe {
            RegQueryValueExW(
                run.0,
                name.as_ptr(),
                std::ptr::null(),
                &mut kind,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        // The string types a String read of the value accepts.
        status == ERROR_SUCCESS && matches!(kind, REG_SZ | REG_EXPAND_SZ | REG_MULTI_SZ)
    }

    pub fn enable() -> anyhow::Result<()> {
        let subkey = wide(RUN_KEY);
        let mut hkey: HKEY = std::ptr::null_mut();
        // SAFETY: `subkey` is NUL-terminated and outlives the call; the class,
        // security attributes and disposition are optional and passed null;
        // `hkey` is a valid place for the handle.
        check(unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                subkey.as_ptr(),
                0,
                std::ptr::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_ALL_ACCESS,
                std::ptr::null(),
                &mut hkey,
                std::ptr::null_mut(),
            )
        })?;
        let run = Key(hkey);
        // Quote the path so spaces in "Program Files" survive a login launch.
        let data = wide(&format!("\"{}\"", exe_path()?));
        let size = u32::try_from(std::mem::size_of_val(data.as_slice()))?;
        let name = wide(VALUE_NAME);
        // SAFETY: `name` is NUL-terminated, `data` holds `size` bytes of
        // NUL-terminated UTF-16, and both outlive the call.
        check(unsafe {
            RegSetValueExW(run.0, name.as_ptr(), 0, REG_SZ, data.as_ptr().cast(), size)
        })?;
        crate::vtt_log!("Autostart enabled (HKCU Run)");
        Ok(())
    }

    pub fn disable() -> anyhow::Result<()> {
        if let Some(run) = open_run_key(KEY_SET_VALUE) {
            let name = wide(VALUE_NAME);
            // SAFETY: `name` is NUL-terminated and outlives the call. The
            // status is ignored: a value already gone is the state wanted.
            let _ = unsafe { RegDeleteValueW(run.0, name.as_ptr()) };
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
