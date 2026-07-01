# TASK-VTT118: Correction dictionary for commonly mistranscribed words

## Context

Emmanuel: whisper mishears recurring words/phrases (e.g. 'ard'->'odd', 'amala vijnana' misspelled). Wants a user-editable list of misheard->correct substitutions applied post-transcription, separate from the existing single initial_prompt field.

## Acceptance criteria

- [x] `Settings.corrections: Vec<(String, String)>` added, parsed from repeated `correction="from=>to"` lines in `settings.conf` (`src/settings.rs`)
- [x] `src/corrections.rs::apply` — case-insensitive whole-word/phrase regex substitution, `NoExpand`-safe, applied in list order
- [x] Wired into `src/main.rs` transcription pipeline, after Whisper output and before `compose_final_text`
- [x] `settings.conf` self-documents the line format (auto-written comment)
- [x] Unit tests added (7 in `corrections.rs`, 4 in `settings.rs`); full suite 88/88 pass, clippy clean, fmt clean, release build green
- [ ] [deferred — operator's own timing] Verified in real daily-use dictation on the operator's machine

## Closing note (2026-07-01)
Design + build done in this session. FEAT-VTT037 itself stays at `backlog`
status pending: (1) a review-gate bug fix upstream in consciousness
(TASK-VTT120) or a direct human approval, and (2) the operator rebuilding
and verifying in daily use on their own schedule — they were actively
dictating through the currently-installed binary while this was built, so
verification was deliberately not forced. This task's own scope (design and
implement the feature) is complete regardless of the feature's promotion
state; tasks auto-flow independent of the feature-level gate.

## Dependencies

- Story: STORY-VTT019
- Directive: DIRECT-VTT002
- Features: FEAT-VTT037
