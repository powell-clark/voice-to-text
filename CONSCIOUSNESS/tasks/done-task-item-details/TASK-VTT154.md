# TASK-VTT154: Expose archiving and denoise in the settings dialog

## Context

`archive`, `archive_dir`, `archive_max_files` and `denoise` are settings-file
only. On 2026-09-03 that cost real time twice: the README pointed at
`~/.config/voice-to-text/settings.conf`, which the app has not read since a
pre-2.0 version, and editing it silently did nothing — no error, no warning,
just a setting that never took. The live file is
`~/.local/share/voice-to-text/settings.conf`, because `main.rs` names the
variable `config_dir` and assigns it `dirs::data_local_dir()`.

A checkbox in the dialog that already edits voice prefix, initial prompt,
corrections and newline behaviour removes that entire class of error. It also
makes the privacy-sensitive one visible: archiving writes your voice and a
transcript of everything you dictate to disk, and burying that in a file most
users never open is the wrong place for it.

Narrower than TASK-VTT051 (GTK settings dialog), whose criteria predate both
settings and list a VAD toggle for a feature that does not exist yet.

## Acceptance criteria

- [x] The settings dialog shows an archiving checkbox reflecting the saved
      `archive` value, with text stating plainly that it saves your voice and
      your words to disk
- [x] The archive location is shown, resolved (not the raw empty-means-default
      setting), so the user can see where recordings actually go
- [x] The dialog shows a denoise checkbox reflecting the saved `denoise` value
- [x] Saving persists both to `settings.conf`, and a saved value survives a
      reload
- [x] The dialog states that these two take effect on restart, because the
      audio layer reads them once at startup — no silent no-op
- [x] Reset Default returns both to their shipped defaults (archiving off,
      denoise off) alongside the other fields
- [x] `cargo test --workspace` passes; clippy and fmt clean

## Deliberately not doing live-apply

`Audio` caches both settings in atomics set once at startup, and `TrayState`
holds no `Audio` handle. Applying live means threading an `Arc<Audio>` through
`Tray::new`, whose signature is shared with `src/tray/portable.rs` — a change
across three files to save a restart on two settings that are set-once in
practice.

Saying "restart to apply" in the dialog is honest and costs the user one
command. Live-apply is worth doing when the dialog grows a setting that is
genuinely toggled often; filed as a follow-up rather than smuggled in here.

## Files

- Modify: `src/tray/linux.rs` (the two controls and their save/reset wiring)

## Pre-mortem

### Failure modes

- The archiving checkbox reads as a feature to try rather than a decision to
  make, and someone enables it without understanding that their voice is being
  written to disk. Mitigation: the label says so in the dialog, in the same
  plain terms as the README section, rather than deferring to documentation the
  user has not read.
- The dialog claims a restart is needed when it is not, or vice versa, and the
  user distrusts the whole dialog. Mitigation: it is needed, because the atomics
  are set once in `main`; the text says exactly that and nothing more.

### Weak assumptions

- That showing the resolved archive path is more useful than showing the raw
  setting. It is, because empty-means-default is exactly the case a user cannot
  interpret, and the default is the one that surprised everyone this morning.

## Dependencies

- Story: STORY-VTT019
- Directive: DIRECT-VTT002


## Evidence

```
cargo test --workspace: 173 passed; 0 failed; 1 ignored   (172 before)
cargo clippy --workspace --all-targets -- -D warnings: clean
cargo build: clean
```

`settings::tests::both_dialog_toggles_survive_a_round_trip` checks both
directions. On is the obvious case; off matters more, because both settings
default off and a one-way test would pass even if saving did nothing at all —
which is the shape of the bug this task exists to remove.

The dialog now carries, after the newline controls:

- a checkbox reading "Archive my recordings — saves my voice AND what I said,
  to disk", stated in the dialog rather than deferred to the README
- the resolved destination beneath it, so empty-means-default is never shown to
  a user as an empty box
- a rumble-filter checkbox
- one line naming the restart command, because the audio layer reads both once
  at startup

Reset Default returns both to off. That mattered enough to test by hand in the
wiring: a Reset that left archiving on would silently keep writing a user's
voice to disk after they asked for defaults.

The window grew 560 to 680 for the four new rows, and is resizable since
TASK-VTT144.

## Follow-up

Live-apply, so the two take effect without a restart, needs an `Arc<Audio>`
threaded through `Tray::new` — a signature shared with `src/tray/portable.rs`.
Worth doing when the dialog grows a setting that is genuinely toggled often.
