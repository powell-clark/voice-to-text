# TASK-VTT072: Unknown command /consciousness:sequence — is it part of pgps or a separate command?

> **Answered 2026-07-21 (product-owner grooming pass):** /consciousness:sequence is a documented
> standalone command — confirmed present in this session's available-skills listing
> ("consciousness:sequence: EXECUTION SPINE — by default print the deterministic spine and suggest
> updates; 'list only' for the pure spine; 'auto' to apply best improvements headless"), and the
> pgps command's own doc text references it as a sibling ("/consciousness:pgps sequence" / "pgps
> sequence"). A `/consciousness:sequence auto` pass ran successfully in this repo just prior to this
> grooming pass, producing the five findings acted on here — direct empirical confirmation the
> command works. Re-tiered to p5. Not force-closed to done: closing a backlog task through the full
> claim → in_progress → in_review → done lifecycle for a doc-only administrative answer is a builder/
> loop action outside the product-owner role's normal mandate (see task_lifecycle claiming rules).
> Recommend the operator close this directly.

## Context

Auto-created from /consciousness:issue.

Report context:
User tried /consciousness:sequence, got 'Unknown command'. Asking whether sequence is part of pgps or a separate skill.

## Acceptance criteria

- [x] Confirm whether /consciousness:sequence is a documented standalone command or part of /consciousness:pgps — ANSWERED: standalone command, also reachable via `/consciousness:pgps sequence`
