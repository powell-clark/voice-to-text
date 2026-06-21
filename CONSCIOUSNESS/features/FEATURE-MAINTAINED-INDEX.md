id|kano|description|story_ids|task_ids|doc
FEAT-VTT001|must-have|Push-to-talk voice recording with PortAudio (16kHz mono WAV)|STORY-VTT001|TASK-VTT001|
FEAT-VTT002|must-have|Whisper transcription with dual backends (whisper.cpp and CTranslate2/faster-whisper)|STORY-VTT001|TASK-VTT002|
FEAT-VTT003|must-have|macOS menu bar app with Cocoa native UI|STORY-VTT001|TASK-VTT003|
FEAT-VTT004|must-have|Linux system tray with GTK3 and AppIndicator|STORY-VTT001|TASK-VTT004|
FEAT-VTT005|must-have|Text injection into focused application (XTest on Linux, Accessibility API on macOS)|STORY-VTT001|TASK-VTT005|
FEAT-VTT006|performance|Multi-language support with 99-language auto-detection|STORY-VTT001|TASK-VTT006|
FEAT-VTT007|must-have|Homebrew cask distribution for macOS|STORY-VTT002|TASK-VTT007|
FEAT-VTT008|must-have|APT PPA distribution for Ubuntu (Noble + Jammy)|STORY-VTT002|TASK-VTT008|
FEAT-VTT009|performance|CUDA GPU acceleration with cuDNN auto-detection|STORY-VTT001|TASK-VTT002|
FEAT-VTT010|delighter|Configurable voice prefix, initial prompt, and hotkey|STORY-VTT001||
FEAT-VTT011|performance|User-configurable initial_prompt passthrough to both transcription backends|STORY-VTT003|TASK-VTT009|
FEAT-VTT022|must-have|Whisper model loaded once in-process worker thread — eliminates spawn-per-press, sub-second transcription regardless of model size|STORY-VTT010|TASK-VTT024,TASK-VTT026,TASK-VTT027,TASK-VTT028,TASK-VTT030,TASK-VTT033,TASK-VTT034|FEAT-VTT022.md
FEAT-VTT023|must-have|Pure Rust transcription — no Python runtime dependency, no pip install, no faster-whisper, no ctranslate2|STORY-VTT010|TASK-VTT025,TASK-VTT031,TASK-VTT032|FEAT-VTT023.md
FEAT-VTT026|must-have|Automatic GGML model download from HuggingFace with sha256 verify and tray progress notifications|STORY-VTT010|TASK-VTT029|FEAT-VTT026.md
FEAT-VTT027|must-have|Debian package builds the Rust binary via cargo (not gcc against C sources)|STORY-VTT011|TASK-VTT035,TASK-VTT036,TASK-VTT038,TASK-VTT039|FEAT-VTT027.md
FEAT-VTT028|must-have|Default model pre-downloaded via postinst so first-run works offline|STORY-VTT011|TASK-VTT037|FEAT-VTT028.md
FEAT-VTT035|must-have|Automated regression testing — cargo unit tests, GitHub Actions CI on every push, local pre-push hook, 67 tests gate every change|STORY-VTT018|TASK-VTT055,TASK-VTT056,TASK-VTT057,TASK-VTT058,TASK-VTT059,TASK-VTT061,TASK-VTT060|FEAT-VTT035.md
