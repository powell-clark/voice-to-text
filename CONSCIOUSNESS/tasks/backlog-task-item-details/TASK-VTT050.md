# TASK-VTT050: Silero VAD integration

## Acceptance Criteria
1. Recording auto-stops within 1s of speech ending (silence detected by Silero VAD)
2. Background noise below a configurable threshold does not trigger recording
3. VAD runs in-process alongside whisper-rs — no additional process or network call
4. The feature is toggleable in `settings.conf`; default is enabled
