# TASK-VTT052: Design CT2 persistent daemon protocol

## Acceptance Criteria
1. [x] Protocol spec is documented: stdin/stdout line-delimited JSON with request/response shapes defined — `docs/CT2-DAEMON-PROTOCOL.md`
2. [x] Spec covers: load_model, transcribe (with audio path), status, shutdown commands
3. [x] Health-check heartbeat interval and failure detection strategy are specified — 5s heartbeat via `status`, 2s response timeout, stdout-EOF as an immediate independent signal
4. [x] ADR filed capturing the decision to use line-delimited JSON over gRPC or Unix socket — ADR-0009

## Evidence, 2026-09-04

Design-only task — no code changes. Two documents produced:
- `docs/CT2-DAEMON-PROTOCOL.md` — the full wire protocol (framing, four
  commands, health checking, error handling, explicit non-goals)
- `CONSCIOUSNESS/architectural-decisions/0009-ct2-daemon-wire-protocol.md` —
  stdio-JSON vs gRPC vs Unix domain socket, with the cross-platform
  (no native Unix sockets on Windows) and dependency-weight arguments that
  decided it

Scope note: whether to build the CT2 daemon feature at all was already
decided by the operator's own user story (STORY-VTT017); this task and its
ADR only scope the wire protocol, not that decision. Implementation is
TASK-VTT053 (the daemon itself) and TASK-VTT054 (settings toggle + fallback),
both already filed and blocked on this task.
