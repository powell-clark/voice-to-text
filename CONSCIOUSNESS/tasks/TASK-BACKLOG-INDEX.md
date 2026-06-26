id|priority|title|story_ids|epic_id|feature_ids|doc|blocked_by|blocks|assignee|parent_task_id|sequence
TASK-VTT098|p2|Windows tray Logs submenu — parity with Linux FEAT-VTT004|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT098.md|||||g1
TASK-VTT101|p3|Windows installer pre-provisions the default model — parity FEAT-VTT028|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT101.md|||||g1a
TASK-VTT097|p3|models::system_cache() returns a Linux path on Windows|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT097.md|||||g1b
TASK-VTT104|p3|macOS universal binary (lipo Intel + Apple Silicon)|STORY-VTT013|DIRECT-VTT002|FEAT-VTT030|TASK-VTT104.md|||||g1c
TASK-VTT102|p2|Rename binary — drop the misleading vtt-linux name|STORY-VTT013|DIRECT-VTT002|FEAT-VTT030|TASK-VTT102.md|||||g2
TASK-VTT103|p3|Evaluate unifying Linux GTK tray onto the portable tray (ADR)|STORY-VTT013|DIRECT-VTT002|FEAT-VTT030|TASK-VTT103.md|||||g2a
TASK-VTT095|p2|Update mechanism — how Windows/macOS users get new versions|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT095.md|||||g3
TASK-VTT056|p0|Monitor v2.0.5 in daily use for 3-5 days — confirm £/é transcriptions type fully in Claude Code TUI, Slack, browsers, terminals; confirm Logs submenu populates on first hover|STORY-VTT018|DIRECT-VTT002|FEAT-VTT035|TASK-VTT056.md|||||b0
TASK-VTT070|p1|Vendor refresh — upgrade rustls-webpki to >=0.103.13 to close RUSTSEC-2026-0104||DIRECT-VTT002||TASK-VTT070.md|||||b1
TASK-VTT060|p1|Delete superseded Launchpad PPA versions via web UI — keep only latest 2 (v2.0.4 + v2.0.5) to free quota from ~5 GB back toward empty|STORY-VTT018|DIRECT-VTT002|FEAT-VTT035|TASK-VTT060.md|||||b2
TASK-VTT064|p1|Windows ARM64 Snapdragon CPU build for Kyle|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT064.md|TASK-VTT082||||b2a
TASK-VTT047|p1|Windows Authenticode code signing|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT047.md|TASK-VTT064||||b3
TASK-VTT040|p1|cargo bundle macOS .app with metal feature, Info.plist with NSMicrophoneUsageDescription|STORY-VTT012|DIRECT-VTT003|FEAT-VTT029|TASK-VTT040.md|||||b4
TASK-VTT041|p2|Verify transcription on 2019 Intel i9 Mac with Radeon Pro 5500M via Metal|STORY-VTT012|DIRECT-VTT003|FEAT-VTT029|TASK-VTT041.md|TASK-VTT040||||b5
TASK-VTT042|p2|Apple Silicon build with coreml + metal features enabled|STORY-VTT012|DIRECT-VTT003|FEAT-VTT029|TASK-VTT042.md|TASK-VTT040||||b6
TASK-VTT043|p2|Apple developer signing + notarisation|STORY-VTT012|DIRECT-VTT003|FEAT-VTT029|TASK-VTT043.md|TASK-VTT041,TASK-VTT042||||b7
TASK-VTT048|p2|GitHub Actions matrix workflow: ubuntu-latest + macos-latest + macos-14 (ARM) + windows-latest|STORY-VTT014|DIRECT-VTT002|FEAT-VTT031|TASK-VTT048.md|TASK-VTT047,TASK-VTT041,TASK-VTT042||||b8
TASK-VTT049|p2|Auto-release on tag push with binaries + .deb + .dmg + .msi attached|STORY-VTT014|DIRECT-VTT002|FEAT-VTT031|TASK-VTT049.md|TASK-VTT048||||b9
TASK-VTT062|p2|Wire Settings.selected_device_index through to audio::Audio::new() so users with multiple mics can pick one from the tray — field is saved to settings.conf but currently unused; audio.rs hardcodes default_input_device()||DIRECT-VTT002||TASK-VTT062.md|||||ba
TASK-VTT050|p2|Integrate silero-rs VAD for auto-stop on silence and noise filtering|STORY-VTT015|DIRECT-VTT002|FEAT-VTT032|TASK-VTT050.md|||||bb
TASK-VTT051|p2|GTK settings dialog replacing settings.conf hand-editing|STORY-VTT016|DIRECT-VTT002|FEAT-VTT033|TASK-VTT051.md|||||bc
TASK-VTT052|p2|Design persistent Python daemon protocol (stdin/stdout line-delimited JSON) for optional CT2 backend|STORY-VTT017|DIRECT-VTT002|FEAT-VTT034|TASK-VTT052.md|||||bd
TASK-VTT053|p2|Implement transcribe_daemon.py with faster-whisper persistent model and IPC loop|STORY-VTT017|DIRECT-VTT002|FEAT-VTT034|TASK-VTT053.md|TASK-VTT052||||be
TASK-VTT054|p2|Settings toggle CT2 vs native backend; spawn daemon at startup when enabled; fall back to whisper-rs if daemon crashes|STORY-VTT017|DIRECT-VTT002|FEAT-VTT034|TASK-VTT054.md|TASK-VTT053||||bf
TASK-VTT023|p3|Add --file flag for batch audio transcription|STORY-VTT009|DIRECT-VTT002|FEAT-VTT021|TASK-VTT023.md|||||bg
TASK-VTT021|p5|Windows WASAPI/Win32/SendInput (superseded by TASK-VTT044 to TASK-VTT047)|STORY-VTT007|DIRECT-VTT004|FEAT-VTT019|TASK-VTT021.md|||||bh
TASK-VTT022|p3|Remove whisper.cpp backend (CANCELLED — ADR-0003 keeps whisper.cpp as sole backend)|STORY-VTT008|DIRECT-VTT002|FEAT-VTT020|TASK-VTT022.md|||||bi
TASK-VTT072|p3|Unknown command /consciousness:sequence — is it part of pgps or a separate command?||||TASK-VTT072.md|||||a0
TASK-VTT073|p3|Can't connect to APGPS — ap.consciousness.london unreachable despite being authenticated||||TASK-VTT073.md|||||a1
TASK-VTT075|p3|PGPS validation error wall is overwhelming for new users — needs friendlier guidance and auto-fix path||||TASK-VTT075.md|||||a2||
TASK-VTT076|p3|Telemetry opt-in should be default at install — 30-day retention agreed at walkthrough, no opt-out required||||TASK-VTT076.md|||||b2b||
TASK-VTT079|p2|Upstream — feature-index status column splits PGPS display but fails validator (dist build inconsistency)||DIRECT-VTT002||TASK-VTT079.md|||||b3
TASK-VTT080|p3|Features and testing are not connected — add per-feature test-status tracking (last_tested field and/or verification reviews)||||TASK-VTT080.md|||||b2c||
TASK-VTT083|p3|Story and task indexes still use 'epic_id' column header instead of 'directive_id' (naming-precision residue; drives fk-asymmetry validator warnings)||||TASK-VTT083.md|||||b2e||
