# TASK-VTT152: Fail the build when the packaged binary is stale

## Context

On 2026-09-03 Emmanuel ran `bash scripts/release-local.sh --install` to pick up
the dictation archive. Four stages printed green, lintian ran, he typed his sudo
password, and the installed binary was still the one built on 19 August with no
archive code in it. Nothing in the output said so.

Three faults compounded, and only the first is this task's:

1. `debian/rules` does not compile. `override_dh_auto_build` checks only that
   `packaging/linux/vtt-linux.prebuilt` exists and is an x86_64 ELF;
   `override_dh_auto_clean` says "No cargo state to clean — binary is
   committed". So the `.deb` ships whatever that file holds, regardless of what
   is on main.
2. `apt install` refuses an unchanged version (`2.3.11` installed, `2.3.11`
   offered) and exits 0, so the script printed "Installed." over a no-op.
3. `vtt.service` carries `Restart=always`, so `pkill` respawned the old binary
   five seconds later and the singleton lock then blocked a direct run.

The prebuilt-binary design is deliberate and correct: Launchpad builds on Ubuntu
Noble, whose Cargo 1.75 cannot parse this crate's edition-2024 manifest, so the
binary is built here and committed. That constraint stays.

What is not defensible is the silence. `release-local.sh` runs
`cargo build --release --offline --locked` as a pre-flight, which proves the
SOURCE compiles and proves nothing about what gets PACKAGED. A session can ship
a two-week-old binary and see nothing but green.

## Acceptance criteria

- [x] `scripts/release-local.sh` exits non-zero, before `debuild`, when
      `packaging/linux/vtt-linux.prebuilt` is older than the newest commit
      touching `src/`, `Cargo.toml` or `Cargo.lock`
- [x] The error names both timestamps, the commit that made it stale, and the
      exact command that refreshes it
- [x] The gate passes silently-but-visibly when the prebuilt is current — one
      line confirming what the `.deb` will actually ship
- [x] Verified by setting the prebuilt's mtime to the real 19 August value and
      confirming the script refuses, then restoring it
- [x] `bash -n scripts/release-local.sh` is clean

## Evidence

Falsification run first, against the exact timestamps of the real failure:

```
$ touch -d "2026-08-19 03:37:00" packaging/linux/vtt-linux.prebuilt
$ bash scripts/release-local.sh
=== Building voice-to-text 2.3.11 .deb ===

[1/4] Vendoring cargo dependencies...
  vendor/ size: 961M

ERROR: packaging/linux/vtt-linux.prebuilt is older than the source it is supposed to build from.
  prebuilt:     2026-08-19 03:37:00
  newest src/:  2026-09-03 08:40:40  (780841f)

The .deb installs this file verbatim, so building now would ship stale code.
Refresh it:
  cargo build --release && cp target/release/vtt-linux packaging/linux/vtt-linux.prebuilt

$ echo $?
1
```

Exit code checked without a pipe, because `head` in the pipeline masks it. The
mtime was restored immediately afterwards and `git diff --stat` shows the file
unchanged — only its timestamp moved, never its content.

Current state passes:

```
[0/4] Prebuilt is current (built 2026-09-03 08:44); it is what the .deb ships.
```

## Why mtime rather than a content hash

A hash of `target/release/vtt-linux` against the prebuilt would be exact, but it
requires a release build to have happened, which is the expensive thing the gate
runs before. mtime against `git log -1 --format=%ct -- src/` needs no build and
catches the failure that actually occurred. It has one false negative — touching
the file without rebuilding — which no honest workflow produces, and one false
positive after a fresh clone, where git does not preserve mtimes; the fix in
that case is the same `cargo build --release` the message already prints.

## Out of scope

- The apt-refuses-same-version behaviour (fault 2). That is a version-bump
  discipline question, not a build gate; `dpkg -i` installs over an unchanged
  version and is the documented local path.
- Whether to keep the committed-binary packaging at all. It exists for a real
  Launchpad constraint and replacing it is a separate decision.

## Dependencies

- Story: STORY-VTT018
- Directive: DIRECT-VTT002
