---
id: FEAT-VTT027
status: maintained
kano: must-have
verified: v2.0.0
---

# FEAT-VTT027: Debian package builds the Rust binary via cargo

## Kano
must-have (p0)

## Description
The Launchpad PPA ships the Rust `vtt-linux` binary (not the legacy C/`Makefile.linux` build). Because Ubuntu Noble's cargo 1.75 cannot parse edition-2024 manifests, `debian/rules` installs a **pre-built binary** (`packaging/linux/vtt-linux.prebuilt`, compiled with current Rust on Emmanuel's machine and committed) rather than running cargo during packaging. Users installing from the PPA receive the actual Rust rewrite. (Mechanism changed from cargo-in-rules to prebuilt at v2.0.2 — see `debian/rules`.)

## User Observable Behaviour
- `sudo apt install voice-to-text` on 2.0.0 installs a `vtt-linux` binary that is the Rust build, not the old C one
- `file /usr/bin/vtt-linux` reports a stripped ELF 64-bit executable of approximately 10 MB (old C binary was ~2-3 MB)
- `strings /usr/bin/vtt-linux | grep -c rust_begin_unwind` returns a positive number (Rust runtime signatures)
- `dpkg -L voice-to-text` lists no Python scripts and no shell scripts (except postinst)
- the source package builds on Launchpad without a Rust toolchain — it validates and installs the committed prebuilt binary (no `gcc`, `cmake`, `g++`, `cargo` in the build path)

## Acceptance Criteria
- [x] **AC-1** — `debian/rules` `override_dh_auto_build` verifies and installs the committed `packaging/linux/vtt-linux.prebuilt` — cargo is NOT run during packaging (Noble cargo 1.75 cannot parse edition-2024) — verified in `debian/rules`
- [x] **AC-2** — The installed `/usr/bin/vtt-linux` is the committed Rust prebuilt binary — verified
- [x] **AC-3** — `debian/control` Build-Depends carries only the shared libs needed to validate the prebuilt (no `rustc`/`cargo` — the binary is built off-box) — verified in `debian/control`
- [x] **AC-4** — `debian/control` Depends contains only runtime shared libraries — no build tools, no `python3` — verified
- [x] **AC-5** — `debuild -S -sa` produces a signed source package; Launchpad builds it successfully for Noble — verified across v2.0.0 through v2.1.1
- [x] **AC-6** — SHA-256 of `target/release/vtt-linux` matches SHA-256 of `/usr/bin/vtt-linux` after `dpkg -i` — verified on local install
- [x] **AC-7** — `debian/changelog` 2.0.0 entry explicitly notes that prior 1.0.x releases shipped the legacy C binary — verified in `debian/changelog`

## Linked Tasks
- TASK-VTT035, TASK-VTT036, TASK-VTT038, TASK-VTT039

## Parent Story
- STORY-VTT011
