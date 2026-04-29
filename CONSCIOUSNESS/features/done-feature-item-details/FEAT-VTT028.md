# FEAT-VTT028: Default model pre-downloaded via postinst

## Kano
must-have (p1)

## Description
On first install from the PPA, a postinst script downloads `ggml-small.en.bin` (~488 MB) to `/usr/share/voice-to-text/models/` so that the user can begin transcribing immediately after install without another download step. Because the .deb file itself stays small (< 5 MB), the PPA upload remains fast and the `apt install` progress bar surfaces the model download clearly.

## User Observable Behaviour
- `sudo apt install voice-to-text` produces visible output: `Downloading default Whisper model (ggml-small.en.bin, 488MB)...` followed by progress (via `curl`)
- On successful install, `/usr/share/voice-to-text/models/ggml-small.en.bin` exists with correct size and SHA
- First launch of VTT: tray appears, `Ready` status within 5 seconds, transcription works without a separate download
- If network was unavailable during install, install still succeeds; first VTT launch prompts the user to either connect to the internet or select a model (graceful fallback)
- On package purge (`apt purge voice-to-text`), the shared model cache at `/usr/share/voice-to-text/models/` is removed; the user's personal cache at `~/.cache/voice-to-text/models/` is preserved

## Acceptance Criteria
1. `debian/postinst` downloads `ggml-small.en.bin` using `curl --fail --silent --show-error -L --max-time 300` to a `.tmp` file, verifies SHA-256, and atomically renames
2. Postinst is idempotent — running twice with the file present is a no-op
3. Postinst network failure prints a user-friendly warning but does not fail the install (`exit 0`)
4. `src/models.rs::model_path` checks `/usr/share/voice-to-text/models/` before `~/.cache/voice-to-text/models/` so system-installed models are preferred when present
5. `debian/postrm` on `purge` removes `/usr/share/voice-to-text/models/`
6. Test fresh Multipass VM: `sudo apt install voice-to-text` shows the download output and the binary can immediately transcribe without a subsequent download prompt

## Linked Tasks
- TASK-VTT037

## Parent Story
- STORY-VTT011
