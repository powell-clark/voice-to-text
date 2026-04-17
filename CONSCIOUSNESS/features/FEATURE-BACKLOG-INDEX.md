# Backlog Features

id|priority|kano|description|story_ids|task_ids|doc
FEAT-VTT022|p0|must-have|Whisper model loaded once in-process worker thread — eliminates spawn-per-press, sub-second transcription regardless of model size|STORY-VTT010|TASK-VTT024,TASK-VTT026,TASK-VTT027,TASK-VTT028,TASK-VTT030,TASK-VTT033,TASK-VTT034|FEAT-VTT022.md
FEAT-VTT023|p0|must-have|Pure Rust transcription — no Python runtime dependency, no pip install, no faster-whisper, no ctranslate2|STORY-VTT010|TASK-VTT025,TASK-VTT031,TASK-VTT032|FEAT-VTT023.md
FEAT-VTT024|p1|performance|Vulkan GPU acceleration on Linux and Windows — universal NVIDIA plus AMD plus Intel without requiring CUDA Toolkit|STORY-VTT010|TASK-VTT025|FEAT-VTT024.md
FEAT-VTT025|p1|performance|Metal GPU acceleration on macOS — Intel Radeon and Apple Silicon both supported|STORY-VTT010|TASK-VTT025|FEAT-VTT025.md
FEAT-VTT026|p1|must-have|Automatic GGML model download from HuggingFace with sha256 verify and tray progress notifications|STORY-VTT010|TASK-VTT029|FEAT-VTT026.md
FEAT-VTT027|p0|must-have|Debian package builds the Rust binary via cargo (not gcc against C sources)|STORY-VTT011|TASK-VTT035,TASK-VTT036,TASK-VTT038,TASK-VTT039|FEAT-VTT027.md
FEAT-VTT028|p1|must-have|Default model pre-downloaded via postinst so first-run works offline|STORY-VTT011|TASK-VTT037|FEAT-VTT028.md
FEAT-VTT029|p1|must-have|macOS .app bundle with accessibility permissions flow for Intel Radeon and Apple Silicon|STORY-VTT012|TASK-VTT040,TASK-VTT041,TASK-VTT042,TASK-VTT043|
FEAT-VTT030|p1|must-have|Windows .msi installer with Start Menu integration|STORY-VTT013|TASK-VTT044,TASK-VTT045,TASK-VTT046,TASK-VTT047|
FEAT-VTT031|p2|must-have|GitHub Actions matrix builds producing signed artefacts for all three platforms on every tag|STORY-VTT014|TASK-VTT048,TASK-VTT049|
FEAT-VTT032|p2|delighter|Silero VAD integration — auto-stop on silence and background-noise suppression|STORY-VTT015|TASK-VTT050|
FEAT-VTT033|p2|delighter|GTK settings dialog replacing settings.conf hand-editing|STORY-VTT016|TASK-VTT051|
FEAT-VTT034|p2|performance|Optional CT2 persistent daemon backend for maximum NVIDIA and x86 CPU speed — user-toggleable, whisper-rs remains default|STORY-VTT017|TASK-VTT052,TASK-VTT053,TASK-VTT054|
FEAT-VTT020|p3|performance|Superseded feature record — kept for archaeological reference|STORY-VTT008|TASK-VTT022|
FEAT-VTT021|p3|delighter|Batch file transcription mode via --file flag|STORY-VTT009|TASK-VTT023|
FEAT-VTT018|p3|must-have|Single Rust codebase for Linux macOS and Windows — partial delivery moved to STORY-VTT010|STORY-VTT005|TASK-VTT014,TASK-VTT015,TASK-VTT016,TASK-VTT017,TASK-VTT018,TASK-VTT019,TASK-VTT020|
FEAT-VTT019|p3|must-have|Windows push-to-talk transcription (superseded by FEAT-VTT030)|STORY-VTT007|TASK-VTT021|
