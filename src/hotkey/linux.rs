// X11 keysym and xlib constants ship CamelCase / Mixed_Case (e.g. XK_End,
// KeyPress, Mod5Mask) because they match the historical Xlib C headers.
// Clippy flags these as "constant in pattern should have an upper case name"
// whenever we match against them. They're re-exported verbatim by the x11
// crate and we can't rename them — allow the lint at module level.
#![allow(non_upper_case_globals)]

use super::{HotkeyCmd, KeyEvent};
use std::sync::mpsc;
use std::thread;
use x11::keysym::*;
use x11::xlib::*;

/// Start the global hotkey monitor in a background thread.
///
/// Returns a sender for commands (e.g. changing the hotkey or stopping).
/// The callback fires on key-down and key-up events.
pub fn start_monitor<F>(initial_keycode: u8, callback: F) -> anyhow::Result<mpsc::Sender<HotkeyCmd>>
where
    F: Fn(KeyEvent) + Send + 'static,
{
    let (cmd_tx, cmd_rx) = mpsc::channel();

    thread::Builder::new()
        .name("hotkey-monitor".into())
        .spawn(move || unsafe {
            monitor_loop(initial_keycode, callback, cmd_rx);
        })?;

    Ok(cmd_tx)
}

/// Get the X11 keycode for Scroll Lock on the given display.
unsafe fn get_scroll_lock_keycode(display: *mut Display) -> u32 {
    let kc = XKeysymToKeycode(display, XK_Scroll_Lock as u64);
    if kc == 0 {
        crate::vtt_log!("Scroll Lock not found, using keycode 78");
        78
    } else {
        kc as u32
    }
}

/// Grab a key on the root window with Num Lock / Caps Lock modifier combinations.
unsafe fn grab_key(display: *mut Display, root: Window, keycode: u32) {
    let modifiers: [u32; 4] = [
        0,                   // No modifiers
        LockMask,            // Caps Lock
        Mod2Mask,            // Num Lock
        LockMask | Mod2Mask, // Both
    ];
    for &m in &modifiers {
        XGrabKey(
            display,
            keycode as i32,
            m,
            root,
            1, // owner_events = True
            GrabModeAsync,
            GrabModeAsync,
        );
    }
    XSync(display, 0);
}

/// Ungrab a key from the root window.
unsafe fn ungrab_key(display: *mut Display, root: Window, keycode: u32) {
    let modifiers: [u32; 4] = [0, LockMask, Mod2Mask, LockMask | Mod2Mask];
    for &m in &modifiers {
        XUngrabKey(display, keycode as i32, m, root);
    }
}

/// Get a human-readable key name for display purposes.
pub fn get_key_name(keycode: u8) -> String {
    if keycode == 0 {
        return "Scroll Lock".into();
    }
    unsafe {
        let display = XOpenDisplay(std::ptr::null());
        if display.is_null() {
            return format!("Key {}", keycode);
        }
        let keysym = XKeycodeToKeysym(display, keycode, 0);
        let name = match keysym as u32 {
            XK_Scroll_Lock => "Scroll Lock",
            XK_Caps_Lock => "Caps Lock",
            XK_Num_Lock => "Num Lock",
            XK_Pause => "Pause",
            XK_Print => "Print Screen",
            XK_Insert => "Insert",
            XK_Home => "Home",
            XK_End => "End",
            XK_Page_Up => "Page Up",
            XK_Page_Down => "Page Down",
            XK_F1 => "F1",
            XK_F2 => "F2",
            XK_F3 => "F3",
            XK_F4 => "F4",
            XK_F5 => "F5",
            XK_F6 => "F6",
            XK_F7 => "F7",
            XK_F8 => "F8",
            XK_F9 => "F9",
            XK_F10 => "F10",
            XK_F11 => "F11",
            XK_F12 => "F12",
            _ => {
                let s = XKeysymToString(keysym);
                if s.is_null() {
                    XCloseDisplay(display);
                    return format!("Key {}", keycode);
                }
                let name = std::ffi::CStr::from_ptr(s).to_string_lossy().into_owned();
                XCloseDisplay(display);
                return name;
            }
        };
        XCloseDisplay(display);
        name.to_string()
    }
}

unsafe fn monitor_loop<F>(initial_keycode: u8, callback: F, cmd_rx: mpsc::Receiver<HotkeyCmd>)
where
    F: Fn(KeyEvent) + Send + 'static,
{
    let display = XOpenDisplay(std::ptr::null());
    if display.is_null() {
        crate::vtt_log!("Hotkey: failed to open X display");
        return;
    }

    // Enable detectable auto-repeat (suppress synthetic KeyRelease+KeyPress pairs)
    let mut supported: i32 = 0;
    XkbSetDetectableAutoRepeat(display, 1, &mut supported);
    if supported == 0 {
        crate::vtt_log!("Detectable auto-repeat not supported; manual filtering active");
    }

    let root = XDefaultRootWindow(display);
    let mut keycode: u32 = if initial_keycode >= 8 {
        initial_keycode as u32
    } else {
        get_scroll_lock_keycode(display)
    };

    grab_key(display, root, keycode);
    crate::vtt_log!("Hotkey monitor started (keycode {})", keycode);

    let mut event: XEvent = std::mem::zeroed();

    loop {
        // Check for commands (non-blocking)
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                HotkeyCmd::SetKeycode(new_kc) => {
                    ungrab_key(display, root, keycode);
                    keycode = if new_kc >= 8 {
                        new_kc as u32
                    } else {
                        get_scroll_lock_keycode(display)
                    };
                    grab_key(display, root, keycode);
                    crate::vtt_log!("Hotkey changed to keycode {}", keycode);
                }
                HotkeyCmd::Stop => {
                    ungrab_key(display, root, keycode);
                    XCloseDisplay(display);
                    crate::vtt_log!("Hotkey monitor stopped");
                    return;
                }
            }
        }

        // Wait for X11 event with timeout (poll every 100ms for commands)
        if XPending(display) == 0 {
            // Use select/poll to wait for X11 fd with timeout
            let fd = XConnectionNumber(display);
            let mut read_fds: libc::fd_set = std::mem::zeroed();
            libc::FD_ZERO(&mut read_fds);
            libc::FD_SET(fd, &mut read_fds);
            let mut timeout = libc::timeval {
                tv_sec: 0,
                tv_usec: 100_000, // 100ms
            };
            let ready = libc::select(
                fd + 1,
                &mut read_fds,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut timeout,
            );
            if ready <= 0 {
                continue;
            }
        }

        while XPending(display) > 0 {
            XNextEvent(display, &mut event);

            match event.get_type() {
                KeyPress => {
                    let key_event = event.key;
                    if key_event.keycode == keycode {
                        callback(KeyEvent::Down);
                    }
                }
                KeyRelease => {
                    let key_event = event.key;
                    if key_event.keycode == keycode {
                        // Auto-repeat filtering: if KeyPress follows immediately, skip both
                        if XPending(display) > 0 {
                            let mut next: XEvent = std::mem::zeroed();
                            XPeekEvent(display, &mut next);
                            if next.get_type() == KeyPress
                                && next.key.keycode == key_event.keycode
                                && (next.key.time.wrapping_sub(key_event.time)) < 20
                            {
                                // Consume the paired KeyPress
                                XNextEvent(display, &mut next);
                                continue;
                            }
                        }
                        callback(KeyEvent::Up);
                    }
                }
                _ => {}
            }
        }
    }
}
