# TASK-VTT125: Re-register the macOS Intel self-hosted runner

## Context

During the v2.3.9 release (2026-07-10) the "Build macOS binary (Intel)
[best-effort]" job sat queued forever: `gh api .../actions/runners` shows
**zero registered runners** — the Intel MacBook's runner registration has
lapsed entirely (GitHub removes self-hosted runners after ~30 days offline).
The release published anyway (job is best-effort by design), but every release
until this is fixed ships without the `vtt-macos-intel` asset.

## Approach

On the 2019 Intel MacBook Pro (must be done on that machine):

1. Get a fresh registration token: repo Settings → Actions → Runners → New
   self-hosted runner (or `gh api -X POST
   repos/powell-clark/voice-to-text/actions/runners/registration-token`).
2. Run `scripts/setup-runner.sh` (the repo's macOS runner setup script) with
   the token; confirm it installs as a service that survives reboot.
3. Verify: `gh api repos/powell-clark/voice-to-text/actions/runners` shows the
   runner online; re-run the v2.3.9 Intel job (`gh run rerun 29067979725
   --job <intel-job-id>`) or let the next release exercise it.

## Acceptance criteria

- [ ] Runner shows online in repo Actions settings
- [ ] Runner survives a MacBook reboot (service install, not foreground run)
- [ ] A Release workflow run produces the macOS Intel asset

## Dependencies

- Directive: DIRECT-VTT003 (macOS voice-to-text — Intel and Apple Silicon)
- Assignee: Emmanuel (physical access to the MacBook required)
