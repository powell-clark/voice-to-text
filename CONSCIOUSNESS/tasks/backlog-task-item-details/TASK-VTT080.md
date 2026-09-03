# TASK-VTT080: Features and testing are not connected — add per-feature test-status tracking (last_tested field and/or verification reviews)

> **Groomed (product-owner grooming pass, 2026-07-21):** this card is no longer "needs review" — it
> already carries a real 2026-06-26 operator decision, a full design, and six concrete acceptance
> criteria (below). Linked story_ids=STORY-VTT018 per this card's own "Migration note" ("Story:
> STORY-VTT018"), reciprocated on STORY-VTT018. Linked directive_id=DIRECT-VTT005 (Cross-platform
> feature parity as a testable spec) rather than DIRECT-VTT002 — DIRECT-VTT005's title is a near-verbatim
> match for this card's own goal, DIRECT-VTT005 was editable this session (DIRECT-VTT002 was not,
> per this session's explicit scope), and DIRECT-VTT005 already carries sibling tasks (TASK-VTT121,
> TASK-VTT123, TASK-VTT129) without a story link, so a directive-only or directive+story link both
> fit the existing pattern here; reciprocated on DIRECT-VTT005. Re-tiered p3 → p2 per this card's own
> "Migration note — re-tier from p3, this is now the active goal"; no target tier was specified in
> the note, so p2 is a one-notch bump, not a guess at p1 or p0 — operator may want to bump further.


## Context

Auto-created from /consciousness:issue.

Report context:
Diagnosis from VTT repo (voice-to-text), schema 20260621090000000:

PROBLEM: There is no first-class way to track how-tested a feature is.
- The feature schema fields are only: id, status, priority, kano, description, story_ids, task_ids, doc. No last_tested / verified_at / test_status / coverage field exists (grepped schema.js — zero hits).
- Acceptance criteria are unstructured free markdown in the detail card body, not a validated schema field.
- Reviews CAN target features (review/cascade.js: tasks-done -> feature review-eligible -> Kano-gated verdict row with reviewed_at), but that path is built for APPROVAL gating, not test tracking — it conflates 'approved to ship' with 'tested', and PGPS never surfaces it as a freshness/coverage signal.

REQUEST (pick depth):
- Minimal: add a last_tested (or verified_at) column to the feature schema (columns_active + columns_done) — a per-feature freshness stamp independent of reviews.
- Proper: make acceptance_criteria a structured field, add a test-coverage/confidence marker, and have the review cascade record VERIFICATION verdicts distinctly from APPROVAL verdicts, surfaced in /pgps.

SEPARATE known issue (do not conflate): the feature terminal index status-column inconsistency where the PGPS display reads a status column to split MAINTAINED vs DONE but the validator rejects it (enforces legacy 6-col). Tracked locally as TASK-VTT079.

## Operator decision (2026-06-26, Emmanuel)

Go with the **"proper" depth, plus a per-platform dimension.** The spec must live
**in the feature cards**, per platform — NOT in a standalone parity document.
"If you add all the feature cards together you get a specification for each
system." So `docs/PLATFORM-PARITY.md` (created this session as a hand-maintained
doc) is the WRONG artifact — it should be **generated from the cards** or deleted.

## Design

1. **Per-platform acceptance criteria in each maintained FEAT-VTT card** — a table:

   ```markdown
   ## Acceptance criteria — per platform
   | # | Criterion | Linux | Windows | macOS |
   |---|-----------|:-----:|:-------:|:-----:|
   | 1 | Types at cursor, no paste gesture | ✅ 2026-06-20 | ✅ 2026-06-26 | 🟡 |
   | 2 | Unicode (£ é ñ emoji) correct      | ✅ | 🟡 untested | ❌ |
   last_tested: { linux: 2026-06-20, windows: 2026-06-26, macos: null }
   verified_by: tests/<file>::<test>   # where automatable
   ```

2. **Schema extension** (ADR required — feature-card schema is a one-way door):
   structured `acceptance_criteria` with a per-platform status + `last_tested`
   per platform + optional `verified_by` test reference.

3. **The check/test system:**
   - **Validator** — a maintained feature must declare per-platform ACs and a
     test status; flag gaps (extends card-validator.ts).
   - **Verification reviews** — record VERIFICATION verdicts per (feature ×
     platform), distinct from APPROVAL verdicts (review/cascade.js).
   - **Automated where possible** — ACs link to cargo tests; the suite reports
     which ACs are covered. Ties to TASK-VTT048 (cross-platform test matrix that
     actually RUNS tests per platform).
   - **Generator** — sums the cards into each platform's spec view; replaces the
     hand-maintained docs/PLATFORM-PARITY.md.

## Acceptance criteria

- [ ] ADR filed for the per-platform feature-card schema change
- [ ] Maintained FEAT-VTT cards carry a per-platform AC table + last_tested
- [ ] Validator fails a maintained feature lacking per-platform ACs / test status
- [ ] Verification reviews recorded per (feature × platform), distinct from approval
- [ ] Per-platform spec is GENERATED from the cards; docs/PLATFORM-PARITY.md retired
- [ ] At least the parity-relevant cards (VTT004/005/012/013/015/026 …) migrated as the pattern

## Migration note

Re-tier from p3 — this is now the active goal (feature parity + clear specs from
the aggregate of maintained feature cards). Story: STORY-VTT018. Relates:
TASK-VTT048, TASK-VTT079, TASK-VTT103.


## Partial resolution note, 2026-09-03

The diagnosis above says no `last_tested` field exists. That is no longer true
of this repo: `FEATURE-ACTIVE-INDEX.md` carries a `last_tested` column and
FEAT-VTT039 populates it. So the tracking half has landed since the report.

What that column cannot do on its own, demonstrated today: FEAT-VTT039
(Re-transcribe last recording from the tray) is must-have, carried
`last_tested=2026-07-17`, and sat seven weeks while an unrelated task
(TASK-VTT150, Archive dictation as training-grade audio) changed the capture
rate its decoder depends on. Nothing connected the two. The regression risk was
caught because the archive task's own pre-mortem named it, not because the
feature index knew a change had touched its dependencies.

A date is a record, not a gate. What the card's remaining criteria are really
after is something that goes STALE on a code change — a feature whose
`last_tested` predates the newest commit touching the files it depends on,
surfaced the way TASK-VTT152 (Fail the build when the packaged binary is stale)
surfaces a stale binary. Same shape, same failure mode: something asserting it
is current while the thing underneath it moved.

The schema-level half (a validated field, structured acceptance criteria) is
upstream consciousness-plugin work and not this repo's to make.
