# Test fixtures

| File | What it is | Source and licence |
|------|------------|--------------------|
| `jfk-ask-not.mp3` | 11 s, 16 kHz mono MP3, about 76 KB: "And so, my fellow Americans, ask not what your country can do for you; ask what you can do for your country." | President John F. Kennedy's inaugural address, 20 January 1961. A work of the United States federal government, so in the public domain in the United States (17 U.S.C. § 105). This copy is `samples/jfk.mp3` from whisper.cpp, which the project already vendors. Used by the smoke test, `.github/workflows/smoke.yml`. |
| `testing-one-two-three.wav`, `testing-one-two-three.mp3` | "Testing one two three four. The quick brown fox jumps over the lazy dog." Synthetic speech. | Generated for this project by `scripts/gen-test-fixture-windows.ps1` with the Windows speech synthesiser. Used by the end-to-end tests in `src/whisper.rs`. |
