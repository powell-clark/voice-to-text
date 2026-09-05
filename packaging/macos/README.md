# macOS packaging

`cargo build --release` produces a working binary on macOS (arm64 and x86_64).

A signed `.app` bundle with `NSMicrophoneUsageDescription` and notarisation
is planned. Assets and `Info.plist` will live here when that work lands.

## Current build

```bash
cargo build --release
./target/release/vtt
```

## Planned

- `cargo bundle` or hand-crafted `.app` structure
- Apple developer signing + notarisation
- DMG packaging for distribution
