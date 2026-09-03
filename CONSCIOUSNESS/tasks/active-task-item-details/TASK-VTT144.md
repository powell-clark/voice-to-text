# TASK-VTT144: Correction dictionary editable from the tray

## Context

Operator p00 steering 2026-08-15: common mispronounced phrases. corrections.rs +
settings.rs shipped (TASK-VTT118) but src/tray/ has zero correction handling —
grep 'correction' src/tray/ returns nothing — so adding a misheard->correct pair
requires hand-editing settings.conf. Add a tray editor beside the existing
initial_prompt textarea.

## Measured starting state, 2026-09-03

- `src/corrections.rs:32` — `format_pairs(&[(String, String)]) -> String` renders
  `misheard => correct`, one per line. Carries `#[allow(dead_code)]`.
- `src/corrections.rs:47` — `parse_pairs(&str) -> Vec<(String, String)>` is its
  inverse, dropping blank lines and half-typed rows. Also `#[allow(dead_code)]`.
- Both were written in TASK-VTT118 for a dialog that was never built. The round
  trip is already unit-tested; nothing about the data model needs designing.
- `src/tray/linux.rs:590` — `show_prompt_dialog` already edits voice prefix,
  initial prompt, newline toggle and newline type in one 500x340 non-resizable
  window, and its Save handler writes the whole `Settings` struct.
- `src/settings.rs` — `corrections: Vec<(String, String)>` is already parsed,
  saved and round-trip tested.

So this task is wiring, not construction: one more section in an existing dialog,
using two functions that already exist and are already tested.

## Acceptance criteria

- [x] The tray's "Customize Transcription Settings" dialog shows a corrections
      editor: a scrollable textarea, one `misheard => correct` pair per line,
      pre-filled from the saved corrections
- [x] Saving parses the textarea with `corrections::parse_pairs` and persists the
      result, so a pair added in the dialog survives a restart
- [x] A half-typed row (no `=>`, empty left side, or empty right side) is dropped
      rather than saved — a rule that eats a word can never be created by
      accident
- [x] Clearing the textarea and saving leaves zero corrections rather than
      leaving the previous list in place
- [x] Reset Default clears the corrections textarea alongside the other fields
- [ ] DEFERRED (operator gate) — the dialog still fits its content after the new
      section; needs eyes on a rebuilt binary, see the note below
- [x] `format_pairs` and `parse_pairs` lose their `#[allow(dead_code)]`
      attributes, because they are now called
- [x] `cargo test --workspace` passes; `cargo clippy --workspace --all-targets --
      -D warnings` is clean

## Evidence

```
cargo test --workspace: 154 passed; 0 failed; 1 ignored
cargo clippy --workspace --all-targets -- -D warnings: clean
cargo build: clean
```

Two new tests cover what the editor newly makes reachable — the round trip
itself was already tested seven ways by TASK-VTT118, so this adds only the cases
the dialog introduces:

- `emptying_the_editor_removes_every_correction` — the save path assigns
  `parse_pairs` unconditionally, so an emptied box must mean "remove them all"
  rather than "no change"; whitespace-only counts as empty
- `a_wrong_arrow_is_dropped_not_guessed` — `->` is the plausible typo for `=>`;
  it must not become a rule and must not take the valid line beside it down

Removing the two `#[allow(dead_code)]` attributes is itself a check: the build
would now fail on an unused-function warning under `-D warnings` if the wiring
were not actually reached.

`buffer_text` was extracted while wiring the second multi-line field, and the
initial-prompt save now uses it too — the same four lines had been inlined.

### The one deferred criterion

The dialog was `set_resizable(false)` at a fixed 500x340 holding five controls;
a sixth clips the button row, which is failure mode 1 in the pre-mortem. It is
now 520x560 and resizable. Section heights sum to roughly 545px — prefix 55,
prompt 110, corrections 155, newline block 110, buttons 35, border 40, spacers
40 — so 560 fits with the resize handle absorbing font-scale variance.

That is construction, not verification. GTK is not unit-testable here and this
repo's story scope calls widget tests brittle and low-ROI, so the check is a
human opening the dialog on a rebuilt binary. It shares a gate with TASK-VTT150
(Archive dictation as training-grade audio): both need
`bash scripts/release-local.sh --install` and a look.

## Test Strategy

The GTK dialog itself is not unit-testable (no display in CI, and the story's
scope note calls widget tests brittle and low-ROI). Test the pure boundary
instead: the `format_pairs`/`parse_pairs` round trip already has coverage, so add
the cases this dialog makes reachable — an emptied textarea, a half-typed row,
and a round trip through `Settings::save`/`load` carrying corrections edited as
text. Verify the widget behaviour by building and opening the dialog.

## Files

- Modify: `src/tray/linux.rs` (the new dialog section and its save/reset wiring),
  `src/corrections.rs` (drop the two `#[allow(dead_code)]`)
- Possibly modify: `src/settings.rs` (only if a corrections-specific test belongs
  there rather than in corrections.rs)

## Pre-mortem

### Failure modes

- The dialog is `set_resizable(false)` at 500x340 and already holds five
  controls; adding a textarea silently clips the Save button, making the whole
  settings dialog unusable rather than just the new section. Mitigation: grow the
  default size and allow resizing, and confirm by opening it.
- A user clears the textarea meaning "remove all my corrections" and the save
  path treats empty as "no change", stranding rules they believe they deleted.
  Mitigation: an explicit acceptance criterion for the empty case.
- `parse_pairs` drops malformed rows silently, so a user who typos `->` instead
  of `=>` sees their rule vanish on save with no explanation. Mitigation: state
  the required format in the label above the textarea, in the dialog, where they
  are typing.

### Weak assumptions

- That corrections are few enough to edit as free text. True today (the operator
  has a handful), and the format is line-oriented so it degrades gracefully; a
  per-row widget list is only worth building if the list grows past a screenful.
- That no other session is mid-edit in `src/tray/linux.rs` — checked with git
  status before starting, and this repo has had one concurrent-edit collision
  already today.

## Dependencies

- Directive: DIRECT-VTT002
- Story: STORY-VTT019
- Features: FEAT-VTT037
