# TASK-VTT135: Self-hosted signed apt repo on GitHub Pages

## Context

ADR-0007 Linux channel: CI job builds apt repo metadata (Packages/Release), GPG-signs it, publishes .deb + index to gh-pages branch. Replaces Launchpad queue wait (2.3.10: 4h40m+ and binary still unpublished) with minutes-after-CI availability. No matrix dependency — ubuntu-latest only; NOT blocked by TASK-VTT047/041/042. Includes migration doc for existing PPA users (new source line + keyring). GPG key generation + repo secret setup included in scope.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Story: STORY-VTT014
- Directive: DIRECT-VTT002
- Features: FEAT-VTT031
