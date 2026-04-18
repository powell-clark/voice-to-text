# TASK-VTT037: Postinst downloads default model

## Context
GGML models are too large to ship inside the .deb (~245 MB for small.en, up to 3 GB for large-v3). Instead, the first-install postinst script downloads the default model to a system-wide cache `/usr/share/voice-to-text/models/` so the user can transcribe immediately after install without a separate download step.

## Acceptance Criteria
1. `debian/postinst` runs on install with root privileges; if `/usr/share/voice-to-text/models/ggml-small.en.bin` is absent, it downloads from `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin`
2. Download uses `curl --fail --silent --show-error -L` with a 300-second timeout; on failure, prints a user-friendly message but does not fail the install ("Model download failed; you can retry by selecting a model in the VTT tray menu when VTT is running")
3. After download, verify sha256 against the known hash; on mismatch, delete the file and log a warning
4. On package upgrade (not first install), do not re-download; check existence of the file and sha
5. The script is idempotent — running it multiple times with the model present does nothing
6. `debian/postrm` on purge removes `/usr/share/voice-to-text/models/` directory contents (leaves user-space cache `~/.cache/voice-to-text/models/` untouched)
7. VTT's `models::ensure_model` checks the system-wide cache first (`/usr/share/voice-to-text/models/`) before falling back to the user cache (`~/.cache/voice-to-text/models/`)

## Technical Approach
```bash
#!/bin/sh
# debian/postinst
set -e

MODEL_DIR=/usr/share/voice-to-text/models
MODEL_FILE="$MODEL_DIR/ggml-small.en.bin"
MODEL_URL="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin"
MODEL_SHA="..." # filled in from real upstream value

case "$1" in
    configure)
        mkdir -p "$MODEL_DIR"
        if [ ! -f "$MODEL_FILE" ]; then
            echo "Downloading default Whisper model (ggml-small.en.bin, 488MB)..."
            if curl --fail --silent --show-error -L --max-time 300 -o "$MODEL_FILE.tmp" "$MODEL_URL"; then
                actual=$(sha256sum "$MODEL_FILE.tmp" | cut -d' ' -f1)
                if [ "$actual" = "$MODEL_SHA" ]; then
                    mv "$MODEL_FILE.tmp" "$MODEL_FILE"
                    echo "Model installed."
                else
                    rm "$MODEL_FILE.tmp"
                    echo "WARNING: Model integrity check failed; download skipped. Use VTT menu to retry."
                fi
            else
                rm -f "$MODEL_FILE.tmp"
                echo "WARNING: Model download failed (no network?). Use VTT menu to retry."
            fi
        fi
        ;;
esac

#DEBHELPER#

exit 0
```

Update `src/models.rs::model_path` to check `/usr/share/voice-to-text/models/` first.

## Test Strategy
Install the package on a fresh VM with network; verify the model file appears after install. Install on a VM without network; verify install succeeds with a warning, and the VTT tray can still launch (it will fail to transcribe until the user manually selects a model and triggers download).

## Files
- `debian/postinst` (create)
- `debian/postrm` (create — remove model cache on purge)
- `src/models.rs` (modify — check system cache first)
