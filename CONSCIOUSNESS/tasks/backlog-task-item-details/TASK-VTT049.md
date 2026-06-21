# TASK-VTT049: Auto-release on tag push with all platform artefacts

## Acceptance Criteria
1. Pushing a `vX.Y.Z` tag triggers a GitHub Release with all platform binaries attached: `.deb`, `.dmg`, `.msi`, `.exe`, and Linux binary
2. Release notes are auto-generated from conventional commits since the last tag
3. The release is created as a draft first — operator approves before it goes public
4. Existing `scripts/release-ppa.sh` PPA push is not broken by the new workflow
