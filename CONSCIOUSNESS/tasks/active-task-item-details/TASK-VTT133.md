# TASK-VTT133: Release 2.3.10 to the PPA — clipboard fix + re-transcribe + accumulated main

## Context
The PPA (and the operator's installed package) is at 2.3.9. `main` has drifted
~11 user-facing commits past the v2.3.9 tag with no PPA release — v2.3.9 went to
the GitHub channel only ("PPA deferred to operator", commit f76754c). The
operator wants a valid apt build so `apt upgrade` picks it up the usual way.

## Acceptance Criteria
1. Cargo.toml and debian/changelog both bumped to 2.3.10, aligned.
2. debian/changelog 2.3.10 entry lists the notable user-facing changes since
   2.3.9 (clipboard-persistence fix, re-transcribe recovery net, --file,
   device-index, security/macos/deps fixes).
3. Release-prep committed and pushed to main (pre-flight requires clean tree +
   local == remote).
4. `scripts/release-ppa.sh` run: pbuilder hard gate green on noble + jammy,
   debuild -S signed, dput to ppa:powellclark/voice-to-text, git tag v2.3.10.
   (Operator-run — needs sudo for pbuilder + GPG signing key.)
5. After Launchpad builds (~15 min), `apt-cache policy voice-to-text` shows
   2.3.10 as candidate; `sudo apt install --only-upgrade voice-to-text` installs
   it; VTT restarts on the new build.

## Technical Approach
- Version bump + changelog + `cargo build` to sync Cargo.lock, commit, push
  (agent — reversible, in-repo).
- The release itself is credential-gated (sudo pbuilder chroots + GPG) so the
  operator runs `bash scripts/release-ppa.sh`; the script builds the prebuilt,
  commits it, runs the hard gate, dputs, and tags.

## Dependencies
- TASK-VTT131 (clipboard fix) + TASK-VTT132 (re-transcribe) — landed on main.
