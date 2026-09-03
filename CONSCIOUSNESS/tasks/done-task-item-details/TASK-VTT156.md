# TASK-VTT156: Log the resolved archiving state at startup

## Context

An archive-enabled binary that writes nothing looks exactly like a working one.

On 2026-09-03 a process holding a deleted inode answered the hotkey for eighty
minutes. Transcription worked perfectly. `archive=1` was set in the right file.
`/usr/bin/vtt-linux` was the correct build with the feature in it. And no
archive entry appeared, with nothing in the log saying why. Emmanuel had no
signal at all — he learned about it because two agent sessions happened to be
watching the filesystem, which is not a mechanism anyone should rely on.

The log already announces the model, the hotkey, the audio device and the
capture rate. Archiving — the one feature that writes the operator's voice to
disk, and the one whose absence is silent — announced nothing.

Suggested by the epc-voice seat, which measured the eighty-minute gap
independently from the consumer side.

## Acceptance criteria

- [x] Every startup logs whether archiving is on or off
- [x] When on, the line names the RESOLVED archive directory, not the raw
      setting — an empty `archive_dir` means "the default" and is exactly the
      value a reader cannot interpret
- [x] When on, the line names the file cap, with 0 rendered as "unbounded"
      rather than as a bare zero
- [x] When off, the line says how to turn it on, so the log answers the question
      rather than only reporting state
- [x] Rumble filtering is reported on the same pass, for the same reason
- [x] `cargo test --workspace` passes; clippy and fmt clean

## Evidence

```
cargo test --workspace: 173 passed; 0 failed; 1 ignored
cargo clippy --workspace --all-targets -- -D warnings: clean
```

The lines, at the point where the settings are read into the audio layer:

```
Archiving ON -> /home/powell-clark/.local/share/voice-to-text/archive (cap 5000)
Rumble filtering off
```

or, unconfigured:

```
Archiving off (set archive=1 in settings.conf to enable)
```

Had this shipped with TASK-VTT150, the eighty-minute gap would have been one
glance at the log at 08:51 rather than a filesystem investigation at 10:12. The
absent line was itself the diagnostic.

## Why this is not just a log line

Three of today's failures shared one shape: something reported success while
old code ran. TASK-VTT152 catches it at build time, TASK-VTT153 at diagnosis
time. This catches it at the only moment the operator is actually looking — the
startup banner he already reads to confirm the app came up.

## Dependencies

- Story: STORY-VTT020
- Directive: DIRECT-VTT002
