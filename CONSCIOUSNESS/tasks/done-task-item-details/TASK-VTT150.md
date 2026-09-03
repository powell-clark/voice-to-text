# TASK-VTT150: Archive dictation as training-grade audio

## Context
Emmanuel dictates into this tool every day. Each recording is a paired audio/text
sample — exactly what the voice-clone project (`~/projects/auxiliary/epc-voice`,
TASK-EV034) needs hours of. Today both halves are thrown away: capture runs at
16 kHz, and the archive is a 20-file debug ring with no transcripts.

This task makes daily dictation training-grade, opt-in and off by default.

Mirrors TASK-EV034 (Extend voice-to-text to archive training-grade audio) in
epc-voice, which carries the full decision register and pre-mortem. Ownership was
handed to this seat by the epc-voice session on 2026-09-03; that session's
settings work is preserved at commit 18479b1.

## Measured starting state, 2026-09-03
- `src/audio.rs:10` — `const SAMPLE_RATE: u32 = 16000;` is both the capture rate and
  the Whisper input rate.
- `src/audio.rs:623` — `resample_to_16k(input, in_rate)` exists, used when a device
  cannot open 16 kHz directly.
- `src/main.rs:888` — `save_and_cleanup` copies the wav to
  `~/.local/share/voice-to-text/recordings/` then calls `prune_recordings(dir, 20)`.
- `src/whisper.rs` — `decode_wav_rejects_non_16khz` test asserts Whisper input stays
  16 kHz; the resample boundary must keep that true.
- Baseline: `cargo test --workspace` → 135 passed, 0 failed, 1 ignored.

## Acceptance criteria

- [x] `cargo test --workspace` passes with at least three new tests: Whisper input
      is 16 kHz from a 48 kHz capture; archiving off writes nothing new; archiving
      on writes wav plus sidecar
- [x] Five existing recordings transcribe to identical text before and after the
      change, outputs pasted below
- [x] `ffprobe` on a newly archived file shows `sample_rate=48000`, `channels=1`
      — and `pcm_s24le`, 24-bit, from a real dictation
- [x] The sidecar json carries id, recorded_at, duration_s, sample_rate, text,
      model, language and app for that same recording
- [x] With the three settings absent, behaviour is byte-identical to today:
      `recordings/` still capped at 20, nothing written to an archive
- [x] `README.md` states what is recorded, where it is stored, how to disable it
      and how to delete it
- [x] The new binary is running and archiving is enabled in Emmanuel's own
      `settings.conf`; enabled 2026-09-03, first archived recording 08:50:53

ACs 3, 4 and 7 are a single gate, not three: they all need Emmanuel to read the
README section and switch archiving on in his own `settings.conf`. Shipping the
code with the setting pre-enabled would be exactly the thing the privacy design
refuses to do, so the task stays in_progress until he decides.

## Evidence

Shipped at commit `4d18734` on main.

### AC 1 — test suite

```
baseline (739fd40): 135 passed; 0 failed; 1 ignored
after   (4d18734): 152 passed; 0 failed; 1 ignored
cargo clippy --workspace --all-targets -- -D warnings: clean
```

Seventeen new tests. The three the card names:
`audio::tests::whisper_input_is_16k_from_a_48k_capture`,
`settings::tests::a_pre_archive_settings_file_keeps_archiving_off` (with
`archive_settings_default_off_and_survive_a_missing_file`), and
`archive::tests::write_archive_lands_wav_and_sidecar_together`.

### AC 2 — transcription unchanged, 5 of 5 identical

`before` is `packaging/linux/vtt-linux.prebuilt` (v2.3.11, pre-change);
`after` is `target/release/vtt-linux` at 4d18734. Same five wavs, `--file` batch
mode, newest five in `~/.local/share/voice-to-text/recordings/`:

```
--- vtt_recording_n0UQ3L.wav ---
before: By the way, we should probably re-get our company's house articles and stuff like that. I don't know if I've got all the proper ones or where they are.
after : By the way, we should probably re-get our company's house articles and stuff like that. I don't know if I've got all the proper ones or where they are.
MATCH
--- vtt_recording_iAFoNH.wav ---
before: Given my directors loan agreements, how much does my business owe me?
after : Given my directors loan agreements, how much does my business owe me?
MATCH
--- vtt_recording_8kbU68.wav ---
before: But you can move them to the other directory and make a note that they don't live there.
after : But you can move them to the other directory and make a note that they don't live there.
MATCH
--- vtt_recording_YpFoxI.wav ---
before: Wait a second pricing pages on on Constance London those exist on AP GPS So don't think you need those
after : Wait a second pricing pages on on Constance London those exist on AP GPS So don't think you need those
MATCH
--- vtt_recording_FMsEEl.wav ---
before: It just can't interrupt any of the existing stuff the core functionality of working on Nprobeck working on Codex working on law code and deploying that
after : It just can't interrupt any of the existing stuff the core functionality of working on Nprobeck working on Codex working on law code and deploying that
MATCH
=== result: 5/5 identical, 0 differ ===
```

The pre-mortem named this as the change's most likely way to fail. It did not.

### Deviation from TASK-EV034's step 2

The card said resample "immediately before the Whisper call", meaning in the
worker. It happens one step earlier, at the end of `stop_recording`, because the
debug `recordings/` wavs are written from the same samples and
`whisper::decode_wav_to_samples` rejects any wav that is not 16 kHz — writing
those at 48 kHz would have broken the re-transcribe-last recovery net, which the
same card requires be left untouched. Resampling once on the finished capture
satisfies both, and the invariant the AC actually asserts (Whisper input is
16 kHz from a 48 kHz capture) is unchanged.

## Follow-up

The sidecar's `app` field is always `null`. Capturing the focused window class
needs an X11/Wayland round-trip on the typing path and the card allows `null`, so
it is out of scope here — worth a task if the corpus turns out to want it.

## Technical Approach
1. Move the resample: capture at 48 kHz, call `resample_to_16k(&samples, 48000)`
   immediately before the Whisper call, so transcription input is unchanged.
2. Settings keys `archive_recordings`, `archive_dir`, `archive_max_files` — landed
   at 18479b1, defaults preserve today's behaviour when absent.
3. Archive write: `<archive_dir>/<YYYY-MM-DD>/vtt_<id>.wav` at 48 kHz 16-bit mono,
   plus `vtt_<id>.json` sidecar.
4. Cap the archive oldest-first, counting across dated directories.
5. README privacy section; CHANGELOG entry through the sanctioned flow.

## Test Strategy
Unit tests on the pure logic (resample boundary, archive path construction, cap
selection). Manual end-to-end: build a local binary, dictate, confirm a 48 kHz wav
and its sidecar appear and the typed text is unchanged.

## Files
- Modify: `src/audio.rs`, `src/main.rs`, `README.md`, `CHANGELOG.md`
- Already modified: `src/settings.rs` (18479b1)

## Out of Scope
- The transcription model, the hotkey, the typing path
- The existing 20-file `recordings/` debug ring (untouched — re-transcribe-last
  must keep working)
- Importing anything into epc-voice (TASK-EV035)
- Uploading archived audio anywhere


## Closing evidence, 2026-09-03 08:50

Emmanuel enabled archiving and dictated. The log line that distinguishes the
builds:

```
[2026-09-03 08:50:36] Audio stream opened (native 48000 Hz mono)
[2026-09-03 08:50:36] Audio capture started (48000 Hz mono)
[2026-09-03 08:50:53] Archived 1.75s at 48000 Hz to
  .../archive/2026-09-03/vtt_20260903T085053_813.wav
```

`ffprobe` on that file:

```
codec_name=pcm_s24le
sample_rate=48000
channels=1
bits_per_raw_sample=24
duration=1.749333
```

Its sidecar, every key present:

```json
{
  "id": "20260903T085053_813",
  "recorded_at": "2026-09-03T08:50:53.813524584+01:00",
  "duration_s": 1.749,
  "sample_rate": 48000,
  "text": "Testing, testing, one, two, three.",
  "model": "large-v3-turbo",
  "language": "en",
  "app": null
}
```

And the check that mattered most, because getting it wrong would have broken a
shipped recovery net rather than merely failing: the debug ring is still 16 kHz.

```
$ ffprobe .../recordings/<newest>.wav
sample_rate=16000
channels=1
```

Archive at 48 kHz 24-bit for the corpus, debug ring at 16 kHz for
re-transcribe-last, one capture feeding both. That separation was the design
risk and it held in production, not just in tests.

## What it took to get here, worth recording

Three faults stood between a correct implementation and a working install, and
none of them announced itself:

- `packaging/linux/vtt-linux.prebuilt` was two weeks old, and `debian/rules`
  installs it verbatim rather than compiling. A full green build shipped
  August code. Now gated by TASK-VTT152 (Fail the build when the packaged
  binary is stale).
- `apt install` refuses an unchanged version number and exits 0, so the
  script printed "Installed." over a no-op. `dpkg -i` is the local path.
- `vtt.service` carries `Restart=always`, so `pkill` respawned the old binary
  within five seconds and the singleton lock then blocked a direct run.

Separately, the README sent readers to `~/.config/voice-to-text/settings.conf`
three times. The app reads `~/.local/share/voice-to-text` — `main.rs` names the
variable `config_dir` and assigns `dirs::data_local_dir()`. A stale `~/.config/`
settings file from October 2025 still exists on the machine, so the wrong path
did not fail loudly; it silently did nothing. Fixed at 0276b94.

## Follow-up: persistence

The first verified archive was produced by a direct run of
`target/release/vtt-linux` after `systemctl --user stop vtt.service`. That is
session-only: `vtt.service` is still `enabled` with
`ExecStart=/usr/bin/vtt-linux`, which is the 18 August build with no archive
code in it. A reboot or any `systemctl --user start` silently returns to 16 kHz
with archiving dead and nothing in the output to notice.

Closing that needs `sudo dpkg -i` of a rebuilt package so `/usr/bin/vtt-linux`
is the archiving binary and systemd owns it again. Flagged to Emmanuel with the
verification command; the acceptance criteria above are met on their own terms
(a real dictation did archive correctly) and this is a deployment step rather
than an unmet criterion.

## Follow-up: sidecar id is not the filename stem

`id` is bare (`20260903T085053_813`); the wav and json are `vtt_<id>.wav` and
`vtt_<id>.json`. A consumer assuming `id == stem` pairs nothing. Surfaced by the
epc-voice seat against the first real artefact and documented in README rather
than changed, because data now exists in the shipped shape.
