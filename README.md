# VTT - Voice to Text

A lightweight macOS menu bar app for instant voice-to-text transcription using OpenAI's Whisper.

## Features

- **Push-to-talk recording**: Hold Right Option key to record, release to transcribe
- **Instant paste**: Automatically pastes transcribed text at cursor position
- **Menu bar app**: Runs quietly in background with status indicator
- **Multiple Whisper models**: Choose from tiny, base, small, medium, or large models
- **Zero-latency recording**: Optimized audio queue for instant recording start
- **Embedded Whisper**: Built-in whisper.cpp for fast local transcription
- **No network required**: All transcription happens locally on your Mac

## Installation

### Quick Install
1. Download the latest VTT.app from releases
2. Move to Applications folder
3. Launch VTT
4. Grant required permissions (Accessibility, Input Monitoring, Microphone)

### Build from Source
```bash
# Clone repository
git clone https://github.com/yourusername/voice-to-text.git
cd voice-to-text

# Build with embedded whisper
make vendor-whisper  # Download whisper.cpp
make whisper-lib     # Build whisper library
make complete        # Build VTT.app

# Install
sudo cp -R VTT.app /Applications/
open /Applications/VTT.app
```

## Usage

1. Click VTT icon in menu bar to see status
2. Hold **Right Option** key to start recording (🎤 appears)
3. Speak your text
4. Release Right Option to stop and transcribe
5. Text is automatically pasted at cursor position

## Permissions Required

VTT needs these macOS permissions to function:
- **Accessibility**: To simulate keyboard paste (Cmd+V)
- **Input Monitoring**: To detect Right Option key press/release
- **Microphone**: To record audio

Grant these in System Settings → Privacy & Security

## Models

Select different Whisper models from the menu:
- **tiny**: Fastest, least accurate (39 MB)
- **base**: Fast, good for quick notes (74 MB)
- **small**: Balanced speed/accuracy (244 MB) - Default
- **medium**: Slower, more accurate (769 MB)
- **large**: Slowest, best accuracy (1550 MB)

Models download automatically on first selection.

## Technical Details

- Pure Objective-C implementation for minimal overhead
- CoreAudio for low-latency audio capture (16kHz mono)
- CGEventTap for global hotkey monitoring
- Embedded whisper.cpp for fast transcription
- Linear resampling for device compatibility
- 4KB audio buffers (~43ms latency)

## Troubleshooting

### Microphone not activating
1. Check Input Monitoring permission for VTT
2. Toggle permission OFF then ON
3. Restart VTT

### No transcription output
1. Enable logging: VTT menu → Logging: On
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

MIT

## Credits

- Built with [whisper.cpp](https://github.com/ggerganov/whisper.cpp)
- Uses OpenAI's Whisper models
- Icon generated programmatically with CoreGraphics