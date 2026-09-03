# TASK-VTT137: Unify release flow — local install + PPA publish in one run

## Context

`scripts/release-local.sh` (build + optional `--install`, no PPA) and
`scripts/release-ppa.sh` (build + pbuilder gate + dput, no local install) are
two scripts the operator must choose between. Emmanuel asked (2026-07-17) for
one flow: build once, install locally for immediate use, and kick off the
publish for everyone else in the same run — the publish side is slow and async,
and there is no reason to wait on it to use your own software.

## What 2026-09-03 added to this task

The unification is the smaller half. The larger half is that the local install
path lies. Emmanuel ran `release-local.sh --install` at 08:44 and again after a
`git pull`, and both times ended up still running the August binary. Five lines
of this script carry two of the three faults:

```bash
sudo apt install -y "$STAGED_DEB"
echo "Installed. Restart VTT with:"
echo "  pkill -f vtt-linux; /usr/bin/vtt-linux &"
```

- `apt install` refuses a package whose version already matches the installed
  one and exits 0. The script then prints "Installed." over a no-op. This is
  guaranteed to happen on every local iteration, because a local build has the
  same version as what is installed — the exact case this flag exists for.
- The restart advice cannot work. `vtt.service` carries `Restart=always`, so
  `pkill` respawns the old binary within five seconds, and the singleton lock
  then refuses the hand-launched replacement. The advice also leaves systemd
  owning nothing (`MainPID: 0`), so a later `systemctl restart` restarts an
  empty unit.

The third fault, a packaged binary older than the source, is already gated by
TASK-VTT152 (Fail the build when the packaged binary is stale). The diagnosis is
now available at runtime via TASK-VTT153 (Verify the running binary matches the
installed one).

## Acceptance criteria

- [x] `--install` uses `dpkg -i`, which installs over an unchanged version, with
      `apt-get -f install` afterwards to settle dependencies
- [x] `--install` verifies the install took rather than trusting the exit code,
      and fails loudly when the on-disk binary did not change
- [x] The restart advice is `systemctl --user restart vtt.service`, never
      `pkill` plus a hand-launched binary
- [x] The script tells the operator to run `vtt-linux --doctor` and says what a
      clean result looks like
- [x] `--install` warns when a running process holds a replaced binary, since
      that is the state a fresh install creates and the one that looks like
      failure
- [x] `bash -n scripts/release-local.sh` is clean and a no-arg run is unchanged
      for anyone not passing `--install`

## Out of scope, and why

Merging `release-ppa.sh` into this script is NOT done here. The two build
different artifacts — `release-ppa.sh` builds a SOURCE package for Launchpad,
`release-local.sh` builds a BINARY `.deb` — and the card's own reality-check
note (2026-07-17) says bolting local-install onto the PPA script is the wrong
shape. The world-facing path is `.github/workflows/release.yml`, tag-triggered.
A true one-command release is bump → changelog → commit → tag → push, and that
belongs in a `release.sh` wrapper or the release-manager agent, gated on a
version-bump decision that is the operator's.

What this pass does is make the half that runs every day tell the truth. A
unified flow built on an install step that silently no-ops would inherit the
defect.

## Pre-mortem

### Failure modes

- `dpkg -i` leaves unmet dependencies where `apt install` would have resolved
  them, turning a working install into a broken package state. Mitigation:
  `apt-get -f install -y` immediately afterwards, which is the documented repair
  and a no-op when nothing is broken.
- The verification compares the wrong thing and passes vacuously. Mitigation:
  compare the installed binary's SHA against the packaged one, not mtime —
  `dpkg` preserves the packaged timestamp, which is precisely what made the
  August binary look current this morning.

### Weak assumptions

- That `systemctl --user restart` is the right advice for every user. It is
  right for a systemd desktop, which is what the `.deb` targets and what the
  package ships a unit for. A user running it by hand is outside the packaged
  path and the doctor covers them.

## Dependencies

- Story: STORY-VTT014
- Directive: DIRECT-VTT002
- Features: FEAT-VTT031


## Evidence

Both new checks exercised against the machine's real state rather than a
contrived one, because the real state happened to contain exactly the two
conditions they exist to catch.

The content comparison, against the `.deb` Emmanuel built and installed:

```
$ dpkg-deb --fsys-tarfile ../voice-to-text_2.3.11_amd64.deb \
    | tar -xO ./usr/bin/vtt-linux | sha256sum
710f179de97290a58c14ab38c48ce2adc13512396a8a0ca1835a69d25a955c99
$ sha256sum /usr/bin/vtt-linux
710f179de97290a58c14ab38c48ce2adc13512396a8a0ca1835a69d25a955c99
```

Equal, so that install genuinely took — which mtime would have denied, since
`dpkg` preserves the packaged timestamp and `/usr/bin/vtt-linux` still reads
18 August.

The stale-process walk, against the live deleted-inode process:

```
detected: 3247235
```

That is the pid whose `/proc/3247235/exe` reads `/usr/bin/vtt-linux (deleted)`.

`bash -n scripts/release-local.sh` is clean.

Together these mean the same run that installs now says both "the install took"
and "the running app has not picked it up yet" — the gap that cost three rounds
of diagnosis this morning, closed at the place it opened.

## Follow-up

The genuine unification — one command that cuts the release for the world and
installs locally — still wants a `release.sh` wrapper doing bump, changelog,
commit, tag, push. It is gated on a version-bump decision that is Emmanuel's:
`Cargo.toml` and `debian/changelog` both read 2.3.11 while five shipped features
sit under `[Unreleased]`. Worth its own task once he decides whether the next
release is 2.4.0.