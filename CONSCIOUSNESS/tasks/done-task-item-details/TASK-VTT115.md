# TASK-VTT115: PGPS feature MAINTAINED/DONE display double-lists — validator rejects the status column the display script requires (consciousness v0.42.9, TASK-VTT079)

> **Likely duplicate (product-owner grooming pass, 2026-07-21):** this card's own body cites
> TASK-VTT079 (Upstream — feature-index status column splits PGPS display but fails validator) as
> the same root cause. TASK-VTT079 has the more complete problem statement — measured before/after
> validation counts, a decision log, and acceptance criteria refined across two updates
> (2026-06-23, 2026-06-27) that already describe exactly the fix this card's "Detail" paragraph
> proposes. Re-tiered to p5 pending operator confirmation to merge/cancel this card in favour of
> TASK-VTT079. Not cancelled here — cancellation/merge of a card is operator-gated per review-gates.

> **Needs review:** the agent created this task during real-time validation and is uncertain about scope or priority. Operator should review and re-tier as appropriate.


## Context

Auto-created from /consciousness:issue.

Report context:
transcripts:
  - chats/claude-code/2026-06-27/session-30126e93.jsonl

Detail: schema.json defines features.columns_done with a 'status' column and main.js (display) honours it (8-col index splits MAINTAINED/DONE correctly), but the validator Rule 31/33 still parses the feature terminal index with the legacy 6-col map id|kano|description|story_ids|task_ids|doc. An 8-col index therefore validates 49/52 with ~48 false 'Invalid reference ID' errors. From a consumer you cannot have both clean validation AND the maintained/done split. Fix: point validator Rule 31/33 at schema.columns_done for the feature terminal index (backward-compatible header detection).

## Acceptance criteria

- [ ] _(to be filled in)_
