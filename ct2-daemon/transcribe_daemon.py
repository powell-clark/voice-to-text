#!/usr/bin/env python3
"""Persistent CTranslate2/faster-whisper transcription daemon.

Implements the protocol in docs/CT2-DAEMON-PROTOCOL.md: one JSON object per
line on stdin (requests) and stdout (responses), four commands (load_model,
transcribe, status, shutdown). TASK-VTT053, protocol designed in TASK-VTT052
/ ADR-0009.

This is the optional CT2 speed backend (FEAT-VTT034, STORY-VTT017) — the
default backend remains whisper-rs in-process (ADR-0003); nothing here runs
unless a user explicitly opts in (TASK-VTT054 wires the settings toggle that
spawns this process).

The protocol-dispatch logic (`dispatch`) takes a `ModelBackend` by
dependency injection specifically so it is unit-testable without loading a
real CT2 model — see test_transcribe_daemon.py's `FakeBackend`.
"""
from __future__ import annotations

import json
import sys
import time
from dataclasses import dataclass, field
from typing import Any, Optional, Protocol


class ModelBackend(Protocol):
    """The real CTranslate2 calls, behind an interface tests can fake."""

    def load(self, model_path: str, device: str, compute_type: str) -> str:
        """Load a model, returning its display name. Raises on failure."""
        ...

    def transcribe(
        self, audio_path: str, language: str, initial_prompt: Optional[str]
    ) -> str:
        """Transcribe one audio file, returning the text. Raises on failure."""
        ...


class FasterWhisperBackend:
    """Real backend: faster-whisper / CTranslate2. Imported lazily so the
    protocol-only logic (and its unit tests) never require faster-whisper
    to be installed."""

    def __init__(self) -> None:
        self._model = None

    def load(self, model_path: str, device: str, compute_type: str) -> str:
        from faster_whisper import WhisperModel  # local import, see class docstring

        self._model = WhisperModel(model_path, device=device, compute_type=compute_type)
        return model_path.rstrip("/").rsplit("/", 1)[-1]

    def transcribe(
        self, audio_path: str, language: str, initial_prompt: Optional[str]
    ) -> str:
        if self._model is None:
            raise RuntimeError("no model loaded")
        lang = None if language == "auto" else language
        segments, _info = self._model.transcribe(
            audio_path, language=lang, initial_prompt=initial_prompt
        )
        return "".join(segment.text for segment in segments).strip()


@dataclass
class DaemonState:
    backend: ModelBackend
    state: str = "idle"  # "idle" | "loading" | "transcribing"
    model_name: Optional[str] = None
    started_at: float = field(default_factory=time.time)


def _error(req_id: Any, message: str) -> dict:
    return {"id": req_id, "ok": False, "error": message}


def _ok(req_id: Any, result: dict) -> dict:
    return {"id": req_id, "ok": True, "result": result}


def dispatch(state: DaemonState, request: dict) -> Optional[dict]:
    """Handle one already-parsed request, returning the response to send
    (or None for a request malformed enough to have no usable id).

    Every branch that can raise from user-controlled input is caught here
    so a real CTranslate2/faster-whisper exception never becomes an
    uncaught traceback on stdout, which would corrupt the line-delimited
    stream for every request after it (docs/CT2-DAEMON-PROTOCOL.md's Error
    handling section).
    """
    req_id = request.get("id")
    cmd = request.get("cmd")
    params = request.get("params")
    if not isinstance(params, dict):
        params = {}

    if cmd == "load_model":
        model_path = params.get("model_path")
        device = params.get("device", "cpu")
        compute_type = params.get("compute_type", "int8")
        if not model_path:
            return _error(req_id, "load_model requires 'model_path'")
        state.state = "loading"
        try:
            state.model_name = state.backend.load(model_path, device, compute_type)
        except Exception as exc:  # noqa: BLE001 - deliberately broad, see docstring
            state.state = "idle"
            return _error(req_id, f"load_model failed: {exc}")
        state.state = "idle"
        return _ok(req_id, {"model_name": state.model_name, "device": device})

    if cmd == "transcribe":
        if state.model_name is None:
            return _error(req_id, "no model loaded")
        audio_path = params.get("audio_path")
        if not audio_path:
            return _error(req_id, "transcribe requires 'audio_path'")
        language = params.get("language", "auto")
        initial_prompt = params.get("initial_prompt")
        state.state = "transcribing"
        start = time.monotonic()
        try:
            text = state.backend.transcribe(audio_path, language, initial_prompt)
        except Exception as exc:  # noqa: BLE001 - deliberately broad, see docstring
            state.state = "idle"
            return _error(req_id, f"transcribe failed: {exc}")
        state.state = "idle"
        duration_ms = int((time.monotonic() - start) * 1000)
        return _ok(req_id, {"text": text, "duration_ms": duration_ms})

    if cmd == "status":
        return _ok(
            req_id,
            {
                "state": state.state,
                "model_loaded": state.model_name is not None,
                "model_name": state.model_name,
                "uptime_s": int(time.time() - state.started_at),
            },
        )

    if cmd == "shutdown":
        return _ok(req_id, {})

    return _error(req_id, f"unknown command: {cmd!r}")


def run(backend: ModelBackend, in_stream, out_stream, err_stream) -> int:
    """The stdin/stdout loop. Split from main() so tests can drive it with
    in-memory streams instead of the real process stdio."""
    state = DaemonState(backend=backend)
    for line in in_stream:
        line = line.strip()
        if not line:
            continue
        try:
            request = json.loads(line)
        except json.JSONDecodeError as exc:
            # Malformed line: log and continue. No id was recoverable, so no
            # response is sent for it (docs/CT2-DAEMON-PROTOCOL.md, Error
            # handling) — this only matters for daemon robustness against a
            # future protocol bug, since Rust never sends malformed JSON.
            print(f"transcribe_daemon: malformed line ignored: {exc}", file=err_stream)
            continue

        response = dispatch(state, request)
        if response is not None:
            out_stream.write(json.dumps(response) + "\n")
            out_stream.flush()

        if request.get("cmd") == "shutdown":
            return 0
    return 0


def main() -> int:
    return run(FasterWhisperBackend(), sys.stdin, sys.stdout, sys.stderr)


if __name__ == "__main__":
    sys.exit(main())
