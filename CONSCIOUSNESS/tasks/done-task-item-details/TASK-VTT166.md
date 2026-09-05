# TASK-VTT166: Migrate remaining parity cards to the DIRECT-VTT005 section

## Context

Filed alongside ADR-0008 and TASK-VTT080. `docs/PLATFORM-PARITY.md` aggregates
19 maintained feature cards with a detailed gap register and per-gap task
links — considerably richer than the 5-card "Cross-platform acceptance
criteria" convention ADR-0008 formalised. Original scope: migrate the
remaining cards to that shape, verify the generator preserves the
gap-register detail, then retire the hand-maintained doc.

## Findings (2026-09-05)

The "19 maintained cards" list was itself stale before touching anything —
checking each card's real frontmatter `status` and description against
`docs/PLATFORM-PARITY.md`'s claims surfaced:

1. **4 of the 19 are `status: done`, not `maintained`:** VTT010, VTT011,
   VTT014, VTT023. Per ADR-0008's own reasoning for excluding VTT015 ("the
   per-platform freshness tracking this ADR adds is meaningless for a done
   card by definition"), these are excluded from migration for the same
   reason, not silently skipped.
2. **3 more describe a genuinely single-platform mechanism** with no shared
   code or comparable cross-OS analogue — VTT015 (already excluded by
   ADR-0008), VTT016 (`release-ppa.sh`, Linux-only tooling), VTT027
   (cargo-in-`debian/rules` build, Linux-only). Same exclusion class as
   VTT015.
3. **The gap register itself was stale**, not just the card list: of its 10
   rows, 6 (VTT093, VTT099, VTT100, VTT094 — already correctly ✅ — plus
   VTT098 and VTT097 and VTT095, shown ❌ but actually done) needed
   correcting. Only TASK-VTT101 and TASK-VTT048 are genuinely still open,
   confirmed against `TASK-DONE-INDEX.md`/`TASK-BACKLOG-INDEX.md` directly
   rather than trusting the doc.
4. **FEAT-VTT004 itself (one of the original 5 migrated cards) was also
   stale** — its own "Logs submenu missing — TASK-VTT098" bullets (Windows
   and macOS) were wrong now that TASK-VTT098 shipped. Corrected in the same
   change, since leaving a known-false line next to new work would be an
   obvious omission.
5. **FEAT-VTT006's AC-5** ("no language UI is exposed in the tray") was
   flatly contradicted by `git grep` — both trays have had a runtime
   English/Multilingual toggle for some time. Corrected.

Real migration set: **12 cards** — VTT001, 004, 005, 006, 008, 012, 013,
017, 022, 026, 028, 035 (the 5 already done under ADR-0008 plus 7 migrated
here: 001, 006, 008, 017, 022, 028, 035). All verified via `scripts/
generate-platform-spec.sh`, which now emits all 12 sections cleanly.

`docs/PLATFORM-PARITY.md` itself: corrected in place (stale card list, stale
gap register rows) rather than retired — see its own "Retirement status"
note. The generator has no equivalent for the doc's §0 (path conventions)
or the consolidated gap-register table, so deleting the doc today would
silently drop both; extending the generator to synthesise them is real,
separate scripting work, split to TASK-VTT172.

## Acceptance criteria

- [x] Every genuinely-maintained, genuinely-cross-platform card from the
      original 19 carries the "Cross-platform acceptance criteria
      (DIRECT-VTT005 parity spec)" section with a `last_tested` line —
      7 newly migrated (001, 006, 008, 017, 022, 028, 035).
- [x] The 7 cards that do NOT qualify (010, 011, 014, 015, 016, 023, 027)
      are excluded with stated reasoning, not silently dropped — recorded
      above and in `docs/PLATFORM-PARITY.md`'s correction note.
- [x] `scripts/generate-platform-spec.sh`'s output verified against the full
      12-card migration set — `bash scripts/generate-platform-spec.sh | grep
      '^## FEAT'` lists exactly the 12 expected cards.
- [x] `docs/PLATFORM-PARITY.md`'s stale content (card list, gap register,
      FEAT-VTT004's own stale bullets) corrected in place — every claim
      re-verified against live `TASK-DONE-INDEX.md`/`TASK-BACKLOG-INDEX.md`
      or the actual source, not copied forward from the old text.
- [ ] DEFERRED, split to TASK-VTT172: extend the generator to also
      synthesise the path-conventions table and the consolidated gap
      register, then retire `docs/PLATFORM-PARITY.md` once that generated
      view is confirmed complete. Not done here — real scripting work
      beyond card-content migration, and rushing it risked exactly the
      "silently dropped detail" failure this task exists to prevent.

## Dependencies

- Directive: DIRECT-VTT005
- Story: STORY-VTT018

## Pre-mortem

### Failure modes

- Trusting the hand-maintained doc's own claims (card list, gap register)
  as ground truth instead of re-verifying against live task indexes and
  source code — this is exactly what made the doc stale in the first
  place, and blindly migrating its content forward would have propagated
  the staleness into the "generated, cannot drift" cards.
- Treating "migrate all 14/19 cards" as the literal instruction when several
  don't actually qualify under ADR-0008's own stated logic — forcing a
  cross-platform table onto a single-platform or `done`-status card
  manufactures false parity rows, the exact failure ADR-0008's VTT015
  exclusion warns against.

### Weak assumptions

- Assumes the 7 migrated cards' per-platform bullets, though verified
  against current source at time of writing, will need re-verification at
  each `last_tested` refresh — this task starts them at `null`
  deliberately (ADR-0008's convention), not as a claim of ongoing
  freshness.
