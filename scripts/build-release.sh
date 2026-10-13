#!/usr/bin/env bash
# Release build with this machine's paths remapped out of the binary.
#
# Usage: bash scripts/build-release.sh [cargo build args, e.g. --offline --locked]
# Output: target/release/vtt
#
# rustc writes absolute source paths into panic locations, and whisper.cpp's C
# and C++ write them through __FILE__, so a binary built under a home directory
# ships the builder's home path and username. --remap-path-prefix (Rust) and
# -ffile-prefix-map (C and C++, which cc and cmake pick up from CFLAGS and
# CXXFLAGS) rewrite those prefixes. Every path is read here at run time; none
# is written into the repo. When several prefixes match, the last one given
# wins, so the broad $HOME comes first and the narrower paths after it.
#
# The build is refused afterwards if the binary still holds $HOME.

set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

# Each prefix in its given and its symlink-resolved form, broadest first.
maps=()
add_map() {
    local from=$1 to=$2 real
    [ -n "$from" ] && [ "$from" != / ] && [ -d "$from" ] || return 0
    case $from in *[[:space:]]*)
        echo "build-release: a build path contains whitespace, which RUSTFLAGS and CFLAGS cannot carry" >&2
        exit 1 ;;
    esac
    maps+=("$from=$to")
    real=$(cd "$from" && pwd -P)
    [ "$real" = "$from" ] || maps+=("$real=$to")
}
add_map "$HOME" "~"
add_map "${RUSTUP_HOME:-$HOME/.rustup}" /rustup
add_map "${CARGO_HOME:-$HOME/.cargo}" /cargo
# From a git worktree, sources can still come from the main checkout (a
# vendor/ directory there, when cargo is set to vendor), so that root is mapped
# too, before this one, since a worktree may sit inside it.
main_git=$(git -C "$root" rev-parse --path-format=absolute --git-common-dir 2>/dev/null || true)
case $main_git in
    */.git) [ "${main_git%/.git}" = "$root" ] || add_map "${main_git%/.git}" /build/voice-to-text ;;
esac
add_map "$root" /build/voice-to-text

rust_flags=""
c_flags=""
for m in "${maps[@]}"; do
    rust_flags="$rust_flags --remap-path-prefix=$m"
    c_flags="$c_flags -ffile-prefix-map=$m"
done
export RUSTFLAGS="${RUSTFLAGS:-}$rust_flags"
export CFLAGS="${CFLAGS:-}$c_flags"
export CXXFLAGS="${CXXFLAGS:-}$c_flags"
echo "build-release: ${#maps[@]} path prefix(es) remapped for rustc and the C/C++ compilers"

cargo build --release "$@"

bin=target/release/vtt
[ -x "$bin" ] || { echo "build-release: $bin was not produced" >&2; exit 1; }
leaks=$(grep -c -a -F -- "$HOME" "$bin" || true)
if [ "$leaks" -ne 0 ]; then
    echo "build-release: REFUSED: $bin still holds the home directory path ($leaks line(s)); do not ship it" >&2
    echo "build-release: path remapping cannot reach a path the source bakes in, such as env!(\"CARGO_MANIFEST_DIR\")" >&2
    exit 1
fi
echo "build-release: $bin holds no home directory path"
