# Voice-to-Text - Features Done

id|location|tested|description
FEAT-VTT001|src/common/transcribe.py|yes|Whisper-based transcription with faster-whisper backend achieving 95% accuracy
FEAT-VTT002|src/linux/keyboard.c, src/macos/VTTDaemon.m|yes|Push-to-talk with Right Alt (macOS) and Scroll Lock (Linux)
FEAT-VTT003|Casks/voice-to-text.rb|yes|Homebrew cask for macOS installation via brew tap
FEAT-VTT004|debian/|yes|APT package for Ubuntu/Debian Linux installation via PPA
FEAT-VTT005|src/common/transcribe.py|yes|Auto-detection of optimal device (CPU/CUDA) and compute type
FEAT-VTT006|src/common/transcribe.py|yes|CUDA GPU acceleration with automatic CPU fallback
FEAT-VTT007|src/linux/gui.c, src/macos/VTTDaemon.m|yes|System tray/menu bar integration for configuration
FEAT-VTT008|src/common/transcribe.py|yes|Multiple model sizes (tiny/base/small/medium/large-v3)
FEAT-VTT009|src/common/transcribe.py|yes|99+ language support with auto-detection
FEAT-VTT010|src/linux/typing.c|yes|XTest text injection into active application (Linux)
FEAT-VTT011|src/macos/VTTDaemon.m|yes|Accessibility API text injection (macOS)
FEAT-VTT012|src/linux/audio.c|yes|PortAudio cross-platform audio recording
