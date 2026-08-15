# TASK-VTT085: Upgrade quinn-proto to >=0.11.15 — close RUSTSEC-2026-0185 (high) reddening CI

## Context

New advisory RUSTSEC-2026-0185: remote memory exhaustion in quinn-proto 0.11.14, fix = upgrade to >=0.11.15. This is the sole red CI job (cargo audit). Likely a one-line 'cargo update -p quinn-proto --precise 0.11.15'. Not a Windows blocker — Windows build job is green.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Directive: DIRECT-VTT002

## Closed without contract

This task reached a terminal state while its acceptance criteria still read
`_(to be filled in)_`. No criteria have been authored retrospectively: the contract
that would have governed this work was never written, and this marker records that
honestly rather than manufacturing one after the fact.
