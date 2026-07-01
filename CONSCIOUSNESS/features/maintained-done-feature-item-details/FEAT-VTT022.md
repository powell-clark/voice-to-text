---
id: FEAT-VTT022
status: maintained
kano: must-have
verified: v2.0.0
---

# FEAT-VTT022: Whisper model loaded once in-process worker thread

## Kano
must-have (p0)

## Description
The Whisper model is loaded into memory exactly once per VTT process lifetime and reused across every push-to-talk transcription. No subprocess is spawned, no model file is re-read from disk between presses, no Python interpreter is involved in the hot path. Model switching is an explicit user action that reloads the context on the same worker thread.

## User Observable Behaviour
- VTT startup logs `Loading model: ggml-small.en.bin in 2.3s` (or similar timing) exactly once; not once per recording
- Tray transitions `Loading model...` → `Ready` during startup
- Every subsequent push-to-talk press logs `Transcribed in 0.4s` or similar; never logs another `Loading model:` line unless the user changes models
- Pressing push-to-talk ten times consecutively produces ten `Transcribed in Xs` lines, zero model-load lines, zero subprocess invocations

## Acceptance Criteria
- [x] **AC-1** — `ps aux | grep vtt` shows exactly one `vtt-linux` process for the lifetime of the app — no child processes spawned for transcription — verified in v2.0.0 daily use
- [x] **AC-2** — `ps aux | grep python3` during transcription returns zero VTT-related processes — verified in v2.0.0
- [x] **AC-3** — Average end-to-end press-to-text latency across ten consecutive 5-second clips on the user's RTX 2060 SUPER is under 500 ms — verified subjectively (~0.3-0.4s typical)
- [x] **AC-4** — VTT log file contains exactly one `Model loaded:` line per model (startup + any user-triggered switches), regardless of how many transcriptions run — verified in v2.0.0
- [x] **AC-5** — Memory usage stays constant after the initial model load — `ps -o rss vtt-linux` shows no growth over 100 consecutive transcriptions — verified in v2.0.x extended use

## Linked Tasks
- TASK-VTT024, TASK-VTT026, TASK-VTT027, TASK-VTT028, TASK-VTT030, TASK-VTT033, TASK-VTT034

## Parent Story
- STORY-VTT010
