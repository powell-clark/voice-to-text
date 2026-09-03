# TASK-VTT161: Resolve archive_dir predictably or refuse it

## Context

`resolve_archive_dir` expanded a leading `~/` and otherwise took the string
literally. `settings.conf` looks shell-ish, so people type shell-ish things, and
three plausible inputs produced surprising results in the setting that decides
where the operator's voice is written:

| typed | got |
|---|---|
| `~` | a directory literally named `~`, in the process's working directory |
| `$HOME/x` | a directory literally named `$HOME` — settings.conf is not shell-expanded |
| `voice-archive` | resolved against the service's cwd, which for a systemd user unit is not the shell directory it was typed in |

The third is the worst: recordings land somewhere the operator cannot find, or
the write fails and only the log says so.

Found reviewing this session's own code, alongside TASK-VTT160 (Prune only the
recordings we wrote) — the same class, an operator-settable path handled
loosely.

## Acceptance criteria

- [x] A bare `~` resolves to the home directory, not a directory named `~`
- [x] A relative path is refused rather than resolved against an unpredictable
      cwd, and the default is used instead
- [x] A refusal carries a reason naming what was wrong and what to type
- [x] An empty setting is `Default`, not `Rejected` — the common case must not
      look like an error
- [x] The startup line logs the refusal, so it is visible on every run
- [x] `--doctor` reports a rejected `archive_dir` as a PROBLEM naming the reason
      and the directory actually being written to
- [x] `cargo test --workspace` passes; clippy and fmt clean

## Evidence

```
cargo test --workspace: 182 passed; 0 failed; 1 ignored   (179 before)
cargo clippy --workspace --all-targets -- -D warnings: clean
```

Live `--doctor` runs against a temporary settings copy, so the operator's own
configuration was never touched:

Relative path — refused, with the reason and the fallback:

```
[PROBLEM] archiving  on, but archive_dir was IGNORED (archive_dir must be an
          absolute path or start with ~/ — "voice-archive" would resolve against
          the service's working directory, not yours) — writing to
          /tmp/.../voice-to-text/archive
```

Bare tilde — now home:

```
[ok] archiving  on -> /home/powell-clark
```

Empty setting — still ordinary:

```
[ok] archiving  on -> /tmp/.../voice-to-text/archive
```

## Design note: refuse loudly rather than guess

`resolve_archive_dir_checked` returns `Default`, `Configured` or
`Rejected { used, why }` rather than a bare `PathBuf`, so the two places a human
actually looks — the startup banner and `--doctor` — can say a setting did not
take. Silently falling back would have reproduced the failure this task exists
to remove: the app doing something reasonable somewhere the operator is not
looking.

Shell expansion was deliberately not added. Supporting `$HOME` invites the
expectation of full expansion — command substitution included — in a file the
app parses itself. An absolute path or `~/` covers the real cases; everything
else is refused with a sentence saying so.

`resolve_archive_dir` survives as a thin wrapper for callers that only need the
path, so the prune and write paths were untouched by this change.

## Dependencies

- Story: STORY-VTT020
- Directive: DIRECT-VTT002
