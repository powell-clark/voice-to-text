# TASK-VTT053: Implement CT2 transcription daemon

## Acceptance Criteria
1. `transcribe_daemon.py` starts, loads a faster-whisper model, and responds to the protocol from TASK-VTT052
2. Daemon responds to a transcribe request within 500ms of receiving audio (model already loaded)
3. Daemon handles shutdown command cleanly and exits 0
4. Unit tests cover: startup, transcribe round-trip, and shutdown over the IPC protocol
