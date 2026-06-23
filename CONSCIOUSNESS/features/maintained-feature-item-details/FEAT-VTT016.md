---
id: FEAT-VTT016
status: maintained
kano: must-have
---

# FEAT-VTT016: One-command PPA release script with pre-flight checks

## Description
`scripts/release-ppa.sh` wraps the entire Launchpad PPA release process: pre-flight checks (clean tree, tests passing, pbuilder build success), debian changelog bump, GPG signing, and `dput` upload. The operator runs a single command and the script stops on any failure before uploading. This replaced a manual multi-step process that regularly resulted in partial or broken PPA uploads.

## Acceptance Criteria
- [x] `bash scripts/release-ppa.sh` runs end-to-end without manual steps — verified across v2.0.0, v2.0.4, v2.0.5 releases
- [x] Script aborts before `dput` if any pre-flight check fails (dirty tree, tests fail, pbuilder fail) — verified by intentionally introducing failures during TASK-VTT011
- [x] pbuilder build is a hard gate — Launchpad build parity is verified before upload — verified
- [x] Script tags the git commit after successful upload — verify in git log after release
- [x] Script is idempotent on failure — safe to re-run after fixing the failure cause — verified in v2.0.5 release workflow

## Linked Tasks
- TASK-VTT011

## Parent Story
- STORY-VTT004
