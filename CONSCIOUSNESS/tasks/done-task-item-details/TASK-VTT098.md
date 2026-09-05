# TASK-VTT098: Windows tray Logs submenu (parity with Linux FEAT-VTT004)

The Linux GTK tray has a Logs submenu showing the last N log entries on hover. The
portable (Windows/macOS) tray has no Logs submenu. Add one that reads the recent
lines from the daily log file.

- [x] Portable tray has a Logs submenu listing recent entries — `src/tray/portable.rs`
      `refresh_logs_submenu`/`set_logs_items`, sharing `logging::list_log_filenames`/
      `format_log_label` with the Linux tray (one implementation, not a second copy)
- [x] Opening it surfaces today's log — clicking an entry opens the file via the
      platform default handler (`open_path`: macOS `open`, Windows `cmd start`),
      matching the Linux tray's actual click-opens-the-file behaviour
- Story: STORY-VTT013 · Directive: DIRECT-VTT004 · Parity §2

## Verification (2026-09-05)
- Commit `8172f58` (already on `main`) implements both criteria.
- Local: `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean,
  `cargo test` — 209 passed, 0 failed.
- CI run 33940167639 (this exact commit) — all 5 jobs green: ubuntu
  fmt+clippy+test+build, macOS arm64 build, Windows x86_64-msvc build, Windows
  ARM64 build, cargo-audit.
