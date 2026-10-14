#!/usr/bin/env bash
# Smoke test: transcribe one known clip with a built vtt and one model, and
# fail unless the transcript holds every expected word.
#
#   bash scripts/ci/smoke-transcribe.sh <vtt-binary> <model> <clip> <word>...
#
# Fails when the binary exits non-zero (an engine crash included), prints
# nothing, or misses an expected word. Runs on Linux, macOS and Windows (Git
# Bash). The model is chosen through settings.conf: on Linux in a throwaway
# data directory; on macOS and Windows, where the app reads one fixed place,
# any settings.conf already there is set aside and put back afterwards.
# Models come from the normal cache and are downloaded on first use.
set -euo pipefail

if [ "$#" -lt 4 ]; then
    echo "usage: $0 <vtt-binary> <model> <clip> <word>..." >&2
    exit 2
fi
bin="$1"
model="$2"
clip="$3"
shift 3

data=""
conf_dir=""
case "$(uname -s)" in
    Linux*)
        data="$(mktemp -d)"
        export XDG_DATA_HOME="$data"
        conf_dir="$data/voice-to-text"
        ;;
    Darwin*) conf_dir="$HOME/Library/Application Support/voice-to-text" ;;
    MINGW* | MSYS* | CYGWIN*)
        conf_dir="$(cygpath -u "$LOCALAPPDATA")/voice-to-text"
        bin="$(cygpath -u "$bin")"
        ;;
    *)
        echo "[smoke] unsupported OS: $(uname -s)" >&2
        exit 2
        ;;
esac
conf="$conf_dir/settings.conf"
backup="$conf.smoke-backup"
restore() {
    if [ -n "$data" ]; then
        rm -r -- "$data"
    elif [ -f "$backup" ]; then
        mv -f -- "$backup" "$conf"
    else
        rm -f -- "$conf"
    fi
}
trap restore EXIT
mkdir -p "$conf_dir"
if [ -z "$data" ] && [ -f "$conf" ]; then
    mv -- "$conf" "$backup"
fi
printf 'model="%s"\nlanguage="en"\n' "$model" > "$conf"

echo "[smoke] $model on $(basename "$clip")"
if ! text="$("$bin" --file "$clip")"; then
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
