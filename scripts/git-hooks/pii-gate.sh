#!/usr/bin/env bash
# Pre-push personal-data gate. scripts/git-hooks/pre-push calls it with git's
# pre-push arguments (<remote name> <remote url>) and stdin; it also works as a
# pre-push hook on its own. Install both with: bash scripts/git-hooks/install.sh
#
# For every commit being pushed it checks the added lines, file names, commit
# message, author and committer fields, annotated tag messages and ref names
# against three layers, and refuses the push on any hit:
#   1. Identifiers derived on this machine at run time and never stored: $HOME
#      and the user's home path in its Linux, macOS and Windows forms, the
#      hostname, and the repo owner's private GitHub repo names (when gh is
#      installed and signed in).
#   2. Generic patterns: an absolute path under any home directory refuses. An
#      email address refuses unless it is git's user.email (read at run time,
#      never written here), a GitHub noreply address, a bot or noreply address,
#      or on a reserved example domain; in added lines of CHANGELOG.md or docs/
#      it warns instead.
#   3. A replacement list, when one exists (PII_LIST, default
#      ~/.local/share/pii-review/replacements.txt), through scrub.py's count
#      subcommand (PII_SCRUB_PY, else git config pii.scrubPath, else scrub.py
#      beside the list). If the list exists and its check cannot run, the push
#      is refused.
# Output is counts and file names only. A matched value is never printed, and a
# file name that itself holds one is withheld.
#
# Portable to bash 3.2 (macOS), Linux and Git Bash on Windows: no associative
# arrays or mapfile, and only POSIX awk, sed, tr, sort and find.

set -u
set -o pipefail

say() { printf '[pii-gate] %s\n' "$*"; }
refuse() { say "REFUSED: $*"; exit 1; }
tildify() {
    case $1 in
        "$HOME"|"$HOME"/*) printf '~%s' "${1#"$HOME"}" ;;
        *) printf '%s' "$1" ;;
    esac
}

remote_name=${1:-}
remote_url=${2:-}

top=$(git rev-parse --show-toplevel 2>/dev/null) || refuse "not inside a git work tree"
cd "$top" || refuse "cannot enter the work tree"

umask 077
T=$(mktemp -d "${TMPDIR:-/tmp}/pii-gate.XXXXXX") || refuse "cannot create a temp directory"
cleanup() {
    find "$T" -type f -exec rm -f {} + 2>/dev/null
    find "$T" -depth -type d -exec rmdir {} + 2>/dev/null
}
trap cleanup EXIT

# ---------------------------------------------------------------- what is pushed

cat > "$T/push"
: > "$T/commits.all"; : > "$T/tags"; : > "$T/refs"; : > "$T/ranges"
nref=0
while read -r lref lsha rref rsha; do
    [ -n "${lsha:-}" ] || continue
    case $lsha in *[!0]*) ;; *) continue ;; esac   # a deletion pushes no content
    nref=$((nref + 1))
    printf '%s\n' "$rref" >> "$T/refs"
    [ "$(git cat-file -t "$lsha" 2>/dev/null)" = tag ] && printf '%s\n' "$lsha" >> "$T/tags"
    case ${rsha:-0} in *[!0]*) old=$rsha ;; *) old= ;; esac
    if [ -n "$old" ] && git cat-file -e "$old^{commit}" 2>/dev/null; then
        printf '%s %s\n' "$lsha" "$old" >> "$T/ranges"
        git rev-list "$lsha" "^$old" >> "$T/commits.all" || refuse "cannot list the commits being pushed"
    elif [ -n "$remote_name" ]; then
        printf '%s -\n' "$lsha" >> "$T/ranges"
        git rev-list "$lsha" --not --remotes="$remote_name" >> "$T/commits.all" || refuse "cannot list the commits being pushed"
    else
        printf '%s .\n' "$lsha" >> "$T/ranges"
        git rev-list "$lsha" >> "$T/commits.all" || refuse "cannot list the commits being pushed"
    fi
done < "$T/push"
[ "$nref" -gt 0 ] || exit 0
sort -u "$T/commits.all" > "$T/commits"
ncommits=$(wc -l < "$T/commits" | tr -d ' ')

# ---------------------------------------------------------------- layer 1: derived identifiers

IDS="$T/ids"
: > "$IDS"
add_id() {   # class mode value; modes: sub (right-bounded substring), tok (whole token), ntok (whole token in a file name)
    v=$(printf '%s' "$3" | tr -d '\r' | tr 'A-Z' 'a-z')
    [ -n "$v" ] && printf '%s\t%s\t%s\n' "$1" "$2" "$v" >> "$IDS"
}

user=$(id -un 2>/dev/null || true)
[ -n "$user" ] || user=${USER:-${USERNAME:-}}
case ${HOME:-/} in /|"") ;; *) add_id home sub "$HOME" ;; esac
if [ -n "$user" ]; then
    add_id home sub "/home/$user"
    add_id home sub "/users/$user"
    add_id home sub "c:\\users\\$user"
    add_id home sub "c:/users/$user"
fi

: > "$T/hosts"
for h in "$(hostname 2>/dev/null)" "$(hostname -s 2>/dev/null)" "$(hostname -f 2>/dev/null)" \
         "$(uname -n 2>/dev/null)" "${HOSTNAME:-}" "${COMPUTERNAME:-}" \
         "$(scutil --get LocalHostName 2>/dev/null)" "$(scutil --get ComputerName 2>/dev/null)"; do
    [ -n "$h" ] || continue
    printf '%s\n' "$h" >> "$T/hosts"
    case $h in *.*) printf '%s\n' "${h%%.*}" >> "$T/hosts" ;; esac
done
sort -u "$T/hosts" -o "$T/hosts"
while IFS= read -r h; do
    lh=$(printf '%s' "$h" | tr 'A-Z' 'a-z')
    case $lh in ''|localhost|localhost.*) continue ;; esac
    [ ${#lh} -ge 2 ] || continue
    add_id host ntok "$h"
    distinctive=0
    case $lh in *[-0-9.\ \']*) distinctive=1 ;; esac
    [ ${#lh} -ge 8 ] && distinctive=1
    if [ $distinctive = 1 ]; then
        add_id host tok "$h"
    else
        # A short plain hostname is often an ordinary word, so only the forms
        # that name a machine count: user@host, host.local and friends, ssh host.
        add_id host sub "@$h"
        for d in local lan home localdomain; do add_id host tok "$h.$d"; done
        add_id host tok "ssh $h"
    fi
done < "$T/hosts"

repo_note="repo names: skipped (no GitHub remote or no gh)"
owner_repo() {
    printf '%s\n' "$1" | sed -n -E 's#^.*github\.com[:/]+([^/]+)/([^/]+)/?$#\1 \2#p' | sed -E 's#\.git$##'
}
or=$(owner_repo "$remote_url")
[ -n "$or" ] || or=$(owner_repo "$(git config --get remote.origin.url 2>/dev/null)")
if [ -n "$or" ] && command -v gh >/dev/null 2>&1; then
    owner=${or%% *}
    self=${or#* }
    gh repo list "$owner" --visibility private --limit 1000 --json name --jq '.[].name' \
        > "$T/repos" 2>/dev/null < /dev/null &
    gpid=$!
    start=$SECONDS
    i=0
    while kill -0 "$gpid" 2>/dev/null && [ $((SECONDS - start)) -lt 20 ] && [ $i -lt 400 ]; do
        sleep 0.1 2>/dev/null || sleep 1
        i=$((i + 1))
    done
    if kill -0 "$gpid" 2>/dev/null; then
        kill "$gpid" 2>/dev/null
        repo_note="repo names: skipped (gh timed out)"
    elif wait "$gpid"; then
        nrepo=0
        tr -d '\r' < "$T/repos" > "$T/repos.lf"
        while IFS= read -r n; do
            [ -n "$n" ] || continue
            [ "$n" = "$self" ] && continue
            [ "$n" = "$owner" ] && continue
            add_id repo sub "$owner/$n"
            case $n in *-*) add_id repo tok "$n" ;; esac
            nrepo=$((nrepo + 1))
        done < "$T/repos.lf"
        repo_note="repo names: $nrepo"
    else
        repo_note="repo names: skipped (gh could not list private repos; not signed in?)"
    fi
fi
sort -u "$IDS" -o "$IDS"

# One stream of records, KIND<TAB>COMMIT<TAB>PATH<TAB>TEXT, over everything pushed.
# KIND: A added line, N file name, H author/committer, M message, T tag, R ref name.
mark="PIIGATE${RANDOM}${RANDOM}X"
PARSE='
index($0, M " C ") == 1 { c = substr($0, length(M) + 4, 7); st = "meta"; next }
index($0, M " H ") == 1 { t = substr($0, length(M) + 4); printf "H\t%s\t%s\t%s\n", c, substr(t, 1, index(t, " ") - 1), t; next }
$0 == M " B" { st = "body"; next }
$0 == M " E" { st = "diff"; path = ""; inhdr = 0; next }
st == "body" { printf "M\t%s\t\t%s\n", c, $0; next }
st != "diff" { next }
/^diff / { inhdr = 1; path = ""; next }
inhdr && /^\+\+\+ / {
    p = substr($0, 5)
    if (p == "/dev/null") { path = ""; next }
    sub(/^"/, "", p); sub(/"$/, "", p); sub(/^b\//, "", p)
    path = p
    printf "N\t%s\t%s\t%s\n", c, path, path
    next
}
/^@@/ { inhdr = 0; np = (match($0, /^@+/) ? RLENGTH - 1 : 1); next }
inhdr { next }
path != "" {
    pre = substr($0, 1, np)
    if (index(pre, "+") && !index(pre, "-")) printf "A\t%s\t%s\t%s\n", c, path, substr($0, np + 1)
}'
: > "$T/stream"
if [ "$ncommits" -gt 0 ]; then
    git log --no-walk=unsorted --stdin -p --cc --root -U0 -a --no-renames --no-relative \
        --no-color --no-ext-diff --no-textconv --no-show-signature --src-prefix=a/ --dst-prefix=b/ \
        --format="$mark C %H%n$mark H author %an <%ae>%n$mark H committer %cn <%ce>%n$mark B%n%B%n$mark E" \
        < "$T/commits" | LC_ALL=C tr -d '\000' | LC_ALL=C awk -v M="$mark" "$PARSE" > "$T/stream" \
        || refuse "cannot read the commits being pushed"
fi
while IFS= read -r t; do
    git cat-file tag "$t" < /dev/null | LC_ALL=C tr -d '\000' \
        | LC_ALL=C awk -v c="$(printf '%s' "$t" | cut -c1-7)" '{ printf "T\t%s\t\t%s\n", c, $0 }' >> "$T/stream" \
        || refuse "cannot read a pushed tag"
done < "$T/tags"
LC_ALL=C awk '{ printf "R\t-\t\t%s\n", $0 }' "$T/refs" >> "$T/stream"

MATCH='
BEGIN {
    FS = "\t"
    while ((getline ln < IDS) > 0) { split(ln, f, "\t"); np++; pc[np] = f[1]; pm[np] = f[2]; pv[np] = f[3]; pl[np] = length(f[3]) }
    close(IDS)
    n = split("user username runner you yourname your-name me name someone somebody example ubuntu linuxbrew shared public default guest", w, " ")
    for (i = 1; i <= n; i++) ph[w[i]] = 1
    re = "(/home/|/Users[/]|[A-Za-z]:\\\\[Uu][Ss][Ee][Rr][Ss]\\\\|[A-Za-z]:/[Uu][Ss][Ee][Rr][Ss]/)[A-Za-z0-9._-]+"
    pre = "^(/home/|/Users[/]|[A-Za-z]:\\\\[Uu][Ss][Ee][Rr][Ss]\\\\|[A-Za-z]:/[Uu][Ss][Ee][Rr][Ss]/)"
}
function boundary(ch, mode) {
    if (ch == "") return 1
    if (mode == "ntok") return ch !~ /[a-z0-9]/
    return ch !~ /[a-z0-9_-]/
}
function occurrences(s, v, vl, mode,   n, off, pos, at) {
    n = 0; off = 0
    while ((pos = index(substr(s, off + 1), v)) > 0) {
        at = off + pos
        if (boundary(substr(s, at + vl, 1), mode) && (mode == "sub" || boundary(at > 1 ? substr(s, at - 1, 1) : "", mode))) n++
        off = at
    }
    return n
}
function allowed(e,   at, l, d) {
    if (e == CFG && CFG != "") return 1
    if (e ~ /@users\.noreply\.github\.com$/ || e == "action@github.com") return 1
    at = index(e, "@"); l = substr(e, 1, at - 1); d = substr(e, at + 1)
    if (l == "noreply" || l == "no-reply" || l == "git" || index(l, "[bot]")) return 1
    if (d ~ /^example\.(com|org|net)$/ || d ~ /\.(example|invalid|test|localhost)$/ || d == "localhost") return 1
    return 0
}
function hit(key, cls, k) {
    if (!(key in seen)) { seen[key] = 1; order[++nk] = key }
    if (!((key, cls) in cnt)) classes[key] = classes[key] " " cls
    cnt[key, cls] += k
    total += k
    split(key, kp, SUBSEP)
    if (kp[2] == "N") dirty[kp[3]] = 1
}
{
    kind = $1; c = $2; path = $3
    text = $4
    for (i = 5; i <= NF; i++) text = text "\t" $i
    s = tolower(text)
    key = c SUBSEP kind SUBSEP path
    for (p = 1; p <= np; p++) {
        if (pm[p] == "ntok" && kind != "N") continue
        k = occurrences(s, pv[p], pl[p], pm[p])
        if (k) hit(key, pc[p], k)
    }
    t = text
    while (match(t, re)) {
        lb = RSTART > 1 ? substr(t, RSTART - 1, 1) : ""
        lb2 = RSTART > 2 ? substr(t, RSTART - 2, 1) : ""
        seg = substr(t, RSTART, RLENGTH)
        t = substr(t, RSTART + RLENGTH)
        # a path inside a word or URL is not a home path, except the Git Bash drive form (slash, drive letter, slash, Users)
        if (lb ~ /[A-Za-z0-9._~-]/ && !(substr(seg, 1, 7) == "/Users/" && lb ~ /[A-Za-z]/ && lb2 == "/")) continue
        sub(pre, "", seg)
        u = tolower(seg)
        if (length(u) < 3 || (u in ph) || u ~ /^\.+$/) continue
        hit(key, "home-path", 1)
    }
    if (kind == "H" || (kind == "T" && substr(text, 1, 7) == "tagger ")) {
        if (match(text, /<[^>]*>/) && !allowed(tolower(substr(text, RSTART + 1, RLENGTH - 2)))) hit(key, "email", 1)
    } else if (kind == "A" || kind == "M" || kind == "T") {
        t = text
        while (match(t, /[A-Za-z0-9][A-Za-z0-9._%+-]*@[A-Za-z0-9-]+(\.[A-Za-z0-9-]+)*\.[A-Za-z][A-Za-z]+/)) {
            e = tolower(substr(t, RSTART, RLENGTH))
            t = substr(t, RSTART + RLENGTH)
            if (allowed(e)) continue
            if (kind == "A" && (path == "CHANGELOG.md" || path ~ /^docs\//)) {
                if (!(key in wmail)) wkey[++nw] = key
                wmail[key]++
            } else hit(key, "email", 1)
        }
    }
}
END {
    shown = 0
    for (i = 1; i <= nk; i++) {
        split(order[i], kp, SUBSEP); c = kp[1]; kind = kp[2]; path = kp[3]
        if (kind == "A" || kind == "N") label = (path in dirty) ? "<file name withheld: it holds an identifier>" : path
        if (kind == "N") label = label " (file name)"
        else if (kind == "H") label = path " field"
        else if (kind == "M") label = "commit message"
        else if (kind == "T") label = "annotated tag"
        else if (kind == "R") label = "ref name"
        line = ""
        n = split(substr(classes[order[i]], 2), cs, " ")
        for (j = 1; j <= n; j++) line = line " " cs[j] "=" cnt[order[i], cs[j]]
        if (++shown <= 200) printf "[pii-gate] HIT %s %s:%s\n", c, label, line
    }
    if (shown > 200) printf "[pii-gate] ... and %d more location(s)\n", shown - 200
    for (i = 1; i <= nw; i++) {
        split(wkey[i], kp, SUBSEP)
        label = (kp[3] in dirty) ? "<file name withheld: it holds an identifier>" : kp[3]
        printf "[pii-gate] WARN %s %s: email=%d (allowed with a warning in CHANGELOG.md and docs/)\n", kp[1], label, wmail[wkey[i]]
    }
    for (p in dirty) print p > DIRTY
    printf "%d\n", total + 0 > TOTAL
}'
cfg_email=$(git config --get user.email 2>/dev/null | tr 'A-Z' 'a-z')
: > "$T/dirty"
LC_ALL=C awk -v IDS="$IDS" -v CFG="$cfg_email" -v DIRTY="$T/dirty" -v TOTAL="$T/total" "$MATCH" "$T/stream" \
    || refuse "the identifier check could not run"
[ -s "$T/total" ] || refuse "the identifier check gave no result"
derived_hits=$(cat "$T/total")

# ---------------------------------------------------------------- layer 3: replacement list

LIST=${PII_LIST:-$HOME/.local/share/pii-review/replacements.txt}
list_hits=0
list_note="list: none at $(tildify "$LIST"), list layer skipped"
if [ -e "$LIST" ]; then
    scrub=${PII_SCRUB_PY:-}
    [ -n "$scrub" ] || scrub=$(git config --get pii.scrubPath 2>/dev/null || true)
    [ -n "$scrub" ] || scrub=$(dirname "$LIST")/scrub.py
    py=$(command -v python3 2>/dev/null || command -v python 2>/dev/null || true)
    [ -r "$LIST" ] || refuse "a replacement list exists at $(tildify "$LIST") but is unreadable"
    [ -f "$scrub" ] || refuse "a replacement list exists at $(tildify "$LIST") but scrub.py is not at $(tildify "$scrub"); set PII_SCRUB_PY or git config --global pii.scrubPath"
    [ -n "$py" ] || refuse "a replacement list exists at $(tildify "$LIST") but python is not on PATH"
    mkdir "$T/L" || refuse "cannot create a temp directory"
    : > "$T/objs"
    while read -r lsha old; do
        case $old in
            -) git rev-list --objects "$lsha" --not --remotes="$remote_name" ;;
            .) git rev-list --objects "$lsha" ;;
            *) git rev-list --objects "$lsha" "^$old" ;;
        esac < /dev/null >> "$T/objs" || refuse "cannot list the objects being pushed"
    done < "$T/ranges"
    git cat-file --batch-check='%(objecttype) %(objectname) %(rest)' < "$T/objs" > "$T/objtypes" \
        || refuse "cannot type the objects being pushed"
    : > "$T/map"
    n=0
    while read -r typ sha rest; do
        [ "$typ" = blob ] || continue
        n=$((n + 1))
        git cat-file blob "$sha" < /dev/null > "$T/L/b$n" || refuse "cannot read a pushed file"
        printf 'b%s\t%s\n' "$n" "$rest" >> "$T/map"
    done < "$T/objtypes"
    sed -n -E 's/^(blob|tree) [0-9a-f]+ (.+)$/\2/p' "$T/objtypes" | sort -u > "$T/names"
    while IFS= read -r p; do
        n=$((n + 1)); printf '%s' "$p" > "$T/L/n$n"; printf 'n%s\t%s\n' "$n" "$p" >> "$T/map"
    done < "$T/names"
    while IFS= read -r c; do
        n=$((n + 1)); git cat-file commit "$c" < /dev/null > "$T/L/c$n" || refuse "cannot read a pushed commit"
        printf 'c%s\tcommit %s\n' "$n" "$(printf '%s' "$c" | cut -c1-7)" >> "$T/map"
    done < "$T/commits"
    while IFS= read -r t; do
        n=$((n + 1)); git cat-file tag "$t" < /dev/null > "$T/L/t$n" || refuse "cannot read a pushed tag"
        printf 't%s\tannotated tag\n' "$n" >> "$T/map"
    done < "$T/tags"
    while IFS= read -r r; do
        n=$((n + 1)); printf '%s' "$r" > "$T/L/r$n"; printf 'r%s\tref name\n' "$n" >> "$T/map"
    done < "$T/refs"

    : > "$T/count.out"
    run_count() {
        "$py" -I "$scrub" --list "$LIST" count "$@" >> "$T/count.out" 2>> "$T/count.err" < /dev/null
        rc=$?
        case $rc in 0|1) ;; *) refuse "the replacement-list check failed (scrub.py exit $rc; its stderr is withheld)" ;; esac
        tail -n 1 "$T/count.out" | grep -q -E "^count: [0-9]+ of $# file\\(s\\) hold a listed value$" \
            || refuse "the replacement-list check gave no summary"
    }
    set --
    k=0
    while IFS= read -r line; do
        set -- "$@" "$T/L/${line%%	*}"
        k=$((k + 1))
        if [ $k -ge 100 ]; then run_count "$@"; set --; k=0; fi
    done < "$T/map"
    [ $k -gt 0 ] && run_count "$@"

    sed -n -E 's#^.*/L/([bnctr][0-9]+)  matches=([1-9][0-9]*)(  entries=([0-9,]+))?$#\1 \2 \4#p' "$T/count.out" > "$T/listhits"
    : > "$T/listtotal"
    LC_ALL=C awk -v MAP="$T/map" -v DIRTY="$T/dirty" -v TOTAL="$T/listtotal" '
        BEGIN {
            while ((getline ln < MAP) > 0) { i = index(ln, "\t"); id = substr(ln, 1, i - 1); lab[id] = substr(ln, i + 1) }
            while ((getline ln < DIRTY) > 0) dirty[ln] = 1
        }
        { h[++nh] = $1; m[$1] = $2; e[$1] = $3; if (substr($1, 1, 1) == "n") dirty[lab[$1]] = 1 }
        END {
            for (i = 1; i <= nh; i++) {
                id = h[i]; t = substr(id, 1, 1)
                if (t == "b") label = (lab[id] in dirty) ? "<file name withheld: it holds an identifier>" : lab[id]
                else if (t == "n") label = "<a file or directory name, withheld>"
                else label = lab[id]
                if (i <= 200) printf "[pii-gate] LIST %s: matches=%s entries=%s\n", label, m[id], e[id]
                total += m[id]
            }
            if (nh > 200) printf "[pii-gate] ... and %d more object(s)\n", nh - 200
            printf "%d\n", total + 0 > TOTAL
        }' "$T/listhits" || refuse "the replacement-list report could not run"
    [ -s "$T/listtotal" ] || refuse "the replacement-list report gave no result"
    list_hits=$(cat "$T/listtotal")
    list_note="list: checked $(wc -l < "$T/map" | tr -d ' ') object(s) against $(tildify "$LIST")"
fi

# ---------------------------------------------------------------- verdict

say "checked $ncommits commit(s) on $nref ref(s); derived: $(cut -f1 "$IDS" | sort | uniq -c | awk '{printf "%s%s=%s", (NR > 1 ? " " : ""), $2, $1}'); $repo_note; $list_note"
if [ "$derived_hits" -gt 0 ] || [ "$list_hits" -gt 0 ]; then
    refuse "$derived_hits identifier hit(s), $list_hits list hit(s). Remove them from the commits (amend or rebase) and push again. No matched value is printed."
fi
say "clean"
exit 0
