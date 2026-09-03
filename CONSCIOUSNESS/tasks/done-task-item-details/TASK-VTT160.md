# TASK-VTT160: Prune only the recordings we wrote

## Context

`collect_archived` matched any `.wav` one level under `archive_dir`, and
`prune_archive` deleted the oldest past `archive_max_files` along with any
`.json` beside it, then removed directories as they emptied.

`archive_dir` is operator-settable and its name invites exactly the wrong
reading. Set it to a folder that already holds audio — a typo, or a fair
interpretation of a setting called "archive dir" — and after the next dictation
the tool silently deletes the operator's own oldest recordings, plus any JSON
sharing their stem, plus the folders they lived in.

Data loss from a config typo, in the one feature whose entire purpose is keeping
audio. Found reviewing this session's own code rather than in use.

## Acceptance criteria

- [x] Pruning descends only into dated directories this code creates
- [x] Pruning matches only `vtt_<id>.wav` names this code writes
- [x] A wav in an unrelated subdirectory of `archive_dir` survives a prune
- [x] A wav sitting inside one of our own dated directories, but not named by
      us, also survives — the likelier accident of the two
- [x] The empty-directory tidy-up also skips directories we did not create
- [x] README and CHANGELOG state that the cap only removes our own files
- [x] `cargo test --workspace` passes; clippy and fmt clean

## Evidence

```
cargo test --workspace: 179 passed; 0 failed; 1 ignored   (176 before)
cargo clippy --workspace --all-targets -- -D warnings: clean
```

`prune_never_touches_files_it_did_not_write` builds the destructive scenario
directly: an `Albums/wedding-speech.wav` in an unrelated folder, two of our own
recordings over the cap, and a `someone-elses.wav` planted inside one of our own
dated directories. Pruning to a cap of 1 removes exactly one file — ours — and
all three foreign files and the unrelated directory survive.

Two narrower tests pin the predicates: `vtt_.wav` (empty id), `vtt_x.flac`,
`vtt_x.json` and `holiday-song.wav` are all rejected as ours; `2026-9-3`,
`20260903` and `Albums` are all rejected as dated directories.

## Why a name check rather than a manifest

A sidecar-driven manifest of what we wrote would be exact, and it would add a
second source of truth that can drift from the filesystem — a manifest lost to a
crash would strand every recording as unprunable, and one that disagreed with
disk would be worse than the name check. The names are ours, deterministic, and
already the pairing key the importer uses.

## Out of scope

- Refusing to archive into a non-empty directory at all. That would be safer
  still, but it breaks the legitimate case of pointing `archive_dir` at an
  existing backup location, and the guard above already makes the destructive
  outcome unrepresentable.

## Dependencies

- Story: STORY-VTT020
- Directive: DIRECT-VTT002
