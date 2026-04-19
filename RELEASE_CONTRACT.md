# Release Contract — voice-to-text is a PRODUCTION REPO

Voice-to-text is a public Launchpad PPA. Every version we publish is visible
to the world forever. Failed Launchpad builds appear on the PPA page as
permanent "build failures" badges. Users notice. Other developers judge.

## The principle

> **No release leaves this machine until the exact build that Launchpad will run
> has succeeded locally.**

This is not a convention. It is a **mechanical gate** in `scripts/release-ppa.sh`.
If the local build fails, `dput` never runs, no tag is pushed, no failure
appears on the PPA. The broken state stays on this machine until it's fixed.

## How the gate works

`scripts/release-ppa.sh` runs this sequence for every release:

1. **Build the release binary** (local rustup) and commit it as `vtt-linux.prebuilt`.
2. **Generate source package** (`debuild -S -sa`) — this also runs lintian.
3. **Chroot build in pbuilder** matching the target distro exactly (`noble`, `jammy`).
   - Same `cargo` version, same apt repos, same network isolation as Launchpad.
   - If this step returns non-zero, the script refuses to `dput` and exits 1.
4. **Only then** does `dput` run.

The gate applies **per distro**. Noble passes → noble uploads. Jammy fails →
noble already uploaded but jammy aborts here and no jammy failure appears on
the PPA.

## One-time setup

Before the first release on a new machine, create the pbuilder chroots:

```bash
sudo pbuilder --create \
    --distribution noble \
    --basetgz /var/cache/pbuilder/noble-base.tgz \
    --mirror http://archive.ubuntu.com/ubuntu \
    --components "main restricted universe multiverse"

sudo pbuilder --create \
    --distribution jammy \
    --basetgz /var/cache/pbuilder/jammy-base.tgz \
    --mirror http://archive.ubuntu.com/ubuntu \
    --components "main restricted universe multiverse"
```

Each chroot is ~500 MB. Build time for a voice-to-text release in a chroot is
~2–3 minutes. Multiply by the number of target distros.

Refresh the chroots every few months (`sudo pbuilder --update`) so they track
Ubuntu security updates.

## The bypass — and when never to use it

```bash
VTT_SKIP_PBUILDER=1 ./scripts/release-ppa.sh
```

This env var bypasses the chroot gate. It exists for two reasons:

1. The first release from a machine that doesn't have pbuilder set up yet
   (rare — should be a one-off).
2. Republishing a known-good release identical to one that already built
   successfully on Launchpad (extremely rare).

**Do not set `VTT_SKIP_PBUILDER` to avoid a failing chroot build.** That is
the exact failure mode this gate was built to prevent. Tonight (2026-04-19),
without this gate, the PPA picked up 2.0.0, 2.0.1, and 2.0.2~jammy1 failed
builds — each one visible to anyone who visits the PPA page. Never again.

## Why we ship a pre-built binary

The Rust dep tree (cpal → coreaudio-sys, arboard → image → moxcms, etc.)
uses crates that require Rust edition 2024. Ubuntu LTS ships cargo 1.75,
which cannot parse edition-2024 Cargo.toml manifests. Source builds on
Launchpad are therefore structurally impossible on current LTS releases.

Rather than fight the dep tree, the `.deb` ships a binary compiled locally
with rustup 1.91. `debian/rules` installs it. This is the same pattern
used by Google Chrome, Zoom, and many proprietary Linux packages.

When Ubuntu ships a newer cargo (25.04+), we can revert to a source-built
approach. The old cargo-build rules are in git at commit `efa5d75`.

## What this protects

- **Public reputation.** The PPA page stays at 100% build success.
- **User trust.** Every version `apt upgrade` offers them actually works.
- **Developer confidence.** We can iterate fast locally because the gate
  catches problems before they are visible.
- **CI/CD discipline.** This is the seed of the eventual GitHub Actions
  matrix — same principle, eventually automated across macOS and Windows.

## Contract summary

| Thing | Rule |
|---|---|
| Public build failures | Zero. The gate exists to prevent them. |
| `VTT_SKIP_PBUILDER=1` | Used twice a year at most, and only for reasons explained in writing. |
| Chroot freshness | Update monthly via `sudo pbuilder --update`. |
| Binary origin | Compiled locally with rustup, committed as `vtt-linux.prebuilt`. |
| First release from a new machine | pbuilder setup first. No exceptions. |

> If you are about to bypass this gate, pause. Every past bypass in this
> project's history caused a visible failure. The gate is cheap. Use it.
