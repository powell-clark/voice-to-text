# Voice to Text - Tools

Utilities for testing, benchmarking, and troubleshooting Voice to Text.

## crash_test

Test crash handler functionality by triggering various crashes.

**Build:**
```bash
cd tools
make crash_test
```

**Usage:**
```bash
./crash_test <test>
```

**Available tests:**
- `segfault` - Null pointer dereference (SIGSEGV)
- `abort` - Call abort() (SIGABRT)
- `fpe` - Division by zero (SIGFPE)
- `stackoverflow` - Infinite recursion

**Example:**
```bash
./crash_test segfault
# Check crash log:
cat /tmp/vtt-crash-test/crash.log
```

## benchmark.sh

Benchmark all transcription models with sample audio.

**Usage:**
```bash
./benchmark.sh <audio_file.wav> [output_file.md]
```

**Create test audio:**
```bash
# Record 10 seconds of speech
arecord -d 10 -f S16_LE -r 16000 -c 1 test_audio.wav

# Run benchmark
./benchmark.sh test_audio.wav results.md
```

**Output:**
- Markdown table with model performance
- Transcription time and realtime speed (e.g., "3.2x")
- System information (CPU, RAM, GPU, CUDA)

**Example output:**
```
| Model         | Backend | Time (s) | Speed | Accuracy | Notes                    |
|---------------|---------|----------|-------|----------|--------------------------|
| CT2 tiny.en   | CT2     | 0.8      | 12.5x | ✓        | The quick brown fox...   |
| CT2 small.en  | CT2     | 1.2      | 8.3x  | ✓        | The quick brown fox...   |
```

## first_run_wizard.sh

Interactive setup wizard for first-time users.

**Usage:**
```bash
./first_run_wizard.sh
```

**Checks:**
- ✅ X11 display availability
- ✅ Python version (>= 3.10)
- ✅ System dependencies (python3, pactl, arecord)
- ✅ Python packages (faster-whisper, ctranslate2)
- ✅ vtt-linux binary installation
- ✅ Microphone access and audio levels

**Features:**
- Interactive microphone test (record + playback)
- Audio level checking
- Troubleshooting guidance

**Example:**
```bash
$ ./first_run_wizard.sh

==========================================
Voice to Text - First Run Setup Wizard
==========================================

✓ X11 display detected: :0
✓ Session type: x11

Checking system dependencies...
✓ python3 found
✓ pactl found
✓ arecord found
✓ Python 3.12.0 (>= 3.10 required)

...

==========================================
Setup Summary
==========================================

✓ All checks passed!

You're ready to use Voice to Text.
```

## Tools Roadmap

### Coming Soon
- [ ] **model_downloader.sh** - Download and verify Whisper models
- [ ] **performance_profiler** - Profile CPU/GPU usage during transcription
- [ ] **integration_test** - End-to-end testing suite
- [ ] **log_analyzer** - Parse and analyze vtt.log for issues
- [ ] **config_validator** - Validate settings.conf syntax

### Future
- [ ] **audio_visualizer** - Real-time recording visualization
- [ ] **model_quantizer** - Convert models to smaller quantized versions
- [ ] **database_explorer** - Browse transcription history (when implemented)

## Development

**Build all tools:**
```bash
cd tools
make
```

**Clean:**
```bash
make clean
```

## Contributing

Add new tools following this pattern:
1. Create tool file in `tools/`
2. Add build target to `tools/Makefile`
3. Document in this README
4. Add example usage

Keep tools focused, lightweight, and well-documented.
