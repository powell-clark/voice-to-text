---
id: FEAT-VTT008
status: maintained
kano: must-have
---

# FEAT-VTT008: APT PPA distribution for Ubuntu (Noble + Jammy)

## Description
VTT is distributed via a Launchpad PPA (`ppa:powellclark/voice-to-text`) for Ubuntu Noble (24.04) and Jammy (22.04). Users install with `sudo add-apt-repository ppa:powellclark/voice-to-text && sudo apt install voice-to-text`. The PPA ships the Rust binary as of v2.0.0.

## Acceptance Criteria
- [x] `sudo apt install voice-to-text` installs successfully on Ubuntu 24.04 Noble from the PPA — verified across v2.0.0, v2.0.4, v2.0.5
- [x] Installed binary is the Rust build (not the legacy C binary) — verified by checking file size and `strings` for Rust runtime markers
- [x] Default Whisper model is downloaded by postinst and VTT transcribes on first launch without a second download prompt — verified
- [x] `apt upgrade` picks up new PPA releases correctly — verified across patch upgrades
- [x] PPA includes at least Noble and Jammy series — verify on Launchpad PPA page

## Linked Tasks
- TASK-VTT008, TASK-VTT039

## Parent Story
- STORY-VTT002
