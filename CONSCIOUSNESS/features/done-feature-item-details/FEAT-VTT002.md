---
id: FEAT-VTT002
status: done
superseded_by: FEAT-VTT022, FEAT-VTT023
kano: must-have
---

# FEAT-VTT002: Whisper transcription with dual backends (done — superseded)

## Description
**SUPERSEDED.** The original VTT supported two transcription backends: whisper.cpp (C++ via subprocess) and CTranslate2/faster-whisper (Python). Both required spawning a subprocess per recording. This dual-backend architecture was retired in ADR-0003 (v2.0.0).

**Successor:** FEAT-VTT022 (in-process whisper-rs) and FEAT-VTT023 (pure Rust, no Python).

## Why Done (not Maintained)
ADR-0003 (committed as TASK-VTT024) retired both the whisper.cpp subprocess and the Python/CT2 backend in favour of whisper-rs loaded in-process. No dual-backend code remains in the codebase after TASK-VTT031 (Python deleted) and TASK-VTT032 (C/ObjC deleted).

## Historical Acceptance Criteria
- [x] Python CT2 backend transcribed audio via faster-whisper subprocess — delivered in original Python version
- [x] whisper.cpp backend transcribed audio via CLI subprocess — delivered in original version
- [x] Backend selection was configurable — delivered
- [x] Both backends retired and deleted in v2.0.0 — verified via TASK-VTT031, TASK-VTT032

## Linked Tasks
- TASK-VTT002, TASK-VTT031, TASK-VTT032

## Parent Story
- STORY-VTT001
