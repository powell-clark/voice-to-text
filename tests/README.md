# Voice to Text - Test Suite

Automated tests for Voice to Text.

## Running Tests

### All Tests
```bash
cd tests
make test
```

### C Tests Only
```bash
cd tests
make
./test_settings
```

### Python Tests Only
```bash
cd tests
python3 test_transcribe.py
```

## Test Coverage

### C Tests (`test_settings.c`)
- ✅ Load default settings
- ✅ Save and load settings persistence
- ✅ Model backend detection (CT2 vs W)

### Python Tests (`test_transcribe.py`)
- ✅ Model name parsing
- ✅ Language mode validation
- ✅ WAV header generation
- ✅ Temp file cleanup

## CI Integration

Tests run automatically on every PR via `.github/workflows/test.yml`:
- Linux build + unit tests
- macOS build verification
- Code linting (clang-tidy, pylint)
- Security scanning
- Debian packaging validation

## Adding New Tests

1. **C tests**: Add to `test_settings.c` or create new `test_*.c` file
2. **Python tests**: Add test class to `test_transcribe.py`
3. **Update Makefile**: Add new test binaries to `TEST_BINARIES`
4. **Run locally**: `make test` before pushing

## Test Requirements

- GCC/Clang with C11 support
- Python 3.10+
- All dependencies from main project

## Future Tests

- [ ] Audio recording tests (mock PortAudio)
- [ ] Text typing tests (verify UTF-8, special chars)
- [ ] Queue tests (buffer management)
- [ ] Integration tests (end-to-end transcription)
- [ ] Performance benchmarks
