#!/usr/bin/env bash
# Fails when agent state is tracked: CONSCIOUSNESS/, .claude/ or chats/ at any depth.
# Run by .github/workflows/tracked-paths.yml; safe to run locally from any directory in the repo.
set -euo pipefail
unset GIT_DIR GIT_WORK_TREE

ROOT="$(git rev-parse --show-toplevel)"
tracked="$(git -C "$ROOT" ls-files -z | tr '\0' '\n' | grep -E '(^|/)(CONSCIOUSNESS|\.claude|chats)/' || true)"
if [ -n "$tracked" ]; then
    count="$(printf '%s\n' "$tracked" | wc -l)"
    echo "::error::$count tracked file(s) under CONSCIOUSNESS/, .claude/ or chats/; agent state stays local"
    printf '%s\n' "$tracked" | sed -n '1,50p'
    exit 1
fi
echo "[tracked-paths] no CONSCIOUSNESS/, .claude/ or chats/ path is tracked"
