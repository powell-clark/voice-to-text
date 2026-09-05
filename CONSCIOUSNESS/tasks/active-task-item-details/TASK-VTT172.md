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

- [ ] `docs/PLATFORM-PATHS.md` (new) carries the path-conventions table
      (settings/model-cache/logs/recordings, Linux vs Windows), moved
      verbatim from `docs/PLATFORM-PARITY.md` §0 — this content isn't tied
      to any single feature card, so it needs its own permanent home before
      the old doc can go.
- [ ] `scripts/generate-platform-spec.sh` prepends that file's content and
      appends a synthesised "Open gaps" section built by scanning every
      migrated card's Cross-platform section for unchecked (`- [ ]`)
      bullets and extracting the referenced `TASK-VTT*` id, in card order
      (no fabricated priority sort — the real priority already lives in
      `TASK-BACKLOG-INDEX.md`, not duplicated here).
- [ ] Diffed the generator's new "Open gaps" output against
      `docs/PLATFORM-PARITY.md`'s current gap-register table line by line —
      every currently-open row (and only the currently-open rows) appears;
      nothing silently dropped, nothing fabricated.
- [ ] `docs/PLATFORM-PARITY.md` deleted once the above is confirmed;
      `git grep` for any remaining reference to it elsewhere in the repo
      (README, other cards, scripts) updated to point at the generated
      output or `docs/PLATFORM-PATHS.md` instead.
- [ ] `cargo fmt`/`clippy`/`test` unaffected (docs/scripts-only change) —
      confirmed green.

## Dependencies

- Directive: DIRECT-VTT005
- Story: STORY-VTT018
- Doc: `docs/PLATFORM-PARITY.md`, `scripts/generate-platform-spec.sh`
