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

## Acceptance criteria

- [ ] _(to be filled in)_
