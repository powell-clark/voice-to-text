# Stability Improvements

This document tracks stability and error handling improvements made to Voice to Text.

## Sprint 1: Crash Handling & Error Infrastructure (Completed)

### Crash Handler (`src/common/crash_handler.{c,h}`)

**Features:**
- Signal handlers for: SIGSEGV, SIGABRT, SIGFPE, SIGILL, SIGBUS
- Automatic backtrace generation on Linux (via `execinfo.h`)
- Crash log persistence to `~/.local/share/voice-to-text/crash.log`
- Previous crash detection on startup
- Clean crash recovery

**Integration:**
- Initialized in `src/linux/main.c` after logging setup
- Logs written before process termination
- Crash logs preserved across restarts

**Usage:**
```c
crash_handler_init("/path/to/log/dir");

if (crash_handler_has_previous_crash()) {
    vtt_log("Previous crash detected at %s", crash_handler_get_log_path());
}
```

### Error Handler (`src/common/error_handler.{c,h}`)

**Features:**
- Centralized error notification system
- Desktop notifications via libnotify (Linux)
- Error categorization and user-friendly messages
- Automatic logging of all errors

**Error Types:**
- `VTT_ERROR_MICROPHONE_INIT` - Microphone initialization failed
- `VTT_ERROR_MICROPHONE_ACCESS` - Cannot access microphone
- `VTT_ERROR_TRANSCRIPTION_TIMEOUT` - Transcription took >60s
- `VTT_ERROR_TRANSCRIPTION_FAILED` - Transcription process failed
- `VTT_ERROR_MODEL_LOAD_FAILED` - Model file not found/corrupt
- `VTT_ERROR_AUDIO_TOO_SHORT` - Recording <0.5s
- `VTT_ERROR_AUDIO_TOO_QUIET` - Amplitude too low

**Usage:**
```c
#include "error_handler.h"

vtt_error_notify(VTT_ERROR_MICROPHONE_INIT, "PortAudio error: paDeviceUnavailable");
```

## Planned Improvements

### Phase 2: Timeout Handling
- [ ] Add transcription timeout (60s max)
- [ ] Kill hung transcription processes
- [ ] Show progress indicator for long transcriptions

### Phase 3: Resource Monitoring
- [ ] Monitor memory usage
- [ ] Detect disk space issues
- [ ] Warn on low disk space for models

### Phase 4: Better Error Recovery
- [ ] Auto-restart audio stream on failure
- [ ] Retry transcription on transient errors
- [ ] Graceful degradation (fallback models)

### Phase 5: Telemetry (Opt-in)
- [ ] Track error frequency
- [ ] Monitor performance metrics
- [ ] Privacy-first (no audio/text data)

## Testing

```bash
# Test crash handler
cd tests
make test_crash_handler

# Test error notifications
make test_error_handler
```

## Performance Impact

- Crash handler: <1ms initialization, minimal runtime overhead
- Error notifications: ~5-10ms per notification
- No impact on recording/transcription performance

## Crash Log Example

```
=== VTT CRASH LOG ===
Time: 2025-01-15 14:32:10
Signal: 11 (SIGSEGV - Segmentation Fault)
PID: 12345

Backtrace:
  ./vtt-linux(+0x4a1c) [0x55f8a9d4a1c]
  /lib/x86_64-linux-gnu/libc.so.6(+0x42520) [0x7f4d2c042520]
  ./vtt-linux(vtt_audio_init+0x42) [0x55f8a9d4f92]
  ./vtt-linux(main+0x15d) [0x55f8a9d505d]

=== END CRASH LOG ===
```

## Monitoring

Check logs:
```bash
# Main log
tail -f ~/.local/share/voice-to-text/vtt.log

# Crash log (if exists)
cat ~/.local/share/voice-to-text/crash.log

# System journal (systemd)
journalctl --user -u vtt -f
```

## Success Metrics

Target: <1% crash rate in production

Current status:
- Crash detection: ✅ Implemented
- Error notifications: ✅ Implemented
- Timeout handling: ⏳ In progress
- Auto-recovery: ❌ Not started
- Telemetry: ❌ Not started

---

Last updated: 2025-01-15
