# TASK-VTT049: Auto-release on tag push with all platform artefacts

## Acceptance Criteria
1. Pushing a `vX.Y.Z` tag triggers a GitHub Release with all platform binaries attached: `.deb`, `.dmg`, `.msi`, `.exe`, and Linux binary
2. Release notes are auto-generated from conventional commits since the last tag
3. The release is created as a draft first — operator approves before it goes public
4. Existing `scripts/release-ppa.sh` PPA push is not broken by the new workflow


## Reality-check note (2026-07-17)

`.github/workflows/release.yml` already implements tag-triggered auto-release
with binaries + .msi attached and draft→publish gating. Remaining delta vs
this card's title: (1) the Linux `.deb` is NOT attached in CI (PPA runs
locally via release-ppa.sh, needs Emmanuel's GPG key) — folding a .deb build
into CI or attaching release-local.sh's output is the open piece; (2) the
`.dmg`/macOS `.app` is parked (no Apple licence, TASK-VTT043). Re-scope to
those two items rather than treating auto-release-on-tag as unbuilt. See
ADR-0007 Context reconciliation note.
