#!/usr/bin/env bash
# Generate a per-platform spec view from maintained feature cards' own
# "Cross-platform acceptance criteria" sections (ADR-0008). TASK-VTT080,
# extended by TASK-VTT172 to replace docs/PLATFORM-PARITY.md entirely: this
# is now the authoritative generated view, prepending the path-conventions
# table (docs/PLATFORM-PATHS.md) and appending a synthesised open-gaps list.
#
# This aggregates what each card already says under DIRECT-VTT005 — it does
# not invent new judgements. Cards without that section (single-platform
# capabilities, or `status: done` cards — see TASK-VTT166 for the exclusion
# reasoning) are skipped, not silently treated as "no gaps".
#
# Usage: scripts/generate-platform-spec.sh [> docs/GENERATED-PLATFORM-SPEC.md]
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

CARDS_DIR="CONSCIOUSNESS/features/maintained-done-feature-item-details"
PATHS_FILE="docs/PLATFORM-PATHS.md"

echo "# Generated per-platform spec (ADR-0008)"
echo
echo "**Generated file — do not hand-edit.** Re-run \`scripts/generate-platform-spec.sh\`"
echo "after editing a card's Cross-platform acceptance criteria section instead."
echo
echo "Aggregated from each maintained card's own \"Cross-platform acceptance"
echo "criteria (DIRECT-VTT005 parity spec)\" section — nothing here is"
echo "hand-written, so it cannot drift from what the cards say. Cards without"
echo "that section are not listed (see TASK-VTT166 for the exclusion"
echo "reasoning — a few cards are single-platform or \`status: done\` and"
echo "genuinely don't qualify)."
echo
echo "Generated: $(date -u '+%Y-%m-%d %H:%M UTC')"
echo

if [ -f "$PATHS_FILE" ]; then
    echo "---"
    echo
    cat "$PATHS_FILE"
    echo
fi

found=0
for card in "$CARDS_DIR"/FEAT-*.md; do
    [ -f "$card" ] || continue
    grep -q "^## Cross-platform acceptance criteria" "$card" || continue
    found=1

    # Cards carry YAML frontmatter (--- ... ---) before the title, so the
    # title is the first "# FEAT-..." heading, not necessarily line 1.
    title=$(grep -m1 '^# FEAT-' "$card" | sed 's/^# //')
    echo "---"
    echo
    echo "## ${title}"
    echo

    # Print the section body: everything from the "Cross-platform..."
    # heading up to (not including) the next "## " heading.
    awk '
        /^## Cross-platform acceptance criteria/ { p=1; next }
        p && /^## / { exit }
        p { print }
    ' "$card"
    echo
done

if [ "$found" -eq 0 ]; then
    echo "No maintained card carries a Cross-platform acceptance criteria section yet." >&2
    exit 1
fi

# --- Open gaps (TASK-VTT172) ---------------------------------------------
# Scan every card's Cross-platform section for unchecked bullets and pull
# out the TASK-VTT id each one references. Card order, not a fabricated
# priority sort — real priority already lives in TASK-BACKLOG-INDEX.md.
echo "---"
echo
echo "## Open gaps"
echo
echo "Every unchecked (\`- [ ]\`) bullet across the cards above that names a"
echo "task — extracted, not hand-maintained. A bullet with no task reference"
echo "is listed as-is; check the card itself for context."
echo

for card in "$CARDS_DIR"/FEAT-*.md; do
    [ -f "$card" ] || continue
    grep -q "^## Cross-platform acceptance criteria" "$card" || continue
    title=$(grep -m1 '^# FEAT-' "$card" | sed 's/^# //')

    section=$(awk '
        /^## Cross-platform acceptance criteria/ { p=1; next }
        p && /^## / { exit }
        p { print }
    ' "$card")

    gaps=$(printf '%s\n' "$section" | grep -E '^\s*- \[ \]')
    [ -z "$gaps" ] && continue

    echo "**${title}**"
    printf '%s\n' "$gaps" | sed 's/^\s*- \[ \] //'
    echo
done
