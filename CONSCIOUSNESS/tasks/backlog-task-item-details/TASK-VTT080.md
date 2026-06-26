# TASK-VTT080: Features and testing are not connected — add per-feature test-status tracking (last_tested field and/or verification reviews)

> **Needs review:** the agent created this task during real-time validation and is uncertain about scope or priority. Operator should review and re-tier as appropriate.


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
