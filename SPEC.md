# Voice to Text - Platform Feature Specification

**Version**: 1.0
**Last Updated**: 2025-10-13
**Purpose**: Define uniform feature set across macOS, Linux, and future platforms

---

## 1. Executive Summary

This document defines the official feature specification for Voice to Text across all platforms. It ensures consistency in user experience, behavior, and configuration across macOS, Linux, and any future Windows or other platform implementations.

### Key Principles
- **Uniform UX** - All platforms should have identical feature sets and menu structures
- **Smart Backend** - UI should be simple, backend should be intelligent
- **Settings Persistence** - All user preferences must be saved and restored
- **Graceful Degradation** - Features unavailable on a platform should degrade gracefully

---

## 2. Core Features (All Platforms)

### 2.1 Push-to-Talk Voice Recording

#### Specification
- User presses and holds a hotkey
- Microphone activates and records audio
- User releases hotkey to stop recording
- Audio is transcribed and text is pasted into focused application

#### Recording Limits
- **Maximum Duration**: 300 seconds (5 minutes) on Linux, 10 seconds on macOS (testing)
- **Behavior on Limit**:
  - Desktop notification appears: "Recording limit reached (300s) - release key to transcribe"
  - Recording stops capturing audio but waits for key release
  - Transcription includes "[Truncated - 300s limit]" prefix
  - User can continue holding key, but no additional audio captured

#### Audio Specifications
- **Sample Rate**: 16kHz (Whisper requirement)
- **Channels**: Mono
- **Format**: 16-bit PCM
- **Codec**: WAV for transcription input
- **Buffer Size**: Platform-specific (4KB on macOS, configurable on Linux)

---

### 2.2 Language Selection

#### Specification
**Status: ✅ Implemented on Linux | ❌ Missing on macOS**

User can select between two language modes:

1. **"English only (fastest)"**
   - Optimized for English transcription
   - Backend automatically uses .en model variants when available
   - Faster transcription (English-only models are optimized)
   - Parameter passed to backend: `language="en"`

2. **"Multilingual (99 languages)"**
   - Supports 99+ languages with automatic detection
   - Backend uses multilingual model variants
   - Parameter passed to backend: `language="auto"`
   - Tiny and base models should be disabled (poor accuracy for 99 languages)

#### UI Requirements
- Radio button selection (mutual exclusion)
- Label shows current selection: "Language: English only" or "Language: Multilingual"
- Selection persists across restarts
- Changing language rebuilds model menu to show/disable appropriate models

#### Backend Behavior
- When "English only" selected and model has .en variant (tiny, base, small, medium):
  - Backend appends .en suffix automatically
  - Example: User selects "W small" → backend uses "W small.en"
- When "Multilingual" selected:
  - Backend uses base model name without .en
  - Example: User selects "W small" → backend uses "W small"
- Large model has no .en variant:
  - Always uses "large-v3" with `language="en"` parameter for English

---

### 2.3 Model Selection

#### Specification
**Status: ✅ Implemented on Linux | ⚠️ Partially Implemented on macOS**

#### UI Design
- Menu shows **simple model names** without .en suffixes
- Radio button selection (only one model can be selected)
- Label shows current selection: "Model: CT2 small"

#### Available Models

| Model | Linux | macOS | Size | Speed | Accuracy | Notes |
|-------|-------|-------|------|-------|----------|-------|
| **W tiny** | ✅ | ✅ | ~39MB | Fastest | Low | Disabled in Multilingual mode |
| **W base** | ✅ | ✅ | ~74MB | Very fast | Medium | Disabled in Multilingual mode |
| **W small** | ✅ | ✅ | ~244MB | Fast | Good | **Recommended for most users** |
| **W medium** | ✅ | ✅ | ~769MB | Slow | Very good | Good for accuracy-critical work |
| **W large** | ✅ | ✅ | ~1550MB | Very slow | Best | Always multilingual (no .en variant) |
| **CT2 tiny** | ✅ | ✅ | ~39MB | Fastest | Low | 5-10x faster with GPU |
| **CT2 base** | ✅ | ✅ | ~74MB | Very fast | Medium | Disabled in Multilingual mode |
| **CT2 small** | ✅ | ✅ | ~244MB | Fast | Good | **Recommended for GPU users** |
| **CT2 medium** | ✅ | ✅ | ~769MB | Slow | Very good | Requires ~2GB GPU memory |
| **CT2 large-v3** | ✅ | ✅ | ~1550MB | Very slow | Best | Requires ~4GB GPU memory |

#### Backend Types
- **W models**: whisper.cpp (C++ implementation, no Python required)
  - Benefits: No dependencies, works offline, good for systems without Python
  - Drawbacks: Slower than CTranslate2

- **CT2 models**: CTranslate2/faster-whisper (Python backend)
  - Benefits: 5-10x faster with GPU, supports CUDA acceleration
  - Drawbacks: Requires Python 3.12+, faster-whisper package

#### Model Download Behavior
- Models download automatically on first use
- Download progress shown in status menu
- Models cached in `~/.cache/whisper/` (Linux) or `~/Library/Caches/whisper/` (macOS)
- Failed downloads retry up to 3 times with resume support

---

### 2.4 Microphone Selection

#### Specification
**Status: ✅ Implemented on Linux | ✅ Implemented on macOS**

- User can select input microphone from menu
- "System Default" option always available
- Menu shows all available audio input devices
- Hot-plug detection: menu updates when devices added/removed
  - macOS: CoreAudio property listener (instant notification)
  - Linux: Polling every 3 seconds (PortAudio limitation)
- Selected device persists across restarts
- Label shows current selection: "Microphone: Built-in Microphone"

---

### 2.5 Hotkey Configuration

#### Specification
**Status: ⚠️ Placeholder on both platforms**

#### Current Implementation
- Linux: Hard-coded to Right Alt (keycode 108)
- macOS: Hard-coded to Right Alt/Option (keycode 61)
- Menu shows: "Hotkey: Right Alt" (not editable yet)

#### Target Implementation (Future)
- User can click "Hotkey: Right Alt" to open hotkey capture dialog
- User presses desired key combination
- Supported modifiers: Ctrl, Alt, Shift, Cmd/Super
- Supported keys: Function keys (F1-F12), modifier keys, special keys
- Hotkey persists across restarts
- Validation prevents conflicts with system hotkeys

---

### 2.6 Transcription Settings

#### Specification
**Status: ✅ Implemented on Linux | ⚠️ Partially Implemented on macOS**

Dialog accessible from menu: "Customize Transcription Settings..."

#### Voice Prefix
- Text prepended to every transcription
- Default: `[Voice] `
- User can customize (e.g., `[Speech]`, `<dictation>`, etc.)
- Persists across restarts
- Field: Text input

#### Initial Prompt (Whisper Context)
- Hint text provided to Whisper for better accuracy
- Default: "Male British English speaker. Programming, business and technical terminology with frequent acronyms and spelled letters."
- Maximum 240 characters (Whisper limitation)
- Character counter with color coding:
  - Normal: 0-199 characters
  - Orange: 200-229 characters
  - Red: 230-240 characters (beeps on 240)
- Field: Multi-line text area with scrolling
- "Reset Default" button restores default prompt
- Persists across restarts

---

### 2.7 Logging

#### Specification
**Status: ✅ Implemented on both platforms**

- Toggle: "Logging: On" / "Logging: Off" in menu
- Log files stored in:
  - Linux: `~/.local/share/voice-to-text/vtt.log`
  - macOS: `~/Library/Logs/VTT/vtt.log` or `/tmp/VTT/vtt.log`
- Log rotation: Automatic when file exceeds 10MB
  - Keeps last 3 rotations (vtt.log.1, vtt.log.2, vtt.log.3)
- Menu item: "Show Logs" opens log file in default text editor
- Logging state persists across restarts

#### Log Format
```
[HH:MM:SS] Event description
[10:30:45] Key pressed - starting recording
[10:30:48] Recording saved: /tmp/vtt_recording_12345.wav
[10:30:49] Transcription: Hello world
```

---

## 3. Platform-Specific Implementation

### 3.1 macOS

#### Technology Stack
- **Language**: Objective-C (ARC)
- **Audio**: CoreAudio (AudioQueue API)
- **Hotkey**: CGEventTap (global keyboard monitoring)
- **Text Input**: CGEvent posting (synthesized keystrokes)
- **UI**: NSStatusBar (menu bar app)
- **Transcription**: whisper.cpp (embedded) or faster-whisper (Python CLI)
- **Permissions Required**:
  - Microphone (audio recording)
  - Accessibility (text pasting)
  - Input Monitoring (hotkey detection)

#### File Structure
```
VTT.app/
├── Contents/
│   ├── MacOS/
│   │   └── VTT              # Main executable
│   ├── Resources/
│   │   ├── AppIcon.icns     # App icon
│   │   └── ggml-small.en.bin # Bundled model (optional)
│   └── Info.plist
```

#### Build System
- `Makefile` (GNU Make)
- XCode Command Line Tools required
- Links against: CoreAudio, CoreFoundation, Cocoa, ApplicationServices, AVFoundation, IOKit

---

### 3.2 Linux

#### Technology Stack
- **Language**: C11 (GCC)
- **Audio**: PortAudio (cross-platform audio I/O)
- **Hotkey**: X11 XRecord extension (global keyboard hook)
- **Text Input**: XTest extension (synthesized keystrokes)
- **UI**: GTK3 + libayatana-appindicator3 (system tray)
- **Transcription**: faster-whisper (Python) or whisper.cpp
- **Threading**: pthreads (worker thread for transcription queue)
- **Notifications**: libnotify (desktop notifications)
- **Permissions Required**: None (X11 automatically granted)

#### File Structure
```
/usr/local/bin/vtt-linux          # Executable
~/.config/systemd/user/vtt.service # Systemd service
~/.local/share/voice-to-text/
├── vtt.log                        # Log file
└── settings.conf                  # User settings
~/.cache/whisper/                  # Model cache
└── ggml-small.en.bin
```

#### Build System
- `Makefile.linux` (GNU Make)
- Links against: portaudio, X11, Xtst, gtk-3, ayatana-appindicator3, notify, glib-2.0
- Systemd service for auto-start on login

---

## 4. Settings Persistence

### 4.1 Settings File Format

#### Linux: `~/.local/share/voice-to-text/settings.conf`
```ini
selected_model=CT2 small
selected_language=en
voice_prefix=[Voice]
initial_prompt=Male British English speaker. Programming, business and technical terminology with frequent acronyms and spelled letters.
selected_device_index=-1
logging_enabled=1
hotkey_code=108
hotkey_modifiers=0
```

#### macOS: NSUserDefaults
```objc
selectedModel = "small"
selectedLanguage = "en"  // NEW - not yet implemented
voicePrefix = "[Voice] "
initialPrompt = "Male British English speaker..."
selectedMicrophoneID = 0
loggingEnabled = YES
hotkeyCode = 61
hotkeyModifiers = 0
```

### 4.2 Settings to Persist

| Setting | Type | Default | Platform |
|---------|------|---------|----------|
| `selected_model` | String | `"small"` | Both |
| `selected_language` | String | `"en"` | Linux ✅, macOS ❌ |
| `voice_prefix` | String | `"[Voice] "` | Both |
| `initial_prompt` | String | See defaults | Both |
| `selected_device_index` | Integer | `-1` (default) | Both |
| `logging_enabled` | Boolean | `true` | Both |
| `hotkey_code` | Integer | 108 (Linux), 61 (macOS) | Both (not editable yet) |
| `hotkey_modifiers` | Integer | `0` | Both (not editable yet) |

---

## 5. User Interface Specification

### 5.1 Menu Structure (All Platforms)

```
┌─────────────────────────────────┐
│ 🎤 VTT                          │ ← Status icon (changes with state)
├─────────────────────────────────┤
│ Status: Ready                   │ ← Current state
├─────────────────────────────────┤
│ Language: English only      ▶   │ ← NEW - Language selector
│   ● English only (fastest)      │   ← Radio button (selected)
│   ○ Multilingual (99 languages) │   ← Radio button
├─────────────────────────────────┤
│ Model: CT2 small            ▶   │ ← Model selector (simplified)
│   Whisper.cpp Models:           │
│   ○ W tiny                       │   ← Radio (disabled if Multilingual)
│   ○ W base                       │   ← Radio (disabled if Multilingual)
│   ○ W small                      │   ← Radio
│   ○ W medium                     │   ← Radio
│   ○ W large                      │   ← Radio
│   ─────────────────────          │
│   CTranslate2 Models:            │
│   ○ CT2 tiny                     │   ← Radio (disabled if Multilingual)
│   ○ CT2 base                     │   ← Radio (disabled if Multilingual)
│   ● CT2 small                    │   ← Radio (selected)
│   ○ CT2 medium                   │   ← Radio
│   ○ CT2 large-v3                 │   ← Radio
├─────────────────────────────────┤
│ Microphone: Built-in        ▶   │ ← Microphone selector
│   ● System Default              │   ← Radio (selected)
│   ○ Built-in Microphone         │   ← Radio
│   ○ USB Microphone              │   ← Radio (hot-plug detected)
├─────────────────────────────────┤
│ Hotkey: Right Alt               │ ← Hotkey (future: clickable)
├─────────────────────────────────┤
│ Customize Transcription...      │ ← Opens settings dialog
├─────────────────────────────────┤
│ Logging: On                     │ ← Toggle
│ Show Logs                       │ ← Opens log file
├─────────────────────────────────┤
│ About Voice to Text             │
│ Quit                            │
└─────────────────────────────────┘
```

### 5.2 Status Icon States

| State | macOS | Linux | Description |
|-------|-------|-------|-------------|
| Ready | VTT ✅ | 🎤 (microphone) | Idle, ready to record |
| Recording | VTT 🔴 | 🔴 (red dot) | Recording in progress |
| Processing | VTT ⏳ | ⏳ (hourglass) | Transcribing audio |
| Error | VTT ⚠️ | ⚠️ (warning) | Error state |
| Loading | VTT ⏳ | ⏳ | Loading model |

---

## 6. Behavioral Specifications

### 6.1 Recording Rejection Heuristics

Recordings should be rejected (not transcribed) if:
- **Duration < 0.3 seconds** - Too short, likely accidental keypress
- **Audio RMS < threshold** - Too quiet, no speech detected
- **File size < 10KB** - Empty or corrupt recording

#### Rejection Behavior
- No transcription performed
- Status returns to "Ready"
- Linux: Types feedback message like `[Transcription activated: audio too short]`
- macOS: Silent rejection (no output)

### 6.2 Auto-.en Model Selection (Backend Logic)

When user selects "English only" language mode:

```c
// Pseudocode for backend logic
if (language == "en" && model_has_en_variant(model)) {
    actual_model = model + ".en";
    log("Auto-selected .en model: %s", actual_model);
} else if (language == "auto") {
    actual_model = model; // Use multilingual
} else {
    actual_model = model;
}
```

#### Model Variants Available
| Model | Has .en variant? | English mode | Multilingual mode |
|-------|-----------------|--------------|-------------------|
| tiny | ✅ Yes | tiny.en | tiny |
| base | ✅ Yes | base.en | base |
| small | ✅ Yes | small.en | small |
| medium | ✅ Yes | medium.en | medium |
| large | ❌ No | large-v3 (with lang=en) | large-v3 (with lang=auto) |
| large-v3 | ❌ No | large-v3 (with lang=en) | large-v3 (with lang=auto) |

### 6.3 Model Quality Filtering

When "Multilingual (99 languages)" is selected:
- **Disable** tiny and base models (both W and CT2)
- Reason: Poor accuracy for 99 languages, English-optimized
- User can still see them but cannot select them
- Tooltip (future): "This model is optimized for English only. Switch to English-only mode to use it."

---

## 7. Testing Specifications

### 7.1 Feature Test Matrix

| Feature | Linux | macOS | Test Type |
|---------|-------|-------|-----------|
| Push-to-talk recording | ✅ | ✅ | Manual + Unit |
| Language selection | ✅ | ❌ | Manual |
| Model selection | ✅ | ⚠️ | Manual |
| Model download | ✅ | ✅ | Integration |
| Microphone selection | ✅ | ✅ | Manual |
| Hot-plug detection | ✅ | ✅ | Manual |
| Settings persistence | ✅ | ⚠️ | Unit |
| Recording truncation | ✅ | ✅ | Manual |
| Desktop notifications | ✅ | ✅ | Manual |
| Text pasting | ✅ | ✅ | Integration |
| Logging toggle | ✅ | ✅ | Manual |
| Auto-.en selection | ✅ | ❌ | Unit |

### 7.2 Gherkin Feature Files (Proposed)

```gherkin
Feature: Multi-language Voice Transcription

  Scenario: Transcribe English with English-only mode
    Given VTT is running
    And language is set to "English only"
    And model is set to "CT2 small"
    When user presses and holds Right Alt
    And user speaks "Hello world"
    And user releases Right Alt
    Then text "[Voice] Hello world" appears in focused app
    And backend used model "CT2 small.en"

  Scenario: Transcribe Spanish with Multilingual mode
    Given VTT is running
    And language is set to "Multilingual"
    And model is set to "CT2 small"
    When user presses and holds Right Alt
    And user speaks "Hola mundo"
    And user releases Right Alt
    Then text "[Voice] Hola mundo" appears in focused app
    And backend used model "CT2 small" with language auto-detect

  Scenario: Recording truncation at max length
    Given VTT is running
    And max recording length is 300 seconds
    When user presses and holds Right Alt for 301 seconds
    Then desktop notification appears: "Recording limit reached (300s)"
    And recording stops capturing audio
    When user releases Right Alt
    Then transcription includes prefix "[Truncated - 300s limit]"

  Scenario: Model menu filtering in Multilingual mode
    Given VTT is running
    When user selects "Multilingual" language mode
    Then model menu shows "W tiny" as disabled
    And model menu shows "W base" as disabled
    And model menu shows "W small" as enabled
    And model menu shows "CT2 tiny" as disabled
    And model menu shows "CT2 base" as disabled
```

---

## 8. Platform Parity Checklist

### 8.1 What Linux Has That macOS Needs

| Feature | Linux | macOS | Priority | Effort |
|---------|-------|-------|----------|--------|
| Language selector (English/Multilingual) | ✅ | ❌ | 🔴 High | Medium |
| Simplified model menu (no .en shown) | ✅ | ⚠️ Partial | 🔴 High | Low |
| Auto-.en model selection in backend | ✅ | ❌ | 🔴 High | Low |
| Model filtering (disable tiny/base for multilingual) | ✅ | ❌ | 🟡 Medium | Low |
| Radio buttons for language | ✅ | ❌ | 🔴 High | Low |
| Radio buttons for models | ✅ | ⚠️ Checkmarks | 🟡 Medium | Low |
| Settings persistence (language) | ✅ | ❌ | 🔴 High | Low |

### 8.2 What macOS Has That Linux Should Keep

| Feature | macOS | Linux | Priority | Effort |
|---------|-------|-------|----------|--------|
| Embedded whisper.cpp library | ✅ | ❌ | 🟢 Low | High |
| Model download with resume | ✅ | ⚠️ Basic | 🟡 Medium | Medium |
| Onboarding wizard (permissions) | ✅ | N/A | N/A | N/A |
| Log rotation (10MB limit) | ✅ | ✅ | ✅ Done | - |

---

## 9. Implementation Roadmap

### Phase 1: macOS Parity (Priority)
1. ✅ **Add language selector** (English/Multilingual)
2. ✅ **Simplify model menu** (remove .en suffixes from UI)
3. ✅ **Implement auto-.en backend logic**
4. ✅ **Add model filtering** (disable tiny/base for multilingual)
5. ✅ **Convert checkmarks to radio buttons**
6. ✅ **Persist language setting** to NSUserDefaults
7. ✅ **Update transcribe calls** to pass language parameter

### Phase 2: Cross-Platform Polish
1. ⬜ **Implement hotkey customization** (both platforms)
2. ⬜ **Add tooltips** to disabled menu items
3. ⬜ **Improve error messages** (consistent across platforms)
4. ⬜ **Add model download progress** (percentage, ETA)
5. ⬜ **Implement model verification** (checksum validation)

### Phase 3: Advanced Features
1. ⬜ **Streaming transcription** (real-time as you speak)
2. ⬜ **Transcription history** (clipboard manager)
3. ⬜ **Custom voice prefix per app** (e.g., "[Email]" in Mail)
4. ⬜ **Punctuation restoration** (automatic capitalization)
5. ⬜ **Speaker diarization** (multiple speakers)

---

## 10. Security Considerations

### 10.1 Open-Source Security Benefits

**Why open-source is SAFER for accessibility apps:**
- ✅ Community can audit code for backdoors
- ✅ No hidden malicious behavior possible
- ✅ OS permission systems provide sandboxing
- ✅ Trust through transparency

### 10.2 Required Permissions (Justified)

| Permission | Platform | Why Needed | Risk Level |
|------------|----------|------------|------------|
| Microphone | Both | Record voice for transcription | 🟢 Low |
| Accessibility | macOS | Paste transcribed text | 🟡 Medium |
| Input Monitoring | macOS | Detect hotkey press/release | 🟡 Medium |
| X11 Access | Linux | Keyboard hook and text input | 🟢 Low |

**Note**: All permissions are standard for voice-to-text apps. No network access, no data collection.

---

## 11. Appendix

### A. Model File Naming Conventions

#### Linux (Whisper.cpp)
```
~/.cache/whisper/
├── ggml-tiny.en.bin      # English-only tiny
├── ggml-tiny.bin         # Multilingual tiny
├── ggml-small.en.bin     # English-only small
├── ggml-small.bin        # Multilingual small
├── ggml-large-v3.bin     # Multilingual large (no .en)
```

#### macOS (Whisper.cpp)
```
~/Library/Caches/whisper/
├── ggml-tiny.en.bin
├── ggml-small.en.bin
├── ggml-large-v3.bin
```

#### Both (CTranslate2/faster-whisper)
```
~/.cache/huggingface/hub/
├── models--Systran--faster-whisper-tiny.en/
├── models--Systran--faster-whisper-small.en/
├── models--Systran--faster-whisper-large-v3/
```

### B. Whisper Language Codes

Common language codes for `language` parameter:
- `en` - English (fastest with .en models)
- `es` - Spanish
- `fr` - French
- `de` - German
- `zh` - Chinese
- `ja` - Japanese
- `ko` - Korean
- `auto` - Auto-detect (multilingual models only)

Full list: https://github.com/openai/whisper#available-models-and-languages

### C. Build Commands Reference

#### macOS
```bash
make vendor-whisper    # Download whisper.cpp submodule
make whisper-lib       # Build whisper.cpp library
make complete          # Build VTT.app with bundled model
```

#### Linux
```bash
make -f Makefile.linux         # Build vtt-linux executable
make -f Makefile.linux install # Install to /usr/local/bin
systemctl --user enable vtt    # Enable auto-start
```

---

## 12. Changelog

### Version 1.0 (2025-10-13)
- Initial specification created
- Documented Linux implementation (with multilingual support)
- Documented macOS implementation (without multilingual support)
- Defined platform parity checklist
- Added testing specifications (Gherkin feature files)

---

## Contributors
- Emmanuel Powell-Clark (powell-clark)
- Claude Code (AI assistant)

## License
Apache License 2.0 - See LICENSE file
