# TASK-VTT167: Package ct2-daemon into the Linux .deb install

## Context

Filed alongside TASK-VTT054. `ct2_client::resolve_daemon_script()` only found
`transcribe_daemon.py` in a self-built checkout (`CARGO_MANIFEST_DIR`-relative)
or a sibling of the running executable — no `.deb`/`.msi` install step placed
`ct2-daemon/` (plus its faster-whisper/ctranslate2 Python deps) alongside the
installed binary. A real end-user install with `backend=ct2` selected would
fail to locate the daemon and silently fall back to native.

Narrowed to the Linux `.deb` slice (2026-09-05): this repo's only working
installer-time file-placement mechanism is `debian/rules`, which I can build
and inspect locally without touching the live system (`dpkg-deb --contents`,
no `sudo`/install needed). The Windows `.msi` and macOS `.app` equivalents
need installer authoring I cannot verify end-to-end from this Linux machine —
split off as TASK-VTT171.

## Acceptance criteria

- [x] `debian/rules`' `override_dh_auto_install` installs
      `ct2-daemon/transcribe_daemon.py` and `ct2-daemon/requirements.txt` to
      `/usr/share/voice-to-text/ct2-daemon/` — the same FHS root
      `src/models.rs::system_cache()` already uses for the downloaded model
      cache, not `/usr/bin` (executables only).
- [x] `resolve_daemon_script()` gains a fourth, Linux-only fallback
      (`system_daemon_script()`, mirroring `models.rs`'s
      `#[cfg(target_os = "linux")]` / `#[cfg(not(target_os = "linux"))]` split)
      that checks that path, so a real `.deb` install (not just a dev
      checkout) can find the daemon.
- [x] No new `Depends` on Python/faster-whisper/ctranslate2 in
      `debian/control` and no auto-`pip install` in `postinst` — the package
      still requires no Python by default (backend defaults to native
      whisper-rs); a `backend=ct2` user installs the daemon's own deps
      manually (`pip install -r
      /usr/share/voice-to-text/ct2-daemon/requirements.txt`).
- [x] Verified via a local, non-installing `.deb` build
      (`scripts/release-local.sh`, no `--install`) that the package actually
      contains the two files at the exact path above with byte-identical
      content to the source tree, and contains none of `ct2-daemon`'s test
      artefacts (`test_transcribe_daemon.py`, `__pycache__`,
      `.pytest_cache`) — `dpkg-deb --contents` / `--fsys-tarfile` +
      `sha256sum`, no `sudo`, no system install.
- [ ] DEFERRED (operator gate, same class as TASK-VTT108/TASK-VTT139): a real
      `apt install`/`dpkg -i` on a live Ubuntu machine, then selecting
      `backend=ct2` in the tray and confirming the daemon actually spawns
      (not just that the file is present) — needs a human with `sudo`, which
      this autonomous session does not use for a live system install.
- Windows `.msi` / macOS `.app` equivalents: split to TASK-VTT171, not part
  of this card's acceptance.

## Dependencies

- Directive: DIRECT-VTT002
- Story: STORY-VTT017
- Features: FEAT-VTT034

## Pre-mortem

### Failure modes

- A stale `packaging/linux/vtt.prebuilt` masked the fact that the .deb ships
  a committed binary, not a fresh build — hit during this task (prebuilt was
  from commit `327b756`'s era, source had moved to `1cb2d03`) and fixed by
  refreshing it from a real `cargo build --release`, same remedy the
  script's own staleness gate prints.
- `debuild`'s Cargo.lock v3 downgrade (Launchpad's old cargo can't parse v4)
  is a deliberate side effect of `scripts/release-local.sh` — reverted back
  to `version = 4` before committing so it doesn't leak into an unrelated
  diff.

### Weak assumptions

- Assumes a `backend=ct2` user is comfortable running one manual `pip
  install` — acceptable per the card's own explicit non-goal (no Python
  required by default), but not verified against an actual first-time
  `backend=ct2` user's experience.
