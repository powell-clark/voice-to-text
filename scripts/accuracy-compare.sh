#!/usr/bin/env bash
# Re-transcribe a fixed set of real recordings under two configurations and
# report every difference. TASK-VTT158.
#
# The point is that "accuracy feels worse" is unfalsifiable, and a model or
# prompt change is currently made blind. This turns both into a diff over the
# operator's own speech.
#
# Usage:
#   scripts/accuracy-compare.sh --baseline <binary> --candidate <binary> [-n N]
#   scripts/accuracy-compare.sh --settings-a <conf> --settings-b <conf> [-n N]
#
# The two modes answer different questions. Two binaries answer "did the code
# change transcription?". Two settings files answer "did the model, prompt or
# corrections change it?" — the same binary either way, so the only variable is
# configuration.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/voice-to-text"
CORPUS_DEFAULT="$DATA_DIR/recordings"
N=10
BASELINE=""
CANDIDATE=""
SETTINGS_A=""
SETTINGS_B=""
CORPUS="$CORPUS_DEFAULT"

while [ $# -gt 0 ]; do
    case "$1" in
        --baseline)   BASELINE="$2"; shift 2 ;;
        --candidate)  CANDIDATE="$2"; shift 2 ;;
        --settings-a) SETTINGS_A="$2"; shift 2 ;;
        --settings-b) SETTINGS_B="$2"; shift 2 ;;
        --corpus)     CORPUS="$2"; shift 2 ;;
        -n)           N="$2"; shift 2 ;;
        -h|--help)    sed -n '2,20p' "$0"; exit 0 ;;
        *) echo "unknown option: $1" >&2; exit 2 ;;
    esac
done

# Prefer the archive when it has enough material: those wavs are paired with the
# transcript that was actually typed, so a future version of this script can
# score against ground truth rather than only diffing two runs.
ARCHIVE="$DATA_DIR/archive"
if [ "$CORPUS" = "$CORPUS_DEFAULT" ] && [ -d "$ARCHIVE" ]; then
    ARCHIVE_COUNT=$(find "$ARCHIVE" -name '*.wav' 2>/dev/null | wc -l)
    if [ "$ARCHIVE_COUNT" -ge "$N" ]; then
        CORPUS="$ARCHIVE"
        echo "corpus: archive ($ARCHIVE_COUNT recordings, paired with transcripts)"
    else
        echo "corpus: debug ring — archive holds only $ARCHIVE_COUNT of $N needed"
    fi
fi

FILES=$(find "$CORPUS" -name '*.wav' -printf '%T@ %p\n' 2>/dev/null \
        | sort -rn | head -n "$N" | cut -d' ' -f2-)
[ -z "$FILES" ] && { echo "no recordings under $CORPUS" >&2; exit 1; }

run_one() {  # run_one <binary> <settings-or-empty> <wav>
    local bin="$1" conf="$2" wav="$3"
    if [ -n "$conf" ]; then
        # Settings live in the data dir; point the whole dir at a temp copy so
        # the operator's own configuration is never written to by a comparison.
        local tmp
        tmp=$(mktemp -d)
        # The app resolves $XDG_DATA_HOME/voice-to-text/settings.conf, not
        # $XDG_DATA_HOME/settings.conf. Writing to the wrong depth made every
        # comparison return "identical" because neither side ever read the
        # variant — a canary prompt in French moved nothing, which is how this
        # was caught rather than reported as a null result.
        mkdir -p "$tmp/voice-to-text"
        cp "$conf" "$tmp/voice-to-text/settings.conf"
        XDG_DATA_HOME="$tmp" timeout 300 "$bin" --file "$wav" 2>/dev/null | tail -1
        rm -rf "$tmp"
    else
        timeout 300 "$bin" --file "$wav" 2>/dev/null | tail -1
    fi
}

if [ -n "$SETTINGS_A" ] && [ -n "$SETTINGS_B" ]; then
    BIN_A="${BASELINE:-./target/release/vtt-linux}"; BIN_B="$BIN_A"
    LABEL_A="settings A"; LABEL_B="settings B"
    CONF_A="$SETTINGS_A"; CONF_B="$SETTINGS_B"
elif [ -n "$BASELINE" ] && [ -n "$CANDIDATE" ]; then
    BIN_A="$BASELINE"; BIN_B="$CANDIDATE"
    LABEL_A="baseline"; LABEL_B="candidate"
    CONF_A=""; CONF_B=""
else
    echo "need either --baseline and --candidate, or --settings-a and --settings-b" >&2
    exit 2
fi

same=0; diff=0; n=0
echo ""
for wav in $FILES; do
    n=$((n + 1))
    a=$(run_one "$BIN_A" "$CONF_A" "$wav")
    b=$(run_one "$BIN_B" "$CONF_B" "$wav")
    if [ "$a" = "$b" ]; then
        same=$((same + 1))
        echo "[$n] SAME  $(basename "$wav")"
    else
        diff=$((diff + 1))
        echo "[$n] DIFF  $(basename "$wav")"
        echo "      $LABEL_A: $a"
        echo "      $LABEL_B: $b"
    fi
done

echo ""
echo "=== $same/$n identical, $diff changed ==="
echo ""
echo "A diff is not a regression on its own — read the pairs. Whichever side"
echo "reads closer to what you actually said is the better configuration, and"
echo "that judgement is yours; this script only finds where to look."
