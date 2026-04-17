# Backlog Tasks

id|priority|title|story_ids|epic_id|feature_ids|doc
TASK-VTT024|p0|ADR-0003 approved and committed|STORY-VTT010|EPIC-VTT002|FEAT-VTT022|TASK-VTT024.md
TASK-VTT025|p0|Add whisper-rs 0.16 to Cargo.toml with vulkan (Linux+Windows) and metal (macOS) features; bump crate version to 2.0.0|STORY-VTT010|EPIC-VTT002|FEAT-VTT023,FEAT-VTT024,FEAT-VTT025|TASK-VTT025.md
TASK-VTT026|p0|Write src/whisper.rs — WhisperEngine owns WhisperContext and WhisperState with load_model, transcribe, switch_model|STORY-VTT010|EPIC-VTT002|FEAT-VTT022|TASK-VTT026.md
TASK-VTT027|p0|Rewrite transcription worker in src/main.rs — owns WhisperEngine, receives samples via channel, produces text via channel|STORY-VTT010|EPIC-VTT002|FEAT-VTT022|TASK-VTT027.md
TASK-VTT028|p0|Route raw f32 samples from audio.rs to worker without WAV round-trip; keep WAV write only for debug recordings archive|STORY-VTT010|EPIC-VTT002|FEAT-VTT022|TASK-VTT028.md
TASK-VTT029|p1|Write src/models.rs — GGML download from huggingface.co with sha256 verify and progress notifications|STORY-VTT010|EPIC-VTT002|FEAT-VTT026|TASK-VTT029.md
TASK-VTT030|p1|Simplify model menu — flat list small, medium, large-v3-turbo, large-v3; tray shows Loading model / Ready / Transcribing|STORY-VTT010|EPIC-VTT002|FEAT-VTT022|TASK-VTT030.md
TASK-VTT031|p0|Delete Python backend — transcribe.py, python3 from debian/control, transcribe_ct2 and transcribe_whisper_cpp from Rust|STORY-VTT010|EPIC-VTT002|FEAT-VTT023|TASK-VTT031.md
TASK-VTT032|p1|Delete dead C/ObjC — src/linux/*.c, src/common/*.c, src/macos/*.m (7638 lines); retire Makefile.linux|STORY-VTT010|EPIC-VTT002|FEAT-VTT023|TASK-VTT032.md
TASK-VTT033|p0|Add #[cfg(unix)] guards to singleton_lock and ctrlc_handler so Windows build compiles|STORY-VTT010|EPIC-VTT002|FEAT-VTT022|TASK-VTT033.md
TASK-VTT034|p0|Build release binary, deploy to /usr/bin/vtt-linux, restart service, verify transcription quality against CT2 baseline recordings|STORY-VTT010|EPIC-VTT002|FEAT-VTT022|TASK-VTT034.md
TASK-VTT035|p0|Rewrite debian/rules — replace Makefile.linux invocation with cargo build --release|STORY-VTT011|EPIC-VTT002|FEAT-VTT027|TASK-VTT035.md
TASK-VTT036|p0|Update debian/control — drop python3/pip/cmake/g++/make from Depends; add rustc/cargo/libclang-dev/libssl-dev to Build-Depends|STORY-VTT011|EPIC-VTT002|FEAT-VTT027|TASK-VTT036.md
TASK-VTT037|p1|Write postinst script that downloads ggml-small.en.bin to /usr/share/voice-to-text/models on first install|STORY-VTT011|EPIC-VTT002|FEAT-VTT028|TASK-VTT037.md
TASK-VTT038|p0|Bump debian/changelog to 2.0.0 with explicit note that the PPA now ships the Rust binary|STORY-VTT011|EPIC-VTT002|FEAT-VTT027|TASK-VTT038.md
TASK-VTT039|p1|dput 2.0.0 to the Launchpad PPA, apt install locally, verify end-to-end transcription from the installed package|STORY-VTT011|EPIC-VTT002|FEAT-VTT027|TASK-VTT039.md
TASK-VTT040|p1|cargo bundle macOS .app with metal feature, Info.plist with NSMicrophoneUsageDescription|STORY-VTT012|EPIC-VTT002|FEAT-VTT029|
TASK-VTT041|p1|Verify transcription on 2019 Intel i9 Mac with Radeon Pro 5500M via Metal|STORY-VTT012|EPIC-VTT002|FEAT-VTT029|
TASK-VTT042|p2|Apple Silicon build with coreml + metal features enabled|STORY-VTT012|EPIC-VTT002|FEAT-VTT029|
TASK-VTT043|p2|Apple developer signing + notarisation|STORY-VTT012|EPIC-VTT002|FEAT-VTT029|
TASK-VTT044|p1|Windows singleton — replace flock with CreateMutexW named mutex|STORY-VTT013|EPIC-VTT002|FEAT-VTT030|
TASK-VTT045|p1|Windows signal handling — replace sigwait with SetConsoleCtrlHandler|STORY-VTT013|EPIC-VTT002|FEAT-VTT030|
TASK-VTT046|p1|cargo-wix .msi installer with Start Menu shortcut|STORY-VTT013|EPIC-VTT002|FEAT-VTT030|
TASK-VTT047|p2|Windows Authenticode code signing|STORY-VTT013|EPIC-VTT002|FEAT-VTT030|
TASK-VTT048|p2|GitHub Actions matrix workflow: ubuntu-latest + macos-latest + macos-14 (ARM) + windows-latest|STORY-VTT014|EPIC-VTT002|FEAT-VTT031|
TASK-VTT049|p2|Auto-release on tag push with binaries + .deb + .dmg + .msi attached|STORY-VTT014|EPIC-VTT002|FEAT-VTT031|
TASK-VTT050|p2|Integrate silero-rs VAD for auto-stop on silence and noise filtering|STORY-VTT015|EPIC-VTT002|FEAT-VTT032|
TASK-VTT051|p2|GTK settings dialog replacing settings.conf hand-editing|STORY-VTT016|EPIC-VTT002|FEAT-VTT033|
TASK-VTT052|p2|Design persistent Python daemon protocol (stdin/stdout line-delimited JSON) for optional CT2 backend|STORY-VTT017|EPIC-VTT002|FEAT-VTT034|
TASK-VTT053|p2|Implement transcribe_daemon.py with faster-whisper persistent model and IPC loop|STORY-VTT017|EPIC-VTT002|FEAT-VTT034|
TASK-VTT054|p2|Settings toggle CT2 vs native backend; spawn daemon at startup when enabled; fall back to whisper-rs if daemon crashes|STORY-VTT017|EPIC-VTT002|FEAT-VTT034|
TASK-VTT023|p3|Add --file flag for batch audio transcription|STORY-VTT009|EPIC-VTT002|FEAT-VTT021|
TASK-VTT021|p3|Windows WASAPI/Win32/SendInput (superseded by TASK-VTT044 to TASK-VTT047)|STORY-VTT007|EPIC-VTT002|FEAT-VTT019|
TASK-VTT022|p3|Remove whisper.cpp backend (CANCELLED — ADR-0003 keeps whisper.cpp as sole backend)|STORY-VTT008|EPIC-VTT002|FEAT-VTT020|
