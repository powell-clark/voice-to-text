# Release Process

Distribution: Ubuntu/Debian via Launchpad PPA (`ppa:powellclark/voice-to-text`).

## How to release (PPA)

```bash
# 1. Bump version in Cargo.toml and debian/changelog, then commit + push

# 2. Run the PPA release script: it builds the binary with
#    scripts/build-release.sh, stages it as packaging/linux/vtt.prebuilt,
#    runs the pbuilder gate and dputs
bash scripts/release-ppa.sh
```

`packaging/linux/vtt.prebuilt` is built at release time and never committed
(it is gitignored). Ubuntu Noble ships Cargo 1.75, which cannot parse edition
2024 manifests, so the Launchpad build step installs the pre-built binary
rather than building from source; `debuild -S` packs it from the working tree
into the 3.0 (native) source tarball. This follows the Google Chrome / Zoom
pattern for proprietary .debs. Revert to the cargo-build path when Ubuntu
ships a newer toolchain (expected 25.04+) — `git show 376d6e0:debian/rules`
is the last cargo-build version.

## Development workflow (local .deb)

```bash
bash scripts/release-local.sh --install
```

This builds via `scripts/build-release.sh`, stages the binary as
`packaging/linux/vtt.prebuilt`, packages a `.deb`, and installs it.

## macOS

`cargo build --release` produces `target/release/vtt` (the binary name is cross-platform).
A signed `.app` bundle is planned in `packaging/macos/` — see the backlog.
