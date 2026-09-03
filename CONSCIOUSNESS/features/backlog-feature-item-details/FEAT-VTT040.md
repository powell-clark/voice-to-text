---
id: FEAT-VTT040
status: backlog
kano: delighter
---

# FEAT-VTT040: Dictation archived as training-grade audio

## Description
Ordinary push-to-talk dictation is kept, opt-in, as paired audio and text: the
recording at the microphone's own sample rate plus a JSON sidecar carrying the
transcript and its metadata, in one folder per day, capped oldest-first.

## Kano
delighter (p1)

Nobody chooses a voice-to-text tool because it can archive a training corpus,
and nobody is disappointed by its absence — which is exactly the delighter
shape. It exists because Emmanuel's voice-clone project (DIRECT-EV001, in
`~/projects/auxiliary/epc-voice`) needs hours of paired audio and text, his
deliberate recording time is capped at two hours by FEAT-EV001 AC-8, and he
already dictates into this tool every day. The corpus was walking past the door.

## Status note (2026-09-03)
Code shipped at `4d18734` on main under TASK-VTT150 (Archive dictation as
training-grade audio). Tests 135 to 152, clippy clean, and the change's most
likely failure — a transcription drift from moving capture to 48 kHz — was
falsified rather than assumed: five existing recordings transcribe to identical
text before and after.

Held at `backlog` rather than promoted. Two things are outstanding and both need
the operator, not more code: the shipped binary on his machine is still the PPA's
v2.3.11, so the feature is not yet running anywhere; and archiving is off until
he has read the README section and enabled it himself. Promote once a real
dictation has produced an archived wav and its sidecar.

## Acceptance criteria

- [x] **AC-1** — Capture runs at 48 kHz; the samples handed to Whisper are still 16 kHz, so
      transcription behaviour is unchanged
- [x] **AC-2** — Five existing recordings transcribe to identical text before and after
- [x] **AC-3** — Three settings keys, all absent by default, with absent behaviour
      byte-identical to the previous release
- [x] **AC-4** — Archiving writes a 48 kHz 16-bit mono wav plus a JSON sidecar carrying id,
      recorded_at, duration_s, sample_rate, text, model, language and app
- [x] **AC-5** — The archive is capped oldest-first across dated directories
- [x] **AC-6** — The existing 20-file `recordings/` debug ring is untouched, so
      re-transcribe-last keeps working
- [x] **AC-7** — `README.md` states what is recorded, where, how to disable it and how to
      delete it
- [ ] **AC-8** — A real dictation on an installed build produces an archived wav and sidecar
      that `ffprobe` confirms at 48 kHz mono

## Privacy contract
This is a public product that now writes users' voices to disk, so the defaults
carry the guarantee: absent settings mean nothing is written, nothing is uploaded
by this feature at any setting, and the README names the folder to delete. Any
future change that weakens one of those three is a change to this contract, not
an implementation detail.

## Out of scope
- Importing anything into epc-voice (TASK-EV035 owns that)
- Uploading archived audio anywhere
- The transcription model, the hotkey, or the typing path
