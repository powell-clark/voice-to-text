id|priority|title|story_ids|directive_id|feature_ids|doc|blocked_by|blocks|assignee|parent_task_id|sequence|expected_duration|story_points
TASK-VTT098|p2|Windows tray Logs submenu — parity with Linux FEAT-VTT004|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT098.md|||||g1
TASK-VTT108|p2|Branded Windows app + installer icon and version info|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT108.md|||||g1b
TASK-VTT101|p3|Windows installer pre-provisions the default model — parity FEAT-VTT028|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT101.md|||||g1a
TASK-VTT104|p4|macOS universal binary (lipo Intel + Apple Silicon)|STORY-VTT013|DIRECT-VTT002|FEAT-VTT030|TASK-VTT104.md|||||g1c
TASK-VTT102|p2|Rename binary — drop the misleading vtt-linux name|STORY-VTT013|DIRECT-VTT002|FEAT-VTT030|TASK-VTT102.md|||||g2
TASK-VTT103|p3|Evaluate unifying Linux GTK tray onto the portable tray (ADR)|STORY-VTT013|DIRECT-VTT002|FEAT-VTT030|TASK-VTT103.md|||||g2a
TASK-VTT095|p2|Update mechanism — how Windows/macOS users get new versions|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT095.md|||||g3
TASK-VTT056|p3|Monitor current release in daily use for 3-5 days — confirm £/é transcriptions type fully in Claude Code TUI, Slack, browsers, terminals; confirm Logs submenu populates on first hover (was scoped to v2.0.5, now superseded by v2.3.7)|STORY-VTT018|DIRECT-VTT002|FEAT-VTT035|TASK-VTT056.md|||||b0
TASK-VTT047|p1|Windows Authenticode code signing|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT047.md|TASK-VTT064||||b3
TASK-VTT040|p4|cargo bundle macOS .app with metal feature, Info.plist with NSMicrophoneUsageDescription|STORY-VTT012|DIRECT-VTT003|FEAT-VTT029|TASK-VTT040.md|||||b4
TASK-VTT041|p4|Verify transcription on 2019 Intel i9 Mac with Radeon Pro 5500M via Metal|STORY-VTT012|DIRECT-VTT003|FEAT-VTT029|TASK-VTT041.md|TASK-VTT040||||b5
TASK-VTT042|p4|Apple Silicon build with coreml + metal features enabled|STORY-VTT012|DIRECT-VTT003|FEAT-VTT029|TASK-VTT042.md|TASK-VTT040||||b6
TASK-VTT043|p4|Apple developer signing + notarisation|STORY-VTT012|DIRECT-VTT003|FEAT-VTT029|TASK-VTT043.md|TASK-VTT041,TASK-VTT042||||b7
TASK-VTT048|p2|GitHub Actions matrix workflow: ubuntu-latest + macos-latest + macos-14 (ARM) + windows-latest|STORY-VTT014|DIRECT-VTT002|FEAT-VTT031|TASK-VTT048.md|TASK-VTT047,TASK-VTT041,TASK-VTT042||||b8
TASK-VTT049|p2|Auto-release on tag push with binaries + .deb + .dmg + .msi attached|STORY-VTT014|DIRECT-VTT002|FEAT-VTT031|TASK-VTT049.md|TASK-VTT048||||b9
TASK-VTT054|p2|Settings toggle CT2 vs native backend; spawn daemon at startup when enabled; fall back to whisper-rs if daemon crashes|STORY-VTT017|DIRECT-VTT002|FEAT-VTT034|TASK-VTT054.md|TASK-VTT053||||bf
TASK-VTT075|p3|PGPS validation error wall is overwhelming for new users — needs friendlier guidance and auto-fix path||||TASK-VTT075.md|||||a2||
TASK-VTT076|p3|Telemetry opt-in should be default at install — 30-day retention agreed at walkthrough, no opt-out required||||TASK-VTT076.md|||||b2b||
TASK-VTT120|p2|Review-gate kano override never resolves (getFeatureKano bug) — OPERATOR-DECISION-PENDING (fix lives in the consciousness plugin repo, not voice-to-text)||DIRECT-VTT002||TASK-VTT120.md|||||b2i||
TASK-VTT125|p2|Re-register the macOS Intel self-hosted runner||DIRECT-VTT003||TASK-VTT125.md|||Emmanuel||b2m
TASK-VTT128|p3|Unscoped validation stall — needs operator review||||TASK-VTT128.md|||||b2n||
TASK-VTT129|p2|Tray Microphone submenu — device picker UX|STORY-VTT016|DIRECT-VTT005|FEAT-VTT033|TASK-VTT129.md|TASK-VTT062||||bo
TASK-VTT130|p3|--file multi-format decode + long-file chunking|STORY-VTT009|DIRECT-VTT002|FEAT-VTT021|TASK-VTT130.md|TASK-VTT023||||bp
TASK-VTT138|p3|Spike hotkey dialog on portable tray|STORY-VTT013|DIRECT-VTT002|FEAT-VTT030|TASK-VTT138.md|||||b2s||
TASK-VTT139|p3|Portable tray parity — settings and About dialogs|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT139.md|||||b2t||
TASK-VTT140|p2|Agent-run cost tracking and seat selection for PGPS execution||DIRECT-VTT002||TASK-VTT140.md|||||b2u||
TASK-VTT143|p4|Rule 63 maintained-directive false positive||DIRECT-VTT002||TASK-VTT143.md|||||b2v||
TASK-VTT151|p3|Spectral subtraction if rumble filtering proves insufficient|STORY-VTT015|DIRECT-VTT002||TASK-VTT151.md|||||b2wGVx||
TASK-VTT164|p3|accuracy-compare.sh --corpus override is neutralised by default-equality check|STORY-VTT018|DIRECT-VTT002||TASK-VTT164.md|||||b2yK0P||
TASK-VTT165|p3|Upstream: validator + VERIFICATION verdict for per-platform ACs|STORY-VTT018|DIRECT-VTT005||TASK-VTT165.md|||||b2znCT||
TASK-VTT166|p3|Migrate remaining parity cards, retire docs/PLATFORM-PARITY.md|STORY-VTT018|DIRECT-VTT005||TASK-VTT166.md|||||b30hfs||
