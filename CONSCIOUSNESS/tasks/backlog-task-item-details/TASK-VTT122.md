# TASK-VTT122: Service restart semantics — Restart=always + tray Quit stops the unit

## Context

Follow-up filed from the TASK-VTT121 diagnosis session (2026-07-10). The systemd
unit (`packaging/linux/vtt.service`) is `Restart=on-failure`, but both tray Quit
paths exit cleanly:

- Portable tray (macOS/Windows, also compiled on Linux): `src/tray/portable.rs:246`
  calls `std::process::exit(0)`.
- Linux GTK tray: `src/tray/linux.rs:238-240` Quit handler.

A clean exit is not a "failure", so systemd never restarts vtt after a crash-free
stop — but the flip side observed live was worse: when vtt died or was stopped,
nothing brought it back, reading to the operator as "service won't restart".

## Approach

1. `packaging/linux/vtt.service`: `Restart=on-failure` → `Restart=always` so any
   exit (crash, kill, OOM) brings vtt back.
2. Make intentional Quit actually stop the unit rather than fight `Restart=always`:
   when running under systemd (detect via `INVOCATION_ID` env var), tray Quit runs
   `systemctl --user stop vtt` (detached) instead of / before `exit(0)`. When not
   under systemd (foreground run, macOS, Windows), plain exit stays.
3. Apply to BOTH quit paths (portable tray + GTK tray) via one shared helper so
   the platforms cannot drift.

## Acceptance criteria

- [ ] `kill -9` of the vtt main process results in systemd restarting it within
      RestartSec (unit has `Restart=always`)
- [ ] Tray Quit under systemd stops the service and it stays stopped (no restart
      loop, `systemctl --user status vtt` shows inactive)
- [ ] Tray Quit when NOT under systemd (plain foreground run) still exits the
      process
- [ ] Both tray implementations (portable + GTK) route through the same quit
      helper
- [ ] cargo test / clippy / fmt green

## Dependencies

- Directive: DIRECT-VTT002 (Linux voice-to-text and shared core)
- Sibling: TASK-VTT121 (stream recovery) — same diagnosis session, independent code paths
