# JSOC Reliability Standard — Rust (v1.0)

A reliability standard for `voice-to-text`. Synthesised from NASA JPL *Power of 10*, JSF AV C++ Coding Standards, Clean Code, Rust API Guidelines, and the *Rustonomicon*.

Voice-to-text runs on users' machines as a push-to-talk app that types into focused windows. Failure modes include: dropped audio, silent model load failures, deadlocks in the worker thread, and GPU state corruption. A panic or deadlock in the hotkey path wedges the entire UX. Safety is a precondition, not a feature.

**SHALL** = blocking. **SHOULD** = requires justification.

---

## §1 — Bounded Execution

| ID | Rule |
|---|---|
| R-1.1 | **SHALL** — Every loop has a statically provable upper bound. Explicit `MAX_` constant, or iterator over a finite collection. |
| R-1.2 | **SHALL** — No bare `loop {}` without a `break` condition verifiable by inspection. |
| R-1.3 | **SHALL** — Every retry sequence has a `MAX_RETRIES` constant and exponential backoff. |
| R-1.4 | **SHALL** — Every subprocess invocation has a timeout or explicit justification that the child is bounded by design. |
| R-1.5 | **SHOULD** — Deep recursion replaced with explicit iteration. |

---

## §2 — State Management

| ID | Rule |
|---|---|
| R-2.1 | **SHALL** — State machines expressed as `enum` discriminated unions. Illegal combinations must be unrepresentable. |
| R-2.2 | **SHALL** — `match` arms exhaustive. Catch-all `_ =>` requires a comment explaining why enumeration is insufficient. |
| R-2.3 | **SHALL** — No `static mut`. Use `OnceCell`, `Mutex<T>`, or const. |
| R-2.4 | **SHOULD** — Command/query separation: functions either mutate OR return state, not both. |

---

## §3 — Type Safety

| ID | Rule |
|---|---|
| R-3.1 | **SHALL** — Zero warnings from `cargo clippy --all-targets -- -D warnings`. |
| R-3.2 | **SHALL** — Every `unsafe` block has a `// SAFETY:` comment stating the invariants the caller upholds. |
| R-3.3 | **SHALL** — Newtype wrappers for entity IDs: `ModelName(String)`, `AbsolutePath(PathBuf)`, etc. Raw `String` for IDs is forbidden. |
| R-3.4 | **SHALL** — No `as` cast between signed/unsigned or narrowing integer types. Use `TryFrom` and handle the error. |
| R-3.5 | **SHOULD** — Prefer `?` over `unwrap()` in non-test code. Every `unwrap()` in library code requires a comment naming the invariant. |
| R-3.6 | **SHOULD** — Functions exported to users of the crate have explicit return types and doc comments. |

---

## §4 — Error Handling

| ID | Rule |
|---|---|
| R-4.1 | **SHALL** — Expected failures return `Result<T, E>`. No panics for business-logic errors. |
| R-4.2 | **SHALL** — Domain-specific error enums with `#[derive(Debug, thiserror::Error)]` OR `anyhow::Error` at application boundaries only. |
| R-4.3 | **SHALL** — No `unwrap()` or `expect()` in any code path the user can reach without developer intent. Audio failures, model load failures, file I/O failures must surface to the tray as status messages, not crash. |
| R-4.4 | **SHALL** — `expect()` messages name the invariant that was violated, not the operation that was attempted. `"hotkey receiver closed — main dropped the sender before shutdown completed"` not `"failed to recv"`. |
| R-4.5 | **SHOULD** — Panic only for genuine programmer errors (out-of-bounds index on a known-valid slice, unreachable match arm). Never for expected runtime conditions. |

---

## §5 — Defensive Programming

| ID | Rule |
|---|---|
| R-5.1 | **SHALL** — Parse, don't validate. `&str` becomes `ModelName(String)` or fails; downstream code takes `ModelName` directly and trusts the type. |
| R-5.2 | **SHALL** — `debug_assert!` for invariants that should never fire in tested code. Real panics come with explanations. |
| R-5.3 | **SHOULD** — Sanitize external input at the boundary: paths, model names, URLs. |

---

## §6 — Resource Management

| ID | Rule |
|---|---|
| R-6.1 | **SHALL** — Bounded queues, buffers, caches have an explicit `MAX_` constant and a strategy for over-capacity behaviour. |
| R-6.2 | **SHALL** — Every type that owns an OS resource (file handle, thread, GPU context, GTK widget, audio stream) implements `Drop` or owns an RAII guard. |
| R-6.3 | **SHALL** — Locks held for the shortest possible scope. Never `.lock().unwrap()` across a `.await` or a blocking I/O call. |
| R-6.4 | **SHOULD** — Prefer `parking_lot::Mutex` over `std::sync::Mutex` in hot paths. |

---

## §7 — Function & Module Design

| ID | Rule |
|---|---|
| R-7.1 | **SHALL** — Functions ≤ 60 lines (excluding test bodies). Exceed only with explicit rationale. |
| R-7.2 | **SHALL** — Cyclomatic complexity ≤ 10. |
| R-7.3 | **SHALL** — Nesting depth ≤ 3. Prefer early-return guard clauses using `?` or `return`. |
| R-7.4 | **SHALL** — Each module expresses one concept. `src/whisper.rs` owns the engine. `src/models.rs` owns the catalogue and downloads. `src/transcribe.rs` is the thin bridge. |
| R-7.5 | **SHALL** — No junk-drawer module names (`utils`, `helpers`, `common`, `misc`, `shared`). Name by concept. |

---

## §8 — Testing

| ID | Rule |
|---|---|
| R-8.1 | **SHALL** — `cargo test` runs cleanly on every commit that touches `src/`. |
| R-8.2 | **SHALL** — State transitions in the transcription worker have unit tests. |
| R-8.3 | **SHOULD** — Integration tests exercise the audio → worker → engine → typer pipeline with a bundled `ggml-tiny.en.bin` fixture. |
| R-8.4 | **SHOULD** — `#[ignore]` tests link to a tracking issue in the comment. |

---

## §9 — Static Analysis

| ID | Rule |
|---|---|
| R-9.1 | **SHALL** — `cargo clippy --all-targets -- -D warnings` zero warnings. |
| R-9.2 | **SHALL** — `cargo fmt --check` passes. |
| R-9.3 | **SHOULD** — `cargo audit` shows no unresolved vulnerabilities on every release. |
| R-9.4 | **SHOULD** — `#[allow(...)]` at module or crate level requires a `TODO` comment with scope and rationale. |

---

## §10 — FFI and `unsafe`

| ID | Rule |
|---|---|
| R-10.1 | **SHALL** — Every `unsafe` block has `// SAFETY:` documenting the invariant upheld. |
| R-10.2 | **SHALL** — `unsafe impl Send`/`Sync` has justification citing the specific thread model. |
| R-10.3 | **SHALL** — FFI boundaries explicit about ownership: who allocates, who frees. |
| R-10.4 | **SHOULD** — Minimise `unsafe` surface — push it to a small, documented module. |

---

## §11 — Concurrency

| ID | Rule |
|---|---|
| R-11.1 | **SHALL** — Never hold a `Mutex` lock across a `.await` or a blocking I/O call. |
| R-11.2 | **SHALL** — Shared mutable state uses atomic types (`AtomicBool`, `AtomicUsize`) when the operation is simple; `Mutex<T>` only when the data is compound. |
| R-11.3 | **SHALL** — Channel senders (`mpsc::Sender`, `crossbeam::Sender`) are explicit about backpressure: bounded channels with `.try_send`, or unbounded with a documented capacity invariant. |
| R-11.4 | **SHALL** — Worker threads catch panics (via `std::panic::catch_unwind` or thread `JoinHandle`) and restart or surface them to the UI. A panicked thread must never silently disappear. |

---

## §12 — Naming and Clarity

| ID | Rule |
|---|---|
| R-12.1 | **SHALL** — Intent-revealing compound names. `migrate_legacy_model_name` not `fix_model`. `run_with_timeout` not `run`. |
| R-12.2 | **SHALL** — Error variants name the cause, not the effect. `ModelNotFound { path }` not `LoadFailed`. |
| R-12.3 | **SHOULD** — Variable names match abstraction level — loop indices can be `i`, domain entities cannot. |
| R-12.4 | **SHALL** — No junk-drawer module names (see R-7.5). |

---

## Output format for reviews

```
JSOC REVIEW: [file or scope]

BLOCKING (SHALL violations):
  R-X.Y  description  path/to/file.rs:line

WARNINGS (SHOULD violations):
  R-X.Y  description  path/to/file.rs:line

Compliance: N% (X blocking, Y warnings across N files reviewed)
```

Report only real violations with `path/to/file.rs:line` references. No padding.
