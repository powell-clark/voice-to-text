#!/usr/bin/env bash
#
# Generate GitHub release notes for a version: the matching CHANGELOG.md section
# followed by a downloads table with direct asset links for every platform.
#
# Usage: scripts/gen-release-notes.sh <version> <repo> <tag>
#   version  e.g. 2.3.1   (no leading v)
#   repo     e.g. powell-clark/voice-to-text
#   tag      e.g. v2.3.1
#
# CI (release.yml) calls this and feeds the output to softprops/action-gh-release
# via body_path. Run it locally to preview a release's notes before tagging.
set -euo pipefail

VERSION="${1:?version required}"
REPO="${2:?repo required}"
TAG="${3:?tag required}"

# Pull the "## [VERSION] — date" section out of CHANGELOG.md, up to the next
# "## [" heading. This is what actually changed in this release.
SECTION="$(awk -v ver="$VERSION" '
  $0 ~ "^## \\[" ver "\\]" { flag = 1; print; next }
  /^## \[/ && flag { exit }
  flag { print }
' CHANGELOG.md)"

if [ -z "$SECTION" ]; then
  SECTION="## v${VERSION}

See the [CHANGELOG](https://github.com/${REPO}/blob/main/CHANGELOG.md)."
fi

# Body = changelog section + downloads table + full-changelog link. The asset
# URLs follow GitHub's predictable releases/download/<tag>/<file> pattern, so
# they resolve once each build job uploads its artifact.
{
  printf '%s\n\n' "$SECTION"
  printf '## Downloads\n\n'
  printf '| Platform | Download | Install |\n'
  printf '|----------|----------|---------|\n'
  printf '| **Windows 11** | [`voice-to-text-installer.msi`](https://github.com/%s/releases/download/%s/voice-to-text-installer.msi) | Run the installer. Push-to-talk = **Scroll Lock**. GPU-accelerated (Vulkan). |\n' "$REPO" "$TAG"
  printf '| **Linux (Ubuntu)** | [`vtt-linux`](https://github.com/%s/releases/download/%s/vtt-linux) or PPA | `sudo add-apt-repository ppa:powellclark/voice-to-text && sudo apt install voice-to-text` |\n' "$REPO" "$TAG"
  printf '| **macOS** (Apple Silicon) | [`vtt-macos`](https://github.com/%s/releases/download/%s/vtt-macos) | Unsigned: `chmod +x vtt-macos && xattr -d com.apple.quarantine vtt-macos`, then run. Signed `.app` is planned. |\n' "$REPO" "$TAG"
  printf '\n**Full changelog:** [%s](https://github.com/%s/blob/main/CHANGELOG.md)\n' "$TAG" "$REPO"
}
