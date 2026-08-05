# TASK-VTT076: Telemetry opt-in should be default at install — 30-day retention agreed at walkthrough, no opt-out required

> **Groomed (product-owner grooming pass, 2026-07-21):** acceptance criteria below are drawn
> directly from the operator feedback already recorded in "Report context" below — no new scope
> invented. This is an upstream consciousness-plugin installer/setup-wizard request, not something
> this consumer repo controls directly. Note for context (not proof the upstream default changed):
> this repo's own CONSCIOUSNESS/config.json currently shows `telemetry.enabled: true` and
> `telemetry.upload: true` — but that reflects this project having been connected via
> `/consciousness:apgps` after the fact, not evidence the *installer's* default prompt changed for
> new projects. Left open for operator/upstream confirmation.

## Context

Auto-created from /consciousness:issue.

Report context:
Issue from voice-to-text project. User feedback: telemetry should be a default that is agreed to at install time (like Fable's 30-day retention model). User wants: opt-in by default, 30-day retention, ability to provide feedback or connect to APGPS without it. Telemetry was off by default and required manual config.json edit to enable.

## Acceptance criteria

- [ ] Setup wizard (`consciousness:init`) prompts for telemetry consent at install time rather than defaulting silently off, matching the walkthrough-agreed 30-day retention model
- [ ] Telemetry retention is documented and enforced at 30 days
- [ ] A user can submit feedback (`/consciousness:issue`) or connect APGPS without opting into telemetry.upload — the two are independent gates, not bundled
- [ ] No manual config.json edit is required to reach the agreed default state from a fresh install
