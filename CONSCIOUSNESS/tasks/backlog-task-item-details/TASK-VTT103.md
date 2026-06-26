# TASK-VTT103: Evaluate unifying the Linux GTK tray onto the portable tray (ADR)

Today there are two tray implementations: `tray/linux.rs` (GTK3/AppIndicator, ~1000
lines) and `tray/portable.rs` (tray-icon/muda, Windows + macOS). Feature drift
between them is the root cause of several parity gaps (e.g. Logs submenu, icon
state). tray-icon supports Linux too (StatusNotifierItem), so one cross-platform
tray could replace both — a large maintainability win and permanent parity.

But switching Linux off the mature GTK tray is a one-way-door UI-mechanism change
(risk of regressing the working Linux experience, Wayland/AppIndicator quirks).
File an ADR weighing: unify-on-portable vs keep-split-and-mirror-features. Decide
before any code move.

- [ ] ADR filed with the decision + rationale + considered alternatives
- [ ] If unify: migration plan with Linux regression checks
- Story: STORY-VTT013 · Directive: DIRECT-VTT002
