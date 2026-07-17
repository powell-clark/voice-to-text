id|title|story_ids|directive_id|feature_ids|doc|expected_duration|story_points
TASK-VTT127|CI contract gate for regression and release workflow parity|STORY-VTT018|DIRECT-VTT002|FEAT-VTT035|TASK-VTT127.md
TASK-VTT106|cargo audit red — RUSTSEC-2026-0186 memmap2 unsound|||||TASK-VTT106.md
TASK-VTT118|Correction dictionary for commonly mistranscribed words|STORY-VTT019|DIRECT-VTT002|FEAT-VTT037|TASK-VTT118.md
TASK-VTT119|cargo audit red — RUSTSEC-2026-0190 anyhow unsound||DIRECT-VTT002||TASK-VTT119.md
TASK-VTT107|Smooth release CI/CD — never hang or fail a publish|STORY-VTT018|DIRECT-VTT002|FEAT-VTT035|TASK-VTT107.md
TASK-VTT094|Start on login / autostart — Windows (HKCU Run) + tray toggle|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT094.md
TASK-VTT093|Windows/macOS tray icon never changes state (recording/processing)|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT093.md
TASK-VTT099|Windows — set clipboard as Ctrl+V fallback after typing|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT099.md
TASK-VTT100|Windows — suppress hotkey auto-repeat (rdev)|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT100.md
TASK-VTT091|Windows tray icon has no menu — pump the Win32 message loop|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT091.md
TASK-VTT092|Windows typing drops characters and reorders text|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT092.md
TASK-VTT086|Fix portable tray model submenu — legacy names don't match catalogue|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT086.md
TASK-VTT089|Windows tray app pops a console window — build windowed|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT089.md
TASK-VTT090|Release pipeline — CHANGELOG-driven notes, macOS binary, download links|STORY-VTT018|DIRECT-VTT002|FEAT-VTT035|TASK-VTT090.md
TASK-VTT088|Enable Vulkan GPU acceleration on the Windows build|STORY-VTT010|DIRECT-VTT004|FEAT-VTT024|TASK-VTT088.md
TASK-VTT082|Build and smoke-test VTT on Windows x86-64 hardware|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT082.md
TASK-VTT087|Windows automated test suite — E2E transcription + expanded unit tests|STORY-VTT018|DIRECT-VTT002|FEAT-VTT035|TASK-VTT087.md
TASK-VTT044|Windows singleton — replace flock with CreateMutexW named mutex|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|
TASK-VTT045|Windows signal handling — replace sigwait with SetConsoleCtrlHandler|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|
TASK-VTT046|cargo-wix .msi installer with Start Menu shortcut|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|
TASK-VTT069|macOS CI job, Rust release workflow, stale ObjC scripts removed||DIRECT-VTT002||
TASK-VTT067|Root-level scripts tidy and RELEASE_SETUP rewrite||DIRECT-VTT002||
TASK-VTT066|Repo structure tidy — remove empty stale CONSCIOUSNESS dirs and build artifacts||DIRECT-VTT002||TASK-VTT066.md
TASK-VTT001|Implement PortAudio recording with quality filters|STORY-VTT001|DIRECT-VTT001|FEAT-VTT001|
TASK-VTT002|Integrate faster-whisper with CUDA GPU acceleration|STORY-VTT001|DIRECT-VTT001|FEAT-VTT002,FEAT-VTT009|
TASK-VTT003|Create macOS menu bar app with Cocoa/Objective-C|STORY-VTT001|DIRECT-VTT001|FEAT-VTT003|
TASK-VTT004|Create Linux system tray with GTK3|STORY-VTT001|DIRECT-VTT001|FEAT-VTT004|
TASK-VTT005|Implement text injection via XTest (Linux) and Accessibility API (macOS)|STORY-VTT001|DIRECT-VTT001|FEAT-VTT005|
TASK-VTT006|Add multi-language support with auto-detection|STORY-VTT001|DIRECT-VTT001|FEAT-VTT006|
TASK-VTT007|Create Homebrew cask formula for macOS distribution|STORY-VTT002|DIRECT-VTT001|FEAT-VTT007|
TASK-VTT008|Create APT PPA for Linux distribution|STORY-VTT002|DIRECT-VTT001|FEAT-VTT008|
TASK-VTT009|Wire initial_prompt setting through to both transcription backends with shell-escaping|STORY-VTT003|DIRECT-VTT001|FEAT-VTT011|
TASK-VTT010|Add large-v3-turbo and distil-large-v3 models, trim obsolete models from menu|STORY-VTT004||FEAT-VTT017|
TASK-VTT011|Create hardened PPA release script with pre-flight checks and git tagging|STORY-VTT004||FEAT-VTT016|
TASK-VTT012|Fix clipboard paste via xclip subprocess replacing broken XSetSelectionOwner|STORY-VTT006||FEAT-VTT012|
TASK-VTT013|Add X11 key auto-repeat filtering and increase max recording to 5 minutes|STORY-VTT006||FEAT-VTT013,FEAT-VTT014|
TASK-VTT014|Architecture decision (ADR-0003): whisper-rs in-process replaces CT2-Python subprocess|STORY-VTT005|DIRECT-VTT002|FEAT-VTT018|
TASK-VTT015|Scaffold Rust project with cross-platform build (cargo, CI)|STORY-VTT005|DIRECT-VTT002|FEAT-VTT018|
TASK-VTT016|Port audio capture to Rust (cpal crate)|STORY-VTT005|DIRECT-VTT002|FEAT-VTT018,FEAT-VTT001|
TASK-VTT017|Port transcription to Rust (initial port via Python subprocess; replaced by TASK-VTT026 in STORY-VTT010)|STORY-VTT005|DIRECT-VTT002|FEAT-VTT018|
TASK-VTT018|Port keyboard simulation to Rust (enigo/rdev)|STORY-VTT005|DIRECT-VTT002|FEAT-VTT018,FEAT-VTT005|
TASK-VTT019|Port Linux GTK tray to Rust|STORY-VTT005|DIRECT-VTT002|FEAT-VTT018,FEAT-VTT004|
TASK-VTT020|Port macOS menu bar to Rust (skeleton)|STORY-VTT005|DIRECT-VTT002|FEAT-VTT018,FEAT-VTT003|
TASK-VTT024|ADR-0003 approved and committed|STORY-VTT010|DIRECT-VTT002|FEAT-VTT022|TASK-VTT024.md
TASK-VTT025|Add whisper-rs 0.16 to Cargo.toml with vulkan (Linux+Windows) and metal (macOS) features; bump crate version to 2.0.0|STORY-VTT010|DIRECT-VTT002|FEAT-VTT023,FEAT-VTT024,FEAT-VTT025|TASK-VTT025.md
TASK-VTT026|Write src/whisper.rs — WhisperEngine owns WhisperContext and WhisperState with load_model, transcribe, switch_model|STORY-VTT010|DIRECT-VTT002|FEAT-VTT022|TASK-VTT026.md
TASK-VTT027|Rewrite transcription worker in src/main.rs — owns WhisperEngine, receives samples via channel, produces text via channel|STORY-VTT010|DIRECT-VTT002|FEAT-VTT022|TASK-VTT027.md
TASK-VTT028|Route raw f32 samples from audio.rs to worker without WAV round-trip; keep WAV write only for debug recordings archive|STORY-VTT010|DIRECT-VTT002|FEAT-VTT022|TASK-VTT028.md
TASK-VTT029|Write src/models.rs — GGML download from huggingface.co with sha256 verify and progress notifications|STORY-VTT010|DIRECT-VTT002|FEAT-VTT026|TASK-VTT029.md
TASK-VTT030|Simplify model menu — flat list small, medium, large-v3-turbo, large-v3; tray shows Loading model / Ready / Transcribing|STORY-VTT010|DIRECT-VTT002|FEAT-VTT022|TASK-VTT030.md
TASK-VTT031|Delete Python backend — transcribe.py, python3 from debian/control, transcribe_ct2 and transcribe_whisper_cpp from Rust|STORY-VTT010|DIRECT-VTT002|FEAT-VTT023,FEAT-VTT002,FEAT-VTT009|TASK-VTT031.md
TASK-VTT032|Delete dead C/ObjC — src/linux/*.c, src/common/*.c, src/macos/*.m (7638 lines); retire Makefile.linux|STORY-VTT010|DIRECT-VTT002|FEAT-VTT023,FEAT-VTT002,FEAT-VTT003|TASK-VTT032.md
TASK-VTT033|Add #[cfg(unix)] guards to singleton_lock and ctrlc_handler so Windows build compiles|STORY-VTT010|DIRECT-VTT002|FEAT-VTT022|TASK-VTT033.md
TASK-VTT035|Rewrite debian/rules — replace Makefile.linux invocation with cargo build --release|STORY-VTT011|DIRECT-VTT002|FEAT-VTT027|TASK-VTT035.md
TASK-VTT036|Update debian/control — drop python3/pip/cmake/g++/make from Depends; add rustc/cargo/libclang-dev/libssl-dev to Build-Depends|STORY-VTT011|DIRECT-VTT002|FEAT-VTT027|TASK-VTT036.md
TASK-VTT037|Write postinst script that downloads ggml-small.en.bin to /usr/share/voice-to-text/models on first install|STORY-VTT011|DIRECT-VTT002|FEAT-VTT028|TASK-VTT037.md
TASK-VTT038|Bump debian/changelog to 2.0.0 with explicit note that the PPA now ships the Rust binary|STORY-VTT011|DIRECT-VTT002|FEAT-VTT027|TASK-VTT038.md
TASK-VTT055|Release v2.0.5 — £/é typing fix + Logs submenu fix — via release-manager with pbuilder hard gate|STORY-VTT018|DIRECT-VTT002|FEAT-VTT035|TASK-VTT055.md
TASK-VTT057|Cargo unit tests for pure logic — 20 tests added across settings, main::compose_final_text, tray::format_log_label|STORY-VTT018|DIRECT-VTT002|FEAT-VTT035|TASK-VTT057.md
TASK-VTT058|GitHub Actions CI — fmt + clippy + test + build on every push and PR, ubuntu-24.04|STORY-VTT018|DIRECT-VTT002|FEAT-VTT035|TASK-VTT058.md
TASK-VTT059|Local git pre-push hook matching CI — installed via scripts/git-hooks/install.sh|STORY-VTT018|DIRECT-VTT002|FEAT-VTT035|TASK-VTT059.md
TASK-VTT061|Local build-archives/ disk cleanup — reduced from 5.7 GB to 24 MB, kept only 2.0.4 + 2.0.5 artefacts|STORY-VTT018|DIRECT-VTT002|FEAT-VTT035|
TASK-VTT063|Windows x86-64 compile-green and CI build job|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT063.md
TASK-VTT065|PGPS neurologist repair — EPIC→DIRECT rename, feature index migration, FK heals, schema.json fix, repo tidy||DIRECT-VTT002|||
TASK-VTT034|Build release binary, deploy to /usr/bin/vtt-linux, restart service, verify transcription quality — completed as part of v2.0.0 release|STORY-VTT010|DIRECT-VTT002|FEAT-VTT022|TASK-VTT034.md
TASK-VTT039|dput 2.0.0 to the Launchpad PPA, apt install locally, verify end-to-end transcription — completed as part of v2.0.0 release|STORY-VTT011|DIRECT-VTT002|FEAT-VTT027,FEAT-VTT008|TASK-VTT039.md
TASK-VTT071|Repo organisation — packaging/ layout and CLAUDE.md rewrite||DIRECT-VTT002||
TASK-VTT068|Claude Code tooling parity with Consciousness||DIRECT-VTT002||
TASK-VTT074|Backlog grooming — sequence keys, dependency edges, detail cards||DIRECT-VTT002||TASK-VTT074.md
TASK-VTT077|Feature card audit — maintained vs done split, full ACs, verification markers||DIRECT-VTT002||TASK-VTT077.md
TASK-VTT078|Feature terminal index conformance — single folder, precise card statuses||DIRECT-VTT002||TASK-VTT078.md
TASK-VTT081|Split directives by platform; ready Windows handoff||DIRECT-VTT002||TASK-VTT081.md
TASK-VTT084|Pre-stage Windows handoff from Linux — verify CI green, add smoke-test script|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT084.md
TASK-VTT085|Upgrade quinn-proto to >=0.11.15 — close RUSTSEC-2026-0185|STORY-VTT018|DIRECT-VTT002|FEAT-VTT035|TASK-VTT085.md
TASK-VTT105|Claude PR-automation workflow parity|STORY-VTT018|DIRECT-VTT002|FEAT-VTT035|TASK-VTT105.md
TASK-VTT110|Harden Debian packaging — xclip dep, postinst sha256 verify, postrm purge cleanup||DIRECT-VTT002|FEAT-VTT012,FEAT-VTT028|
TASK-VTT111|Correct stale feature-card acceptance criteria + add README CI badge||DIRECT-VTT002|FEAT-VTT017,FEAT-VTT027,FEAT-VTT035|
TASK-VTT113|Consciousness health repair and feature review backfill|STORY-VTT018|DIRECT-VTT002||
TASK-VTT121|Audio capture must recover when input device changes or suspends||DIRECT-VTT005||TASK-VTT121.md
TASK-VTT122|Service restart semantics — Restart=always + tray Quit stops the unit||DIRECT-VTT002||TASK-VTT122.md
TASK-VTT126|PPA source tarball must exclude .claude/ and chats/||DIRECT-VTT002||TASK-VTT126.md
TASK-VTT070|Vendor refresh — rustls-webpki security upgrade||DIRECT-VTT002||TASK-VTT070.md
TASK-VTT124|Guard the SIGSEGV at the whisper/FFI boundary — coredumps + JSOC audit||DIRECT-VTT002||TASK-VTT124.md
TASK-VTT114|macOS clipboard auto-paste sends Ctrl+V — must be Cmd+V|STORY-VTT012|DIRECT-VTT003|FEAT-VTT012|TASK-VTT114.md
TASK-VTT112|Add stored per-model SHA-256 verification to src/models.rs||DIRECT-VTT002|FEAT-VTT026|TASK-VTT112.md
TASK-VTT123|Copy last transcription tray menu item|STORY-VTT018|DIRECT-VTT005|FEAT-VTT038|TASK-VTT123.md
TASK-VTT062|Wire selected_device_index through to audio::Audio::new()||DIRECT-VTT002||TASK-VTT062.md
TASK-VTT109|Windows autostart on by default — first-run enable + tray off-switch|STORY-VTT013|DIRECT-VTT004|FEAT-VTT030|TASK-VTT109.md
TASK-VTT023|Batch file transcription via --file flag|STORY-VTT009|DIRECT-VTT002|FEAT-VTT021|TASK-VTT023.md
TASK-VTT131|Clipboard persists without a clipboard manager (FEAT-VTT038 X11 regression)|STORY-VTT018|DIRECT-VTT002|FEAT-VTT038|TASK-VTT131.md
TASK-VTT132|Re-transcribe last recording tray item — decode newest WAV, re-type|STORY-VTT018|DIRECT-VTT002|FEAT-VTT039|TASK-VTT132.md
