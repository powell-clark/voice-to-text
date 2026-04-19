# TASK-VTT061: Local build-archives/ disk cleanup

## Context
`build-archives/` on this machine is 5.7 GB of accumulated release artefacts — `.tar.xz` source tarballs, `.changes`, `.buildinfo`, `.upload`, and a scattering of `.deb` files from every local release back to v1.0.12. Not in git, not on Launchpad, just local disk bloat.

Most is safe to delete. The only files worth keeping are the artefacts for the CURRENTLY PUBLISHED PPA version(s) so Emmanuel can re-dput if Launchpad loses an upload.

## Acceptance Criteria
1. `build-archives/` contains only the latest 2 versions' artefacts (v2.0.4 + v2.0.5 after this story ships)
2. Total size under 50 MB
3. No git-tracked files were affected (build-archives/ is not in the repo, confirm via `git status`)
4. scripts/release-local.sh (or whichever release script writes to this dir) is NOT modified — we are just cleaning up its output

## Technical Approach
```
cd build-archives
ls -lh | head
# Confirm what is there, identify which files match v2.0.4 and v2.0.5
# Move the keepers to a temp location, nuke the rest, move keepers back
mkdir /tmp/vtt-keep
mv voice-to-text_2.0.4* voice-to-text_2.0.5* /tmp/vtt-keep/
rm -f voice-to-text_*
mv /tmp/vtt-keep/* .
rmdir /tmp/vtt-keep
du -sh .
```

## Test Strategy
Post-cleanup, run `scripts/release-local.sh` for a dry run — it should still find what it needs. Alternatively, verify by hand that the latest two source.changes files are still present and parseable with `dcmd cat voice-to-text_2.0.5_source.changes`.

## Files
- No file changes — disk hygiene only
