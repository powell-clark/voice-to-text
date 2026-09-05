---
id: FEAT-VTT006
status: maintained
kano: performance
---

# FEAT-VTT006: Multi-language support with auto-detection

## Description
Whisper supports 99 languages. VTT exposes language selection via `settings.conf`. When set to `auto`, Whisper detects the language from the audio. When set to a specific language code (e.g. `en`, `fr`, `ja`), Whisper constrains transcription to that language for faster and more accurate results.

## Acceptance Criteria
- [x] **AC-1** — `language = auto` in `settings.conf` produces correct transcription for English, French, and Spanish recordings — verified subjectively
- [x] **AC-2** — `language = en` produces English-only transcription even when foreign words are spoken — verified
- [x] **AC-3** — The language setting is read from `settings.conf` at startup and passed to the WhisperEngine — verified in `src/settings.rs` and `src/whisper.rs`
- [x] **AC-4** — Invalid language codes do not crash the process — they fall back to `auto` with a log warning — verify with `language = xyz`
- [x] **AC-5** — CORRECTED (2026-09-05, TASK-VTT166): this AC previously read "no language UI is exposed in the tray" — stale. Both trays now expose a runtime English/Multilingual toggle (`language_item` in `src/tray/linux.rs`, `lang_en`/`lang_multi` in `src/tray/portable.rs`), verified via `git grep`. The original claim was true only of an earlier version of the tray, before the toggle was added, and nobody updated this card when it landed.

## Cross-platform acceptance criteria (DIRECT-VTT005 parity spec)
Anchored to `docs/PLATFORM-PARITY.md` §1.4 and §2 (tray language submenu). Language selection itself is fully shared (`settings.rs`, `whisper.rs`); the runtime English/Multilingual toggle lives in each platform's own tray module.

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates.

**🐧 Linux — ✅ works** (this card)
- [x] `settings.conf` `language` field drives `WhisperEngine` — shared code
- [x] Tray language submenu (English / Multilingual) with auto `.en`-suffix switching on the model name — `src/tray/linux.rs`

**🪟 Windows — ✅ works**
- [x] Same shared `settings.rs`/`whisper.rs` language handling
- [x] Tray language submenu (English / Multilingual) — `src/tray/portable.rs`

**🍎 macOS — 🟡 partial**
- [x] Same shared `settings.rs`/`whisper.rs` language handling (identical binary code path to Windows)
- [ ] Tray language submenu compiles (`src/tray/portable.rs` is shared across Windows/macOS) but is untested without a `.app` bundle (FEAT-VTT029) to actually run the menu-bar item

## Linked Tasks
- TASK-VTT006

## Parent Story
- STORY-VTT001
