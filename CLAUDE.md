# Voice-to-Text

This is a public repository on GitHub. Keep it secure and professional.

## Build commands

| Platform | Command | Output |
|----------|---------|--------|
| Linux    | `cargo build --release` | `target/release/vtt-linux` |
| macOS    | `cargo build --release` | `target/release/vtt-linux` |
| Windows  | `cargo build --release` | `target/release/vtt-linux.exe` |
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

### Other scripts

| Script | Purpose |
|--------|---------|
| `scripts/vendor_whisper.sh` | Vendor `whisper.cpp` into `third_party/` |
| `scripts/setup-runner.sh` | Set up the macOS GitHub Actions self-hosted runner |
| `scripts/install-dev.sh` | Set up the local dev environment |
| `scripts/git-hooks/` | Git hooks (install via `install-dev.sh`) |

## PPA information (Linux only)

- Launchpad account: `powellclark` (no hyphen)
- PPA target: `ppa:powellclark/voice-to-text`
- dput target: `powellclark-voice-to-text`
- Pre-built binary: `packaging/linux/vtt-linux.prebuilt`
  (Ubuntu Noble ships Cargo 1.75, which cannot parse edition-2024 manifests;
  `release-ppa.sh` rebuilds and re-commits this file automatically on each release)

## Homebrew tap (macOS) — SEPARATE REPO

macOS `brew install` is served from a **separate repo**, not this one:

- Repo: `powell-clark/homebrew-voice-to-text` (private) — the tap `brew tap powell-clark/voice-to-text` resolves to (Homebrew strips the `homebrew-` prefix)
- Local checkout: `~/projects/aux/voice-to-text-homebrew` (dir name differs from the GitHub repo name — its `origin` remote is `homebrew-voice-to-text`)
- Contents: `Formula/voice-to-text.rb` (source build) + `Casks/voice-to-text.rb` (binary app)
- Install: `brew tap powell-clark/voice-to-text && brew install voice-to-text`

**All Homebrew/macOS-cask changes go in that repo, never here.** A grep of this
tree finds no cask — that is expected; do not conclude Homebrew is unsupported.

⚠️ The tap has NOT tracked the v2.x Rust line. Formula is pinned to `v0.2.0`;
the Cask is `v0.3.16` (Python era) pointing at a `file://` local-dev path that
only exists on Emmanuel's machine — so a normal user's `brew install` of the
current version fails. Bringing it current is **blocked on the macOS `.app`
bundle (FEAT-VTT029)** — without a v2.x `.app` there is nothing to package.
Tracked as FEAT-VTT036. No release automation updates the tap.

## Packaging layout

```
packaging/linux/     — vtt-linux.prebuilt + vtt.service (Launchpad + systemd)
packaging/windows/   — notes; wix/ stays at root (cargo-wix tool convention)
packaging/macos/     — placeholder for planned .app bundle
debian/              — Debian package metadata (must stay at root for dpkg)
wix/                 — WiX installer template (must stay at root for cargo-wix)
```

## Feature status: done vs maintained

The PGPS status field currently blurs two distinct terminal states — separate them when grooming:

- **done** — a one-time achievement that stays true forever with no per-release effort (e.g. the
  pure-Rust/no-Python rewrite, dual-backend removal, ObjC removal). Once shipped, never revisited.
- **maintained** — works, but must be re-tested every release or it can silently break (e.g. the
  key-repeat filter, the resident model, auto model download, the apt PPA). An ongoing burden,
  not a one-off win.

Don't auto-stamp a feature `[maintained]` by default — check whether it's genuinely a one-time
"done" or a per-release "maintained" burden. When an autonomous card audit sets a feature status
based on absence-of-code, verify it matches actual intent before treating it as settled — absence
of code in this tree does not mean a capability is unsupported (see the Homebrew tap note above).

## Commit messages

Use conventional commit style (`feat:`, `fix:`, `chore:`, etc.).
Focus on the *why*, not the *what*.

## Multi-machine workflow

Claude Code runs on three machines (macOS, Linux, Windows), all on `main`.

1. `git pull origin main` before starting work
2. Commit small and focused
3. Push after each commit
4. Pull again after pushing to pick up cross-machine changes
