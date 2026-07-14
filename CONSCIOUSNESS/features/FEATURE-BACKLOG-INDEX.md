id|priority|kano|description|story_ids|task_ids|doc|last_tested
FEAT-VTT024|p1|performance|Vulkan GPU acceleration on Linux and Windows — universal NVIDIA plus AMD plus Intel without requiring CUDA Toolkit (Linux done, Windows blocked on port)|STORY-VTT010|TASK-VTT025,TASK-VTT088|FEAT-VTT024.md
FEAT-VTT025|p1|performance|Metal GPU acceleration on macOS — Intel Radeon and Apple Silicon both supported (blocked on macOS port)|STORY-VTT010|TASK-VTT025|FEAT-VTT025.md
FEAT-VTT037|p1|performance|Correction dictionary — implemented and unit-tested this session (settings.rs, corrections.rs, main.rs); held at backlog pending human approval (review-gate bug, see card) and real-use verification|STORY-VTT019|TASK-VTT118|FEAT-VTT037.md
FEAT-VTT029|p1|must-have|macOS .app bundle with accessibility permissions flow for Intel Radeon and Apple Silicon|STORY-VTT012|TASK-VTT040,TASK-VTT041,TASK-VTT042,TASK-VTT043|
FEAT-VTT036|p1|must-have|Homebrew tap (powell-clark/voice-to-text-homebrew) has not tracked the v2.x line — Cask is local-dev-only (file:// path), Formula pinned v0.2.0, so Mac users cannot brew install the current version. Blocked on FEAT-VTT029 (.app bundle); once a v2.x .app ships, update Formula+Cask each release|STORY-VTT012||
FEAT-VTT030|p1|must-have|Windows .msi installer with Start Menu integration|STORY-VTT013|TASK-VTT044,TASK-VTT045,TASK-VTT046,TASK-VTT047,TASK-VTT063,TASK-VTT064,TASK-VTT098,TASK-VTT108,TASK-VTT101,TASK-VTT097,TASK-VTT104,TASK-VTT102,TASK-VTT103,TASK-VTT095,TASK-VTT094,TASK-VTT093,TASK-VTT099,TASK-VTT100,TASK-VTT091,TASK-VTT092,TASK-VTT086,TASK-VTT089,TASK-VTT082,TASK-VTT084,TASK-VTT109|
FEAT-VTT031|p2|must-have|GitHub Actions matrix builds producing signed artefacts for all three platforms on every tag|STORY-VTT014|TASK-VTT048,TASK-VTT049|
FEAT-VTT032|p2|delighter|Silero VAD integration — auto-stop on silence and background-noise suppression|STORY-VTT015|TASK-VTT050|
FEAT-VTT033|p2|delighter|GTK settings dialog replacing settings.conf hand-editing|STORY-VTT016|TASK-VTT051,TASK-VTT129|
FEAT-VTT034|p2|performance|Optional CT2 persistent daemon backend for maximum NVIDIA and x86 CPU speed — user-toggleable, whisper-rs remains default|STORY-VTT017|TASK-VTT052,TASK-VTT053,TASK-VTT054|
FEAT-VTT020|p3|performance|Superseded feature record — kept for archaeological reference|STORY-VTT008|TASK-VTT022|
FEAT-VTT021|p3|delighter|Batch file transcription mode via --file flag|STORY-VTT009|TASK-VTT023,TASK-VTT130|
FEAT-VTT018|p3|must-have|Single Rust codebase for Linux macOS and Windows — partial delivery moved to STORY-VTT010|STORY-VTT005|TASK-VTT014,TASK-VTT015,TASK-VTT016,TASK-VTT017,TASK-VTT018,TASK-VTT019,TASK-VTT020|
FEAT-VTT019|p3|must-have|Windows push-to-talk transcription (superseded by FEAT-VTT030)|STORY-VTT007|TASK-VTT021|
