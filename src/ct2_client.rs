//! Rust-side client for the optional CT2 daemon backend (FEAT-VTT034,
//! TASK-VTT054). Protocol: docs/CT2-DAEMON-PROTOCOL.md, decided in ADR-0009,
//! implemented on the Python side in TASK-VTT053 (ct2-daemon/transcribe_daemon.py).
//!
//! Split into pure request/response handling (testable without a real
//! subprocess) and the IO loop (spawn, pipes, a reader thread) that drives
//! it — mirrors the same split `ct2-daemon/transcribe_daemon.py` uses on the
//! Python side, for the same reason: the protocol logic should be
//! verifiable without paying for a model load in every test run.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::Duration;

/// Timeout for `transcribe`/`status`/`shutdown` — generous for CPU inference
/// on a long recording (measured: 44s of audio took 1.14s with `tiny.en`;
/// 30s covers a much larger model on a slower machine with room to spare).
const CALL_TIMEOUT: Duration = Duration::from_secs(30);

/// Timeout for `load_model` specifically. Measured against this project's
/// own falsification: a first-time `large-v3-turbo` CTranslate2 download
/// exceeded the 30s call timeout, which killed a genuinely-succeeding load
/// rather than a hung one (TASK-VTT054 smoke test, 2026-09-04) — model
/// loading and model downloading are a different latency class from
/// inference and need their own, much longer, budget.
const LOAD_MODEL_TIMEOUT: Duration = Duration::from_secs(600);

/// Locate `transcribe_daemon.py`. There is no packaged install location yet
/// (tracked as a follow-up — this feature has never shipped in a .deb/.msi),
/// so today this only resolves for a binary built and run from this repo
/// checkout (`cargo build --release` per this project's own documented
/// build step), via an explicit override, or a copy placed next to the
/// running executable. Returns `None` rather than a guessed path — the
/// caller then skips CT2 spawning and falls back to native, exactly as if
/// the user had not opted in.
pub fn resolve_daemon_script() -> Option<std::path::PathBuf> {
    if let Ok(over) = std::env::var("VTT_CT2_DAEMON_SCRIPT") {
        let p = std::path::PathBuf::from(over);
        if p.is_file() {
            return Some(p);
        }
    }
    // Dev/self-built checkout: compiled in at build time, so this only
    // resolves on the machine that built the binary, which matches how
    // every current build of this app is run (CLAUDE.md's build table).
    let dev_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("ct2-daemon")
        .join("transcribe_daemon.py");
    if dev_path.is_file() {
        return Some(dev_path);
    }
    // A future packaged install could place a copy next to the executable.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let sibling = dir.join("ct2-daemon").join("transcribe_daemon.py");
            if sibling.is_file() {
                return Some(sibling);
            }
        }
    }
    None
}

/// Build one request line (no trailing newline — the caller adds it).
/// Pure: no IO, so a protocol-shape bug shows up as a unit-test failure
/// rather than a silent wire mismatch discovered only against a real daemon.
fn build_request(id: u64, cmd: &str, params: Value) -> String {
    serde_json::to_string(&json!({"id": id, "cmd": cmd, "params": params}))
        .expect("request json is always serialisable — no non-finite floats, no cycles")
}

/// Parse one raw response line for a call made with request `id`, returning
/// the `result` object on success or a human-readable error otherwise.
/// Treats an id mismatch the same as a daemon-reported error — this client
/// only ever has one call outstanding at a time, so a mismatch means the
/// daemon and client have desynchronised, which is exactly the "stop
/// trusting this daemon" condition the caller already handles.
fn parse_response(id: u64, raw: &str) -> Result<Value, String> {
    let parsed: Value =
        serde_json::from_str(raw).map_err(|e| format!("malformed response json: {e}"))?;
    let resp_id = parsed.get("id").and_then(Value::as_u64);
    if resp_id != Some(id) {
        return Err(format!(
            "response id {:?} did not match request id {id}",
            resp_id
        ));
    }
    match parsed.get("ok").and_then(Value::as_bool) {
        Some(true) => Ok(parsed.get("result").cloned().unwrap_or(Value::Null)),
        Some(false) => Err(parsed
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("daemon reported failure with no error message")
            .to_string()),
        None => Err("response missing 'ok' field".to_string()),
    }
}

/// Extract the transcribed text from a successful `transcribe` result.
fn extract_text(result: &Value) -> Result<String, String> {
    result
        .get("text")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "transcribe result missing 'text'".to_string())
}

/// Client for a spawned CT2 daemon process. Becomes permanently `dead`
/// after any failed call — a fragile mid-session daemon restart is not
/// worth the complexity TASK-VTT054 asks for ("falls back ... without user
/// intervention"), so the fallback is for the rest of this app session,
/// not just the one recording.
pub struct Ct2Client {
    child: Child,
    stdin: ChildStdin,
    responses: Receiver<String>,
    next_id: u64,
    dead: bool,
}

impl Ct2Client {
    /// Spawn `python3 <daemon_script>` and load a model. Returns `None` on
    /// any failure (missing python3, daemon crash, load_model error, or a
    /// timeout) — spawning the optional backend must never abort the app;
    /// the caller keeps using whisper-rs exactly as if CT2 were never
    /// requested.
    pub fn spawn(
        daemon_script: &Path,
        model_path: &str,
        device: &str,
        compute_type: &str,
    ) -> Option<Self> {
        let mut child = match Command::new("python3")
            .arg(daemon_script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                crate::vtt_log!(
                    "CT2 daemon: failed to spawn python3 {}: {e}",
                    daemon_script.display()
                );
                return None;
            }
        };

        let stdin = child.stdin.take()?;
        let stdout = child.stdout.take()?;
        if let Some(stderr) = child.stderr.take() {
            thread::spawn(move || {
                for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                    crate::vtt_log!("CT2 daemon stderr: {line}");
                }
            });
        }

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
            // stdout EOF: sender drops here, so a subsequent recv_timeout
            // sees Disconnected — the "daemon died" signal (docs/CT2-DAEMON-PROTOCOL.md).
        });

        let mut client = Ct2Client {
            child,
            stdin,
            responses: rx,
            next_id: 0,
            dead: false,
        };
        match client.call_with_timeout(
            "load_model",
            json!({"model_path": model_path, "device": device, "compute_type": compute_type}),
            LOAD_MODEL_TIMEOUT,
        ) {
            Ok(_) => Some(client),
            Err(e) => {
                crate::vtt_log!("CT2 daemon: load_model failed: {e}");
                // A timed-out or failed load_model leaves the daemon process
                // itself in an unknown state (possibly still busy, possibly
                // wedged) — kill it explicitly rather than relying on Drop's
                // cooperative shutdown, which assumes the daemon is free to
                // read the next command. Measured leak this fixes:
                // TASK-VTT054 smoke test, 2026-09-04.
                let _ = client.child.kill();
                let _ = client.child.wait();
                None
            }
        }
    }

    /// Transcribe one recording already written to `wav_path` (16 kHz mono —
    /// the same contract as `--file` and the live capture pipeline). Returns
    /// `None` on any failure; the caller falls back to whisper-rs.
    pub fn transcribe(
        &mut self,
        wav_path: &Path,
        language: &str,
        initial_prompt: &str,
    ) -> Option<String> {
        if self.dead {
            return None;
        }
        let prompt = if initial_prompt.trim().is_empty() {
            Value::Null
        } else {
            json!(initial_prompt)
        };
        let resp = self.call(
            "transcribe",
            json!({
                "audio_path": wav_path.to_string_lossy(),
                "language": language,
                "initial_prompt": prompt,
            }),
        );
        match resp {
            Ok(result) => match extract_text(&result) {
                Ok(text) => Some(text),
                Err(e) => {
                    crate::vtt_log!(
                        "CT2 daemon: {e} — falling back to whisper-rs for this session"
                    );
                    self.dead = true;
                    None
                }
            },
            Err(e) => {
                crate::vtt_log!("CT2 daemon: {e} — falling back to whisper-rs for this session");
                self.dead = true;
                None
            }
        }
    }

    pub fn is_alive(&self) -> bool {
        !self.dead
    }

    fn call(&mut self, cmd: &str, params: Value) -> Result<Value, String> {
        self.call_with_timeout(cmd, params, CALL_TIMEOUT)
    }

    fn call_with_timeout(
        &mut self,
        cmd: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, String> {
        self.next_id += 1;
        let id = self.next_id;
        let line = build_request(id, cmd, params);
        writeln!(self.stdin, "{line}").map_err(|e| format!("write to daemon stdin failed: {e}"))?;
        self.stdin
            .flush()
            .map_err(|e| format!("flush to daemon stdin failed: {e}"))?;
        match self.responses.recv_timeout(timeout) {
            Ok(raw) => parse_response(id, &raw),
            Err(RecvTimeoutError::Timeout) => Err(format!("no response within {timeout:?}")),
            Err(RecvTimeoutError::Disconnected) => {
                Err("daemon stdout closed (process died)".to_string())
            }
        }
    }
}

impl Drop for Ct2Client {
    fn drop(&mut self) {
        // Best-effort: the process may already be dead, which is exactly
        // why some Ct2Client instances reach Drop in the first place.
        let _ = self.call("shutdown", json!({}));
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_shape_matches_the_protocol() {
        let line = build_request(7, "transcribe", json!({"audio_path": "/tmp/a.wav"}));
        let parsed: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(parsed["id"], 7);
        assert_eq!(parsed["cmd"], "transcribe");
        assert_eq!(parsed["params"]["audio_path"], "/tmp/a.wav");
    }

    #[test]
    fn parse_response_extracts_result_on_success() {
        let raw = r#"{"id": 1, "ok": true, "result": {"text": "hello"}}"#;
        let result = parse_response(1, raw).unwrap();
        assert_eq!(result["text"], "hello");
    }

    #[test]
    fn parse_response_surfaces_the_daemons_own_error_message() {
        let raw = r#"{"id": 1, "ok": false, "error": "no model loaded"}"#;
        let err = parse_response(1, raw).unwrap_err();
        assert_eq!(err, "no model loaded");
    }

    #[test]
    fn parse_response_rejects_a_mismatched_id() {
        let raw = r#"{"id": 2, "ok": true, "result": {}}"#;
        let err = parse_response(1, raw).unwrap_err();
        assert!(err.contains("did not match"), "got: {err}");
    }

    #[test]
    fn parse_response_rejects_malformed_json_without_panicking() {
        let err = parse_response(1, "not json at all").unwrap_err();
        assert!(err.contains("malformed"), "got: {err}");
    }

    #[test]
    fn parse_response_rejects_missing_ok_field() {
        let raw = r#"{"id": 1, "result": {}}"#;
        let err = parse_response(1, raw).unwrap_err();
        assert!(err.contains("missing 'ok'"), "got: {err}");
    }

    #[test]
    fn extract_text_reads_the_transcribe_result_shape() {
        let result = json!({"text": "clean up all DNS", "duration_ms": 340});
        assert_eq!(extract_text(&result).unwrap(), "clean up all DNS");
    }

    #[test]
    fn extract_text_errors_cleanly_when_text_is_absent() {
        let result = json!({"duration_ms": 340});
        assert!(extract_text(&result).is_err());
    }

    #[test]
    #[ignore = "spawns the real python3 daemon and downloads a real CT2 model \
                (faster-whisper tiny.en) -- mirrors whisper::tests::e2e_transcribes_spoken_digits_from_fixture"]
    fn spawn_load_and_transcribe_against_the_real_daemon() {
        let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("ct2-daemon")
            .join("transcribe_daemon.py");
        let mut client = Ct2Client::spawn(&script, "tiny.en", "cpu", "int8")
            .expect("daemon should spawn and load tiny.en");
        assert!(client.is_alive());

        // Any 16kHz mono wav from the debug ring works; this test only
        // proves the wire round-trip, not transcription accuracy.
        let recordings = dirs::data_local_dir()
            .unwrap()
            .join("voice-to-text/recordings");
        let wav = std::fs::read_dir(&recordings)
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.path())
            .find(|p| p.extension().is_some_and(|e| e == "wav"))
            .expect("at least one recording in the debug ring");

        let text = client.transcribe(&wav, "en", "");
        assert!(text.is_some(), "expected real transcribed text, got None");
        assert!(client.is_alive());
    }

    /// Regression test for a real leak this task's own smoke test found
    /// (2026-09-04): a `load_model` failure left the spawned `python3`
    /// process running, unowned, because `Drop`'s cooperative shutdown
    /// assumes the daemon is free to read the next command — not true when
    /// it's still busy (or wedged) inside the call that just failed. Fast
    /// and deterministic (a bogus model name fails before any download), so
    /// unlike the real-model test above this one is not `#[ignore]`d.
    #[test]
    fn spawn_kills_the_process_when_load_model_fails() {
        let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("ct2-daemon")
            .join("transcribe_daemon.py");

        let before = count_transcribe_daemon_processes();
        let client = Ct2Client::spawn(&script, "this-model-does-not-exist-zzz", "cpu", "int8");
        assert!(client.is_none(), "a bogus model name must fail load_model");

        // The kill is synchronous in spawn() (child.wait() blocks until the
        // process actually exits), so no polling/sleep is needed here.
        let after = count_transcribe_daemon_processes();
        assert_eq!(
            after, before,
            "spawn() must leave no orphaned transcribe_daemon.py process behind on failure"
        );
    }

    fn count_transcribe_daemon_processes() -> usize {
        std::process::Command::new("pgrep")
            .arg("-fc")
            .arg("transcribe_daemon.py")
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .parse()
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }
}
