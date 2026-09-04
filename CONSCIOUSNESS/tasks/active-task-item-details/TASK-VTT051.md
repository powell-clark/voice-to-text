# TASK-VTT051: GTK settings dialog

## Acceptance Criteria
1. [x] Tray menu has a "Settings…" item that opens a GTK dialog — the existing
   "Customize Transcription Settings..." item/dialog is renamed and extended
   (it already consolidated voice prefix, initial prompt, corrections,
   newline, archive, and denoise into one dialog; this task adds the rest)
2. [x] Dialog exposes: hotkey (button into the existing capture dialog),
   language (radio), model (radio, from the same catalogue the tray uses),
   initial_prompt (pre-existing), input device (combo box, basic list+save —
   the fuller hot-plug/live-rebuild UX stays TASK-VTT129's scope)
   — [ ] **VAD toggle: DEFERRED.** No `vad` field exists in `Settings` yet —
   TASK-VTT050 (Silero VAD integration) is unclaimed pending an operator
   decision on which crate to depend on (see that card's crate research).
   Nothing to toggle exists until it lands; adding a dead checkbox would be
   worse than omitting it. Add this control when TASK-VTT050 ships.
3. [x] Changes save to `settings.conf` on the dialog's "Save" button (matches
   the existing dialog's convention for its other fields) and most take
   effect immediately; input device explicitly notes "restart required" in
   its own label, matching the existing pattern for denoise/archive
4. [x] (best-effort, not empirically click-tested — see Verification below)
   Standard GTK3 widgets only (Entry, TextView, RadioButton, CheckButton,
   ComboBoxText, Button) — all Tab-focusable by default; nothing sets
   `can_focus(false)` on any new widget

## Verification, 2026-09-04

- `cargo build --release`: clean, 0 warnings
- `cargo clippy --all-targets -- -D warnings`: clean
- `cargo test --workspace`: 182 passed, 0 failed, 1 ignored (pre-existing,
  unrelated — downloads a model)
- **Live visual verification NOT performed.** This machine's `:1` display is
  the operator's actual live desktop, with a real `vtt.service` already
  running (production settings, real hotkey) and other visible work in
  progress. Launched an isolated instance (separate `XDG_DATA_HOME`, its own
  temp settings copy, does not touch the real service) to screenshot the new
  dialog, but the resulting screenshot captured the operator's live screen —
  another session's private work was visible in it. Deleted the screenshot,
  killed the isolated test instance immediately, and stopped rather than
  continue driving the operator's live desktop for a cosmetic check. The
  production `vtt.service` (pid unchanged throughout) was never touched.
  **A human should do a 10-second check**: open the tray, click "Settings...",
  confirm it renders and Tab moves through the fields in a sane order.
