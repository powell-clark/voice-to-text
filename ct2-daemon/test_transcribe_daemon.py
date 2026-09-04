"""Unit tests for the IPC protocol logic in transcribe_daemon.py.

These use FakeBackend so they run in milliseconds with no faster-whisper
model download, mirroring the Rust suite's convention of keeping the fast
default test run free of network/model dependencies (see
whisper::tests::e2e_transcribes_spoken_digits_from_fixture, marked ignored
for the same reason). TASK-VTT053 acceptance: startup, transcribe
round-trip, and shutdown over the IPC protocol.
"""
import io
import json

import pytest

from transcribe_daemon import DaemonState, dispatch, run


class FakeBackend:
    """Stands in for FasterWhisperBackend: no real model, no CTranslate2."""

    def __init__(self, transcribe_text: str = "hello world", fail_load: bool = False,
                 fail_transcribe: bool = False):
        self.transcribe_text = transcribe_text
        self.fail_load = fail_load
        self.fail_transcribe = fail_transcribe
        self.loaded_with: tuple | None = None
        self.transcribed: list[tuple] = []

    def load(self, model_path: str, device: str, compute_type: str) -> str:
        if self.fail_load:
            raise RuntimeError("simulated load failure")
        self.loaded_with = (model_path, device, compute_type)
        return model_path.rsplit("/", 1)[-1]

    def transcribe(self, audio_path: str, language: str, initial_prompt) -> str:
        if self.fail_transcribe:
            raise RuntimeError("simulated transcribe failure")
        self.transcribed.append((audio_path, language, initial_prompt))
        return self.transcribe_text


def _req(req_id, cmd, **params):
    return {"id": req_id, "cmd": cmd, "params": params}


# --- dispatch(): startup / load_model ---------------------------------


def test_load_model_success_reports_model_name_and_updates_state():
    backend = FakeBackend()
    state = DaemonState(backend=backend)

    resp = dispatch(state, _req(1, "load_model", model_path="/models/large-v3", device="cpu",
                                 compute_type="int8"))

    assert resp == {"id": 1, "ok": True, "result": {"model_name": "large-v3", "device": "cpu"}}
    assert state.model_name == "large-v3"
    assert state.state == "idle"
    assert backend.loaded_with == ("/models/large-v3", "cpu", "int8")


def test_load_model_missing_path_is_a_clean_error_not_a_crash():
    state = DaemonState(backend=FakeBackend())

    resp = dispatch(state, _req(1, "load_model", device="cpu"))

    assert resp["ok"] is False
    assert "model_path" in resp["error"]
    assert state.model_name is None


def test_load_model_backend_exception_becomes_ok_false_not_a_traceback():
    state = DaemonState(backend=FakeBackend(fail_load=True))

    resp = dispatch(state, _req(1, "load_model", model_path="/bad/path"))

    assert resp["ok"] is False
    assert "simulated load failure" in resp["error"]
    assert state.state == "idle"  # recovers, does not get stuck in "loading"


def test_load_model_replaces_an_already_loaded_model():
    backend = FakeBackend()
    state = DaemonState(backend=backend)
    dispatch(state, _req(1, "load_model", model_path="/models/small"))

    dispatch(state, _req(2, "load_model", model_path="/models/large-v3"))

    assert state.model_name == "large-v3"
    assert backend.loaded_with == ("/models/large-v3", "cpu", "int8")


# --- dispatch(): transcribe round-trip ---------------------------------


def test_transcribe_round_trip_returns_text_and_duration():
    backend = FakeBackend(transcribe_text="clean up all DNS")
    state = DaemonState(backend=backend)
    dispatch(state, _req(1, "load_model", model_path="/models/large-v3"))

    resp = dispatch(state, _req(2, "transcribe", audio_path="/tmp/rec.wav", language="en",
                                 initial_prompt="programming terms"))

    assert resp["ok"] is True
    assert resp["result"]["text"] == "clean up all DNS"
    assert isinstance(resp["result"]["duration_ms"], int)
    assert resp["result"]["duration_ms"] >= 0
    assert backend.transcribed == [("/tmp/rec.wav", "en", "programming terms")]
    assert state.state == "idle"  # returns to idle after finishing, not stuck "transcribing"


def test_transcribe_without_a_loaded_model_is_an_explicit_error():
    state = DaemonState(backend=FakeBackend())

    resp = dispatch(state, _req(1, "transcribe", audio_path="/tmp/rec.wav"))

    assert resp == {"id": 1, "ok": False, "error": "no model loaded"}


def test_transcribe_missing_audio_path_is_a_clean_error():
    state = DaemonState(backend=FakeBackend())
    dispatch(state, _req(1, "load_model", model_path="/models/small"))

    resp = dispatch(state, _req(2, "transcribe", language="en"))

    assert resp["ok"] is False
    assert "audio_path" in resp["error"]


def test_transcribe_backend_exception_becomes_ok_false_and_recovers_state():
    state = DaemonState(backend=FakeBackend(fail_transcribe=True))
    dispatch(state, _req(1, "load_model", model_path="/models/small"))

    resp = dispatch(state, _req(2, "transcribe", audio_path="/tmp/rec.wav"))

    assert resp["ok"] is False
    assert "simulated transcribe failure" in resp["error"]
    assert state.state == "idle"


def test_transcribe_defaults_language_to_auto_and_prompt_to_none():
    backend = FakeBackend()
    state = DaemonState(backend=backend)
    dispatch(state, _req(1, "load_model", model_path="/models/small"))

    dispatch(state, _req(2, "transcribe", audio_path="/tmp/rec.wav"))

    assert backend.transcribed == [("/tmp/rec.wav", "auto", None)]


# --- dispatch(): status (doubles as the heartbeat) ---------------------


def test_status_before_any_model_load():
    state = DaemonState(backend=FakeBackend())

    resp = dispatch(state, _req(1, "status"))

    assert resp["result"]["state"] == "idle"
    assert resp["result"]["model_loaded"] is False
    assert resp["result"]["model_name"] is None
    assert resp["result"]["uptime_s"] >= 0


def test_status_after_a_model_load_reports_it():
    state = DaemonState(backend=FakeBackend())
    dispatch(state, _req(1, "load_model", model_path="/models/large-v3"))

    resp = dispatch(state, _req(2, "status"))

    assert resp["result"]["model_loaded"] is True
    assert resp["result"]["model_name"] == "large-v3"


# --- dispatch(): shutdown and unknown commands -------------------------


def test_shutdown_replies_ok_with_empty_result():
    state = DaemonState(backend=FakeBackend())

    resp = dispatch(state, _req(1, "shutdown"))

    assert resp == {"id": 1, "ok": True, "result": {}}


def test_unknown_command_is_a_clean_error_not_a_crash():
    state = DaemonState(backend=FakeBackend())

    resp = dispatch(state, _req(1, "does_not_exist"))

    assert resp["ok"] is False
    assert "does_not_exist" in resp["error"]


def test_dispatch_echoes_the_request_id_even_on_error():
    state = DaemonState(backend=FakeBackend())

    resp = dispatch(state, _req(42, "unknown_cmd"))

    assert resp["id"] == 42


# --- run(): the actual stdin/stdout loop, over in-memory streams -------


def test_run_over_the_wire_protocol_end_to_end():
    """The full journey: load_model, transcribe, shutdown, as real
    newline-delimited JSON lines — proves the framing, not just dispatch()."""
    backend = FakeBackend(transcribe_text="via CLI and TMUX")
    requests = [
        _req(1, "load_model", model_path="/models/large-v3"),
        _req(2, "transcribe", audio_path="/tmp/rec.wav", language="en"),
        _req(3, "shutdown"),
    ]
    in_stream = io.StringIO("\n".join(json.dumps(r) for r in requests) + "\n")
    out_stream = io.StringIO()
    err_stream = io.StringIO()

    exit_code = run(backend, in_stream, out_stream, err_stream)

    assert exit_code == 0
    lines = [json.loads(l) for l in out_stream.getvalue().splitlines() if l]
    assert len(lines) == 3
    assert lines[0]["ok"] is True and lines[0]["result"]["model_name"] == "large-v3"
    assert lines[1]["ok"] is True and lines[1]["result"]["text"] == "via CLI and TMUX"
    assert lines[2] == {"id": 3, "ok": True, "result": {}}


def test_run_stops_reading_after_shutdown_even_with_more_lines_queued():
    backend = FakeBackend()
    requests = [_req(1, "shutdown"), _req(2, "status")]
    in_stream = io.StringIO("\n".join(json.dumps(r) for r in requests) + "\n")
    out_stream = io.StringIO()
    err_stream = io.StringIO()

    run(backend, in_stream, out_stream, err_stream)

    lines = [json.loads(l) for l in out_stream.getvalue().splitlines() if l]
    assert len(lines) == 1  # the queued status request is never processed
    assert lines[0]["id"] == 1


def test_run_ignores_a_malformed_line_and_logs_it_without_crashing():
    backend = FakeBackend()
    in_stream = io.StringIO('not valid json\n' + json.dumps(_req(1, "status")) + "\n")
    out_stream = io.StringIO()
    err_stream = io.StringIO()

    exit_code = run(backend, in_stream, out_stream, err_stream)

    assert exit_code == 0
    lines = [json.loads(l) for l in out_stream.getvalue().splitlines() if l]
    assert len(lines) == 1  # only the valid status request produced a response
    assert lines[0]["id"] == 1
    assert "malformed line ignored" in err_stream.getvalue()


def test_run_ignores_blank_lines():
    backend = FakeBackend()
    in_stream = io.StringIO("\n\n" + json.dumps(_req(1, "status")) + "\n\n")
    out_stream = io.StringIO()

    run(backend, in_stream, out_stream, io.StringIO())

    lines = [json.loads(l) for l in out_stream.getvalue().splitlines() if l]
    assert len(lines) == 1


@pytest.mark.skipif(
    True,
    reason=(
        "Real-model smoke test, opt-in only (mirrors the Rust suite's "
        "e2e_transcribes_spoken_digits_from_fixture, ignored by default -- "
        "downloads a real CTranslate2 model). Remove the skipif and run "
        "directly to exercise FasterWhisperBackend against a real model."
    ),
)
def test_real_faster_whisper_backend_loads_and_transcribes():
    from transcribe_daemon import FasterWhisperBackend

    backend = FasterWhisperBackend()
    state = DaemonState(backend=backend)
    resp = dispatch(state, _req(1, "load_model", model_path="tiny.en", device="cpu",
                                 compute_type="int8"))
    assert resp["ok"] is True
