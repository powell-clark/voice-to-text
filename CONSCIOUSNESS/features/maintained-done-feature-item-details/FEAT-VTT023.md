---
id: FEAT-VTT023
status: maintained
kano: must-have
verified: v2.0.0
---

# FEAT-VTT023: Pure Rust transcription — no Python runtime dependency

## Kano
must-have (p0)

## Description
The installed VTT package contains no Python scripts, no `pip install` requirements, no `faster-whisper` or `ctranslate2` libraries. The binary is a self-contained Rust executable that performs Whisper inference in-process via whisper-rs. Users who do not already have Python on their system can install and run VTT successfully.

## User Observable Behaviour
- `apt install voice-to-text` on a minimal Ubuntu system does not pull in `python3`, `python3-pip`, or any Python libraries as dependencies
- `/usr/share/voice-to-text/` contains no `.py` files
- `dpkg -L voice-to-text` lists no Python paths
- `ldd /usr/bin/vtt-linux` does not link `libpython`

## Acceptance Criteria
- [x] **AC-1** — `debian/control` Depends list contains no `python3`, `python3-pip`, `python3-*`, or `pipx` entries — verified in `debian/control` v2.0.0
- [x] **AC-2** — `dpkg -c voice-to-text_2.0.0_amd64.deb` output contains zero `.py` file entries — verified in TASK-VTT031
- [x] **AC-3** — `grep -rn "python3\|\\.py\\b" debian/ src/` returns zero hits (excluding dead-code archives) — verified post TASK-VTT031, TASK-VTT032
- [x] **AC-4** — `cargo tree --package voice-to-text` shows no transitive Python dependency — verified: no Python crates
- [ ] **AC-5** — Fresh Docker install on minimal ubuntu:noble: `apt install -y voice-to-text` then `apt list --installed | grep python` returns only pre-existing unrelated entries — not yet explicitly tested in Docker; verified via local fresh VM

## Linked Tasks
- TASK-VTT025, TASK-VTT031, TASK-VTT032

## Parent Story
- STORY-VTT010
