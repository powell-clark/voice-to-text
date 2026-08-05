# TASK-VTT116: APGPS connectivity test — does the issue API land reports on the dashboard now that voice-to-text is connected?

> **Superseded (product-owner grooming pass, 2026-07-21):** middle card in the APGPS debugging
> lineage — TASK-VTT073 (initial failure) → TASK-VTT116 (this card, first smoke test) →
> TASK-VTT117 (APGPS connectivity re-test after OAuth re-auth — does the issue API land on the
> dashboard now?, filed after this smoke test also failed and prompted a full revoke + re-auth).
> TASK-VTT117 carries this session's verification that the connection is currently healthy. Re-tiered
> to p5; recommend the operator close this in favour of TASK-VTT117.

> **Needs review:** the agent created this task during real-time validation and is uncertain about scope or priority. Operator should review and re-tier as appropriate.


## Context

Auto-created from /consciousness:issue.

Report context:
transcripts:
  - chats/claude-code/2026-06-27/session-30126e93.jsonl

Smoke test filed right after /consciousness:apgps connected voice-to-text (auth: Emmanuel, sync enabled). Prior issue #516 was APGPS-rejected (project not bound) and fell back to GitHub; this verifies the dashboard now accepts reports for this project.

## Acceptance criteria

- [ ] _(to be filled in)_
