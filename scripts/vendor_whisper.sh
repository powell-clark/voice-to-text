#!/usr/bin/env bash
set -euo pipefail

DEST="third_party/whisper.cpp"
REPO_URL="https://github.com/ggerganov/whisper.cpp"

if [ -d "$DEST/.git" ] || [ -d "$DEST/.svn" ] || [ -d "$DEST/.hg" ]; then
  echo "whisper.cpp already present in $DEST"
  exit 0
fi

if [ -d "$DEST" ] && [ -n "$(ls -A "$DEST" 2>/dev/null || true)" ]; then
  echo "Directory $DEST exists and is not empty. Aborting to avoid overwriting."
  exit 1
fi

mkdir -p third_party

if command -v git >/dev/null 2>&1; then
  if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "Adding whisper.cpp as a submodule..."
    git submodule add --depth=1 "$REPO_URL" "$DEST" || {
      echo "Falling back to plain clone..."; \
      git clone --depth=1 "$REPO_URL" "$DEST"; \
    }
  else
    echo "Cloning whisper.cpp..."
    git clone --depth=1 "$REPO_URL" "$DEST"
  fi
else
  echo "git not found. Please install git and rerun."
  exit 1
fi

echo "Done. Source at $DEST"

