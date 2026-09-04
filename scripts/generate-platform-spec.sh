#!/usr/bin/env bash
# Generate a per-platform spec view from maintained feature cards' own
# "Cross-platform acceptance criteria" sections (ADR-0008). TASK-VTT080.
#
# This aggregates what each card already says under DIRECT-VTT005 — it does
# not invent new judgements. Cards without that section (single-platform
# capabilities, or ones not yet migrated — see TASK-VTT166) are skipped, not
# silently treated as "no gaps".
#
# Usage: scripts/generate-platform-spec.sh [> docs/GENERATED-PLATFORM-SPEC.md]
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

CARDS_DIR="CONSCIOUSNESS/features/maintained-done-feature-item-details"

echo "# Generated per-platform spec (ADR-0008)"
echo
echo "**Generated file — do not hand-edit.** Re-run \`scripts/generate-platform-spec.sh\`"
echo "after editing a card's Cross-platform acceptance criteria section instead."
echo
echo "Aggregated from each maintained card's own \"Cross-platform acceptance"
echo "criteria (DIRECT-VTT005 parity spec)\" section — nothing here is"
echo "hand-written, so it cannot drift from what the cards say. Cards without"
echo "that section are not listed (see TASK-VTT166 for full migration; the"
echo "richer, still-authoritative hand-maintained view is"
echo "\`docs/PLATFORM-PARITY.md\`)."
echo
echo "Generated: $(date -u '+%Y-%m-%d %H:%M UTC')"
echo

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
