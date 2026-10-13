# Voice-to-Text

This is a public repository on GitHub. Keep it secure and professional.

## Build commands

| Platform | Command | Output |
|----------|---------|--------|
| Linux    | `cargo build --release` | `target/release/vtt` |
| macOS    | `cargo build --release` | `target/release/vtt` |
| Windows  | `cargo build --release` | `target/release/vtt.exe` |
| All      | `cargo clean` | removes `target/` |

### Windows installer (.msi)

```bash
cargo wix    # requires: cargo install cargo-wix
```

### Linux .deb (local, no PPA)

```bash
bash scripts/release-local.sh [--install]
```

### Linux PPA release

```bash
bash scripts/release-ppa.sh   # pbuilder hard-gate, then dput
```

### Scripts

| Script | Purpose |
|--------|---------|
| `scripts/vendor_whisper.sh` | Vendor `whisper.cpp` into `third_party/` |
| `scripts/build-release.sh` | Release build with this machine's paths remapped out of the binary; every shipped binary comes from it |
| `scripts/install-dev.sh` | Install a local `VTT.app` build into /Applications (macOS) |
| `scripts/git-hooks/` | Git hooks, including the pre-push personal-data gate (install via `bash scripts/git-hooks/install.sh`) |

Never register a self-hosted runner on this repository: it is public, and a
fork pull request could run code on the runner's machine.

## PPA information (Linux only)

- Launchpad account: `powellclark` (no hyphen)
- PPA target: `ppa:powellclark/voice-to-text`
- dput target: `powellclark-voice-to-text`
- Pre-built binary: `packaging/linux/vtt.prebuilt`. Never commit it, or any
  other compiled binary. `release-ppa.sh` and `release-local.sh` build it with
  `build-release.sh` at release time and stage it untracked; `debuild -S`
  packs it from the working tree into the source tarball, and `debian/rules`
  installs it verbatim. The pre-push gate refuses any ELF, PE or Mach-O file.

## Homebrew tap (macOS) — separate repository

macOS Homebrew packaging lives in a separate tap repository, not this one.
Make every Homebrew Formula or Cask change there. A grep of this tree finds no
Formula or Cask; do not conclude from that that Homebrew is unsupported.

The tap repository is private and renamed, and its revival for the v2 line is
pending. Do not point users at `brew install` until the revived tap ships. Its
Cask does not install the current version; a v2 Cask needs the macOS `.app`
bundle. No release automation updates the tap.

## Packaging layout

```
packaging/linux/     — vtt.service (systemd); vtt.prebuilt is staged here at release time
packaging/windows/   — notes; wix/ stays at root (cargo-wix tool convention)
packaging/macos/     — placeholder for planned .app bundle
debian/              — Debian package metadata (must stay at root for dpkg)
wix/                 — WiX installer template (must stay at root for cargo-wix)
```

## Commit messages

Use conventional commit style (`feat:`, `fix:`, `chore:`, etc.).
Focus on the *why*, not the *what*.
Never add an AI or bot attribution trailer; the commit-msg hook refuses one.
