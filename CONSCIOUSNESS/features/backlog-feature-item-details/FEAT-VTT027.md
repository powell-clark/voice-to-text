# FEAT-VTT027: Debian package builds the Rust binary via cargo

## Kano
must-have (p0)

## Description
`debian/rules` invokes `cargo build --release`, producing the Rust `vtt-linux` binary and installing it as the package payload. The legacy `Makefile.linux` build path is retired. Users installing from the Launchpad PPA receive the actual Rust rewrite for the first time in 2.0.0.

## User Observable Behaviour
- `sudo apt install voice-to-text` on 2.0.0 installs a `vtt-linux` binary that is the Rust build, not the old C one
- `file /usr/bin/vtt-linux` reports a stripped ELF 64-bit executable of approximately 10 MB (old C binary was ~2-3 MB)
- `strings /usr/bin/vtt-linux | grep -c rust_begin_unwind` returns a positive number (Rust runtime signatures)
- `dpkg -L voice-to-text` lists no Python scripts and no shell scripts (except postinst)
- `apt-get build-dep voice-to-text` pulls in `rustc`, `cargo`, `libclang-dev` — not `gcc`, `cmake`, `g++`

## Acceptance Criteria
1. `debian/rules` `override_dh_auto_build` runs `cargo build --release --locked`
2. `debian/rules` `override_dh_auto_install` installs from `target/release/vtt-linux`
3. `debian/control` Build-Depends lists `rustc`, `cargo`, `libclang-dev`, `libvulkan-dev` and removes `portaudio19-dev` (unused after Rust rewrite)
4. `debian/control` Depends contains only runtime shared libraries — no build tools, no `python3`
5. `debuild -S -sa` produces a signed source package; Launchpad builds it successfully for Noble
6. SHA-256 of `target/release/vtt-linux` matches SHA-256 of `/usr/bin/vtt-linux` after `dpkg -i`
7. `debian/changelog` 2.0.0 entry explicitly notes that prior 1.0.x releases shipped the legacy C binary

## Linked Tasks
- TASK-VTT035, TASK-VTT036, TASK-VTT038, TASK-VTT039

## Parent Story
- STORY-VTT011
