---
id: FEAT-VTT008
status: maintained
kano: must-have
---

# FEAT-VTT008: APT PPA distribution for Ubuntu (Noble + Jammy)

## Description
VTT is distributed via a Launchpad PPA (`ppa:powellclark/voice-to-text`) for Ubuntu Noble (24.04) and Jammy (22.04). Users install with `sudo add-apt-repository ppa:powellclark/voice-to-text && sudo apt install voice-to-text`. The PPA ships the Rust binary as of v2.0.0.

## Acceptance Criteria
- [x] **AC-1** — `sudo apt install voice-to-text` installs successfully on Ubuntu 24.04 Noble from the PPA — verified across v2.0.0, v2.0.4, v2.0.5
- [x] **AC-2** — Installed binary is the Rust build (not the legacy C binary) — verified by checking file size and `strings` for Rust runtime markers
- [x] **AC-3** — Default Whisper model is downloaded by postinst and VTT transcribes on first launch without a second download prompt — verified
- [x] **AC-4** — `apt upgrade` picks up new PPA releases correctly — verified across patch upgrades
- [x] **AC-5** — PPA includes at least Noble and Jammy series — verify on Launchpad PPA page

## Cross-platform acceptance criteria (DIRECT-VTT005 parity spec)
Anchored to `docs/PLATFORM-PARITY.md` §6 (packaging/distribution). Each platform's installer is a genuinely different mechanism (apt/dpkg vs `cargo wix` MSI vs none yet) achieving the same "one native-feeling install" outcome — same shape as FEAT-VTT004's tray section comparing GTK vs tray-icon/muda.

last_tested: { linux: null, windows: null, macos: null } — ADR-0008 starts freshness tracking clean rather than backfilling guessed dates.

**🐧 Linux — ✅ works** (this card)
- [x] `.deb` via Launchpad PPA, `debian/postinst` pre-provisions the default model (FEAT-VTT028)
- [x] Branded icon + version info embedded in the binary is N/A here (no Windows-style VERSIONINFO resource on ELF); `.desktop` file carries the app metadata instead

**🪟 Windows — 🟡 partial**
- [x] `.msi` via `cargo wix` (`wix/main.wxs`), built only from `release.yml` on a tag push — Start Menu shortcut, branded icon + ProductVersion/FileVersion resource (TASK-VTT108)
- [ ] Default model NOT pre-provisioned by the installer — first launch downloads ~465 MB with only a tray-tooltip progress indicator, no `.deb`-postinst equivalent — TASK-VTT101 (open)
- [ ] In-place upgrade (`MajorUpgrade`) not yet verified against a real prior-version install — TASK-VTT168 (open)

**🍎 macOS — ❌ missing**
- [ ] No `.app` bundle or `.dmg`/installer pipeline exists yet — blocked on TASK-VTT040

## Linked Tasks
- TASK-VTT008, TASK-VTT039

## Parent Story
- STORY-VTT002
