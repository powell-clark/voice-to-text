# Generated per-platform spec (ADR-0008)

**Generated file — do not hand-edit.** Re-run `scripts/generate-platform-spec.sh`
after editing a card's Cross-platform acceptance criteria section instead.

Aggregated from each maintained card's own "Cross-platform acceptance
criteria (DIRECT-VTT005 parity spec)" section — nothing here is
hand-written, so it cannot drift from what the cards say. Cards without
that section are not listed (see TASK-VTT166 for full migration; the
richer, still-authoritative hand-maintained view is
`docs/PLATFORM-PARITY.md`).

Generated: 2026-09-04 21:36 UTC

---

## FEAT-VTT004: Linux system tray with GTK3 and AppIndicator

Anchored to `CONSCIOUSNESS/artifacts/PARITY-MATRIX.md` (capability 5 — system tray/menu). The tray capability spans this card (Linux/GTK) and the portable tray (`src/tray/portable.rs`) used on macOS + Windows.

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates; the statuses below are believed current as of this card's last edit but have not been freshly re-verified under this field.

**🐧 Linux — ✅ works** (this card)
- [x] GTK3 + libappindicator tray: status, model picker, Logs submenu, hotkey picker, quit — `src/tray/linux.rs`

**🍎 macOS — 🟡 partial**
- [ ] `tray/portable.rs` (tray-icon + muda) shows status, model picker, quit — compiles and ships, but a menu-bar item needs NSApplication/`.app` activation; untested without the `.app` (FEAT-VTT029)
- [ ] No Logs submenu and no hotkey picker (parity gap vs Linux)

**🪟 Windows — 🟡 partial**
- [x] `tray/portable.rs` shows status, model picker, logging + autostart toggles, quit; Win32 message pump drives it — `src/main.rs:399-419` (TASK-VTT091)
- [ ] Logs submenu missing (parity gap vs Linux) — TASK-VTT098
- [ ] No hotkey picker (config-file only)


---

## FEAT-VTT005: Text injection into focused application (Linux via XTest)

Anchored to `CONSCIOUSNESS/artifacts/PARITY-MATRIX.md` (capability 3 — text injection).

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates; the statuses below are believed current as of this card's last edit but have not been freshly re-verified under this field.

**🐧 Linux — ✅ works**
- [x] Text types at the cursor via `xdotool type` (primary) with `enigo`/x11rb fallback — `src/typing.rs:41-152`
- [x] £/é/ñ/emoji type correctly; no duplicate characters in X11/XWayland

**🍎 macOS — 🟡 partial**
- [ ] Text types at the cursor via `enigo` once Accessibility permission is granted — code path exists (`src/typing.rs` non-Windows branch) but is untuned char-by-char and `xdotool` is always absent
- [ ] Accessibility-permission prompt presented to the user — BLOCKED on the `.app` bundle (FEAT-VTT029); a raw binary cannot register the prompt
- [ ] Verified typing into Safari, Notes, Terminal on a shipped build

**🪟 Windows — ✅ works**
- [x] Text types at the cursor via batched `enigo.text()` SendInput per newline segment — `src/typing.rs:69-105`
- [x] No dropped or reordered characters (regression fixed in TASK-VTT092)


---

## FEAT-VTT012: Clipboard paste via xclip subprocess

Anchored to `CONSCIOUSNESS/artifacts/PARITY-MATRIX.md` (capability 4 — clipboard paste fallback).

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates; the statuses below are believed current as of this card's last edit but have not been freshly re-verified under this field.

**🐧 Linux — ✅ works**
- [x] Transcription written to the clipboard via `xclip`/`arboard`; `xclip` in `debian/control` Depends — `src/typing.rs:187-252`, `debian/control:36` (TASK-VTT110)
- [x] Manual Ctrl+V pastes correctly; no crash across 50+ writes

**🍎 macOS — 🟡 partial (TASK-VTT114)**
- [x] Transcription reaches the clipboard via `arboard` (manual Cmd+V works) — `src/typing.rs:187-216`
- [ ] Auto-paste keystroke uses **Cmd+V** on macOS — currently sends Ctrl+V (no-op); tracked as TASK-VTT114

**🪟 Windows — ✅ works**
- [x] Transcription written to the clipboard via `arboard` for manual Ctrl+V (no auto-paste, parity with the Linux fallback) — `src/typing.rs:99-103` (TASK-VTT099)


---

## FEAT-VTT013: X11 key auto-repeat filtering

Anchored to `CONSCIOUSNESS/artifacts/PARITY-MATRIX.md` (capability 6 — hotkey auto-repeat handling).

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates; the statuses below are believed current as of this card's last edit but have not been freshly re-verified under this field.

**🐧 Linux — ✅ works** (this card)
- [x] Held hotkey = exactly one recording — X11 `XkbSetDetectableAutoRepeat` + manual KeyPress/KeyRelease pairing — `src/hotkey/linux.rs`

**🍎 macOS — ✅ works**
- [x] `rdev` listener filters auto-repeat; held key = one recording — `src/hotkey/portable.rs` (TASK-VTT100)
- [ ] Requires Input-Monitoring permission (no prompt flow without `.app`, FEAT-VTT029)

**🪟 Windows — ✅ works**
- [x] `rdev` listener filters auto-repeat; live keycode remap via SetKeycode — `src/hotkey/portable.rs` (TASK-VTT100)


---

## FEAT-VTT026: Automatic GGML model download from HuggingFace

Anchored to `CONSCIOUSNESS/artifacts/PARITY-MATRIX.md` (capability 8 — model download). Behaviour is identical on all platforms (one `src/models.rs` path); the only gap is cross-cutting.

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates; the statuses below are believed current as of this card's last edit but have not been freshly re-verified under this field.

**🐧 Linux / 🍎 macOS / 🪟 Windows — ✅ works (uniform)**
- [x] Selecting an uncached model downloads from HuggingFace over HTTPS (rustls) with tray progress and atomic `.tmp`→rename — `src/models.rs:135-193`
- [x] Cache path resolves per-OS via `dirs::cache_dir`: `~/.cache` (Linux), `~/Library/Caches` (macOS), `%LOCALAPPDATA%` (Windows)

**Cross-cutting gap (all platforms) → TASK-VTT112**
- [ ] In-app download verifies the bytes against a stored expected SHA-256 — NOT implemented (only the Linux `debian/postinst` pre-download hard-verifies, TASK-VTT110)


