# Voice to Text - Model Management Guide

> **⚠️ PARTIALLY STALE — UPDATED: 2025-10-13**
>
> This document describes the v1.x model architecture with a dual CT2 +
> whisper.cpp backend. v2.0 (April 2026) removed the CT2 Python backend
> entirely and consolidated on in-process whisper-rs (GGML models) for
> all platforms. ADR-0003 captures the rewrite rationale.
>
> **Authoritative sources for current model behaviour:**
> - `src/models.rs` — the catalogue of supported GGML models with URLs
>   and sizes (source of truth as of v2.0+)
> - `src/models.rs::resolve_variant` — how English/multilingual toggles
>   pick the right variant
> - `debian/postinst` — which model is pre-downloaded on install
> - `CHANGELOG.md` — what changed per release
>
> Specific staleness below:
> - "CT2 small" menu entries no longer exist; model menu derives from
>   `models::MODELS`
> - `transcribe.c` (old C backend) deleted in v2.0
> - "small.en ~244MB" is wrong; actual size is 466MB (see models.rs)
> - Python-based download logic replaced by in-process `models::ensure`

**Purpose**: Complete explanation of model selection, download, and caching behavior across platforms

---

## 🔄 Complete Model Selection Flow

### 1️⃣ User Selection in Menu

User sees simple, clean names:
```
Language: English only ✓
Model: CT2 small ✓
```

No confusing .en suffixes, no technical details - just the basics.

### 2️⃣ Backend Intelligence (transcribe.c:32-55)

The backend automatically selects the optimal model variant:

```c
// User selected: "CT2 small", Language: "en"
// Backend checks: Does this model have a .en variant?

if (language == "en" && has_en_version(model)) {
    actual_model = "CT2 small.en";  // ← Auto-appends .en
    log("Auto-selected .en model: CT2 small.en (English mode)");
}
```

**Models with .en variants:**
- ✅ **tiny, base, small, medium** → have .en versions (English-optimized)
- ❌ **large, large-v3** → NO .en versions (always multilingual)

**Why .en models?**
- Faster transcription (English-only, skip 98 other languages)
- Better accuracy for English
- Smaller model size in some cases

---

## 🎯 Selection Matrix

This table shows exactly which model file is used for each combination:

| User Selection | Language Mode | Backend Transform | Model File Used |
|----------------|---------------|-------------------|-----------------|
| **W tiny** | English only | `W tiny.en` | `~/.cache/whisper/ggml-tiny.en.bin` |
| **W tiny** | Multilingual | `W tiny` | `~/.cache/whisper/ggml-tiny.bin` |
| **W small** | English only | `W small.en` | `~/.cache/whisper/ggml-small.en.bin` |
| **W small** | Multilingual | `W small` | `~/.cache/whisper/ggml-small.bin` |
| **W large** | English only | `W large-v3` + lang=en | `~/.cache/whisper/ggml-large-v3.bin` |
| **W large** | Multilingual | `W large-v3` + lang=auto | `~/.cache/whisper/ggml-large-v3.bin` |
| **CT2 tiny** | English only | `tiny.en` | HuggingFace: `faster-whisper-tiny.en` |
| **CT2 small** | English only | `small.en` | HuggingFace: `faster-whisper-small.en` |
| **CT2 small** | Multilingual | `small` | HuggingFace: `faster-whisper-small` |
| **CT2 large-v3** | Any | `large-v3` | HuggingFace: `faster-whisper-large-v3` |

---

## 🚀 Two Backend Paths

### Path A: Whisper.cpp (W models) - Lines 60-118

**Technology**: C++ implementation, no Python required

**Process:**
```c
// User selected: "W small", Language: "en"
// Backend transforms: "W small" → "W small.en"

Model file: ~/.cache/whisper/ggml-small.en.bin
Command: whisper-cli -m ggml-small.en.bin --language en audio.wav
```

**Model File Locations (Linux/macOS):**
```
~/.cache/whisper/                 # Linux
~/Library/Caches/whisper/         # macOS

├── ggml-tiny.en.bin              # English-only (~39MB)
├── ggml-tiny.bin                 # Multilingual (~39MB)
├── ggml-base.en.bin              # English-only (~74MB)
├── ggml-base.bin                 # Multilingual (~74MB)
├── ggml-small.en.bin             # English-only (~244MB)
├── ggml-small.bin                # Multilingual (~244MB)
├── ggml-medium.en.bin            # English-only (~769MB)
├── ggml-medium.bin               # Multilingual (~769MB)
└── ggml-large-v3.bin             # Multilingual only (~1550MB)
```

**Naming Convention:**
- Pattern: `ggml-{model_name}{.en}.bin`
- `.en` suffix = English-only optimized
- No suffix = Multilingual (99 languages)

### Path B: CTranslate2 (CT2 models) - Lines 120-152

**Technology**: Python faster-whisper library, GPU-accelerated

**Process:**
```c
// User selected: "CT2 small", Language: "en"
// Backend transforms: "CT2 small" → "small.en"

Python script: transcribe.py audio.wav small.en en
```

**Model File Locations:**
```
~/.cache/huggingface/hub/

├── models--Systran--faster-whisper-tiny.en/
│   └── [model files: model.bin, config.json, etc.]
├── models--Systran--faster-whisper-tiny/
├── models--Systran--faster-whisper-small.en/
├── models--Systran--faster-whisper-small/
├── models--Systran--faster-whisper-medium.en/
├── models--Systran--faster-whisper-medium/
└── models--Systran--faster-whisper-large-v3/
```

**Python Loading:**
```python
# transcribe.py:139
from faster_whisper import WhisperModel

# This line triggers automatic download if not cached:
model = WhisperModel("small.en", device="cuda", compute_type="float16")
```

---

## 📥 Model Download Behavior

### **Linux Implementation**

#### CT2 Models (faster-whisper)
**Current behavior: Silent background download** ❌

```python
# transcribe.py:139
model = WhisperModel("small.en")
# Downloads ~244MB from HuggingFace
# NO progress indication
# NO status updates
# User sees "Transcribing..." for 5-10 minutes on first use
```

**User Experience:**
- ❌ First transcription with new model: Long wait with no feedback
- ❌ No progress bar
- ❌ No percentage shown
- ❌ Just appears to hang

**Improvement needed:** Add progress callbacks from faster-whisper

#### W Models (whisper.cpp)
**Current behavior: Manual download required** ⚠️

```bash
# User must manually run:
cd third_party/whisper.cpp/models
bash download-ggml-model.sh small

# Shows progress:
# small.en: Downloading...
# small.en: 112MB / 244MB (45%)
# small.en: Download complete
```

**User Experience:**
- ✅ Clear progress indication
- ✅ Percentage shown
- ❌ Manual step required (not automatic)
- ❌ User must know to run script

### **macOS Implementation** ✅

**Automatic download with full progress tracking:**

From `VTTDaemon.m:1955-2170`:

```objective-c
// User selects "small" model from menu
// → Checks if model file exists
// → If not: Starts automatic download

// UI Updates:
// Menu: "Downloading small... 45%"
// Status bar: "VTT ⏬" (download icon)
// Progress: "small: 112MB / 244MB (45%)"

// Features:
// ✅ Automatic download triggered by selection
// ✅ Progress percentage shown in menu
// ✅ Status icon changes (⏬ downloading)
// ✅ Retry mechanism (3 attempts)
// ✅ Resume support for interrupted downloads
// ✅ Desktop notification on completion
// ✅ Error handling with user feedback
```

**Download URL:**
```
https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{model}.bin
```

**Download Command:**
```bash
curl -L -C - \  # -C - enables resume
  --progress-bar \
  -o ~/Library/Caches/whisper/ggml-small.en.bin.part \
  https://huggingface.co/.../ggml-small.en.bin

# On success: rename .part → .bin
# On failure: keep .part for resume on retry
```

**User Experience:**
- ✅ Automatic download when model selected
- ✅ Real-time progress updates
- ✅ Visual feedback (icon, status, percentage)
- ✅ Resume support (don't re-download on failure)
- ✅ Success notification
- ✅ Graceful error handling

---

## 🔍 Complete Example: User Session Flow

**Scenario:** User selects "CT2 small" + "English only" and speaks "Hello world"

### Step-by-Step Execution:

```
1. GUI: User clicks "CT2 small"
   → Saved to settings.conf: selected_model=CT2 small

2. GUI: User clicks "English only"
   → Saved to settings.conf: selected_language=en

3. USER PRESSES RIGHT ALT
   → Keyboard hook detects keypress
   → Audio recording starts
   → GUI shows: "Status: Recording..."
   → Icon changes to 🔴

4. USER SPEAKS: "Hello world"
   → Audio buffered to memory (16kHz mono PCM)
   → Max 300 seconds (5 minutes) on Linux

5. USER RELEASES RIGHT ALT
   → Audio stops
   → Saved to: /tmp/vtt_recording_123456789.wav
   → GUI shows: "Status: Loading model..."
   → Icon changes to ⏳

6. transcribe.c:21 called with:
   audio_path = "/tmp/vtt_recording_123456789.wav"
   model = "CT2 small"
   language = "en"

7. transcribe.c:38 checks:
   language == "en" ✓
   "small" has .en version? ✓

8. transcribe.c:50 appends:
   "CT2 small" → "CT2 small.en"
   Log: "Auto-selected .en model: CT2 small.en (English mode)"

9. transcribe.c:60 detects:
   Has "CT2" prefix? ✓
   → Use CTranslate2 backend

10. transcribe.c:134 strips prefix:
    "CT2 small.en" → "small.en"

11. transcribe.c:151 constructs command:
    python3.12 transcribe.py /tmp/vtt_recording_123456789.wav small.en en

12. transcribe.py:139 loads model:
    model = WhisperModel("small.en", device="cuda", compute_type="float16")

    First time? Downloads from HuggingFace (~244MB, 5-10 min)
    Already cached? Loads from ~/.cache/huggingface/hub/ (instant)

13. transcribe.py:143 transcribes:
    segments, info = model.transcribe(
        audio_path,
        language="en",  # Use English language model (faster)
        beam_size=5,
        vad_filter=False,
        word_timestamps=True
    )

14. transcribe.py:160 combines segments:
    text = " ".join([seg.text for seg in segments])
    → "Hello world"

15. transcribe.py:214 prints to stdout:
    print("Hello world")

16. transcribe.c:165 reads stdout:
    buffer = "Hello world"

17. transcribe.c:196 returns:
    return strdup("Hello world")

18. main.c:103 adds voice prefix:
    text = "[Voice] Hello world"

19. main.c:120 types text:
    vtt_typing_type_text(&app->typing, "[Voice] Hello world")
    → Uses XTest to simulate keystrokes
    → Text appears in focused application

20. GUI shows: "Status: Ready"
    Icon changes to ✅

21. Cleanup:
    remove("/tmp/vtt_recording_123456789.wav")
```

**Total time (after first download):**
- Recording: 0-300 seconds (user dependent)
- Model loading: ~100ms (cached)
- Transcription: ~500ms (GPU) or ~5s (CPU)
- Typing: ~50ms per character
- **Total: < 1 second for short recordings** 🚀

---

## 📊 Performance Comparison

### Model Size vs Speed vs Accuracy

| Model | Size | Download Time | Transcription Speed (GPU) | Transcription Speed (CPU) | Accuracy |
|-------|------|---------------|---------------------------|---------------------------|----------|
| **W tiny** | 39MB | 30 sec | ~1s (3s audio) | ~5s | Low (WER ~10%) |
| **W base** | 74MB | 1 min | ~1.5s | ~8s | Medium (WER ~7%) |
| **W small** | 244MB | 3-5 min | ~2s | ~15s | Good (WER ~5%) |
| **W medium** | 769MB | 10-15 min | ~5s | ~45s | Very good (WER ~4%) |
| **W large-v3** | 1550MB | 20-30 min | ~10s | ~90s | Best (WER ~3%) |
| **CT2 tiny** | 39MB | 30 sec | ~0.3s | ~3s | Low (WER ~10%) |
| **CT2 base** | 74MB | 1 min | ~0.4s | ~5s | Medium (WER ~7%) |
| **CT2 small** | 244MB | 3-5 min | **~0.5s** | ~8s | Good (WER ~5%) |
| **CT2 medium** | 769MB | 10-15 min | ~1.5s | ~25s | Very good (WER ~4%) |
| **CT2 large-v3** | 1550MB | 20-30 min | ~3s | ~60s | Best (WER ~3%) |

**Legend:**
- WER = Word Error Rate (lower is better)
- GPU = NVIDIA with CUDA 12.6 + cuDNN
- CPU = INT8 quantization
- Times based on 3-second audio sample

**Recommended:**
- **General use**: CT2 small (best balance)
- **Maximum speed**: CT2 tiny (GPU)
- **Maximum accuracy**: CT2 large-v3
- **Offline/no GPU**: W small
- **Low bandwidth**: W tiny

---

## 🔧 Bundled Models (macOS Only)

### Current Implementation

macOS can bundle ONE model in the app:

```
VTT.app/
├── Contents/
│   ├── MacOS/VTT
│   └── Resources/
│       └── ggml-small.en.bin  # Bundled model (~244MB)
```

**Advantages:**
- ✅ Works immediately after install (no download wait)
- ✅ Useful for demos/testing
- ✅ Faster first launch

**Disadvantages:**
- ❌ Increases app size by ~244MB
- ❌ Only one model can be bundled
- ❌ Users still need to download other models

**Current bundle:** None (models downloaded on-demand)

**Recommendation:** Bundle small.en for best first-impression UX

---

## 🛠️ Improvements Needed

### Linux Priority Improvements

1. **Add progress tracking for CT2 downloads** 🔴 High Priority
   ```python
   # Add to transcribe.py
   from faster_whisper.utils import download_model

   def download_with_progress(model_name):
       print(f"Downloading {model_name}...", file=sys.stderr)
       # Show progress percentage
       # Update GUI status
   ```

2. **Auto-download W models on selection** 🟡 Medium Priority
   ```c
   // Add to gui.c on_model_selected()
   if (!model_file_exists(model)) {
       start_download_with_progress(model);
   }
   ```

3. **Add download resume support** 🟢 Low Priority
   - Keep .part files on failure
   - Resume on retry

4. **Show download status in menu** 🔴 High Priority
   ```
   Status: Downloading small.en... 45%
   Model: small.en ⏬
   ```

### macOS Priority Improvements

1. **Add CT2 model support** 🟡 Medium Priority
   - Currently only supports whisper.cpp
   - Add faster-whisper Python backend
   - 5-10x speedup potential

2. **Match Linux language selector** 🔴 High Priority
   - Add "English only" vs "Multilingual" menu
   - Auto-.en selection logic
   - Model filtering (disable tiny/base for multilingual)

3. **Bundle small.en model** 🟢 Low Priority
   - Better first-run experience
   - ~244MB app size increase

---

## 📚 Model Information Links

### Official Sources

- **Whisper Models**: https://github.com/openai/whisper#available-models-and-languages
- **whisper.cpp**: https://github.com/ggerganov/whisper.cpp
- **faster-whisper**: https://github.com/SYSTRAN/faster-whisper
- **Model Downloads**: https://huggingface.co/ggerganov/whisper.cpp

### Model Specifications

All Whisper models are trained on:
- 680,000 hours of multilingual data
- 99+ languages supported
- Supervised learning on web data
- OpenAI Whisper architecture

**.en models** (English-only):
- Same architecture, English-only training data
- ~30% faster inference
- Better English accuracy
- Available for: tiny, base, small, medium
- NOT available for: large, large-v3

**large-v3**:
- Most accurate model
- Multilingual only (no .en variant)
- Released December 2023
- Improved handling of accents and noise

---

## 🔐 Model Integrity

### Checksum Verification (TODO)

Currently, no checksum verification is performed. This should be added for security:

```bash
# After download, verify SHA256
sha256sum ggml-small.en.bin
# Should match: [official checksum from HuggingFace]
```

**Security risk:** Corrupted or tampered models could produce incorrect transcriptions.

**Recommendation:** Add checksum verification to SPEC.md Phase 2.

---

## 📝 Summary

### Key Takeaways

1. **UI Simplicity**: Menu shows "W small", backend intelligently selects "W small.en" for English
2. **.en = Faster**: English-only models skip 98 other languages → 30% speedup
3. **Two Backends**: whisper.cpp (C++) vs CTranslate2 (Python, 5-10x faster)
4. **Auto-Download**: CT2 automatic (silent), W manual on Linux, both automatic on macOS
5. **No Bundling**: Models too large (244MB-1.5GB), always downloaded
6. **Cache Locations**:
   - Linux W: `~/.cache/whisper/`
   - macOS W: `~/Library/Caches/whisper/`
   - CT2: `~/.cache/huggingface/hub/`

### Platform Differences

| Feature | Linux | macOS |
|---------|-------|-------|
| W models auto-download | ❌ Manual | ✅ Automatic |
| W download progress | ✅ CLI | ✅ GUI |
| CT2 download progress | ❌ Silent | N/A (no CT2) |
| Model bundling | ❌ No | ✅ Optional |
| Download retry | ❌ No | ✅ 3 attempts |
| Resume support | ❌ No | ✅ Yes |

### Recommendations

**For Linux:**
- ✅ Add GUI progress tracking for CT2 downloads
- ✅ Auto-download W models on selection
- ✅ Match macOS UX for downloads

**For macOS:**
- ✅ Add CT2 backend support (5-10x speedup)
- ✅ Add language selector (English/Multilingual)
- ✅ Consider bundling small.en (~244MB)

---

*Last Updated: 2025-10-13*
*Contributors: Emmanuel Powell-Clark, Claude Code*
