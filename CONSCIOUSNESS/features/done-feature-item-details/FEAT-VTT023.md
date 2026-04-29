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
1. `debian/control` Depends list contains no `python3`, `python3-pip`, `python3-*`, or `pipx` entries
2. `dpkg -c voice-to-text_2.0.0_amd64.deb` output contains zero `.py` file entries
3. `grep -rn "python3\|\\.py\\b" debian/ src/` returns zero hits (excluding dead-code archives)
4. `cargo tree --package voice-to-text` shows no transitive Python dependency
5. Fresh Docker install on minimal ubuntu:noble: `apt install -y voice-to-text` then `apt list --installed | grep python` returns only pre-existing unrelated entries

## Linked Tasks
- TASK-VTT025, TASK-VTT031, TASK-VTT032

## Parent Story
- STORY-VTT010
