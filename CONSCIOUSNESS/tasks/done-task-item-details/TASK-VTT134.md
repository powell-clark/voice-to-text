# TASK-VTT134: Track PPA build durations — record upload→build-start wait and build time per release

> **Groomed (product-owner grooming pass, 2026-07-21):** acceptance criteria below are drawn
> directly from this card's own "Candidate approach" — no new scope invented. Links (STORY-VTT018,
> DIRECT-VTT002, FEAT-VTT035) already correctly set in TASK-BACKLOG-INDEX.md; no change needed there.


## Context

Operator reports Launchpad PPA builds take ~5h and it is unclear whether that is queue-wait or genuine build time (surfaced 2026-07-17 while apt still showed 2.3.9 despite a 2.3.10 release). Want a durable record per release: version, upload time, build-start time (queue wait = start-upload), build-finish time (build time = finish-start), pass/fail. Candidate approach: a small step in release-ppa.sh (or a post-upload poll) that reads the Launchpad build record via its API and appends a row to a tracked log (e.g. packaging/linux/ppa-build-times.tsv), so trends are visible in git. May warrant promotion to a feature card. Verify against the actual Launchpad build-record timestamps, not wall-clock guesses.

## Acceptance criteria

- [x] release-ppa.sh reads the Launchpad build record via its API after each
      PPA upload — new step calls scripts/record-ppa-times.sh, best-effort so a
      failed API call never fails a release
- [x] Per-release row appended to packaging/linux/ppa-build-times.tsv carrying
      version, series, arch, upload, build-start, build-finish, binary-published
      and build state
- [x] Queue-wait and build-time are derived from the logged timestamps, along
      with publish-wait and the upload-to-available total
- [x] Timestamps come from the actual Launchpad build record (getBuildRecords)
      and the actual binary publication (getPublishedBinaries) — nothing estimated
- [x] Re-running the script updates an existing row rather than duplicating it,
      so a release recorded at upload time can be completed once the binary lands
- [ ] DEFERRED (operator): confirm whether this warrants promotion to a feature
      card. Five releases of real data are now in the log, so the question is
      answerable whenever the operator wants to take it.

## Findings

Backfilled 2.0.5, 2.1.0, 2.3.9 and 2.3.10. The result overturns the premise
that "PPA builds take ~5h":

    version   queue wait   build time   publish wait   total
    2.0.5     2s           2m45s        17m29s         20m16s
    2.1.0     45m35s       2m45s        58m45s         1h47m05s
    2.3.9     16s          19m11s       42m35s         1h02m02s
    2.3.10    28s          7m51s        3h54m12s       4h02m31s

Building is never the bottleneck — it ranges 2-19 minutes and is stable.
Queue wait is usually seconds. The dominant and wildly variable term is
Launchpad's publisher cycle AFTER the build succeeds: 17m, 58m, 42m, then
3h54m. For 2.3.10 that is 97% of the wait.

This is direct evidence for TASK-VTT135 (Self-hosted signed apt repo on
GitHub Pages): no build-side optimisation can address a delay that happens
entirely after the build finishes.

## Dependencies

- Story: STORY-VTT018
- Directive: DIRECT-VTT002
- Features: FEAT-VTT035
