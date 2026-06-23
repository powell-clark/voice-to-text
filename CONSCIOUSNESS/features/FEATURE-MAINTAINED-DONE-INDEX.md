id|kano|description|story_ids|task_ids|doc
FEAT-VTT001|must-have|Push-to-talk voice recording (16kHz mono, cpal, raw f32 to Whisper worker)|STORY-VTT001|TASK-VTT001,TASK-VTT016|FEAT-VTT001.md
FEAT-VTT002|must-have|Whisper transcription with dual backends — DONE, superseded by FEAT-VTT022 and FEAT-VTT023 (ADR-0003)|STORY-VTT001|TASK-VTT002,TASK-VTT031,TASK-VTT032|FEAT-VTT002.md
FEAT-VTT003|must-have|macOS menu bar app with Cocoa native UI — DONE, ObjC deleted in v2.0.0; Rust macOS in backlog (FEAT-VTT029)|STORY-VTT001|TASK-VTT003,TASK-VTT020,TASK-VTT032|FEAT-VTT003.md
FEAT-VTT004|must-have|Linux system tray with GTK3 and AppIndicator|STORY-VTT001|TASK-VTT004,TASK-VTT019|FEAT-VTT004.md
FEAT-VTT005|must-have|Text injection into focused application via XTest on Linux (macOS skeleton exists but not distributed)|STORY-VTT001|TASK-VTT005,TASK-VTT018|FEAT-VTT005.md
FEAT-VTT006|performance|Multi-language support with 99-language auto-detection|STORY-VTT001|TASK-VTT006|FEAT-VTT006.md
FEAT-VTT007|must-have|Homebrew cask distribution for macOS — DONE, Python era only; Rust macOS on backlog|STORY-VTT002|TASK-VTT007|FEAT-VTT007.md
FEAT-VTT008|must-have|APT PPA distribution for Ubuntu (Noble + Jammy)|STORY-VTT002|TASK-VTT008,TASK-VTT039|FEAT-VTT008.md
FEAT-VTT009|performance|CUDA GPU acceleration — DONE, superseded by Vulkan via whisper-rs (FEAT-VTT024)|STORY-VTT001|TASK-VTT002,TASK-VTT031|FEAT-VTT009.md
FEAT-VTT010|delighter|Configurable hotkey, voice prefix, and initial prompt via settings.conf|STORY-VTT001||FEAT-VTT010.md
FEAT-VTT011|performance|User-configurable initial_prompt passthrough to whisper-rs (single backend as of v2.0.0)|STORY-VTT003|TASK-VTT009|FEAT-VTT011.md
FEAT-VTT012|must-have|Clipboard paste via xclip subprocess instead of broken XSetSelectionOwner|STORY-VTT006|TASK-VTT012|FEAT-VTT012.md
FEAT-VTT013|must-have|X11 key auto-repeat filtering to prevent spurious recordings|STORY-VTT006|TASK-VTT013|FEAT-VTT013.md
FEAT-VTT014|performance|5-minute max recording duration (was 2 minutes)|STORY-VTT006|TASK-VTT013|FEAT-VTT014.md
FEAT-VTT015|performance|SystemD service inherits DISPLAY from user session (Wayland-ready)|STORY-VTT006||FEAT-VTT015.md
FEAT-VTT016|must-have|One-command PPA release script with pre-flight checks and git tagging|STORY-VTT004|TASK-VTT011|FEAT-VTT016.md
FEAT-VTT017|performance|large-v3-turbo and distil-large-v3 model support replacing obsolete models|STORY-VTT004|TASK-VTT010|FEAT-VTT017.md
FEAT-VTT022|must-have|Whisper model loaded once in-process worker thread — eliminates spawn-per-press, sub-second transcription regardless of model size|STORY-VTT010|TASK-VTT024,TASK-VTT026,TASK-VTT027,TASK-VTT028,TASK-VTT030,TASK-VTT033,TASK-VTT034|FEAT-VTT022.md
FEAT-VTT023|must-have|Pure Rust transcription — no Python runtime dependency, no pip install, no faster-whisper, no ctranslate2|STORY-VTT010|TASK-VTT025,TASK-VTT031,TASK-VTT032|FEAT-VTT023.md
FEAT-VTT026|must-have|Automatic GGML model download from HuggingFace with sha256 verify and tray progress notifications|STORY-VTT010|TASK-VTT029|FEAT-VTT026.md
FEAT-VTT027|must-have|Debian package builds the Rust binary via cargo (not gcc against C sources)|STORY-VTT011|TASK-VTT035,TASK-VTT036,TASK-VTT038,TASK-VTT039|FEAT-VTT027.md
FEAT-VTT028|must-have|Default model pre-downloaded via postinst so first-run works offline|STORY-VTT011|TASK-VTT037|FEAT-VTT028.md
FEAT-VTT035|must-have|Automated regression testing — cargo unit tests, GitHub Actions CI on every push, local pre-push hook, 67 tests gate every change|STORY-VTT018|TASK-VTT055,TASK-VTT056,TASK-VTT057,TASK-VTT058,TASK-VTT059,TASK-VTT061,TASK-VTT060|FEAT-VTT035.md
