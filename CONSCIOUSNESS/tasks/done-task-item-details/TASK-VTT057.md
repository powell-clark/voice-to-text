# TASK-VTT057: Cargo unit tests for pure logic

## Context
The repo has zero tests today — no `tests/` directory, no `#[test]` in `src/`. GTK, enigo, and X11 are hard to unit-test (they need a display, a keyboard, and a live compositor). But significant pure logic exists that is cheap and fast to test and would catch real regressions:
- `src/settings.rs` — settings load/save round-trip (the `settings.conf` parser)
- `src/tray/linux.rs:build_logs_menu` — filename-to-label formatting (Today / Yesterday / YYYY-MM-DD)
- `src/audio.rs` — buffer truncation boundary at the configured max seconds
- `src/main.rs` worker loop — `final_text` prefix/`[Truncated]` composition

The v2.0.5 typing bug (splitting at first non-ASCII) would not have been caught by unit tests — it is integration. This task does NOT try to cover that class. It covers the pure-logic class that sits alongside.

## Acceptance Criteria
1. `cargo test` runs and exits 0 on this machine and in CI
2. At least one test per module listed below, with meaningful assertions (not just `assert!(true)`)
3. Test execution under 2 seconds total (pure unit, no I/O except temp files for settings round-trip)
4. Tests are inline `#[cfg(test)] mod tests { … }` blocks in the file they test, except where a shared fixture makes a `tests/` directory clearer

## Technical Approach
**Settings round-trip (`src/settings.rs`):**
- Write a Settings to a tempdir, read it back, assert field-by-field equality
- Handle the `logging_enabled` default, custom hotkey keycodes, and selected_model edge cases (empty, unicode, missing)

**Log filename labels (`src/tray/linux.rs`):**
- Extract the label-formatting logic into a pure `fn format_log_label(filename: &str, today: &str, yesterday: &str) -> String`
- Test: today matches → "Today (MM-DD)"; yesterday matches → "Yesterday (MM-DD)"; other → "YYYY-MM-DD"; malformed filename → returns filename unchanged

**Audio buffer truncation (`src/audio.rs`):**
- Find the max-samples constant, test that `push_samples` into a buffer at capacity returns a truncation flag
- If the current implementation inlines this, extract a pure helper

**Transcription prefix composition (`src/main.rs`):**
- Extract the `final_text` match into a pure `fn compose_final_text(is_truncated: bool, prefix: &str, trimmed: &str) -> String`
- Test: truncated flag prepends `[Truncated] `; trimmed already starts with prefix → no duplication; empty prefix → unchanged trimmed

Prefer extracting helpers into pure functions over contriving tests around the current control flow. That is the refactor the tests encourage, which is the point.

## Test Strategy
The tests ARE the test strategy. They must run in CI (TASK-VTT058) and the pre-push hook (TASK-VTT059).

## Files
- `src/settings.rs` — add `#[cfg(test)] mod tests`
- `src/tray/linux.rs` — extract `format_log_label`, add tests
- `src/audio.rs` — extract truncation helper if not already pure, add tests
- `src/main.rs` — extract `compose_final_text`, add tests
- `Cargo.toml` — `[dev-dependencies]` tempfile already present, no new deps
