# TASK-VTT060: Delete superseded Launchpad PPA versions

## Context
Emmanuel observed the PPA is at ~5 GB of 8 GB quota and suspected the .deb bundles the 488 MB Whisper model. It does not — `dpkg-deb -I build-archives/voice-to-text_2.0.0_amd64.deb` shows the .deb is 5.9 MB, and `debian/postinst` downloads the model on install. The real quota consumer is accumulated historical versions.

Launchpad retains every published version of every source package across every distro series until explicitly deleted. Voice-to-text has shipped 1.0.0 → 2.0.4 across jammy and noble, each with source tarball + buildlogs + binary .deb per architecture. That is the 5 GB.

## Acceptance Criteria
1. On https://launchpad.net/~powellclark/+archive/ubuntu/voice-to-text/+packages, all versions below v2.0.3 are deleted (kept: v2.0.3, v2.0.4, v2.0.5)
2. Launchpad quota falls meaningfully — target is under 2 GB
3. `sudo apt policy voice-to-text` on this machine still offers v2.0.5 for install (deletion does not break the current version)
4. No in-flight installs are broken — old versions remain downloadable for a short grace period per Launchpad's behaviour

## Technical Approach
Manual operation via Launchpad web UI — there is no bulk delete CLI. Steps:
1. Visit https://launchpad.net/~powellclark/+archive/ubuntu/voice-to-text/+packages
2. Filter to "any status"
3. For each obsolete version, click "Delete" and supply a short reason ("superseded, freeing quota")
4. Confirm the quota bar on the PPA overview page drops
5. Wait up to 24 hours for Launchpad's garbage collection to actually reclaim the space (deletion is immediate from the user view but GC is asynchronous)

## Test Strategy
Before/after screenshots of the PPA quota bar. Post-cleanup, run `sudo apt update && apt-cache policy voice-to-text` on a test system and confirm only v2.0.3+ versions are offered.

## Files
- No file changes — operational task on Launchpad only
