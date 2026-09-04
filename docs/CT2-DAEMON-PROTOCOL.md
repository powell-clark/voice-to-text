# CT2 daemon wire protocol

**Status:** design only (TASK-VTT052) — `transcribe_daemon.py` does not exist yet;
implementing it is TASK-VTT053, and wiring a settings toggle + fallback is
TASK-VTT054. Nothing in this document is implemented.

**Why this exists:** `FEAT-VTT034` / `STORY-VTT017` — an optional persistent
CTranslate2 (`faster-whisper`) backend for NVIDIA/x86 users who want maximum
inference speed, with `whisper-rs` remaining the default (ADR-0003 already
retired the old *per-recording* CT2 subprocess; this is a *persistent* daemon,
a different shape, not a revival of the old design). See ADR-0009 for why the
transport is line-delimited JSON over stdio rather than gRPC or a Unix socket.

## Framing

One JSON object per line, UTF-8, terminated by `\n`, in both directions.
A JSON string never contains a raw newline (control characters are escaped
as `\n` by every conformant encoder), so a line boundary is always a message
boundary — the daemon and the Rust side both read with a plain line reader,
never a length-prefixed or streaming JSON parser.

**Requests** (Rust → daemon, on the daemon's stdin):
```json
{"id": 1, "cmd": "load_model", "params": { ... }}
```

**Responses** (daemon → Rust, on the daemon's stdout):
```json
{"id": 1, "ok": true, "result": { ... }}
{"id": 1, "ok": false, "error": "human-readable message"}
```

`id` is a Rust-assigned request counter (starts at 1, increments per
request), echoed back unchanged so an out-of-order or delayed response can
still be matched to its request — commands are expected to complete in
order in practice (the daemon processes one request at a time), but the
`id` correlation costs nothing and removes the ordering assumption as a
protocol requirement.

The daemon's **stderr** is never protocol traffic — it is free-form
diagnostic logging Rust may capture for `vtt_log!` but never parses.

## Commands

### `load_model`

Request:
```json
{"id": 1, "cmd": "load_model", "params": {
  "model_path": "/home/user/.cache/voice-to-text/models/ct2-large-v3-turbo",
  "device": "cuda",
  "compute_type": "float16"
}}
```
- `device`: `"cpu"` or `"cuda"` (CTranslate2's own device strings)
- `compute_type`: CTranslate2 quantisation, e.g. `"int8"`, `"float16"`, `"float32"`

Response:
```json
{"id": 1, "ok": true, "result": {"model_name": "large-v3-turbo", "device": "cuda"}}
```
A `load_model` while a model is already loaded replaces it (unloads the old
one first) — there is no separate `unload_model` command; loading a new
model is how an old one is released.

### `transcribe`

Request:
```json
{"id": 2, "cmd": "transcribe", "params": {
  "audio_path": "/tmp/voice-to-text-scratch/rec-1725484800.wav",
  "language": "en",
  "initial_prompt": "Male British English speaker..."
}}
```
- `audio_path`: a 16 kHz mono WAV path (same contract as the Rust `--file`
  path and the live capture pipeline — the daemon does not resample)
- `language`: `"auto"` or an ISO 639-1 code, matching `settings.conf`'s
  existing `language` field
- `initial_prompt`: nullable; matches `settings.conf`'s `initial_prompt`

Response:
```json
{"id": 2, "ok": true, "result": {"text": "transcribed text", "duration_ms": 340}}
```
`duration_ms` is the daemon's own inference wall-clock time (excludes I/O
and JSON overhead) — this is what TASK-VTT053's "within 500ms of receiving
audio" acceptance criterion measures against.

A `transcribe` request while no model is loaded is an error
(`"ok": false, "error": "no model loaded"`), never an implicit `load_model`
— the two stay explicit and separate so a caller always knows which state
transition it triggered.

### `status`

Request:
```json
{"id": 3, "cmd": "status", "params": {}}
```
Response:
```json
{"id": 3, "ok": true, "result": {
  "state": "idle",
  "model_loaded": true,
  "model_name": "large-v3-turbo",
  "uptime_s": 142
}}
```
`state` is one of `"idle"`, `"loading"`, `"transcribing"`. This command
doubles as the heartbeat (see Health checking) — there is no separate
`ping`/`pong` pair, one less message shape to keep in sync between the two
languages.

### `shutdown`

Request:
```json
{"id": 4, "cmd": "shutdown", "params": {}}
```
Response:
```json
{"id": 4, "ok": true, "result": {}}
```
The daemon replies, flushes stdout, then exits `0`. Rust waits for the
process to actually exit (bounded — see Health checking's timeout) before
treating the daemon as stopped; a `shutdown` response without a subsequent
process exit within the timeout is treated the same as a hang.

## Health checking

Rust sends a `status` request every 5 seconds as the heartbeat. If no
response arrives within 2 seconds of a heartbeat request, or the daemon's
stdout closes (EOF) at any time, Rust treats the daemon as dead:
1. Send `SIGTERM` (Unix) / terminate the process handle (Windows) if it is
   still alive.
2. Fall back to `whisper-rs` for the in-flight and all subsequent requests
   (TASK-VTT054's fallback requirement) — the fallback is silent to the
   transcription result, but the tray's "Backend: CT2" label switches to
   "Backend: Native (CT2 unavailable)" so the user can see why speed
   changed, rather than the daemon dying invisibly.
3. Log the failure via `vtt_log!` with enough detail (last stderr lines,
   exit code if available) to diagnose without needing to reproduce it.

A `transcribe` request already in flight when the heartbeat fails uses its
own timeout, not the heartbeat's 2 seconds — `TASK-VTT053`'s 500ms
same-model-loaded target does not bound a cold model load or a pathologically
long recording, so the in-flight timeout should be generous (a fixed cap,
exact value left to TASK-VTT053/054's implementation) and only then falls
back, rather than aborting a nearly-finished real transcription on the
heartbeat's much tighter clock.

## Error handling

- A line that fails to parse as JSON at all: the daemon logs it to stderr
  and continues reading the next line — one malformed line never crashes
  the daemon or the connection, and (since no `id` could be extracted) no
  response is sent for it. Rust does not send malformed JSON by
  construction, so this path only matters for daemon robustness against a
  future protocol bug, not normal operation.
- A line that parses as JSON but has an unknown `cmd` or missing/wrong-typed
  `params`: the daemon responds `{"id": <echoed>, "ok": false, "error": "..."}`
  — always with the `id` it *could* read, even when the rest of the request
  is invalid.
- Any exception during a command's execution (a real `faster-whisper` error,
  a missing audio file, an OOM) is caught and turned into an `"ok": false`
  response with a human-readable `error` string, never an uncaught Python
  traceback on stdout (a traceback would corrupt the line-delimited JSON
  stream for every request after it).

## Non-goals

- No streaming/partial transcription results — `transcribe` is
  request/response, matching the app's own hold-to-record-then-transcribe
  model; there is nothing today that would consume a partial result.
- No concurrent requests — the daemon processes one command at a time by
  design (a single CT2 model instance is not meant to serve overlapping
  transcriptions); Rust's own single-worker-thread model already only ever
  has one recording in flight, so this is not a new constraint.
- No authentication/encryption — stdio between a parent process and its own
  spawned child is not a network-exposed channel; there is no remote peer
  to authenticate.
