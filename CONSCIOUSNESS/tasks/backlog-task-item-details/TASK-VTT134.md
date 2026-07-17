# TASK-VTT134: Track PPA build durations — record upload→build-start wait and build time per release

> **Needs review:** the agent created this task during real-time validation and is uncertain about scope or priority. Operator should review and re-tier as appropriate.


## Context

Operator reports Launchpad PPA builds take ~5h and it is unclear whether that is queue-wait or genuine build time (surfaced 2026-07-17 while apt still showed 2.3.9 despite a 2.3.10 release). Want a durable record per release: version, upload time, build-start time (queue wait = start-upload), build-finish time (build time = finish-start), pass/fail. Candidate approach: a small step in release-ppa.sh (or a post-upload poll) that reads the Launchpad build record via its API and appends a row to a tracked log (e.g. packaging/linux/ppa-build-times.tsv), so trends are visible in git. May warrant promotion to a feature card. Verify against the actual Launchpad build-record timestamps, not wall-clock guesses.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Story: STORY-VTT018
- Directive: DIRECT-VTT002
- Features: FEAT-VTT035
