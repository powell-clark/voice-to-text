# Linux packaging

## Files

- `vtt.prebuilt` — pre-built x86_64 ELF binary, built at release time and never
  committed (it is gitignored). `scripts/release-ppa.sh` and
  `scripts/release-local.sh` build it with `scripts/build-release.sh` and stage
  it here; `debuild -S` packs it into the source tarball so Launchpad PPA builds
  install it without re-compiling. Ubuntu Noble ships Cargo 1.75, which cannot
  parse edition-2024 manifests in the dependency tree.
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

Builds a fresh binary, stages it here (untracked), runs pbuilder, and dput to
`ppa:powellclark/voice-to-text`.

## Local .deb

```bash
bash scripts/release-local.sh [--install]
```
