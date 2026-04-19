# TASK-VTT055: Release v2.0.5 — typing £/é fix + Logs submenu fix

## Context
Two runtime bugs are fixed on main (not yet tagged or released):
- **`src/typing.rs`** — previously split text at first non-ASCII char and pasted the remainder via Ctrl+V. In Claude Code TUI and many other apps, plain Ctrl+V is not bound to paste, so everything from the first £/é/— onward was silently lost. Fix: type every char via `Key::Unicode(c)`; clipboard paste is now a narrow fallback only for chars enigo actively fails on.
- **`src/tray/linux.rs`** — Logs submenu rebuilt on `logs_item.connect_activate`, which fires after GTK reveals the submenu, so first open always saw stale content. Fix: rebuild on the parent `menu.connect_show` signal instead. Also added `(log dir unreadable: …)` fallback item so silent failures become visible.

Both compiled locally on branch main, binary running since 22:55 BST.

## Acceptance Criteria
1. `debian/changelog` has a v2.0.5 entry describing both fixes with commit hashes
2. `Cargo.toml` version bumped to 2.0.5
3. `scripts/release-local.sh` (or release-manager agent) builds the .deb locally, installs it, typing a transcription with £ types the full text in Claude Code TUI
4. The pbuilder hard gate passes for both jammy and noble targets — no dput until `pbuilder --build` exits 0 in a clean chroot (see commits 56ad357 + b10cee3)
5. `dput powellclark-voice-to-text ../voice-to-text_2.0.5_source.changes` uploads cleanly; Launchpad web UI shows Published for Noble within 30 minutes
6. `sudo apt update && sudo apt install --only-upgrade voice-to-text` on this machine upgrades to 2.0.5 and the About dialog shows "Voice to Text 2.0.5"
7. `git tag v2.0.5` pushed to origin

## Technical Approach
Hand off to the release-manager agent. It already knows the pbuilder gate and changelog format. Commands to the agent: `/agent release-manager` → "production release 2.0.5, typing £/é fix and tray logs submenu fix, no breaking changes".

## Test Strategy
Post-install manual smoke test:
1. Dictate "£100 an hour é naïve — done" into Claude Code TUI → full string appears
2. Dictate same into Firefox URL bar → full string appears
3. Open tray → Logs → first hover shows today's file without needing a second open
4. Check `~/.local/share/voice-to-text/vtt-$(date +%Y-%m-%d).log` for "Typing completed (N chars typed directly)" log line (new format) and no fallback-paste messages for normal text

## Files
- `Cargo.toml` — version bump
- `debian/changelog` — new stanza
- No source changes beyond what is already on main (typing.rs + tray/linux.rs)
