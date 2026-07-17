# TASK-VTT137: Unify release flow — local install + PPA publish in one run

## Context

Today scripts/release-local.sh (build + optional local --install, no PPA) and scripts/release-ppa.sh (build + pbuilder gate + dput to Launchpad, no local install) are two separate scripts the operator must choose between or run manually back-to-back. Operator explicitly asked (2026-07-17) for one flow: build once, install locally immediately for own instant use, AND kick off the publish step for everyone else in the same run — since the publish side (Launchpad today, or the future TASK-VTT135 self-hosted apt repo) is slow/async anyway and there's no reason the operator should wait on it to use their own software. Proposed shape: extend release-ppa.sh (or a new release.sh wrapper) to, after the pbuilder hard-gate passes, both (a) sudo apt install the freshly-built .deb locally and (b) dput/publish for the world, rather than requiring two separate manual invocations. Not a one-way-door architecture change — purely a workflow/script consolidation. No dedicated 'release-manager' agent exists for voice-to-text specifically; the generic release-manager agent type in this environment's shared-agents is scoped to the Consciousness plugin project, not this repo.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Story: STORY-VTT014
- Directive: DIRECT-VTT002
- Features: FEAT-VTT031


## Reality-check note (2026-07-17)

The world-facing release path is NOT release-ppa.sh alone — it is
`.github/workflows/release.yml` (tag-triggered: build+publish all platforms)
PLUS the local `scripts/release-ppa.sh` for the Launchpad PPA (GPG key, local
only). So the unified flow the operator wants is: one action that (a) cuts the
release for the world (bump → changelog → commit → tag → push tag, which fires
release.yml) AND (b) installs locally immediately (release-local.sh --install)
so the operator is on it without waiting on any queue. The natural home is the
release-manager AGENT (.claude/agents/release-manager.md) orchestrating both,
or a thin release.sh wrapper — NOT bolting local-install onto release-ppa.sh
(which builds SOURCE packages for Launchpad, a different artifact than the
binary .deb release-local.sh installs). The sudo steps (pbuilder gate, apt
install) remain operator-gated by design.
