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

- [x] ADR filed with the decision + rationale + considered alternatives — **already shipped as `CONSCIOUSNESS/architectural-decisions/0005-unify-tray-implementations.md`** (filed 2026-07-17, per TASK-VTT103, this exact task). Status: Accepted — alternative (b), keep the split and mirror parity, with (a) unify reopened only if a time-boxed hotkey-dialog spike succeeds. Covers exactly what this card asks: considered alternatives (a) unify-on-portable, (b) keep-split-mirror, (c) unify-with-fallback, each with pros/cons; a Recommendation section; Consequences for each path.
- [x] If unify: migration plan with Linux regression checks — N/A, vacuously satisfied. ADR-0005 decided AGAINST unifying (alternative (b)); its own "Migration plan sketch" section is explicitly headed "only if (a) is approved after the spike" and (a) was not approved. A migration plan would only be required if unification were chosen.
- Story: STORY-VTT013 · Directive: DIRECT-VTT002

## Shipped-check evidence, 2026-09-04

Found while resuming task selection: this task's own acceptance criteria are
already met by prior work. `CONSCIOUSNESS/architectural-decisions/0005-unify-tray-implementations.md`'s
own Status line reads "Filed per TASK-VTT103 (STORY-VTT013, DIRECT-VTT002)" —
this task's own detail card cites this exact ADR as its output. (Checked
`git log --oneline -- CONSCIOUSNESS/architectural-decisions/0005-...md`: it
landed via a later index/migration-repair commit, not a dedicated
task-closing commit, so the commit trail alone doesn't show the link — the
ADR's own text is the actual evidence here.) Verified the ADR's content
directly — Context, Decision, three Considered Alternatives with pros/cons
each, Consequences, and a Recommendation section are all present, matching
"ADR filed with the decision + rationale + considered alternatives"
precisely. Closing with this citation rather than re-doing already-completed
work.
