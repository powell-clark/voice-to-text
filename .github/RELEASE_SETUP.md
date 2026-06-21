# Release Process

Distribution: Ubuntu/Debian via Launchpad PPA (`ppa:powellclark/voice-to-text`).

## How to release (PPA)

```bash
# 1. Build the release binary locally
cargo build --release
cp target/release/vtt-linux packaging/linux/vtt-linux.prebuilt

# 2. Bump version in Cargo.toml and debian/changelog, then commit + push

# 3. Run the PPA release script (pbuilder gate + dput)
bash scripts/release-ppa.sh
```

`packaging/linux/vtt-linux.prebuilt` is committed to the repo because Ubuntu
Noble ships Cargo 1.75, which cannot parse edition 2024 manifests. The
Launchpad build step installs the pre-built binary rather than building from
source. This follows the Google Chrome / Zoom pattern for proprietary .debs.
Revert to the cargo-build path when Ubuntu ships a newer toolchain (expected
25.04+) — see `debian/rules` git history commit efa5d75.

## Development workflow (local .deb)

```bash
bash scripts/release-local.sh --install
```

This builds via `cargo build --release`, packages a `.deb`, and installs it.

## macOS

`cargo build --release` produces `target/release/vtt-linux` (the binary name is cross-platform).
A signed `.app` bundle is planned in `packaging/macos/` — see the backlog.

## GitHub Actions CI runner (macOS, optional)

To run macOS CI jobs locally without paying GitHub's $0.08/min:

```bash
# Get a runner token from:
# https://github.com/powell-clark/voice-to-text/settings/actions/runners
bash scripts/setup-runner.sh YOUR_TOKEN_HERE
```
