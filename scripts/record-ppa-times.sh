#!/bin/bash
set -euo pipefail

# Record Launchpad build-record timings for a released version (TASK-VTT134).
#
# Usage: scripts/record-ppa-times.sh [VERSION]
#        VERSION defaults to the top of debian/changelog.
#
# Reads the real build record and the real binary publication record from the
# Launchpad API — never wall-clock guesses — and appends one row per
# (version, series) to packaging/linux/ppa-build-times.tsv.
#
# The row separates three intervals that get conflated as "the PPA is slow":
#   queue_wait      build start  - upload         (waiting for a builder)
#   build_time      build finish - build start    (actually compiling)
#   publish_wait    binary published - build finish (Launchpad's publisher cycle)
# Measured on 2.3.10 these were 28s, 7m51s and 3h54m — so build tuning buys
# nothing and the publisher cycle is the whole story.
#
# `apt install` works when the BINARY is published, not when the source is.
# Availability is therefore always read from getPublishedBinaries.

cd "$(dirname "$0")/.."

PPA_OWNER="powellclark"
PPA_NAME="voice-to-text"
API="https://api.launchpad.net/devel/~${PPA_OWNER}/+archive/ubuntu/${PPA_NAME}"
OUT="packaging/linux/ppa-build-times.tsv"

VERSION="${1:-$(head -1 debian/changelog | grep -oP '\(.*?\)' | tr -d '()')}"

for tool in curl jq; do
    command -v "$tool" >/dev/null || { echo "ERROR: $tool is required"; exit 1; }
done

echo "=== Recording Launchpad timings for ${VERSION} ==="

builds=$(curl -sf "${API}?ws.op=getBuildRecords&source_name=${PPA_NAME}") \
    || { echo "ERROR: could not reach the Launchpad build-records API"; exit 1; }
binaries=$(curl -sf "${API}?ws.op=getPublishedBinaries&binary_name=${PPA_NAME}&exact_match=true") \
    || { echo "ERROR: could not reach the Launchpad published-binaries API"; exit 1; }

if [ ! -f "$OUT" ]; then
    {
        printf '# Launchpad timings per (version, series), from the build record (TASK-VTT134).\n'
        printf '# queue_wait = build_start - uploaded; build_time = build_finish - build_start;\n'
        printf '# publish_wait = binary_published - build_finish; upload_to_available = the total.\n'
        printf '# apt installs BINARIES: availability comes from getPublishedBinaries, never getPublishedSources.\n'
        printf 'version\tseries\tarch\tuploaded_utc\tbuild_start_utc\tbuild_finish_utc\tbinary_published_utc\tqueue_wait\tbuild_time\tpublish_wait\tupload_to_available\tstate\n'
    } > "$OUT"
fi

# Seconds between two ISO-8601 instants, or empty when either is missing.
delta() {
    [ -n "${1:-}" ] && [ -n "${2:-}" ] || { echo ""; return; }
    echo $(( $(date -d "$2" +%s) - $(date -d "$1" +%s) ))
}

# Seconds as 4h02m37s / 11m07s / 28s, dropping units that are zero.
human() {
    local s="${1:-}"
    [ -n "$s" ] || { echo "pending"; return; }
    local h=$(( s / 3600 )) m=$(( (s % 3600) / 60 )) sec=$(( s % 60 ))
    if   [ "$h" -gt 0 ]; then printf '%dh%02dm%02ds\n' "$h" "$m" "$sec"
    elif [ "$m" -gt 0 ]; then printf '%dm%02ds\n' "$m" "$sec"
    else                      printf '%ds\n' "$sec"
    fi
}

rows=0
while IFS=$'\t' read -r ver series arch created started built state; do
    [ -n "$ver" ] || continue

    published=$(printf '%s' "$binaries" | jq -r --arg v "$ver" \
        '[.entries[] | select(.binary_package_version == $v) | .date_published] | first // ""')

    qw=$(delta "$created" "$started")
    bt=$(delta "$started" "$built")
    pw=$(delta "$built" "$published")
    total=$(delta "$created" "$published")

    if grep -q "^${ver}	${series}	" "$OUT" 2>/dev/null; then
        # Re-recording an existing row: availability may have landed since.
        grep -v "^${ver}	${series}	" "$OUT" > "${OUT}.tmp" && mv "${OUT}.tmp" "$OUT"
        echo "  updating existing row for ${ver} ${series}"
    fi

    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
        "$ver" "$series" "$arch" "$created" "$started" "$built" \
        "${published:-pending}" "$(human "$qw")" "$(human "$bt")" \
        "$(human "$pw")" "$(human "$total")" "$state" >> "$OUT"

    echo "  ${ver} ${series}/${arch}: queued $(human "$qw"), built $(human "$bt"), published after $(human "$pw") — total $(human "$total")"
    rows=$((rows + 1))
done < <(printf '%s' "$builds" | jq -r --arg v "$VERSION" '
    .entries[]
    | select(.source_package_version | startswith($v))
    | [ .source_package_version,
        (.distro_series_link | split("/") | last),
        .arch_tag,
        .datecreated,
        (.date_started // ""),
        (.datebuilt // ""),
        .buildstate ]
    | @tsv')

if [ "$rows" -eq 0 ]; then
    echo "  no build records found for ${VERSION} — has it been uploaded yet?"
    exit 1
fi

echo ""
echo "Wrote ${rows} row(s) to ${OUT}"
echo "Rows still reading 'pending' mean the binary is not in the apt index yet;"
echo "re-run this script later to fill them in."
