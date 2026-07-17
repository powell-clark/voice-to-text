# TASK-VTT140: Agent-run cost tracking and seat selection for PGPS execution

## Context

Operator (2026-07-17): now that agent-run cost is trackable, use it to (a) record the token/dollar cost of each agent dispatch that executes a PGPS task, and (b) choose the correct/cheapest-capable model+effort seat per task (per the seat-arrangement / model-selection guidance in ~/.claude/advisor-lanes/model-selection.md — tier by task shape, buy ceiling before floor on effort). Substrate that already exists: CONSCIOUSNESS/stream/model-attribution.jsonl logs per-turn requested vs responding model (currently many rows show responding_model=unverifiable — fixing that verification is a likely sub-part). Goal: per-task cost visibility so dispatch decisions (haiku for mechanical, sonnet for build, opus/fable for hard review) are evidence-based rather than guessed, closing the loop the 0-budget exercise opened. Scope this properly next session; likely warrants a feature card + breaking into: (1) reliable responding-model attribution, (2) cost-per-task rollup, (3) seat-recommendation surfaced at claim time.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Directive: DIRECT-VTT002
