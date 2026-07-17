# ADR-0007: Deterministic cross-platform release distribution

**Status:** Proposed
**Date:** 2026-07-17
**Context:** voice-to-text

## Context

The operator needs releases that are deterministic across all three target
platforms (Linux, Windows, macOS) — available predictably, within minutes of
CI finishing, not sitting on a third party's opaque queue. Today's reality
proves the gap on every axis.

**Linux — the PPA has three stages users conflate, only one of which matters.**
`scripts/release-ppa.sh` uploads a source package via `dput` to
`ppa:powellclark/voice-to-text` (Launchpad account `powellclark`), which then
passes through:

1. Source **Published** (`getPublishedSources` — the upload was accepted)
2. Build reported **"Successfully built"** in the Launchpad UI
3. Binary **published to the apt index** (`getPublishedBinaries`) — the *only*
   stage `apt install`/`apt upgrade` actually observes

On 2026-07-17, release 2.3.10 took 3h05m just to reach source-Published
(`uploaded_utc` 10:09:11Z → `source_published_utc` 13:14:40Z per the newly
seeded `packaging/linux/ppa-release-times.tsv`, TASK-VTT134), and the binary
was still unpublished 4+ hours after upload — `apt` kept serving 2.3.9 while
the Launchpad UI showed "Successfully built" for both the `noble` and
`jammy1` targets. This latency is inherent to Canonical's shared free build
farm queue and **cannot be fixed or paid around from this project's side** —
`debian/rules` already documents that the PPA doesn't even compile Rust
(Noble's cargo 1.75 can't parse edition-2024 manifests; the pre-built
`packaging/linux/vtt-linux.prebuilt` binary is installed verbatim), so the
entire multi-hour wait is queue time, not build time.

**Windows and macOS have automated build+publish pipelines already — what
they lack is *signing*.** A tag-triggered CI workflow already exists at
`.github/workflows/release.yml` (`on: push tags: v*`): it builds the Linux
binary (`ubuntu-24.04`), the Windows `.msi` (`cargo-wix` + Vulkan SDK,
`windows-latest`), and the macOS arm64/Intel binaries (`macos-latest` /
`macos-13`), creates the GitHub release as a draft, attaches each asset, and
un-drafts once all platforms succeed. So the "tag-triggered matrix that
publishes every platform" that this ADR's Decision proposes as the target is
**already ~80% built and live**. The genuinely-missing pieces are narrower:
Windows Authenticode code-signing (TASK-VTT047, backlog) and update mechanism
(TASK-VTT095); a macOS `.app` bundle (TASK-VTT040 / FEAT-VTT029) plus
signing/notarisation (TASK-VTT043, now PARKED — no Apple licence) and a
current Homebrew tap (`CLAUDE.md` records the tap's Formula pinned to `v0.2.0`
and its Cask pointing at a `file://` path only on Emmanuel's machine,
FEAT-VTT036); the Linux `.deb`/apt-repo channel (this ADR's core, TASK-VTT135),
which `release.yml` deliberately does NOT touch (the PPA `dput` needs
Emmanuel's GPG key and runs locally via `scripts/release-ppa.sh`).

**PGPS drift to reconcile (surfaced 2026-07-17):** TASK-VTT048 ("GitHub
Actions matrix workflow…") and TASK-VTT049 ("Auto-release on tag push…") sit
in backlog as if greenfield, but `release.yml` already satisfies the core of
both. They should be re-scoped to the actual remaining delta (attach the
`.deb`; add signing) or closed against the live workflow — not implemented
from scratch. Noted on both cards.

Net effect: release availability today is non-deterministic on Linux (waits on
Launchpad's build farm) and *unsigned* on Windows/macOS (the CI pipeline runs,
but ships binaries browsers/Gatekeeper flag as unverified) — three different
gaps, not "no pipeline at all."

## Decision (proposed)

Make a tag-triggered GitHub Actions matrix the single source of truth for
release artifacts, and publish each platform to a channel with deterministic
availability (minutes after CI, under this project's control) rather than a
third party's queue:

- **Linux** — a self-hosted, CI-built, GPG-signed apt repository hosted on
  **GitHub Pages** (zero new accounts/credentials; a CI job pushes the repo
  files to a `gh-pages` branch), with Cloudflare R2 (S3-compatible,
  `aws s3 sync --endpoint-url`) as an optional later upgrade if
  private/branded hosting is ever wanted. apt availability becomes a
  function of CI duration (minutes), not Launchpad queue depth (hours,
  unbounded). The Launchpad PPA is demoted to an optional
  secondary/discovery channel, or retired.
- **Windows** — CI-built and Authenticode-signed `.msi` attached to GitHub
  Releases, with an in-app/update-check pulling from Releases.
- **macOS** — **PARKED** (see Priority & sequencing below): the end-state
  is a CI-built, Apple developer-signed and notarised `.dmg`/`.app` with
  the Homebrew cask pointing at the Release artifact, but signing requires
  a paid Apple Developer licence the operator is deferring indefinitely.
- **GitHub Releases is the canonical artifact store** — every per-platform
  channel (apt repo, Homebrew cask, Windows update-check) pulls from the
  same CI-produced artifacts rather than each platform building or sourcing
  its own.
- Deploy→available latency is recorded per platform, extending the tracking
  log seeded today at `packaging/linux/ppa-release-times.tsv` (TASK-VTT134)
  to Windows and macOS once their pipelines exist.

## Considered Alternatives

### (a) Stay on the Launchpad PPA and accept the latency

**Pros:** zero migration cost; Launchpad already has the `powellclark`
account, the PPA target, and `dput` wired up; users who already added the
PPA keep working unchanged.

**Cons / risks:** non-deterministic (observed 4h40m+ and still pending on
2026-07-17); opaque (the UI's "Successfully built" state actively misleads —
it is not the binary-published state `apt` needs); unfixable from this
project's side since it depends on Canonical's shared free build farm queue
depth, which this project has no lever over. Rejected as the sole channel.

### (b) Self-hosted apt repo on Cloudflare R2 — optional later upgrade

**Pros:** deterministic (minutes after CI, not queue-dependent); full
control over publish timing and retention; aligns with the operator's
existing config-as-code-over-console-click-ops preference and R2 usage
elsewhere in the federation; S3-compatible API means no new tooling
(`aws s3 sync --endpoint-url` against the R2 endpoint) beyond credentials
already in the operator's toolbox.

**Cons / risks:** requires a GPG signing key plus secret management in CI
(a new key-lifecycle and rotation burden that Launchpad currently absorbs
for free); requires provisioning an R2 bucket and CI credentials; requires
CI changes to build the apt repo metadata (`Packages`, `Release`, signing)
on every tag; existing users must add a new apt source line and import a
new keyring — a one-time but real user-facing migration step.

### (c) Self-hosted apt repo on GitHub Pages — recommended for Linux

**Pros:** simplest possible (no bucket/credentials to provision, uses the
repo's existing GitHub hosting), zero additional cost, zero new accounts —
the whole channel is a CI job pushing files to a `gh-pages` branch.

**Cons / risks:** public-only (no private/staged channel option) and less
control over caching/invalidation than an R2 bucket. Neither matters for
this project today — the repo and releases are already public. If
private/branded hosting is ever wanted, migrating to (b) later is a
same-files move (repoint the apt source URL), not a redesign.

### (d) Manual local builds per machine (`scripts/release-local.sh`)

**Pros:** already exists, works today, zero additional infrastructure;
useful as a stopgap and for local `--install` testing.

**Cons / risks:** not scalable or deterministic across machines or users —
it produces a single machine's `.deb`, not a distributable release channel.
Rejected as the release *strategy*; retained as a local dev/test tool
regardless of this decision.

## Consequences

**Positive:**

- Deterministic, minutes-after-CI availability on Linux, replacing an
  unbounded and opaque third-party queue.
- One consistent model across all three platforms: CI builds the artifact,
  GitHub Releases stores it canonically, a thin per-platform channel
  (apt repo / Homebrew cask / Windows update-check) republishes it.
- Deploy→available latency becomes measurable and trackable per platform,
  extending the pattern TASK-VTT134 already established for the PPA.

**Negative:**

- New GPG signing-key lifecycle and rotation responsibility for the apt
  repo, where Launchpad previously carried this burden.
- New CI secrets to manage: R2 bucket credentials, GPG private key,
  eventually Authenticode and Apple Developer signing credentials.
- One-time user-facing migration: existing PPA users must add a new apt
  source and keyring to keep receiving updates once/if the PPA is demoted
  or retired.
- Ongoing CI ownership: the release pipeline itself becomes something this
  project builds and maintains, rather than delegating build/hosting to
  Canonical's infrastructure.

## Recommendation (pending operator sign-off)

Adopt the CI-as-single-source-of-truth model described above. Sequence
Linux first — the self-hosted apt-repo-on-Pages path is the current pain
(TASK-VTT134's own tracking log exists because of it) and is fully
buildable in headless CI today with no outstanding signing prerequisites.
Windows and macOS follow as their respective signing prerequisites
(Authenticode, Apple notarisation) land, since publishing unsigned binaries
to a new channel would trade one trust problem for another.

This ADR is the umbrella decision that several already-tracked roadmap items
realise — it does not itself implement any of them:

- TASK-VTT048 — GitHub Actions matrix workflow (`ubuntu-latest` +
  `macos-latest` + `macos-14` ARM + `windows-latest`), STORY-VTT014
- TASK-VTT047 — Windows Authenticode code signing, STORY-VTT013
- TASK-VTT049 — Auto-release on tag push with binaries + `.deb` + `.dmg` +
  `.msi` attached, STORY-VTT014
- TASK-VTT095 — Update mechanism: how Windows/macOS users get new versions,
  STORY-VTT013
- TASK-VTT040 / FEAT-VTT029 — macOS `.app` bundle, STORY-VTT012
- TASK-VTT043 — Apple developer signing + notarisation, STORY-VTT012
- FEAT-VTT036 — Homebrew tap currency (blocked on FEAT-VTT029, then updated
  every release from the same canonical artifact)
- TASK-VTT134 — Track PPA build durations / deploy→available tracking,
  STORY-VTT018 (the log this ADR's Context section draws on, and the
  pattern to extend to Windows/macOS)

## References

- `debian/rules` — pre-built-binary rationale (Noble cargo 1.75 / edition
  2024 constraint; the PPA never compiles Rust)
- `scripts/release-ppa.sh` — current `dput` upload flow, pbuilder hard gate
- `scripts/release-local.sh` — local `.deb` build/install, alternative (d)
- `packaging/linux/ppa-release-times.tsv` — deploy→available tracking log
  seeded 2026-07-17 (TASK-VTT134), source of the 2.3.10 latency evidence
  above
- `CLAUDE.md` — PPA information and Homebrew tap sections
- ADR-0003 — whisper-rs in-process model (established precedent: prefer
  vendored/self-contained artifacts over third-party runtime dependencies)
- ADR-0006 — Audio decode + resample dependency (established precedent:
  file the ADR before the one-way-door dependency/infrastructure choice)
- STORY-VTT012, STORY-VTT013, STORY-VTT014, STORY-VTT018
- DIRECT-VTT002, DIRECT-VTT003, DIRECT-VTT004

## Priority & sequencing (operator steer, 2026-07-17)

Operator priority of care: **Linux > macOS > Windows.** Practical
regression-test order (fastest to slowest to verify): Linux, then Windows
(same machine), then macOS.

Implementation sequence:

1. **Linux — self-hosted signed apt repo (this ADR's core).** Next up;
   unblocked; highest value (fixes the current Launchpad pain). Hosting:
   **GitHub Pages is the recommended starting point** — free, no new accounts,
   GitHub serves the repo files over HTTPS; the only tradeoff is the hosting
   repo is public. Cloudflare R2 (a low-cost S3-compatible bucket uploaded via
   the `aws` CLI) is the alternative if private/branded hosting is wanted later.
   Both serve the same apt files — the choice is only *where they live*.
2. **Windows — MSI on GitHub Releases (TASK-VTT047 for signing).** Second;
   the operator has the machine to regression-test it. **Cost flag:** an
   Authenticode certificate is *also* a paid item (traditional OV certs run
   roughly £150–400/yr; Azure Trusted Signing is ~$10/mo) — the same budget
   logic that parks macOS may apply. Interim path that costs nothing: ship
   the unsigned `.msi` on GitHub Releases (users see a one-time SmartScreen
   "unrecognised app" warning but install fine). Signing remains the
   end-state; whether/when to pay for it is an operator decision.
3. **macOS — PARKED.** Apple Developer ID signing + notarization needs a paid
   Apple license the operator is deferring indefinitely for budget reasons. macOS
   deterministic distribution is therefore parked; revisit when budget allows. An
   unsigned/local or Homebrew-from-source path may be an interim option but is out
   of scope until this un-parks.
