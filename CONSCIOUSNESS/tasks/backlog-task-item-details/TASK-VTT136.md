# TASK-VTT136: Fill 2.3.10 available_utc once Launchpad binary publishes

## Context

Launchpad has not yet published the 2.3.10 binary to the apt index (6+ hours after upload as of 2026-07-17 15:12 BST; source published 13:14:40Z, binary still pending for both noble build 33408583 and jammy1 build 33408588). Check with: curl -s 'https://api.launchpad.net/devel/~powellclark/+archive/ubuntu/voice-to-text?ws.op=getPublishedBinaries&binary_name=voice-to-text&exact_match=true' and look for a 2.3.10 entry with status Published. Once found, fill the pending available_utc and deploy_to_available columns for both rows in packaging/linux/ppa-release-times.tsv (currently 'pending'). Do this check FIRST at next session open, before starting TASK-VTT135, since it is a 30-second check with no dependency on other work.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Story: STORY-VTT018
- Directive: DIRECT-VTT002
- Features: FEAT-VTT035
