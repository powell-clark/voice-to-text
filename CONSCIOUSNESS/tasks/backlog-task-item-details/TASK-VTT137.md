# TASK-VTT137: Unify release flow — local install + PPA publish in one run

## Context

Today scripts/release-local.sh (build + optional local --install, no PPA) and scripts/release-ppa.sh (build + pbuilder gate + dput to Launchpad, no local install) are two separate scripts the operator must choose between or run manually back-to-back. Operator explicitly asked (2026-07-17) for one flow: build once, install locally immediately for own instant use, AND kick off the publish step for everyone else in the same run — since the publish side (Launchpad today, or the future TASK-VTT135 self-hosted apt repo) is slow/async anyway and there's no reason the operator should wait on it to use their own software. Proposed shape: extend release-ppa.sh (or a new release.sh wrapper) to, after the pbuilder hard-gate passes, both (a) sudo apt install the freshly-built .deb locally and (b) dput/publish for the world, rather than requiring two separate manual invocations. Not a one-way-door architecture change — purely a workflow/script consolidation. No dedicated 'release-manager' agent exists for voice-to-text specifically; the generic release-manager agent type in this environment's shared-agents is scoped to the Consciousness plugin project, not this repo.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Story: STORY-VTT014
- Directive: DIRECT-VTT002
- Features: FEAT-VTT031
