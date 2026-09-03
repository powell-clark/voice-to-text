# TASK-VTT155: Rename config_dir to data_dir throughout

## Context

`src/main.rs` assigned `dirs::data_local_dir()` to a variable named
`config_dir`, and the name propagated into `settings.rs`, `archive.rs` and both
tray backends. On 2026-09-03 that single misnomer produced three separate
wrong-path errors, none of which failed loudly:

1. The README documented `~/.config/voice-to-text/settings.conf` three times,
   including the command telling a user how to delete their archive. The app
   reads `~/.local/share/voice-to-text`.
2. The epc-voice session read this code, saw `config_dir.join("archive")`,
   concluded the archive root was under `~/.config`, told this session its
   correction was wrong, and committed the wrong path into TASK-EV035 (Build
   the dictation corpus importer). Its importer would have found nothing and
   reported zero imported minutes rather than an error.
3. A stale `~/.config/voice-to-text/settings.conf` from October 2025 still
   exists on the machine, so `archive=1` written there was accepted by the
   filesystem and never read. The operator was sent to it twice.

The name was the common cause. Two readers who both understood the code
independently derived the wrong path from it, which is the definition of a
misleading identifier rather than a careless reader.

## Acceptance criteria

- [x] No `config_dir` identifier remains in `src/`, except `dirs::config_dir()`
      where the real config directory is genuinely meant
- [x] `dirs::config_dir()` in `--doctor` is untouched — it reports the stale
      `~/.config` file and must keep pointing there
- [x] `Settings`'s doc comment states which directory it means and why the name
      changed, so the next reader does not re-derive the old assumption
- [x] `archive::resolve_archive_dir` says the same at its point of use
- [x] `cargo test --workspace` passes unchanged; clippy and fmt clean
- [x] Behaviour is provably identical: `--doctor` resolves the same paths before
      and after

## Evidence

```
renamed: 45 identifiers across 5 files
  src/main.rs        23
  src/settings.rs    18
  src/archive.rs      2
  src/tray/linux.rs   1
  src/tray/portable.rs 1

cargo test --workspace: 173 passed; 0 failed; 1 ignored   (unchanged)
cargo clippy --workspace --all-targets -- -D warnings: clean
```

Two exceptions survive deliberately, and both were checked by hand rather than
left to the regex:

- `dirs::config_dir()` at `main.rs:1005` — the real API, used by `--doctor` to
  report the stale file. Renaming it would have made the doctor look in the
  data directory for a file that is only ever in the config directory, turning
  a working check into a silent no-op. Protected by substitution before the
  rename ran.
- `worker_config_dir` — a compound the word-boundary regex could not see,
  because `\b` does not sit between `worker_` and `config_dir`. Caught by the
  post-rename assertion failing rather than by inspection, which is the reason
  the assertion was there.

Behaviour identical, from a live run after the rename:

```
  [ok     ] settings        /home/powell-clark/.local/share/voice-to-text/settings.conf
  [PROBLEM] stale settings  /home/powell-clark/.config/voice-to-text/settings.conf exists and is NOT read
  [ok     ] archiving       on -> /home/powell-clark/.local/share/voice-to-text/archive
```

Same paths, same findings, same problem detected. A pure rename should be
invisible at runtime and this one is.

## Note on the first attempt

The first run of the rename script asserted and stopped before writing
anything, because `main.rs` still held two occurrences after the substitution.
That was the right outcome: a rename that silently left two identifiers behind
would have compiled if they happened to be unused, or produced a confusing
error if not. The assertion turned a half-finished rename into a stopped one.

## Out of scope

- The stale `~/.config/voice-to-text/settings.conf` on the operator's machine.
  `--doctor` reports it; deleting another program's leftovers uninvited is not
  this task's business.

## Dependencies

- Story: STORY-VTT018
- Directive: DIRECT-VTT002
