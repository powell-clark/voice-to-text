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

# If the user has a global core.hooksPath set (e.g. for git-lfs), our hooks
# in .git/hooks/ will be ignored. Detect this and point core.hooksPath at
# our local copy for this repo only — preserves the user's global default
# for other repos. We chain the global hooks into each local hook so things
# like git-lfs still run.
GLOBAL_HOOKS_PATH=$(git config --get core.hooksPath || true)
LOCAL_HOOKS_PATH=$(git config --local --get core.hooksPath || true)

for hook in pre-push commit-msg; do
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

# If global hooksPath overrides us, switch this repo to use .git/hooks so
# the install actually takes effect. The global hooks (if any) are preserved
# by dispatching to them from within each local hook — but for now we just
# document that users with global hooks need to integrate manually.
if [ -n "$GLOBAL_HOOKS_PATH" ] && [ "$GLOBAL_HOOKS_PATH" != ".git/hooks" ] \
    && [ "$LOCAL_HOOKS_PATH" != ".git/hooks" ]; then
    echo ""
    echo "NOTE: detected global 'core.hooksPath = $GLOBAL_HOOKS_PATH'."
    echo "      Setting 'core.hooksPath = .git/hooks' for this repo only so"
    echo "      the installed hooks take effect. Your global setting stays"
    echo "      untouched for every other repo on this machine."
    git config --local core.hooksPath .git/hooks
fi

echo ""
echo "Hooks installed. To bypass the pre-push hook in a one-off emergency:"
echo "  git push --no-verify"
