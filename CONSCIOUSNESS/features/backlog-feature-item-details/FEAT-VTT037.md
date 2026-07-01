---
id: FEAT-VTT037
status: backlog
kano: performance
---

# FEAT-VTT037: Correction dictionary for commonly mistranscribed words

## Status note (2026-07-01)
Implementation complete on this session's branch (settings.rs, corrections.rs,
main.rs wiring): 88 tests pass, clippy/fmt clean, release build green. All four
ACs below are code-verified. Held at `backlog` rather than promoted to
`maintained`/`done` — the review-gate requires human approval (see Requires-human
note), and separately the operator has not yet rebuilt/reinstalled to verify in
daily dictation use (reasonably deferred: they were actively using the
currently-installed binary to dictate this very session). Promote once both the
approval and a real-use check land — no further coding needed first.

## Requires-human (discovered gate bug, not by design)
`packages/core/review/readiness.ts::getFeatureKano` only reads
`FEATURE-ACTIVE-INDEX.md` to resolve a feature's kano tier for the
review-gates kano_overrides. This repo's `FEATURE-ACTIVE-INDEX.md` is always
empty — features go straight from backlog to maintained/done — so the kano
override for `performance` (agent-gate) never resolves and every feature here
silently falls back to the base `human`-gate default. Filed as TASK-VTT120
against this repo since it affects every feature closure, not just this one.

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
- [x] **AC-1** — `settings.conf` gains a user-editable correction list: repeated `correction="misheard=>correct"` lines, parsed into `Settings.corrections: Vec<(String, String)>` — verified in `src/settings.rs` (`Settings::load`/`save`) + unit tests
- [x] **AC-2** — Corrections apply after Whisper inference and before clipboard paste, as case-insensitive whole-word/phrase matches (no partial-word corruption) — verified in `src/corrections.rs::apply` (`\b`-anchored regex) + unit tests, wired in `src/main.rs` before `compose_final_text`
- [x] **AC-3** — Corrections are additive to, not a replacement for, the existing `initial_prompt` field — `initial_prompt` still biases inference first; `corrections::apply` runs after, on the transcribed text
- [x] **AC-4** — Documented — `settings.conf` self-documents the line format via an auto-written comment (verified by `settings_conf_documents_the_correction_line_format` test)

## Resolved (were open questions at capture time)
- File format: inline in `settings.conf` (reused the existing `prompt=` escape/parse machinery), no new file
- GUI: v1 ships settings.conf-only, matching current cross-platform reality (macOS/Windows have no settings GUI at all yet — tracked separately as FEAT-VTT033, backlog p2). A Linux tray textarea editor is a natural follow-up, not filed as its own task since it's optional polish, not a gap.

## Linked Tasks
- TASK-VTT118

## Parent Story
- STORY-VTT019
