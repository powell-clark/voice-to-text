# TASK-VTT040: macOS .app bundle with metal feature

## Acceptance Criteria
1. `cargo bundle --release` produces a `VoiceToText.app` on macOS
2. `Info.plist` contains `NSMicrophoneUsageDescription` so the OS shows a permission prompt
3. The app launches, shows the menu-bar icon, and transcribes a test phrase on the Intel i9 Mac
4. Build compiles with `--features metal` enabled
