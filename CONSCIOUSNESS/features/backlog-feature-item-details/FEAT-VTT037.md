---
id: FEAT-VTT037
status: backlog
kano: performance
---

# FEAT-VTT037: Correction dictionary for commonly mistranscribed words

## Kano
performance (p1)

## Description
Whisper reliably mishears certain recurring words and proper nouns for this user
— e.g. "ard" comes out as "odd", "amala vijnana" comes out spelled arbitrarily.
The existing `initial_prompt` setting (FEAT-VTT011, edited via the Linux tray's
"Customize Transcription Settings..." dialog or by hand in `settings.conf` on
macOS/Windows) is a single free-text bias fed to Whisper before inference — it
helps but does not deterministically fix a specific known mishearing.

This feature adds a second, complementary mechanism: a user-editable list of
`misheard -> correct` word/phrase pairs, applied as a post-transcription
substitution pass before the text is pasted. Unlike `initial_prompt`, corrections
are exact and guaranteed — once "ard" -> "odd" is added, that specific mistake
never reaches the clipboard again.

## User Observable Behaviour
- User adds a correction pair (e.g. `odd -> ard`) via settings
- Next transcription containing "odd" (as a whole word) pastes "ard" instead
- Corrections are case-insensitive on match, case-preserving on the replacement text as configured
- Multi-word phrases are supported (e.g. `amala vajrayana -> Amala Vijnana`)
- An empty or missing correction list is a no-op — transcription behaves exactly as today

## Acceptance Criteria
- [ ] **AC-1** — `settings.conf` gains a user-editable correction list (format TBD at build time — candidates: repeated `correction="misheard=>correct"` lines, or a separate `corrections.txt`)
- [ ] **AC-2** — Corrections apply after Whisper inference and before clipboard paste, as whole-word/phrase matches (no partial-word corruption, e.g. correcting "ard" must not mangle "hard")
- [ ] **AC-3** — Corrections are additive to, not a replacement for, the existing `initial_prompt` field — both mechanisms coexist
- [ ] **AC-4** — Documented in README/settings comments so the feature is discoverable without reading source

## Open questions (for build time)
- File format for the correction list — inline in `settings.conf` vs. a dedicated file
- Whether the Linux "Customize Transcription Settings..." dialog (src/tray/linux.rs) gains a matching editor, or v1 ships settings.conf-only (macOS/Windows currently have no settings GUI at all — tracked separately as FEAT-VTT033, backlog p2)

## Linked Tasks
- TASK-VTT118

## Parent Story
- STORY-VTT019
