# Linux packaging

## Files

- `vtt-linux.prebuilt` — pre-built x86_64 ELF binary committed to the tree so
  Launchpad PPA builds can install it without re-compiling. Ubuntu Noble ships
  Cargo 1.75, which cannot parse edition-2024 manifests in the dependency tree.
  `scripts/release-ppa.sh` rebuilds and re-commits this file on each release.
- `vtt.service` — systemd user service unit installed to
  `/usr/lib/systemd/user/vtt.service` by the Debian package.

## Packaging directories

- `debian/` at the project root — dpkg/debuild expect it there (conventional).
- `wix/` at the project root — cargo-wix expects it there (tool convention).
- This directory holds assets referenced by `debian/rules` that do not belong
  to the Debian package metadata itself.

## PPA release

```bash
bash scripts/release-ppa.sh
```

Builds a fresh binary, commits it here, runs pbuilder, and dput to
`ppa:powellclark/voice-to-text`.

## Local .deb

```bash
bash scripts/release-local.sh [--install]
```
