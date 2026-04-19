# Backlog Tasks

id|priority|title|story_ids|epic_id|feature_ids|doc
TASK-VTT056|p0|Monitor v2.0.5 in daily use for 3-5 days — confirm £/é transcriptions type fully in Claude Code TUI, Slack, browsers, terminals; confirm Logs submenu populates on first hover|STORY-VTT018|EPIC-VTT002|FEAT-VTT035|TASK-VTT056.md
TASK-VTT060|p1|Delete superseded Launchpad PPA versions via web UI — keep only latest 2 (v2.0.4 + v2.0.5) to free quota from ~5 GB back toward empty|STORY-VTT018|EPIC-VTT002|FEAT-VTT035|TASK-VTT060.md
TASK-VTT061|p2|Local build-archives/ disk cleanup — 5.7 GB of historical source tarballs and .changes files can be deleted except for the latest release's artefacts|STORY-VTT018|EPIC-VTT002|FEAT-VTT035|TASK-VTT061.md
TASK-VTT034|p0|Build release binary, deploy to /usr/bin/vtt-linux, restart service, verify transcription quality against CT2 baseline recordings|STORY-VTT010|EPIC-VTT002|FEAT-VTT022|TASK-VTT034.md
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
