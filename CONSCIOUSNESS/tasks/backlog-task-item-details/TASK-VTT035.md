# TASK-VTT035: Rewrite debian/rules to cargo build

## Context
`debian/rules` line 14 invokes `$(MAKE) -f Makefile.linux`, compiling the legacy C sources with gcc. After TASK-VTT032 deletes those C files, this invocation fails. The replacement must invoke `cargo build --release` and install the resulting `target/release/vtt-linux`.

## Acceptance Criteria
1. `override_dh_auto_build` runs `cargo build --release --locked` (the `--locked` flag enforces Cargo.lock usage for reproducible builds)
2. `override_dh_auto_install` installs `target/release/vtt-linux` to `debian/voice-to-text/usr/bin/vtt-linux` with mode 0755
3. `override_dh_auto_clean` runs `cargo clean` and removes any stale `target/` directory
4. The rules file continues to install `vtt.service` and `voice-to-text.desktop` as before
5. The Python install step (if present) is removed — no `transcribe.py` gets copied to `/usr/share/voice-to-text/`
6. Building the package with `debuild -us -uc` completes without errors; the resulting `.deb` contains `/usr/bin/vtt-linux` as the Rust binary
7. The Debian build uses the vendored crate sources at `vendor/` (already present) when network is unavailable at build time; if not, falls back to crates.io via `~/.cargo/registry`

## Technical Approach
```make
#!/usr/bin/make -f
export CARGO_HOME := $(CURDIR)/debian/cargo-home
export RUST_BACKTRACE := 1

%:
	dh $@

override_dh_auto_build:
	cargo build --release --locked

override_dh_auto_install:
	install -D -m 0755 target/release/vtt-linux debian/voice-to-text/usr/bin/vtt-linux
	install -D -m 0644 vtt.service debian/voice-to-text/usr/lib/systemd/user/vtt.service
	install -D -m 0644 debian/voice-to-text.desktop debian/voice-to-text/usr/share/applications/voice-to-text.desktop

override_dh_auto_clean:
	cargo clean || true
	rm -rf target debian/cargo-home

override_dh_auto_test:
	@echo "Skipping cargo test (tests added in STORY-VTT014)"
```

## Test Strategy
`debuild -S -sa` builds a source package successfully. `sbuild --dist=noble ../voice-to-text_2.0.0-1.dsc` (or local `pbuilder`) produces a working `.deb`. `dpkg -c voice-to-text_2.0.0_amd64.deb` confirms `/usr/bin/vtt-linux` is present and no Python files are installed.

## Files
- `debian/rules` (rewrite)
