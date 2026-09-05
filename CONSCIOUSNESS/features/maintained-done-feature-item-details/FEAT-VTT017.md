---
id: FEAT-VTT017
status: maintained
kano: performance
---

# FEAT-VTT017: large-v3-turbo and distil-large-v3 model support

## Description
The tray model menu offers `large-v3-turbo` alongside `small`, `medium`, and `large-v3`. The newer turbo model delivers better accuracy at lower VRAM cost. Obsolete `large-v1`/`large-v2` entries are gone; the legacy `distil-large-v3` name auto-migrates to `large-v3-turbo` rather than appearing as its own menu entry.

## Acceptance Criteria
- [x] **AC-1** — `large-v3-turbo` appears in the tray model submenu — verified in v2.0.x tray
- [x] **AC-2** — `distil-large-v3` / `distil-large-v3.5` legacy names auto-migrate to `large-v3-turbo` (`src/main.rs`) rather than appearing as a separate menu entry — verified
- [x] **AC-3** — Obsolete `large-v1` / `large-v2` entries no longer appear; the menu is small.en, small, medium.en, medium, large-v3-turbo, large-v3 — verified in `src/models.rs` MODELS
- [x] **AC-4** — Selecting `large-v3-turbo` triggers download if not cached, then loads and transcribes — verified in testing
- [ ] **AC-5** — [deferred → TASK-VTT112] Stored per-model SHA-256 allowlist verification — NOT implemented; `src/models.rs` computes and logs the download hash but does not check it against an expected constant

## Cross-platform acceptance criteria (DIRECT-VTT005 parity spec)
Anchored to `docs/PLATFORM-PARITY.md` §1.6. The catalogue itself (`src/models.rs::MODELS`) is fully shared; only the tray menu construction around it differs.

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates.

**🐧 Linux — ✅ works** (this card)
- [x] `rebuild_model_menu` filters `MODELS` to multilingual entries only, title-cases the label, and auto-strips `.en` when switching to multilingual — `src/tray/linux.rs`

**🪟 Windows — 🟡 partial**
- [x] Model submenu is generated from the real `MODELS` catalogue (v2.3.2), no hardcoded list
- [ ] Iterates ALL of `MODELS` unfiltered and shows the raw `info.name` — no multilingual-only filter, no title-casing, no `.en` auto-switch — so Windows/macOS can show both `.en` and non-`.en` variants as separate entries where Linux collapses them into one radio group (documented in ADR-0005, still true as of this migration — `git grep multilingual src/tray/portable.rs` returns nothing)

**🍎 macOS — 🟡 partial**
- [ ] Same `src/tray/portable.rs` code as Windows — same unfiltered-list gap, untested without a `.app` bundle (FEAT-VTT029)

## Linked Tasks
- TASK-VTT010

## Parent Story
- STORY-VTT004
