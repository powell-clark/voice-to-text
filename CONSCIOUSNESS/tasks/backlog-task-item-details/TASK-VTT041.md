# TASK-VTT041: Verify transcription on 2019 Intel i9 Mac

## Acceptance Criteria
1. VoiceToText.app from TASK-VTT040 runs on the 2019 Intel i9 MacBook Pro with Radeon Pro 5500M
2. Metal GPU acceleration is confirmed active (no fallback to CPU-only)
3. Transcription latency is under 2s for a 5-second recording with the small.en model
4. No crashes or GPU errors in Console.app during a 10-minute session
