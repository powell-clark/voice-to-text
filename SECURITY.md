# Security Policy

## Supported versions

Only the **latest released version** on the
[Launchpad PPA](https://launchpad.net/~powellclark/+archive/ubuntu/voice-to-text)
is supported for security fixes. Users should `apt upgrade voice-to-text`
to the current release before reporting a vulnerability — most issues are
already addressed there.

## Reporting a vulnerability

Please **do not** open a public GitHub issue for security-sensitive reports.
Email instead:

**emmanuel@powellclark.com** — PGP key published on
[keys.openpgp.org](https://keys.openpgp.org/) under the same email.

Include:

1. A description of the vulnerability
2. Steps to reproduce
3. Affected versions (`vtt --version`)
4. Your environment (distro, desktop environment, X11 vs Wayland)
5. Any proof-of-concept exploit code (optional but helpful)

You will get an acknowledgment within 7 days. Fixes are pushed to the PPA
within 30 days of confirmation for anything more serious than a crash;
immediately for remote or privilege-escalating issues.

## Scope

Voice to Text runs as an unprivileged user-space daemon and does not
accept network input for control (only fetches GGML model files from
HuggingFace over HTTPS). The relevant threat surface is:

- **Audio capture**: cpal opens the default input device. If an attacker
  can set the PulseAudio default, they can feed audio — but that requires
  existing user-level compromise.
- **Global hotkey**: X11 XGrabKey grabs the configured key. On X11 this
  is visible to other clients (standard X11 limitation).
- **Text injection**: enigo types transcribed text into the focused
  window. If VTT is tricked into transcribing audio at a wrong time, it
  would type into whatever is focused — but triggering requires the user
  to hold the hotkey.
- **Model downloads**: fetched over HTTPS with SHA-256 hash logged (not
  yet verified against a manifest — see
  `CONSCIOUSNESS/adr/0003-whisper-rs-in-process-model.md`).

## Non-issues

These are **not** security bugs:

- "VTT typed the wrong text into the terminal" — transcription quality
  is Whisper-dependent, not a security issue.
- "Someone saw my hotkey in the tray menu" — cosmetic.
- Missing vulnerability scanner findings from retired dependencies
  (gtk-rs 0.18 is flagged unmaintained; we plan to port to gtk4-rs
  separately but the specific vulnerability has to exist in the code
  we actually use).

## Credit

Reporters who responsibly disclose will be credited in the relevant
CHANGELOG.md entry (unless you request anonymity).
