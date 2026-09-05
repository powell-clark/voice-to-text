# TASK-VTT172: Extend the platform-spec generator and retire docs/PLATFORM-PARITY.md

## Context

Split from TASK-VTT166 (2026-09-05) — see that card for the card-migration
work already done. `scripts/generate-platform-spec.sh` aggregates each
maintained feature card's own "Cross-platform acceptance criteria" section
cleanly, but `docs/PLATFORM-PARITY.md` also carries two views the generator
has no equivalent for:

- **§0, platform path conventions** — a small table of Linux/Windows path
  mappings (settings, model cache, logs, recordings) that isn't tied to any
  single feature card.
- **The consolidated "Parity gap register"** — a priority-sorted table of
  every open/closed gap with its task link, read at a glance rather than
  scattered across 12 cards' individual bullet lists.

Retiring the hand-maintained doc before the generator can reproduce (or an
operator explicitly accepts losing) these two views would be a silent
regression in how this project tracks parity.

## Acceptance criteria

- [x] `docs/PLATFORM-PATHS.md` (new) carries the path-conventions table
      (settings/model-cache/logs/recordings, Linux vs Windows), moved
      verbatim from `docs/PLATFORM-PARITY.md` §0, plus a correction (its
      `system_cache()` gap note pointed at TASK-VTT097, which is done —
      updated to say so).
- [x] `scripts/generate-platform-spec.sh` now prepends that file's content
      and appends a synthesised "Open gaps" section built by scanning every
      migrated card's Cross-platform section for unchecked (`- [ ]`)
      bullets and printing them per card, in card order (no fabricated
      priority sort — the real priority already lives in
      `TASK-BACKLOG-INDEX.md`, not duplicated here).
- [x] Diffed the generator's new "Open gaps" output against the old
      gap-register table's 10 rows by hand: 6 correctly absent (shipped —
      TASK-VTT093/094/097/098/099/100), 4 correctly present (TASK-VTT101,
      TASK-VTT048, TASK-VTT168, and TASK-VTT092's on-hardware AC). That last
      one required a real correction, not just a migration: FEAT-VTT005's
      Windows section had marked TASK-VTT092 `[x]` done, but TASK-VTT092's
      own card still has an unchecked AC-1 and says "needs on-hardware
      verification" — the feature card was overclaiming. Fixed to `[ ]` and
      restored the honest caveat so the generator doesn't silently drop it.
      New gaps surfaced beyond the old 10-row table too (e.g. FEAT-VTT012's
      Cmd+V-vs-Ctrl+V macOS gap, TASK-VTT114) — the generated view is richer
      than the hand-maintained one it replaces, not just equivalent.
- [x] `docs/PLATFORM-PARITY.md` deleted. `git grep` found only expected
      remaining references: already-released `CHANGELOG.md` blocks
      (append-only, not rewritten), ADR-0008/ADR-0009 (active ADRs —
      editing requires operator approval, and their references are
      historical/accurate for when they were written, not live pointers),
      and already-`done` task cards (historical record). The one live,
      user-facing pointer — `README.md` — updated to link
      `docs/GENERATED-PLATFORM-SPEC.md` instead, and that file regenerated
      and committed (was stale since the original 5-card ADR-0008 commit).
- [x] `cargo fmt`/`clippy --all-targets -D warnings`/`cargo test --release`
      (209 tests) all green — docs/scripts-only change, confirmed no
      regression.

## Dependencies

- Directive: DIRECT-VTT005
- Story: STORY-VTT018
- Doc: `docs/PLATFORM-PARITY.md`, `scripts/generate-platform-spec.sh`
