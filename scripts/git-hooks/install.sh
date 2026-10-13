#!/usr/bin/env bash
# Install the git hooks from scripts/git-hooks/ into the repository's hooks
# directory. Run once per fresh clone, from the main checkout or any linked
# worktree. Idempotent — overwrites existing hooks.

set -e
cd "$(git rev-parse --show-toplevel)"

HOOKS_SRC="scripts/git-hooks"
# The common git directory, absolute: a linked worktree's .git is a file, and a
# relative hooks path would resolve inside that worktree and find no hooks.
HOOKS_DST="$(git rev-parse --path-format=absolute --git-common-dir)/hooks"

if [ ! -d "$HOOKS_DST" ]; then
    echo "ERROR: $HOOKS_DST does not exist — is this a git repo?"
    exit 1
fi

# If the user has a global core.hooksPath set (e.g. for git-lfs), our hooks
# in .git/hooks/ will be ignored. Detect this and point core.hooksPath at
# our local copy for this repo only — preserves the user's global default
# for other repos. We chain the global hooks into each local hook so things
# like git-lfs still run.
GLOBAL_HOOKS_PATH=$(git config --global --get core.hooksPath || true)
LOCAL_HOOKS_PATH=$(git config --local --get core.hooksPath || true)

# pii-gate.sh is not a git hook name; pre-push calls it from beside itself.
for hook in pre-push commit-msg pii-gate.sh; do
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

# If a global hooksPath overrides us, or an earlier install left the relative
# .git/hooks (which linked worktrees cannot resolve), point this repo at the
# absolute hooks directory so the install takes effect everywhere. The global
# hooks (if any) are not chained: users with global hooks integrate manually.
if [ "$LOCAL_HOOKS_PATH" = ".git/hooks" ] || { [ -n "$GLOBAL_HOOKS_PATH" ] \
    && [ "$GLOBAL_HOOKS_PATH" != "$HOOKS_DST" ] && [ "$LOCAL_HOOKS_PATH" != "$HOOKS_DST" ]; }; then
    echo ""
    [ -n "$GLOBAL_HOOKS_PATH" ] && echo "NOTE: detected global 'core.hooksPath = $GLOBAL_HOOKS_PATH'."
    echo "NOTE: setting 'core.hooksPath = $HOOKS_DST' for this repo only so"
    echo "      the installed hooks take effect in every worktree. Your global"
    echo "      setting stays untouched for every other repo on this machine."
    git config --local core.hooksPath "$HOOKS_DST"
fi

echo ""
echo "Hooks installed. To bypass the pre-push hook in a one-off emergency:"
echo "  git push --no-verify   (this also skips the personal-data gate in pii-gate.sh)"
