# 9. CT2 daemon transport: line-delimited JSON over stdio, not gRPC or a Unix socket

Date: 2026-09-04

## Status

Accepted. Filed per TASK-VTT052 (STORY-VTT017, DIRECT-VTT002). Scope is the
transport/wire-format decision only — whether to build the optional CT2
daemon backend at all was already decided by the operator's own user story
(STORY-VTT017: "As Emmanuel I want an optional persistent CT2 Python daemon
backend..."); this ADR does not re-open that.

## Context

`FEAT-VTT034` wants an optional, user-toggleable persistent CTranslate2
(`faster-whisper`) backend for NVIDIA/x86 users chasing maximum inference
speed, with `whisper-rs` staying the default (`TASK-VTT054`). This means a
long-lived Python child process (`transcribe_daemon.py`, `TASK-VTT053`)
that the Rust binary spawns, talks to per-recording, and must detect the
death of.

ADR-0003 (whisper-rs in-process model) already named the shape of the
fallback path it would have taken had whisper-rs proved unsuitable: "a
long-running `transcribe_daemon.py` process ... communicating with Rust over
stdin/stdout line-delimited JSON." This ADR is the first time that shape is
actually being built (as an optional speed path, not a fallback), so it is
worth deciding deliberately rather than just inheriting the old text —
three transports were considered.

## Decision

**Line-delimited JSON over the daemon's own stdin/stdout pipes** (full
protocol: `docs/CT2-DAEMON-PROTOCOL.md`).

## Considered Alternatives

### (a) Line-delimited JSON over stdio — chosen

**Pros:**
- Zero new listening surface — stdio pipes between a parent and its spawned
  child exist automatically at process-spawn time and close automatically
  at child-exit time. No socket file, no port, no permissions to manage,
  no cleanup-on-crash logic beyond "the pipe closed" (which is itself the
  daemon-died signal `docs/CT2-DAEMON-PROTOCOL.md`'s health check already
  needs).
- **Identical on Windows, macOS, and Linux.** `std::process::Command`'s
  piped stdin/stdout work the same way on every platform this project
  ships (`CLAUDE.md`'s three-machine workflow). A Unix domain socket does
  not exist on Windows at all without falling back to named pipes there —
  a second, platform-specific implementation this project would then have
  to maintain and test on hardware it mostly doesn't have (see the
  Windows ARM64/macOS Intel runner gaps already tracked elsewhere in this
  backlog).
- Trivially debuggable by hand: `python3 transcribe_daemon.py` in a
  terminal, typing a JSON line and reading the JSON line back, no client
  tooling required. `grpcurl` or a raw socket client would be needed for
  the alternatives below.
- No new Rust dependency beyond a JSON library (`serde`/`serde_json` — not
  currently in `Cargo.toml`, but a much smaller and more common addition
  than a gRPC stack). Python's `json` module is stdlib, no new dependency
  there at all.
- Matches ADR-0003's own already-written fallback description, so the two
  ADRs describe one consistent design rather than two different ones for
  what is architecturally the same kind of process.

**Cons / risks:**
- Single client only — stdio is inherently one parent, one child. Not a
  limitation here: exactly one Rust process ever talks to exactly one
  daemon it spawned itself; nothing needs a second client.
- No built-in schema/type safety the way protobuf gives gRPC — a typo in a
  field name is a runtime error, not a compile-time one, on the Python
  side at least (Rust can still get compile-time checking via
  `serde`-derived structs for its half of the protocol). Mitigated by the
  protocol doc being the single source of truth both sides implement
  against, and by keeping the command set small (four commands).
- Hand-rolled framing (newline-delimited) rather than a battle-tested
  library's framing. Low risk in practice: JSON's own string-escaping
  guarantees a serialized object never contains a raw newline, so "one
  line = one message" is a genuine invariant, not a heuristic.

### (b) gRPC (protobuf over HTTP/2)

**Pros:**
- Strongly-typed schema (`.proto`) shared by both languages, generated
  bindings, built-in streaming support if ever needed later.
- Mature tooling (`grpcurl`, reflection, well-understood error model).

**Cons / risks:**
- Substantial new dependency chain on both sides: `tonic`/`prost` (Rust)
  and `grpcio` or `grpclib` (Python) — each pulls in its own transitive
  tree (HTTP/2 stack, protobuf runtime) for a protocol that, per
  `docs/CT2-DAEMON-PROTOCOL.md`, has exactly four commands and no
  streaming requirement today.
- Needs a listening address (TCP port or a Unix/named-pipe socket
  underneath gRPC's transport either way) — reintroduces the
  socket-lifecycle and cross-platform concerns of alternative (c) *in
  addition to* the dependency cost, for no capability this design
  currently needs.
- Firewall/security-software friction on Windows from a process opening a
  listening TCP port, even loopback-only — an avoidable support-burden
  category for a purely-local IPC need.
- Disproportionate to the problem: this project's whole recent direction
  (ADR-0003's pure-Rust rewrite, ADR-0006 preferring hound over a
  heavier decode stack) has been toward fewer, lighter dependencies, not
  more. gRPC for a four-command local pipe cuts against that pattern
  without a corresponding benefit.

### (c) A raw Unix domain socket (no gRPC)

**Pros:**
- Avoids stdio's single-client limitation (irrelevant here, see (a)'s
  cons) and gRPC's dependency weight.
- Slightly more conventional "server" shape if this ever needed multiple
  concurrent clients — not a requirement today.

**Cons / risks:**
- **Does not exist on Windows.** Named pipes are the Windows analogue but
  are a different API with different semantics (message vs byte mode,
  different permission model) — this project would need two
  implementations (Unix socket + Windows named pipe) to cover the same
  three platforms stdio covers for free. Given this project's Windows
  support is already the platform most short of parity (see
  `docs/PLATFORM-PARITY.md`), adding a second platform-specific IPC path
  is exactly the kind of asymmetry that document exists to catch.
  macOS's socket support is Unix-like, so only Windows is the outlier —
  but it is an outlier this project explicitly ships to.
- Socket-file lifecycle to manage (create in a writable temp dir, correct
  permissions so another local user can't connect, delete on clean
  shutdown, handle a stale socket file left by a crashed prior daemon) —
  all of it work stdio needs none of, since the pipe *is* the process
  lifetime.
- No debugging convenience over stdio for this use case — `socat`/`nc -U`
  vs. typing into the child's stdin directly; roughly a wash, not a
  reason to prefer the socket.

## Consequences

Adopting (a) means `Cargo.toml` gains `serde`/`serde_json` (small, common,
no transitive weight worth noting) when TASK-VTT053/054 actually implement
the daemon and its Rust-side client. No new build-time dependency is added
by this ADR itself — it is a design decision, not an implementation. The
protocol is intentionally small (four commands, no streaming) so the
stdio choice's "no schema enforcement" con stays low-risk; if the daemon
ever needs to serve genuinely concurrent clients or a schema-strict
contract, that would be a reason to revisit this ADR, not a reason to have
chosen differently today.

## References

- TASK-VTT052 — Design CT2 persistent daemon protocol
- `docs/CT2-DAEMON-PROTOCOL.md` — the full wire protocol this ADR's decision produces
- ADR-0003 — whisper-rs in-process model (names this same stdio-JSON shape as its own fallback path)
- ADR-0006 — Audio decode/resample dependency (this project's precedent for preferring a lighter dependency over a heavier one when both meet the need)
- STORY-VTT017 — the operator's user story authorizing the CT2 daemon feature itself
- TASK-VTT053 — Implement CT2 transcription daemon
- TASK-VTT054 — Settings toggle for CT2 daemon backend
- `docs/PLATFORM-PARITY.md` — Windows parity gaps this ADR's cross-platform reasoning references
