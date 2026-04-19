#!/usr/bin/env bash
# Install the git hooks from scripts/git-hooks/ into .git/hooks/.
# Run once per fresh clone. Idempotent — overwrites existing hooks.

set -e
cd "$(git rev-parse --show-toplevel)"

HOOKS_SRC="scripts/git-hooks"
HOOKS_DST=".git/hooks"

if [ ! -d "$HOOKS_DST" ]; then
    echo "ERROR: $HOOKS_DST does not exist — is this a git repo?"
    exit 1
fi

for hook in pre-push; do
    src="$HOOKS_SRC/$hook"
    dst="$HOOKS_DST/$hook"
    if [ ! -f "$src" ]; then
        echo "skip: $src does not exist"
        continue
    fi
    cp "$src" "$dst"
    chmod +x "$dst"
    echo "installed: $dst"
done

echo ""
echo "Hooks installed. To bypass the pre-push hook in a one-off emergency:"
echo "  git push --no-verify"
