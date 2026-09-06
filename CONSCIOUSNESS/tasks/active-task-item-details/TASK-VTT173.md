# TASK-VTT173: Recent transcriptions submenu reads archive sidecars

## Context

Re-transcribe last recording only ever reaches N=1, and 2026-09-06 11:12 proved that is the wrong depth: the operator lost a batch of dictations from a cleared prompt box, clicked it, and got back a byte-identical retype of the clip already on screen (log: vtt-2026-09-06.log 11:12:44-11:12:45). Two defects. (1) Depth — the recovery net reaches one clip; the loss was several clips back. (2) Cost and feedback — it re-runs whisper on audio whose exact transcript is already stored verbatim in the archive .json sidecar 'text' field, and because the retype was identical to what was already there the operator read it as 'nothing came through'. Fix: a tray 'Recent transcriptions' submenu listing the last N by time plus a first-words preview, read straight from ~/.local/share/voice-to-text/archive/<date>/*.json (296 sidecars exist for 2026-09-06 alone), copying the chosen entry to clipboard or retyping it. Zero whisper cost, reaches as far back as the archive retains. FEAT-VTT039 acceptance criteria all passed as written — this is a scope extension of the recovery net, not a regression.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Directive: DIRECT-VTT002
- Story: STORY-VTT018
- Features: FEAT-VTT039

## Pre-mortem

### Failure modes

- _(to be filled in)_

### Weak assumptions

- _(to be filled in)_
