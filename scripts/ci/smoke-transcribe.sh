#!/usr/bin/env bash
# Smoke test: transcribe one known clip with a built vtt and one model, and
# fail unless the transcript holds every expected word.
#
#   bash scripts/ci/smoke-transcribe.sh <vtt-binary> <model> <clip> <word>...
#
# Fails when the binary exits non-zero (an engine crash included), prints
# nothing, or misses an expected word. Runs against a throwaway settings
# directory, so it never reads or changes the user's own settings; models
# come from the normal cache and are downloaded on first use.
set -euo pipefail

if [ "$#" -lt 4 ]; then
    echo "usage: $0 <vtt-binary> <model> <clip> <word>..." >&2
    exit 2
fi
bin="$1"
model="$2"
clip="$3"
shift 3

data="$(mktemp -d)"
trap 'rm -r -- "$data"' EXIT
mkdir -p "$data/voice-to-text"
printf 'model="%s"\nlanguage="en"\n' "$model" > "$data/voice-to-text/settings.conf"

echo "[smoke] $model on $(basename "$clip")"
if ! text="$(XDG_DATA_HOME="$data" "$bin" --file "$clip")"; then
    echo "[smoke] FAIL: vtt --file exited non-zero with $model" >&2
    exit 1
fi
echo "[smoke] transcript: $text"
if [ -z "${text//[[:space:]]/}" ]; then
    echo "[smoke] FAIL: $model produced no text" >&2
    exit 1
fi
lower="$(printf '%s' "$text" | tr '[:upper:]' '[:lower:]')"
missing=()
for word in "$@"; do
    case "$lower" in
        *"$word"*) ;;
        *) missing+=("$word") ;;
    esac
done
if [ "${#missing[@]}" -gt 0 ]; then
    echo "[smoke] FAIL: $model missed: ${missing[*]}" >&2
    exit 1
fi
echo "[smoke] PASS: $model heard: $*"
