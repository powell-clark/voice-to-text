//! TASK-VTT138 spike: prototype a "press-a-key-now" hotkey capture dialog
//! on `tao` (the windowing crate `tray-icon`/`muda` already use), for a
//! UX-fidelity comparison against `linux.rs`'s GTK `show_hotkey_dialog`
//! (`connect_key_press_event` / `connect_key_release_event`, 8-255 X11
//! keycode validation, `HotkeyCmd::SetKeycode`).
//!
//! `tao` is a `[dev-dependencies]`-only addition (see Cargo.toml) — this
//! example never ships in any release binary on any platform.
//!
//! Run interactively:
//!   cargo run --example hotkey_capture_spike
//! Automated proof (used for this spike's evidence — no human needed):
//!   cargo run --example hotkey_capture_spike &
//!   sleep 1 && xdotool key --window <id> a
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::keyboard::{KeyCode, NativeKeyCode};
use tao::window::WindowBuilder;

/// Mirrors `linux.rs::show_hotkey_dialog`'s validation: GTK accepts raw X11
/// keycodes in 8..=255. `tao` 0.30's Linux backend is GTK-based too (its
/// `NativeKeyCode::Gtk(u16)` carries the raw GDK/X11 keycode directly) — no
/// offset conversion needed, but it also means the "avoid GTK" motivation
/// for unifying trays does not fully hold for the Linux windowing layer
/// itself, only for the tray/menu layer.
fn validate_x11_keycode(code: u32) -> Result<u8, String> {
    if (8..=255).contains(&code) {
        Ok(code as u8)
    } else {
        Err(format!("keycode {code} outside GTK's accepted 8-255 range"))
    }
}

fn main() {
    let event_loop = EventLoopBuilder::new().build();
    let window = WindowBuilder::new()
        .with_title("VTT hotkey capture spike (TASK-VTT138) — press any key")
        .with_inner_size(tao::dpi::LogicalSize::new(420.0, 120.0))
        .build(&event_loop)
        .expect("failed to create spike window");

    eprintln!(
        "SPIKE window id: {:?} — press a key now (or `xdotool key --window <id> <key>`)",
        window.id()
    );

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        event: key_event, ..
                    },
                ..
            } => {
                if key_event.state != tao::event::ElementState::Pressed {
                    return;
                }
                let raw_code: Option<u32> = match key_event.physical_key {
                    KeyCode::Unidentified(NativeKeyCode::Gtk(code)) => Some(code as u32),
                    KeyCode::Unidentified(other_native) => {
                        eprintln!(
                            "SPIKE RESULT: Unidentified but non-Gtk NativeKeyCode: {other_native:?}"
                        );
                        None
                    }
                    named => {
                        // to_scancode() converts a named KeyCode back to the
                        // platform raw scancode via tao's platform layer.
                        let scancode = named.to_scancode();
                        eprintln!(
                            "SPIKE RESULT: matched a named KeyCode::{named:?} — \
                             to_scancode() = {scancode:?}"
                        );
                        scancode
                    }
                };

                if let Some(code) = raw_code {
                    match validate_x11_keycode(code) {
                        Ok(keycode) => {
                            println!("SPIKE RESULT: captured raw keycode {keycode} (valid 8-255 range, ready for HotkeyCmd::SetKeycode)");
                        }
                        Err(e) => {
                            println!("SPIKE RESULT: captured raw keycode {code}, REJECTED: {e}");
                        }
                    }
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}
