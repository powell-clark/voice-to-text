# Voice to Text - Windows 11 Installation Guide

## System Requirements

- **OS:** Windows 11 (64-bit)
- **RAM:** 4GB minimum, 8GB+ recommended
- **Storage:** 1GB free space (includes Python runtime and default model)
- **Microphone:** Any USB or built-in microphone
- **GPU (Optional):** NVIDIA GPU with CUDA support for 5-10x faster transcription

## Installation

### Method 1: Installer (Recommended)

1. Download `VoiceToText-1.0.0-win64-setup.exe` from [Releases](https://github.com/powell-clark/voice-to-text/releases)
2. Run the installer
3. Check "Start Voice to Text automatically on system boot" (recommended)
4. Click Install
5. On first run, the app will download the default Whisper model (~250MB)

### Method 2: Portable ZIP

1. Download `VoiceToText-1.0.0-win64-portable.zip`
2. Extract to any folder (e.g., `C:\Tools\VoiceToText\`)
3. Run `vtt-windows.exe`
4. To enable auto-start, create a shortcut in your Startup folder

---

## First-Time Setup

### 1. Grant Microphone Permission

Windows will prompt for microphone access on first run:

1. Click "Yes" when prompted
2. Or manually: **Settings → Privacy → Microphone** → Allow `vtt-windows.exe`

### 2. Test Your Microphone

1. Look for the system tray icon (📢)
2. Right-click → **Microphone** → Select your microphone
3. Default is usually correct for built-in mics

### 3. Test Recording

1. Press and hold **Scroll Lock** key
2. Speak: "This is a test"
3. Release **Scroll Lock**
4. Text should appear in your active window

---

## Usage

### Basic Operation

1. The app runs in the system tray (bottom-right corner, look for 📢 icon)
2. **Press and hold Scroll Lock** to record
3. **Speak** your message
4. **Release Scroll Lock** to transcribe
5. Text appears instantly in your active application

### Hotkey Options

**Default:** Scroll Lock (recommended - rarely used key)

**Alternative Keys:**
- Right-click tray icon → **Hotkey** → Select F12, Pause, or other keys
- Currently customization requires editing `%APPDATA%\voice-to-text\settings.conf`
- Full GUI hotkey selector coming in v1.1

---

## Configuration

### Model Selection

**Small model (default)** - Best balance of speed and accuracy

- Right-click tray icon → **Model** → Select:
  - **CT2 tiny** - Fastest, lowest accuracy (testing only)
  - **CT2 base** - Fast, good for simple dictation
  - **CT2 small** ✅ **Recommended** - Best balance
  - **CT2 medium** - Slower, higher accuracy (requires 2GB GPU RAM)
  - **CT2 large-v3** - Slowest, best accuracy (requires 4GB GPU RAM)

### Language Settings

- **English only (fastest)** ✅ Default
  - Optimized for English
  - Faster transcription
  - Better accuracy for English

- **Multilingual (99 languages)**
  - Auto-detects language
  - Supports Chinese, Spanish, French, German, Japanese, Arabic, Hindi, and more
  - Slightly slower

To change: Right-click tray icon → **Language** → Select mode

### Transcription Settings

Right-click → **Customize Transcription...**

- **Voice Prefix:** Text prepended to transcriptions (default: `[Voice] `)
- **Initial Prompt:** Hint for better accuracy
  - Default: "Male British English speaker. Programming, business and technical terminology with frequent acronyms and spelled letters."
  - Customize for your accent/vocabulary
  - Max 240 characters

---

## GPU Acceleration (NVIDIA)

For **5-10x faster transcription** with NVIDIA GPUs:

### Prerequisites

1. NVIDIA GPU (GTX 10-series or newer, RTX recommended)
2. CUDA Toolkit 12.x
3. cuDNN 8.x or 9.x

### Installation Steps

```powershell
# 1. Download CUDA Toolkit 12.x from NVIDIA
# https://developer.nvidia.com/cuda-downloads

# 2. Download cuDNN from NVIDIA (requires free account)
# https://developer.nvidia.com/cudnn

# 3. Extract cuDNN to CUDA directory
# Copy cudnn*64_8.dll to C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.x\bin

# 4. Restart Voice to Text
# Right-click tray icon → Quit, then restart
```

### Verify GPU Acceleration

Open Command Prompt and run:

```batch
cd "C:\Program Files\Voice to Text"
vtt-transcribe.exe --test-gpu
```

Expected output for NVIDIA GPUs:
```
✓ CUDA GPU detected with cuDNN - using GPU acceleration
✓ GPU: NVIDIA GeForce RTX 2060 SUPER
```

If you see "CUDA not available", CUDA/cuDNN is not properly installed.

---

## Troubleshooting

### Issue: No transcription appears

**Solution 1:** Check microphone selection
- Right-click tray icon → **Microphone** → Try different devices
- Windows may select wrong default after plugging/unplugging devices

**Solution 2:** Check Windows microphone permission
- **Settings → Privacy → Microphone** → Ensure `vtt-windows.exe` is allowed

**Solution 3:** Enable logging and check for errors
- Right-click tray icon → **Logging: On**
- Right-click → **Show Logs**
- Look for errors in `%APPDATA%\voice-to-text\vtt.log`

### Issue: Hotkey not working

**Solution 1:** Try different key
- Some keyboards don't have Scroll Lock
- Edit `%APPDATA%\voice-to-text\settings.conf`:
  ```
  hotkey=123  # VK_F12 for F12 key
  ```
- Common virtual key codes:
  - F12: 123
  - Pause: 19
  - Right Alt: 165

**Solution 2:** Check if another app is using the key
- Close other apps that may register global hotkeys
- Try rebooting Windows

### Issue: Text appears garbled or as boxes

**Solution:** Windows uses clipboard paste for Unicode
- This is normal for emoji, Chinese, special characters
- ASCII text uses direct keyboard simulation for speed
- If English text is garbled, report as bug on GitHub

### Issue: Transcription is slow

**Solution 1:** Use smaller model
- Right-click → **Model** → **CT2 tiny** or **CT2 base**

**Solution 2:** Install GPU acceleration (see above)
- RTX 2060: ~0.5s for 10s audio with small model
- CPU only: ~3-5s for 10s audio with small model

**Solution 3:** Disable antivirus realtime scanning (temporarily)
- Some antivirus software slows down AI model loading

### Issue: "Python backend not found"

**Solution:** Reinstall using the installer
- Ensure you're using the official installer, not manually extracted files
- Installer bundles Python runtime; manual extraction may miss files

### Issue: Audio too quiet / "no audio detected"

**Solution:** Increase microphone volume
- **Settings → System → Sound → Input**
- Set microphone volume to 80-100%
- Test with Windows Voice Recorder first to verify mic works

---

## File Locations

- **Executable:** `C:\Program Files\Voice to Text\vtt-windows.exe`
- **Settings:** `%APPDATA%\voice-to-text\settings.conf`
- **Logs:** `%APPDATA%\voice-to-text\vtt.log`
- **Models:** `%USERPROFILE%\.cache\whisper\` (downloaded on demand)
- **Recordings:** `%TEMP%\vtt_recording_*.wav` (temporary, cleaned up automatically)

---

## Uninstallation

### Standard Uninstall

1. **Settings → Apps → Installed apps**
2. Find **Voice to Text**
3. Click **⋮** → **Uninstall**

### Manual Cleanup (optional)

After uninstalling, you can manually delete:

```batch
REM Settings and logs
rmdir /s /q "%APPDATA%\voice-to-text"

REM Downloaded Whisper models (frees ~250MB)
rmdir /s /q "%USERPROFILE%\.cache\whisper"

REM Temporary recordings
del /q "%TEMP%\vtt_recording_*.wav"
```

---

## FAQ

**Q: Does this send my voice to the internet?**
A: No. 100% offline. All transcription happens locally on your PC.

**Q: Why is the first transcription slow?**
A: The Whisper model is being loaded into memory. Subsequent transcriptions are much faster.

**Q: Can I use this in Zoom/Teams/Slack?**
A: Yes! The hotkey works in any application. Text is typed as if you typed it manually.

**Q: Does it work with languages other than English?**
A: Yes, 99+ languages are supported. Change to "Multilingual" mode in settings.

**Q: How accurate is it?**
A: With small model: ~95% accuracy for clear English speech. Large model: ~98% accuracy. Comparable to professional dictation software.

**Q: Can I dictate code?**
A: Yes! Works great for Python, JavaScript, etc. Customize the "Initial Prompt" to include programming keywords.

**Q: Does it work with multiple monitors?**
A: Yes, text appears in whichever window has focus, regardless of monitor.

**Q: Can I change the hotkey to Ctrl+Space or other combinations?**
A: Currently requires editing settings file manually. GUI hotkey customization coming in v1.1.

---

## Support

- **Issues:** https://github.com/powell-clark/voice-to-text/issues
- **Discussions:** https://github.com/powell-clark/voice-to-text/discussions
- **Email:** emmanuel@powellclark.com

---

## Credits

- **whisper.cpp:** Fast C++ implementation of OpenAI Whisper
- **faster-whisper:** 5-10x faster Python implementation using CTranslate2
- **OpenAI Whisper:** State-of-the-art speech recognition models

---

**Made with ❤️ for developers, writers, and anyone tired of typing.**

⭐ Star this repo if it saved your wrists: https://github.com/powell-clark/voice-to-text
