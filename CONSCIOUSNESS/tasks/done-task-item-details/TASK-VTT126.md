# TASK-VTT126: PPA source tarball must exclude .claude/ and chats/

## Context

Found live during the v2.3.9 PPA upload (2026-07-10 05:47): lintian errored
`source-is-missing [.claude/worktrees/agent-ada3e52e4564f98c2/packaging/linux/vtt-linux.prebuilt]`.
A leftover agent worktree under `.claude/worktrees/` was swept into both the
noble and jammy source tarballs because `debian/source/options` tar-ignores
`.git`, `CONSCIOUSNESS`, `target`, `vendor` etc. but not `.claude` (session
plumbing, worktrees) or `chats/` (local transcript archive, gitignored but
present on disk). Nothing sensitive shipped this time (CONSCIOUSNESS and .git
were already excluded, repo is public) but the upload was bloated and lintian
flagged it; a future worktree or chat file must never ride along.

## Approach

1. Add `tar-ignore = ".claude"` and `tar-ignore = "chats"` to
   `debian/source/options`.
2. Remove the stale merged worktree (`git worktree remove` + branch delete)
   that triggered the error.

## Acceptance criteria

- [ ] `debian/source/options` tar-ignores `.claude` and `chats`
- [ ] No stale worktrees under `.claude/worktrees/` (git worktree list shows
      only the main checkout)
- [ ] Next `release-ppa.sh` run produces no `source-is-missing` lintian error
      referencing `.claude/**` *(deferred: verifiable on the next release run)*

## Dependencies

- Directive: DIRECT-VTT002 (Linux voice-to-text and shared core)
- Related: FEAT-VTT008 (APT PPA distribution), FEAT-VTT016 (release script)
