# Backlog Tasks

id|priority|title|story_ids|epic_id|feature_ids|doc
TASK-VTT014|p0|Architecture decision: Rust crate selection (cpal, whisper-rs, enigo)|STORY-VTT005|||
TASK-VTT015|p0|Scaffold Rust project with cross-platform build (cargo, CI)|STORY-VTT005||FEAT-VTT018|
TASK-VTT016|p1|Port audio capture to Rust (cpal crate)|STORY-VTT005||FEAT-VTT018|
TASK-VTT017|p1|Port transcription to Rust (whisper-rs bindings)|STORY-VTT005||FEAT-VTT018|
TASK-VTT018|p1|Port keyboard simulation to Rust (enigo/rdev)|STORY-VTT005||FEAT-VTT018|
TASK-VTT019|p1|Port Linux GTK tray to Rust|STORY-VTT005||FEAT-VTT018|
TASK-VTT020|p1|Port macOS menu bar to Rust|STORY-VTT005||FEAT-VTT018|
TASK-VTT021|p1|Windows implementation (WASAPI audio, Win32 tray, SendInput keyboard)|STORY-VTT007||FEAT-VTT019|
TASK-VTT022|p2|Remove whisper.cpp backend and W model menu entries|STORY-VTT008||FEAT-VTT020|
TASK-VTT023|p2|Add --file flag for batch audio transcription|STORY-VTT009||FEAT-VTT021|
