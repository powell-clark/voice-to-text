id|kano|status|description|story_ids|task_ids|doc|last_tested
FEAT-VTT001|must-have|maintained|Push-to-talk voice recording (16kHz mono cpal raw f32 to Whisper worker)|STORY-VTT001|TASK-VTT001,TASK-VTT016|FEAT-VTT001.md|
FEAT-VTT002|must-have|removed|(v2.0.0 ADR-0003) Whisper transcription dual backends — superseded by FEAT-VTT022 and FEAT-VTT023|STORY-VTT001|TASK-VTT002,TASK-VTT031,TASK-VTT032|FEAT-VTT002.md|
FEAT-VTT003|must-have|deprecated|macOS menu-bar app — original Objective-C UI (v0.3.x) replaced by the cross-platform Rust tray in v2.0.0; native Mac UI is wanted and continues as FEAT-VTT029 under DIRECT-VTT003|STORY-VTT001|TASK-VTT003,TASK-VTT020,TASK-VTT032|FEAT-VTT003.md|
FEAT-VTT004|must-have|maintained|Linux system tray with GTK3 and AppIndicator|STORY-VTT001|TASK-VTT004,TASK-VTT019|FEAT-VTT004.md|
FEAT-VTT005|must-have|maintained|Text injection into focused application via XTest on Linux (macOS skeleton exists not distributed)|STORY-VTT001|TASK-VTT005,TASK-VTT018|FEAT-VTT005.md|
FEAT-VTT006|performance|maintained|Multi-language support with 99-language auto-detection|STORY-VTT001|TASK-VTT006|FEAT-VTT006.md|
FEAT-VTT007|must-have|deprecated|Homebrew cask (Python v0.3.x era, delivered) — Homebrew install for macOS is wanted and stays; Rust-era cask tracked as FEAT-VTT036 under DIRECT-VTT003|STORY-VTT002|TASK-VTT007|FEAT-VTT007.md|
FEAT-VTT008|must-have|maintained|APT PPA distribution for Ubuntu (Noble + Jammy)|STORY-VTT002|TASK-VTT008,TASK-VTT039|FEAT-VTT008.md|
FEAT-VTT009|performance|removed|(v2.0.0) CUDA GPU acceleration — superseded by Vulkan via whisper-rs FEAT-VTT024|STORY-VTT001|TASK-VTT002,TASK-VTT031|FEAT-VTT009.md|
FEAT-VTT010|delighter|done|Configurable hotkey voice prefix and initial prompt via settings.conf|STORY-VTT001||FEAT-VTT010.md|
FEAT-VTT011|performance|done|User-configurable initial_prompt passthrough to whisper-rs (single backend as of v2.0.0)|STORY-VTT003|TASK-VTT009|FEAT-VTT011.md|
FEAT-VTT012|must-have|maintained|Clipboard paste via xclip subprocess instead of broken XSetSelectionOwner|STORY-VTT006|TASK-VTT012,TASK-VTT110,TASK-VTT114|FEAT-VTT012.md|
FEAT-VTT013|must-have|maintained|X11 key auto-repeat filtering to prevent spurious recordings|STORY-VTT006|TASK-VTT013|FEAT-VTT013.md|
FEAT-VTT014|performance|done|5-minute max recording duration (was 2 minutes)|STORY-VTT006|TASK-VTT013|FEAT-VTT014.md|
FEAT-VTT015|performance|done|SystemD service inherits DISPLAY from user session (Wayland-ready)|STORY-VTT006||FEAT-VTT015.md|
FEAT-VTT016|must-have|maintained|One-command PPA release script with pre-flight checks and git tagging|STORY-VTT004|TASK-VTT011|FEAT-VTT016.md|
FEAT-VTT017|performance|maintained|large-v3-turbo model support (distil names auto-migrate) replacing obsolete large-v1/v2|STORY-VTT004|TASK-VTT010,TASK-VTT111|FEAT-VTT017.md|
FEAT-VTT022|must-have|maintained|Whisper model loaded once in-process worker thread — sub-second transcription regardless of model size|STORY-VTT010|TASK-VTT024,TASK-VTT026,TASK-VTT027,TASK-VTT028,TASK-VTT030,TASK-VTT033,TASK-VTT034|FEAT-VTT022.md|
FEAT-VTT023|must-have|done|Pure Rust transcription — no Python runtime dependency no pip install no faster-whisper no ctranslate2|STORY-VTT010|TASK-VTT025,TASK-VTT031,TASK-VTT032|FEAT-VTT023.md|
FEAT-VTT026|must-have|maintained|Automatic GGML model download from HuggingFace with sha256 verify and tray progress notifications|STORY-VTT010|TASK-VTT029,TASK-VTT112|FEAT-VTT026.md|
FEAT-VTT027|must-have|maintained|Debian package builds the Rust binary via cargo (not gcc against C sources)|STORY-VTT011|TASK-VTT035,TASK-VTT036,TASK-VTT038,TASK-VTT039,TASK-VTT111|FEAT-VTT027.md|
FEAT-VTT028|must-have|maintained|Default model pre-downloaded via postinst so first-run works offline|STORY-VTT011|TASK-VTT037,TASK-VTT110|FEAT-VTT028.md|
FEAT-VTT035|must-have|maintained|Automated regression testing — cargo unit tests GitHub Actions CI local pre-push hook 72 tests gate every change|STORY-VTT018|TASK-VTT055,TASK-VTT056,TASK-VTT057,TASK-VTT058,TASK-VTT059,TASK-VTT061,TASK-VTT060,TASK-VTT107,TASK-VTT090,TASK-VTT087,TASK-VTT085,TASK-VTT105,TASK-VTT111,TASK-VTT127,TASK-VTT133,TASK-VTT134,TASK-VTT136|FEAT-VTT035.md|
FEAT-VTT038|must-have|maintained|Copy last transcription from the tray menu — recovery net when typed output is lost|STORY-VTT018|TASK-VTT123,TASK-VTT131|FEAT-VTT038.md|
