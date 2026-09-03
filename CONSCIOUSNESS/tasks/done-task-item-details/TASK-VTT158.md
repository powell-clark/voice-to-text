# TASK-VTT158: Track transcription accuracy across models and settings

## Context

Emmanuel, 2026-09-03: "Do you have a way of tracking our accuracy over time?
Because I feel like it's not as accurate as it has been."

There was no way to answer that. Nothing measured transcription quality, so a
model change, a prompt change or a code change was made blind and judged by
feel. He also said he would probably do a model update, which without
measurement is the same blind swap again.

## Acceptance criteria

- [x] A harness re-transcribes a fixed set of the operator's own recordings
      under two binaries OR two settings files and reports every line that
      differs
- [x] It never writes to the operator's real settings — a comparison cannot
      disturb the configuration it is measuring
- [x] It prefers the archive corpus when that holds enough recordings, since
      those are paired with the transcript actually produced, and falls back to
      the debug ring otherwise
- [x] `--file` batch mode reproduces the live path, so a comparison measures the
      app rather than a different code path
- [x] The harness is falsified before being trusted — a deliberately wrong
      configuration must move the output
- [x] `cargo test --workspace` passes; clippy and fmt clean

## Two defects found while building it

Both would have made every measurement worthless, and both were silent.

**`--file` passed `None` as the prompt.** Batch mode is the tool you reach for
to debug a bad transcription, and it did not use the initial prompt at all, so
it could produce different text from the hotkey on the same audio with nothing
saying why. It also meant every settings A/B returned "identical" for any prompt
change, because neither side ever saw a prompt. Now passes the prompt and
applies corrections, matching the live path.

**The harness wrote its variant to the wrong directory depth.** The app resolves
`$XDG_DATA_HOME/voice-to-text/settings.conf`; the script wrote
`$XDG_DATA_HOME/settings.conf`. Every comparison silently read the operator's
real settings on both sides.

The second was caught only because the harness was falsified before being
trusted: a canary prompt written in French produced 4 of 4 identical, which is
impossible if the prompt is being read. With the depth fixed, the same canary
turns "Yeah, clean up all DNS" into "Je ne sais pas, on a des données". An 8/8
identical had already been produced and would have been reported as a clean
null result.

## The investigation, 2026-09-03

Six candidate causes for the reported degradation. Four ruled out, two show
improvement, none supports it.

| Candidate | Finding | Verdict |
|---|---|---|
| Today's code changes | pid 3247235 held a deleted inode; the August binary answered every press | ruled out |
| Model change | `large-v3-turbo` in every log line across all retained logs | ruled out |
| Initial prompt | A/B over 10 recordings: 4 changed, 2 better, 2 worse | not the cause |
| Room noise floor | -27.2 -> -41.8 dBFS | improved ~15 dB |
| Speech-to-noise | 17.5 -> 21.3 dB median | improved |
| Discard / hallucination rate | 0 too-quiet, 0 blank-skips today | unchanged |

Recording level did halve — median amplitude 32767 on 2026-08-27 against 18499
on 2026-09-03 — but a median of 32767 is full-scale, meaning half of those
recordings were clipping. 18499 is roughly -5 dBFS, still hotter than the -18 to
-12 dBFS target. Less clipping in a quieter room is a better signal, not a worse
one. Input gain is 0.97, near maximum, so level is not raisable in software; the
VideoMic GO II is a shotgun and aim and distance are the only levers.

### The prompt hypothesis, tested and rejected

The initial prompt contains correction-pair syntax (`odd=ADR`, `odds=ADRs`)
while zero correction pairs are configured, and it is 214 of 240 characters of
comma-jammed word list that reads nothing like speech. Whisper treats the prompt
as decoder context, not a substitution table, so this looked wrong.

Measured rather than assumed. Cleaning the prompt and moving the pairs into real
corrections changed 4 of 10 transcriptions:

- current kept "with and without the hyphen"; cleaned dropped "with and"
- current gave "All approved."; cleaned gave "Well approved."
- current gave "Have that form on the button?"; cleaned gave "How about that
  form on the button?"
- current gave "Sorry,Post-Hog"; cleaned gave "Sorry, post-Hog"

Two better each way. NOT shipped. The reasoning was sound and the measurement
said no — the same outcome as TASK-VTT145 (Suppress steady background noise
before inference), where an obviously-correct audio improvement measured mixed
on real speech.

### What is honestly unresolved

Either the degradation lies in something none of these six metrics captures, or
it is the salience of a few recent errors rather than a trend. Today's A/B
surfaced real ones, and a handful is enough to feel like a slide. No cause was
manufactured to close the question.

The durable answer is the record itself: from the moment archiving runs, every
recording is stored with the text it produced, and this harness diffs any two
configurations over them. The next time the question is asked it is a
measurement rather than an argument.

## Evidence

```
cargo test --workspace: 173 passed; 0 failed; 1 ignored
cargo clippy --workspace --all-targets -- -D warnings: clean
canary (French prompt): 3/3 changed  -> harness genuinely varies
prompt A/B (10 recordings): 6/10 identical, 4 changed
```

## Follow-up

The corpus is currently the 20-file debug ring, which rotates. Once the archive
accumulates, the harness switches to it automatically and gains a stable,
growing, transcript-paired corpus. Scoring against that stored transcript as
ground truth — rather than only diffing two runs — is the natural next step and
is not built.

## Dependencies

- Story: STORY-VTT018
- Directive: DIRECT-VTT002
