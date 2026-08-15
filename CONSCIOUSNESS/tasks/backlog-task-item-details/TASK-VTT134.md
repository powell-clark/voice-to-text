# TASK-VTT134: Track PPA build durations — record upload→build-start wait and build time per release

> **Groomed (product-owner grooming pass, 2026-07-21):** acceptance criteria below are drawn
> directly from this card's own "Candidate approach" — no new scope invented. Links (STORY-VTT018,
> DIRECT-VTT002, FEAT-VTT035) already correctly set in TASK-BACKLOG-INDEX.md; no change needed there.


## Context

Operator reports Launchpad PPA builds take ~5h and it is unclear whether that is queue-wait or genuine build time (surfaced 2026-07-17 while apt still showed 2.3.9 despite a 2.3.10 release). Want a durable record per release: version, upload time, build-start time (queue wait = start-upload), build-finish time (build time = finish-start), pass/fail. Candidate approach: a small step in release-ppa.sh (or a post-upload poll) that reads the Launchpad build record via its API and appends a row to a tracked log (e.g. packaging/linux/ppa-build-times.tsv), so trends are visible in git. May warrant promotion to a feature card. Verify against the actual Launchpad build-record timestamps, not wall-clock guesses.

## Acceptance criteria

- [ ] release-ppa.sh (or a post-upload poll step) reads the Launchpad build record via its API after each PPA upload
- [ ] Per-release row appended to a tracked log (e.g. packaging/linux/ppa-build-times.tsv): version, upload time, build-start time, build-finish time, pass/fail
- [ ] Queue-wait (build-start − upload) and build-time (build-finish − build-start) are derivable from the logged timestamps, not wall-clock guesses
- [ ] Timestamps are read from the actual Launchpad build record, not estimated
- [ ] Operator confirms whether this warrants promotion to a feature card once the log has a few releases of real data

## Dependencies

- Story: STORY-VTT018
- Directive: DIRECT-VTT002
- Features: FEAT-VTT035
