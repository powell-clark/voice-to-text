# Voice to Text

Voice-to-text using OpenAI Whisper. Hold Right Alt to speak, release when finished. Your text appears in any focused input box.

## How It Works

1. Hold **Right Alt** (Right Option key)
2. Speak
3. Release when finished
4. Text appears in your focused input box

**No internet required** - all transcription happens locally on your Mac.

## Installation

```bash
brew tap powell-clark/voice-to-text
brew install --cask voice-to-text
```

Voice to Text automatically installs to /Applications.

### Build from Source
```bash
# Clone repository
git clone https://github.com/powell-clark/voice-to-text.git
cd voice-to-text

# Build with embedded whisper
make vendor-whisper  # Download whisper.cpp
make whisper-lib     # Build whisper library
make complete        # Build VTT.app

# Install
cp -R VTT.app /Applications/
open /Applications/VTT.app
```

## First Run Setup

**IMPORTANT:** Grant these three permissions in System Settings → Privacy & Security or the app won't work:
- **Microphone** - To record your voice
- **Accessibility** - To paste text
- **Input Monitoring** - To detect Right Alt key

The app will prompt you on first launch.

## Choose a Model

Select a model from the menu bar. **We recommend starting with small (244 MB)** - it provides the best balance of speed and accuracy.

- **tiny** (39 MB) - Fastest, less accurate
- **base** (74 MB) - Fast, good for simple dictation
- **small** (244 MB) - **Recommended** - good accuracy, reasonable speed
- **medium** (769 MB) - Slower, more accurate
- **large** (1550 MB) - Slowest, best accuracy

**Larger models take longer to transcribe but are more accurate.** Models download automatically on first selection.

## Microphone Selection

Choose your microphone from the menu bar. **Use your system default microphone** (usually "Built-in Microphone") for best results.

## Technical Details

- Pure Objective-C implementation for minimal overhead
- CoreAudio for low-latency audio capture (16kHz mono)
- CGEventTap for global hotkey monitoring
- Embedded whisper.cpp for fast transcription
- Linear resampling for device compatibility
- 4KB audio buffers (~43ms latency)

## Troubleshooting

### Microphone not activating
1. Check Input Monitoring permission for Voice to Text
2. Toggle permission OFF then ON
3. Restart Voice to Text

### No transcription output
1. Enable logging: Voice to Text menu → Logging: On
2. Check logs: `log stream --predicate 'process == "VTT"'`
3. Verify whisper model is downloaded

### Key not detected
1. Check System Settings → Keyboard → Modifier Keys
2. Ensure Right Option isn't remapped
3. Try with logging enabled to see key events

## Building

Requirements:
- macOS 11.0+
- Xcode Command Line Tools
- CMake (for whisper.cpp)

The Makefile handles:
- Building embedded whisper.cpp library
- Compiling VTT daemon
- Creating app bundle with icon
- Bundling whisper model

## Architecture

```
VTT.app/
├── Contents/
│   ├── MacOS/
│   │   ├── VTT              # Main executable
│   │   └── whisper-cli       # Fallback CLI (optional)
│   ├── Resources/
│   │   ├── AppIcon.icns      # App icon
│   │   └── ggml-small.en.bin # Bundled model
│   └── Info.plist
```

## License

Apache 2.0

## Credits

- Built with [whisper.cpp](https://github.com/ggerganov/whisper.cpp)
- Uses OpenAI's Whisper models
- Icon generated programmatically with CoreGraphics
