# TASK-VTT042: Apple Silicon build with coreml + metal features

## Acceptance Criteria
1. `cargo build --release --features coreml,metal` succeeds on an Apple Silicon Mac
2. The resulting binary runs natively (not under Rosetta 2)
3. CoreML model conversion runs on first launch; subsequent runs use cached CoreML model
4. Transcription accuracy and latency are at least equal to the Linux whisper-rs baseline
