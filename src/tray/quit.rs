// Shared quit helper for both tray implementations (portable + GTK), so the
// platforms cannot drift (TASK-VTT122).
//
// The systemd unit uses `Restart=always`, so a plain `exit(0)` under systemd
// would be treated as a restartable exit and vtt would come straight back —
// an intentional Quit would loop forever. When systemd manages us, stop the
// unit instead; systemd records the stop as intentional and does not restart.

/// Decide whether Quit must go through `systemctl --user stop` rather than a
/// plain process exit. systemd sets `INVOCATION_ID` for every process it
/// starts; an empty value is treated the same as unset.
fn should_stop_unit(invocation_id: Option<&str>) -> bool {
    invocation_id.is_some_and(|id| !id.is_empty())
}

/// Quit the application from a tray menu.
///
/// Under systemd: launch `systemctl --user stop vtt` detached, then exit.
/// The detached stop lands within RestartSec, so even if systemd briefly
/// schedules a restart after our exit(0), the stop cancels it and the unit
/// stays inactive. Outside systemd (foreground run, macOS, Windows): plain
/// exit, same as before.
pub fn quit() -> ! {
    if should_stop_unit(std::env::var("INVOCATION_ID").ok().as_deref()) {
        crate::vtt_log!("Quit requested (systemd) — stopping vtt unit");
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "stop", "vtt"])
            .spawn();
    } else {
        crate::vtt_log!("Quit requested — exiting process");
    }
    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stops_unit_when_invocation_id_set() {
        assert!(should_stop_unit(Some("b0f24d1a2a6c4f5f9d4b7a1c3e5f7a9b")));
    }

    #[test]
    fn plain_exit_when_invocation_id_unset() {
        assert!(!should_stop_unit(None));
    }

    #[test]
    fn plain_exit_when_invocation_id_empty() {
        assert!(!should_stop_unit(Some("")));
    }
}
