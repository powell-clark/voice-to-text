# ADR-0006: Audio decode + resample dependency for multi-format `--file`

**Status:** Proposed
**Date:** 2026-07-17
**Context:** voice-to-text

## Context

TASK-VTT023 shipped `--file`/`-f` batch transcription for 16 kHz mono WAV only
(`run_file_mode` in `src/main.rs:756`, `whisper::decode_wav_to_samples` in
`src/whisper.rs:94`). That function is deliberately narrow: it uses `hound` to
read WAV frames, rejects any file whose `spec.sample_rate != 16_000` with an
actionable error suggesting `ffmpeg -ar 16000 -ac 1`, and down-mixes stereo by
channel averaging. Criteria 2 and 3 of that card were deferred into
TASK-VTT130 because both need real dependency/architecture decisions rather
than a quick add:

1. Accepting `.mp3`, `.m4a`, `.flac` (criterion 2) requires a container/codec
   decoder plus a resampler — whisper.cpp's `WhisperState::full()` (called
   from `WhisperEngine::transcribe` in `src/whisper.rs:43`) only ever accepts
   16 kHz mono `f32` PCM. Arbitrary input sample rates reaching exactly
   16 kHz is the hard part, not container parsing.
2. Long files (>5 min) processed in bounded-memory chunks with stderr
   progress (criterion 3) is a separate concern — whisper already accepts a
   full buffer today, so chunking is a windowing/looping change over
   `engine.transcribe`, not a decode-dependency choice.

This ADR covers criterion 2's one-way-door dependency choice, per
`authorship-flow` (file the ADR before adding the crates) and the TASK-VTT130
card's own instruction. Criterion 3 is addressed briefly at the end but does
not gate on this decision.

### Packaging constraints that make this a real trade-off

- **Linux (.deb):** Since v2.0.2 the PPA does **not** build from source on
  Launchpad. `debian/rules` installs a binary committed at
  `packaging/linux/vtt-linux.prebuilt`, built locally by Emmanuel with
  rustup (currently Rust 1.91) and pushed via `scripts/release-ppa.sh`. The
  reason, per `debian/rules`'s own comment: Ubuntu Noble ships cargo 1.75,
  which cannot parse edition-2024 `Cargo.toml` manifests that several modern
  transitive dependencies (`getrandom 0.3`, `moxcms`, `pxfm`, `toml_edit`,
  pulled in via `cpal`/`arboard`) now require. `Cargo.lock` is also
  defensively pinned to lockfile format v3 for the same reason. This means:
  a new pure-Rust decode/resample dependency does **not** break the Noble
  build path (nothing ever compiles there), but it does add to local build
  time and to the binary Emmanuel commits to the repo on every release.
- **Windows (.msi) / macOS (.dmg):** Built from source — Windows via
  `cargo wix` locally or in CI, macOS via the self-hosted runner
  (`scripts/setup-runner.sh`) in `.github/workflows/release.yml`. These
  paths use a current Rust toolchain, so edition-2024 transitive deps are
  not a blocker there, but any *new runtime system dependency* (e.g. a
  shelled-out binary) would need to be bundled or documented as a user
  prerequisite on both platforms, which today's dependency set (`cpal`,
  `whisper-rs`, `reqwest` with `rustls-tls`) deliberately avoids — the
  project's existing pattern (ADR-0003: remove Python; `reqwest` rustls
  instead of an OpenSSL system dep) is "vendor into the binary, don't
  require the user to install anything else."
- **CI (`ci.yml`):** Runs `cargo fmt`, `cargo clippy -D warnings`,
  `cargo test --release`, `cargo build --release` on `ubuntu-24.04` on every
  push/PR. Any new dependency is exercised there before it ever reaches a
  release build.

## Decision drivers

- Reach 16 kHz mono `f32` from `.wav`, `.mp3`, `.m4a`, `.flac` (and ideally
  other containers) regardless of the input's native sample rate/channel
  layout.
- Preserve the "no new runtime system dependency" pattern already
  established by removing Python (ADR-0003) and by choosing `rustls` over
  system OpenSSL.
- Keep the Linux prebuilt-binary release flow working unchanged — the
  decision must not require Launchpad to compile anything.
- Keep Windows/macOS packaging (`cargo wix`, self-hosted runner `.dmg`)
  free of new install-time prerequisites for end users.
- Minimise added build time/binary size where a smaller option satisfies
  the requirement.

## Considered Alternatives

### (a) Pure-Rust: `symphonia` (decode) + `rubato` (resample)

`symphonia` is a pure-Rust, no-`unsafe`-by-default multi-format demuxer/
decoder (MP3, AAC/M4A via its `isomp4`+`aac` feature set, FLAC, WAV, Vorbis,
etc., selected per-format via Cargo features to control binary size).
`rubato` is a pure-Rust sinc-based sample-rate converter with steady-state
and asynchronous resamplers — the standard choice for accurate arbitrary
in-rate → out-rate conversion (44.1/48 kHz mic and download audio → whisper's
required 16 kHz).

**Pros**

- Zero new runtime system dependency on any platform — consistent with the
  existing "everything vendors into the binary" pattern.
- One dependency addition compiles identically on Linux/macOS/Windows; no
  per-platform packaging branches.
- Does not touch the Noble PPA path at all (nothing compiles there); only
  affects the local `cargo build --release` that produces
  `vtt-linux.prebuilt`, and the already-source-built Windows/macOS paths.
- `symphonia`'s per-format Cargo features let us opt into exactly
  `mp3`/`aac`/`flac`/`isomp4` (skip Vorbis/ALAC/etc.) to bound the size cost.

**Cons**

- Two new crates (plus their transitive trees) — larger `Cargo.lock`, more
  supply-chain surface, more code to reason about when something decodes
  wrong.
- Real binary-size and link-time cost. `whisper-rs-sys` already dominates
  build time (bundled whisper.cpp compile, ADR-0003); adding a demux/decode/
  resample stack lengthens `cargo build --release` further (with
  `lto = true`, `opt-level = 2` already set in `[profile.release]`) and the
  final stripped binary grows — needs to be measured, not assumed small.
- **Must verify before merging:** whether `symphonia`'s or `rubato`'s own
  transitive dependency trees pull in anything requiring edition 2024 (the
  exact failure class that forced the Noble prebuilt-binary workaround in
  the first place). This wouldn't break the PPA (which never compiles on
  Noble now) but would be worth knowing, since it forecloses ever reverting
  to a Noble source-build path per the note in `debian/rules`.
- New pure-Rust decode path is unexercised code needing its own test
  fixtures (mirroring the existing `decode_wav_to_samples` unit tests) —
  ongoing maintenance surface, not a one-time cost.

### (b) Shell out to `ffmpeg`

**Pros**

- Minimal new Rust code — `std::process::Command` piping raw PCM out of
  `ffmpeg -i <in> -f f32le -ar 16000 -ac 1 -`. No new crate, no binary-size
  growth, no new supply-chain surface in `Cargo.lock`.
- `ffmpeg` already handles every format users will realistically throw at
  `--file`, correctly, including ones `symphonia` doesn't (Opus, WMA, video
  containers with an audio track) — broader coverage than any Rust crate
  combination.
- The current WAV-only error message already tells users to run
  `ffmpeg -ar 16000 -ac 1` by hand — this option simply automates what the
  code already recommends as the manual workaround.

**Cons**

- Introduces the exact runtime system dependency this project has
  consistently avoided (Python removal in ADR-0003; `rustls` over OpenSSL).
  `--file` would silently fail (or need a "please install ffmpeg" error
  path) on any machine without it on `$PATH`.
- Debian packaging: `debian/control` would need `ffmpeg` added to
  `Depends` (it is not currently listed, and pulls in a nontrivial
  dependency chain — libavcodec etc. — onto every install, even for users
  who never touch `--file`), or be relegated to a soft `Recommends` with a
  degraded/error UX when absent.
- Windows `.msi` (`cargo wix`) and macOS `.dmg`/self-hosted-runner builds
  have no equivalent of `apt`'s dependency resolution — the installer would
  either need to bundle an `ffmpeg.exe`/`ffmpeg` binary (increases installer
  size similarly to option (a), while also raising licensing/attribution
  questions for a bundled GPL/LGPL ffmpeg build) or tell users to install it
  themselves, which is a materially worse experience than a self-contained
  binary and a support burden ("why doesn't `--file` work") that's opaque
  compared to a compile-time-caught missing crate feature.
- Subprocess spawn + pipe adds a per-invocation cost and a new failure mode
  (missing binary, non-zero exit, partial pipe writes) to handle — echoes
  the exact subprocess fragility ADR-0003 moved *away* from for the core
  transcription path (spawn tax, silently-stuck processes). Reintroducing a
  subprocess dependency for the file-decode path cuts against that
  direction even though it's a different subprocess.

### (c) Defer multi-format; ship only long-file WAV chunking now

**Pros**

- No new dependency at all — chunking is pure control-flow over the
  existing `decode_wav_to_samples` output and `engine.transcribe`, so it
  ships with zero risk to build time, binary size, or packaging.
- Delivers real user value (criterion 3) immediately while the decode
  question gets more design time.
- Keeps `--file` scope minimal and matches the "ship the interim path"
  spirit already in the current error message (rejecting non-16 kHz WAV
  with an `ffmpeg` suggestion rather than silently mis-transcribing).

**Cons**

- Leaves criterion 2 (the feature most likely to be asked for — "why can't
  I just point it at an mp3") unresolved; users must still pre-convert with
  external tooling before `--file` works.
- Doesn't remove the eventual need for this same ADR decision — only
  delays it.

### (d) Other option considered and rejected: `dasp` instead of `rubato` for resampling

`dasp` is a lower-level, more general DSP toolkit; its resampling is
simpler (linear/Sinc via `dasp_interpolate`) but less turn-key for
exact target-rate conversion than `rubato`'s purpose-built `Resampler`
trait and pre-tuned sinc parameters. `rubato` is the more actively
maintained, narrowly-scoped choice for "convert this input rate to this
output rate accurately" and is what the TASK-VTT130 card already names as
the primary candidate. Rejected as the default pick, not as unusable —
if `rubato`'s transitive dependencies prove heavier than expected, `dasp`
is the fallback to re-evaluate within option (a), not a reason to switch
to option (b) or (c).

## Recommendation (pending operator sign-off)

**Option (a): `symphonia` + `rubato`, feature-gated to the formats named in
the card (`mp3`, `isomp4`/`aac`, `flac`, plus the existing `wav` path kept on
`hound` since it already works and is well-tested).**

Rationale: it is the only option that satisfies "accept common formats at
any input sample rate" without reintroducing a system runtime dependency
that this project has twice already engineered away (CT2/Python in
ADR-0003, OpenSSL via `reqwest`'s `rustls-tls` feature). It leaves the
Linux PPA release flow completely untouched (nothing new compiles on
Launchpad), and it keeps Windows/macOS packaging free of a bundled
ffmpeg binary and its licensing questions. The real costs — larger
`Cargo.lock`, longer local build time, bigger stripped binary — are
measurable and bounded by feature selection, not open-ended risks like
"does the user have ffmpeg on `$PATH`."

**Before implementation**, TASK-VTT130's builder should: (1) run
`cargo tree` (or add the crates in a scratch branch and `cargo build
--release`) to confirm neither `symphonia` nor `rubato`'s dependency trees
require edition 2024 or otherwise regress the Noble-adjacent toolchain
story, and (2) measure the stripped-binary size delta and note it in the
task's actual-proof evidence, since "larger build/binary" here is a stated
trade-off, not a hand-waved one.

**Chunking (criterion 3)** is independent of this decision and does not
need to wait on it: it can ship first, or in parallel, as pure control flow
— window the decoded `f32` buffer into overlapping segments above a size
threshold (e.g. >5 min at 16 kHz = ~4.8M samples), transcribe each via the
existing `engine.transcribe`, concatenate to stdout, and emit
`eprintln!("chunk {n}/{total}...")`-style progress to stderr per the
existing `--file` convention of stdout-transcript-only /
stderr-progress-and-diagnostics (already established in `run_file_mode`'s
model-download progress callback at `src/main.rs:784`).

## Consequences

### Positive

- `--file` gains real multi-format support without adding a system
  dependency or degrading the "one self-contained binary" story that
  Windows/macOS/Linux packaging all currently rely on.
- Feature-gating `symphonia` bounds the binary-size/compile-time cost to
  the formats actually named in the acceptance criteria.
- Chunking can land independently, so criterion 3's user value isn't
  blocked on the decode dependency debate.

### Negative

- `Cargo.lock` grows, local release-binary build time increases on top of
  the already-nontrivial `whisper-rs-sys` compile, and the committed
  `vtt-linux.prebuilt` binary gets larger on every future release.
- New decode/resample code path needs its own unit-test fixtures (mirroring
  `decode_wav_to_samples`'s mono/stereo/rate-rejection tests) to avoid
  silently shipping a mis-resampled or mis-channel-mixed decoder — ongoing
  maintenance, not one-time.
- Supply-chain surface grows: two more crates (plus transitive trees) whose
  CVEs/breaking changes this project now tracks.

### Neutral

- No change to the Noble PPA build path — it never compiles Rust today and
  won't after this decision either.
- No change to whisper.cpp/`whisper-rs` itself; this only affects how audio
  reaches the existing 16 kHz mono `f32` buffer `engine.transcribe` expects.

## Rollback

If `symphonia`+`rubato` prove unacceptably heavy (binary size, build time)
or an edition-2024 transitive requirement surfaces that the operator judges
too risky, fall back to option (b) (`ffmpeg` shell-out, gated behind a
`Recommends`-only Debian dependency and a clear "install ffmpeg" error on
Windows/macOS) or option (c) (defer multi-format indefinitely, keep shipping
only WAV + chunking). Because this ADR's scope is isolated to the decode/
resample step of `--file` batch mode, reverting does not touch the core
push-to-talk transcription path or `WhisperEngine`.

## References

- TASK-VTT130 — `--file` multi-format decode + long-file chunking
  (`CONSCIOUSNESS/tasks/backlog-task-item-details/TASK-VTT130.md`)
- TASK-VTT023 — Batch file transcription via `--file` flag, done
  (`CONSCIOUSNESS/tasks/done-task-item-details/TASK-VTT023.md`)
- `src/whisper.rs:94` — `decode_wav_to_samples` (current WAV-only decode path)
- `src/whisper.rs:43` — `WhisperEngine::transcribe` (16 kHz mono `f32` input
  contract)
- `src/main.rs:756` — `run_file_mode` (`--file` batch entry point)
- `debian/rules` — prebuilt-binary rationale (Noble cargo 1.75 / edition
  2024 constraint)
- `scripts/release-ppa.sh` — local rustup build + `vtt-linux.prebuilt`
  commit flow
- ADR-0003 — whisper-rs in-process model (established precedent: remove
  Python/system-process dependencies, prefer vendored/pure-Rust)
- `symphonia` — https://github.com/pdeljanov/Symphonia (pure-Rust media
  demux/decode)
- `rubato` — https://github.com/HEnquist/rubato (pure-Rust sample-rate
  conversion)
