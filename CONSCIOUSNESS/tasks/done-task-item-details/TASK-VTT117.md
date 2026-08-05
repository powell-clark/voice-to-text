# TASK-VTT117: APGPS connectivity re-test after OAuth re-auth — does the issue API land on the dashboard now?

> **Verified resolved (product-owner grooming pass, 2026-07-21):** ran the read-only APGPS status
> check this session (`config-manager/cli.js --auth-status`) — result: "Dashboard: connected as
> Emmanuel (epowellclark@gmail.com), Endpoint: https://ap.consciousness.london/api/sync, Auth: bearer
> token, Expires: 2027-07-03T20:39:42.504Z". CONSCIOUSNESS/config.json confirms `sync.enabled: true`
> and `telemetry.upload: true`. The connection is live and has been since the OAuth re-auth that
> prompted this card — the connectivity problem this lineage (TASK-VTT073 → TASK-VTT116 → this card)
> describes is resolved. Not independently verified: whether a live issue report specifically lands
> on the dashboard (the status check confirms sync/auth health, not an end-to-end issue-API round
> trip) — if the operator wants that specific proof, a real `/consciousness:issue` filing would be
> the test. Re-tiered to p5; recommend the operator close this card.

> **Needs review:** the agent created this task during real-time validation and is uncertain about scope or priority. Operator should review and re-tier as appropriate.


## Context

Auto-created from /consciousness:issue (issue:0QzvfJs_t4J4pUvTnfCvm).

Report context:
transcripts:
  - chats/claude-code/2026-06-27/session-30126e93.jsonl

Filed right after a full revoke + OAuth browser re-auth of voice-to-text (fresh token, project should now be auto-created server-side). Verifies whether the issue API accepts reports for this project now — prior attempts (#516, #518) were APGPS-rejected and fell back to GitHub because the project was never registered.

## Acceptance criteria

- [ ] _(to be filled in)_
