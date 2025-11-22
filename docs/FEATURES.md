## New Features and Developer Tools

This document describes recent enhancements to Voice to Text, including stability improvements, Wayland support, and comprehensive developer tools.

---

## 🛡️ Stability & Error Handling

### Crash Handler
Automatic crash detection and recovery system that helps diagnose issues.

**Features:**
- Signal handlers for SIGSEGV, SIGABRT, SIGFPE, SIGILL, SIGBUS
- Automatic backtrace generation using `execinfo.h`
- Crash log persistence at `~/.local/share/voice-to-text/crash.log`
- Previous crash detection on startup

**Location:** `src/common/crash_handler.{c,h}`

**Testing:** `tools/crash_test.c` - Trigger various crash scenarios

### Error Notifications
Desktop notifications for common error conditions with actionable guidance.

**Error Types:**
- Microphone initialization failures
- Microphone access denied
- Transcription timeouts
- Model loading errors
- Generic system errors

**Features:**
- libnotify integration for desktop popups
- User-friendly error messages with solutions
- Automatic notification dismissal

**Location:** `src/common/error_handler.{c,h}`

### Transcription Timeout
Prevents hung transcriptions from blocking the application.

**Features:**
- 60-second timeout with automatic process termination
- Fork/exec pattern for subprocess management
- SIGKILL enforcement for unresponsive processes
- Desktop notification on timeout

**Location:** `src/common/transcribe_timeout.{c,h}`

---

## 🎤 Recording Indicator

Real-time visual feedback during recording sessions.

**Features:**
- Desktop notification on recording start
- Live duration updates every second
- Warning when approaching max duration (last 10s)
- "Transcribing..." notification after recording stops

**Implementation:**
- GLib timeout integration (1s intervals)
- libnotify for persistent notifications
- Urgency escalation for warnings

**Location:** `src/common/recording_indicator.{c,h}`

---

## 🌊 Wayland Support (Experimental)

Native support for Wayland display protocol with compositor-specific features.

### Wayland Detection
Runtime detection of Wayland session and compositor.

**Detected Compositors:**
- GNOME Shell / Mutter
- KDE Plasma / KWin
- Sway / wlroots
- Hyprland
- Others via XDG environment variables

**Features:**
- Session type detection (`XDG_SESSION_TYPE`)
- Compositor identification via process detection
- XWayland availability check
- Helpful error messages and fallbacks

**Location:** `src/linux/wayland_detect.{c,h}`

### Wayland Keyboard (D-Bus)
Global keyboard shortcuts via D-Bus integration.

**Supported Compositors:**
- GNOME Shell (via `org.gnome.Shell.AcceleratorActivated`)
- KDE Plasma (via KGlobalAccel)

**Status:** Experimental - requires manual configuration

**Location:** `src/linux/keyboard_wayland_dbus.c`

### Wayland Typing
Text injection for Wayland sessions using multiple methods.

**Methods (auto-detected):**
1. **ydotool** - Userspace input device (recommended)
2. **dotool** - Alternative input tool
3. **Clipboard + Paste** - Fallback method

**Location:** `src/linux/typing_wayland.{c,h}`

**Documentation:** `docs/WAYLAND_SETUP.md`

---

## 🔧 Developer Tools

### Model Downloader
Interactive tool for downloading Whisper models from HuggingFace.

**Features:**
- Lists all available models with sizes
- Download with progress indicators (wget/curl)
- Verifies existing downloads
- Model metadata display

**Usage:**
```bash
./tools/download_model.sh list          # List available models
./tools/download_model.sh small.en      # Download specific model
```

**Models:** tiny, base, small, medium, large-v3 (English + multilingual)

### History Viewer (CLI)
Interactive terminal UI for browsing recording history.

**Features:**
- List all recordings with timestamps
- Play recordings (aplay/paplay/ffplay)
- View transcriptions from log file
- Search recordings
- Delete recordings
- Export recordings to directory

**Usage:**
```bash
./tools/history_viewer.sh               # Interactive mode
./tools/history_viewer.sh --list        # List recordings
./tools/history_viewer.sh --play 5      # Play recording #5
```

### History Viewer (GUI)
Tkinter-based graphical interface for recording management.

**Features:**
- Sortable table view with timestamps
- Real-time search/filter
- Double-click to play
- Transcription preview pane
- Bulk export functionality
- Delete with confirmation

**Usage:**
```bash
./tools/history_viewer_gui.py
```

**Requirements:** Python 3 + tkinter (usually pre-installed)

### Performance Profiler
CPU, memory, and GPU monitoring during transcription.

**Features:**
- 10Hz sampling rate
- Metrics: CPU%, memory (MB), GPU utilization, GPU memory
- CSV export for analysis
- Python visualization support (matplotlib/pandas)
- Average and peak calculations

**Usage:**
```bash
./tools/profile.sh test.wav small.en ./results
```

**Output:**
- `results/metrics.csv` - Raw time-series data
- `results/transcription.txt` - Transcription output
- Console summary with averages and peaks

### Memory Leak Detector
Valgrind-based leak analysis with GTK suppressions.

**Features:**
- Automated valgrind execution
- Leak summary parsing and categorization
- Suppressions for GTK/GLib false positives
- AddressSanitizer build target (`make asan`)

**Usage:**
```bash
./tools/leak_detector.sh ./leak_results 30
```

**Build Variants:**
```bash
make -f Makefile.linux asan    # AddressSanitizer build
make -f Makefile.linux debug   # Debug symbols
```

### System Monitor
Real-time application statistics and resource usage.

**Features:**
- Process status (running/stopped)
- CPU and memory usage
- Recording statistics (count, total time, disk usage)
- Recent activity from logs
- JSON export for integration

**Usage:**
```bash
./tools/system_monitor.sh stats         # Show current stats
./tools/system_monitor.sh watch         # Auto-refresh every 2s
./tools/system_monitor.sh json          # Export as JSON
```

### Comprehensive Test Suite
70+ automated tests covering all components.

**Test Categories:**
- Build system validation
- Wayland support modules
- Crash handler functionality
- Error handling
- Recording indicator
- Developer tools
- Python backend
- Memory leak detection
- Configuration files
- Documentation
- Git repository

**Usage:**
```bash
./tools/run_tests.sh                    # Run all tests
RUN_BUILD_TEST=1 ./tools/run_tests.sh   # Include build test
SHOW_SKIPPED=1 ./tools/run_tests.sh     # Show skipped tests
```

**Output:** Color-coded pass/fail/skip with summary statistics

### First-Run Wizard
Automated setup and validation for new installations.

**Checks:**
- X11/Wayland session detection
- System dependencies (gcc, python3, etc.)
- Python version (>= 3.10)
- Python packages (faster-whisper, ctranslate2)
- Binary installation
- Whisper model availability
- Microphone functionality
- Audio level validation

**New Features:**
- Automatic model download prompt
- Integrated with model downloader
- Validates model installation

**Usage:**
```bash
./tools/first_run_wizard.sh
```

---

## 📦 Installation & Packaging

### Universal Installer
One-script installation for multiple Linux distributions.

**Features:**
- Auto-detects distribution (Ubuntu, Fedora, Arch)
- Installs system dependencies
- Installs Python packages
- Builds application from source
- Creates systemd user service
- Offers to download models
- Supports custom install prefix

**Usage:**
```bash
./tools/install.sh                      # Install to /usr/local
INSTALL_PREFIX=$HOME/.local ./tools/install.sh   # User install
```

**Supported Distros:**
- Ubuntu / Debian / Pop!_OS / Linux Mint
- Fedora / RHEL / CentOS
- Arch Linux / Manjaro

### Uninstaller
Clean removal with optional user data preservation.

**Features:**
- Stops and removes systemd service
- Removes binaries and tools
- Optional user data removal
- Separate Whisper model handling
- Preserves data by default

**Usage:**
```bash
./tools/uninstall.sh
```

### Configuration Wizard
Post-install environment optimization.

**Features:**
- Environment detection (X11/Wayland, compositor)
- Hotkey configuration guidance
- Model selection and download
- Text injection method selection
- Auto-start configuration

**Usage:**
```bash
./tools/configure.sh
```

---

## 📊 Build System Enhancements

### AddressSanitizer Build
Fast leak and bug detection during development.

**Usage:**
```bash
make -f Makefile.linux asan
./vtt-linux
```

**Detects:**
- Memory leaks
- Use-after-free
- Buffer overflows
- Stack corruption
- Undefined behavior

### Debug Build
Optimized for debugging with full symbols.

**Usage:**
```bash
make -f Makefile.linux debug
gdb ./vtt-linux
```

**Features:**
- `-g` debug symbols
- `-O0` no optimization
- `-DDEBUG` preprocessor flag

---

## 📝 Documentation

- `README.md` - Main user documentation
- `docs/WAYLAND_SETUP.md` - Wayland configuration guide
- `docs/FEATURES.md` - This file
- `CLAUDE.md` - Development guidelines
- `tools/valgrind.supp` - Valgrind suppressions
- Inline code documentation

---

## 🔬 Testing

All features have been tested with:
- ✅ 70+ automated tests (`tools/run_tests.sh`)
- ✅ Manual testing on Ubuntu 24.04
- ✅ Wayland (GNOME) and X11 sessions
- ✅ Memory leak analysis (valgrind + AddressSanitizer)
- ✅ Crash handler validation

---

## 🚀 Future Enhancements

Potential improvements for consideration:

1. **Native Wayland Protocol Support**
   - virtual-keyboard-v1 for text input
   - ext-session-lock-v1 for global shortcuts
   - Layer shell integration

2. **Advanced Features**
   - Streaming transcription (real-time feedback)
   - Custom wake words
   - Voice commands
   - Multi-speaker support

3. **Platform Expansion**
   - Windows support (WASAPI, Windows hotkeys)
   - BSD support

4. **Quality of Life**
   - GUI for all configuration
   - Transcription editing/correction
   - Custom vocabulary
   - Punctuation improvements

---

## 📈 Version History

### Recent Updates

**Stability Sprint:**
- Crash handler with backtraces
- Error notifications
- Transcription timeout enforcement
- Recording indicator
- Comprehensive logging

**Wayland Sprint:**
- Session and compositor detection
- D-Bus keyboard integration
- ydotool/clipboard typing methods
- Wayland setup documentation

**Developer Tools Sprint:**
- Model downloader
- History viewers (CLI + GUI)
- Performance profiler
- Memory leak detector
- System monitor
- Test suite (70+ tests)
- Installation scripts

See `git log` for detailed commit history.

---

## 🤝 Contributing

For development guidelines, see `CLAUDE.md`.

For bug reports and feature requests:
https://github.com/powell-clark/voice-to-text/issues
