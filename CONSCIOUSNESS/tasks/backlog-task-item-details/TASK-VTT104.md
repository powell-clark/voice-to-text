# TASK-VTT104: macOS universal binary (lipo Intel + Apple Silicon into one)

Today the release ships two macOS binaries (vtt-macos-intel, vtt-macos-arm64).
A single universal binary (`lipo -create`) would be one download that runs on
both — nicer UX, matches the historical v0.3.x universal builds. Add a combine
job that takes the two native artifacts and lipos them into `vtt-macos`.

- [ ] Release attaches a universal `vtt-macos` (Intel + arm64) verified with
      `lipo -info` and `file`
- [ ] Notes offer the universal as the primary macOS download
- Story: STORY-VTT013 · Directive: DIRECT-VTT002 · Parity §6
