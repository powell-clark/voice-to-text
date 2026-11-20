# Test fixtures

| File | What it is | Source and licence |
|------|------------|--------------------|
| `jfk-ask-not.mp3` | 11 s, 16 kHz mono MP3, about 76 KB: "And so, my fellow Americans, ask not what your country can do for you; ask what you can do for your country." | President John F. Kennedy's inaugural address, 20 January 1961. A work of the United States federal government, so in the public domain in the United States (17 U.S.C. § 105). This copy is `samples/jfk.mp3` from whisper.cpp, which the project already vendors. Used by the smoke test, `.github/workflows/smoke.yml`. |
| `testing-one-two-three.wav`, `testing-one-two-three.mp3` | "Testing one two three four. The quick brown fox jumps over the lazy dog." Synthetic speech. | Generated for this project by `scripts/gen-test-fixture-windows.ps1` with the Windows speech synthesiser. Used by the end-to-end tests in `src/whisper.rs`. |
| `wav-golden/s16-16k.wav`, `wav-golden/s16-48k.wav` | 16-bit mono PCM at 16 kHz and 48 kHz, 1023 samples from `audio::golden_wav_samples`. | Written by hound 3.5.1 through `audio::write_wav`, before the in-house writer replaced it (TASK-VTT211). The `wav_write_golden` tests require the in-house writer to reproduce them byte for byte. |
| `wav-golden/s24-16k.wav`, `wav-golden/s24-48k.wav` | 24-bit mono PCM (WAVE_FORMAT_EXTENSIBLE) at 16 kHz and 48 kHz, the same 1023 samples. | Written by hound 3.5.1 through `archive::write_wav_at`, as above. |
| `wav-golden/*.hound.f32` | The samples hound's reader returned for each golden WAV, as raw little-endian f32. | Decoded by hound 3.5.1 through `whisper::decode_wav_native`. The `wav_read_golden` tests require the reader to return them bit for bit. |
| `wav-golden/testing-one-two-three.hound.sha256` | The sample count and SHA-256 of the f32 samples hound's reader returned for `testing-one-two-three.wav`. | As above. |

Never regenerate a file in `wav-golden/` with the code it checks: a golden file the writer or reader produced itself proves nothing.
