#!/usr/bin/env bash
# Validate the source-controlled regression gates themselves.
#
# This is intentionally dependency-free: it runs before Cargo is installed in
# CI and from the local pre-push hook. It protects the contract between the
# four local/CI checks and the release workflow's availability guarantees.

set -euo pipefail

ROOT="${REPO_ROOT:-$(git rev-parse --show-toplevel)}"
CI_WORKFLOW="$ROOT/.github/workflows/ci.yml"
HOOK="$ROOT/scripts/git-hooks/pre-push"
RELEASE_WORKFLOW="$ROOT/.github/workflows/release.yml"

fail() {
    echo "[regression-gates] ERROR: $*" >&2
    exit 1
}

require_literal() {
    local file="$1"
    local literal="$2"
    local description="$3"

    grep -Fq -- "$literal" "$file" || fail "$description ($file)"
}

for file in "$CI_WORKFLOW" "$HOOK" "$RELEASE_WORKFLOW"; do
    [[ -f "$file" ]] || fail "missing required file: $file"
done

# These commands are the regression gate. Keep the hook and GitHub Actions in
# lockstep so a green local push means the same thing as a green CI run.
for check in \
    'cargo fmt --all -- --check' \
    'cargo clippy --release --all-targets -- -D warnings' \
    'cargo test --release' \
    'cargo build --release'; do
    require_literal "$CI_WORKFLOW" "$check" "CI is missing gate: $check"
    require_literal "$HOOK" "$check" "pre-push is missing gate: $check"
done

# CI must run for the protected branch and its pull requests.
require_literal "$CI_WORKFLOW" '  push:' 'CI is missing push trigger'
require_literal "$CI_WORKFLOW" '    branches: [main]' 'CI push trigger does not target main'
require_literal "$CI_WORKFLOW" '  pull_request:' 'CI is missing pull_request trigger'

job_has_timeout() {
    local job="$1"

    awk -v target="  ${job}:" '
        $0 == target { in_job=1; found=0; next }
        in_job && /^  [A-Za-z0-9_-]+:/ { exit(found ? 0 : 1) }
        in_job && /^    timeout-minutes:/ { found=1 }
        END {
            if (in_job && found) exit 0
            exit 1
        }
    ' "$RELEASE_WORKFLOW"
}

# Every release job must fail instead of waiting forever on a queued runner.
for job in build-and-release build-windows-msi build-macos-arm build-macos-intel publish-release; do
    require_literal "$RELEASE_WORKFLOW" "  ${job}:" "release workflow is missing job: $job"
    job_has_timeout "$job" || fail "release job has no timeout-minutes: $job"
done

# The Intel runner is explicitly best-effort and must never block publication.
publish_block="$(awk '
    $0 == "  publish-release:" { in_publish=1; next }
    in_publish && /^  [A-Za-z0-9_-]+:/ { exit }
    in_publish { print }
' "$RELEASE_WORKFLOW")"
publish_needs="$(grep -F 'needs:' <<<"$publish_block" | head -1)"

grep -Fq 'needs: [build-and-release, build-windows-msi, build-macos-arm]' <<<"$publish_needs" \
    || fail 'publish-release must wait for Linux, Windows, and macOS ARM only'
grep -Fq 'build-macos-intel' <<<"$publish_needs" \
    && fail 'publish-release must not depend on the best-effort Intel build'

echo "[regression-gates] CI, pre-push, and release workflow contracts are valid"
