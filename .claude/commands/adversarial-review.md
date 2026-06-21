---
allowed-tools: Bash(gh pr view:*), Bash(gh pr diff:*), Bash(gh pr checks:*), Bash(gh pr comment:*), Bash(gh api:*), Bash(git diff:*), Bash(git log:*), Bash(git show:*), Bash(git status:*), Bash(git rev-parse:*), Bash(git ls-files:*), Bash(git grep:*), Bash(git branch:*), Bash(git remote:*)
description: Adversarial review of a PR — assume it is broken until proven otherwise
---

# Adversarial PR review command

Review a pull request from the stance that **it is broken until it proves otherwise**. You are not a cheerleader. Your job is to find the way this change fails, and to refuse to approve work that has not earned it.

To do this, follow these steps precisely:

1. Use an agent to check if the PR (a) is closed/merged, or (b) already has an adversarial-review comment (check for the exact HTML marker `<!-- adversarial-review-bot:marker -->` in the PR comments — ignore other bot comments). If so, do not proceed.
2. Use an agent to view the PR (`gh pr view`, `gh pr diff`, `gh pr checks`) and the linked issue if any. Ask it to return: what the PR claims to do, what files/functions changed, what issue (if any) it claims to fix.
3. Launch parallel agents, each attacking a different failure mode. Be specific and evidence-driven — every finding must cite a file and line:
   - **Reproduction agent**: Does the problem the PR claims to fix actually exist on the base branch? If the PR fixes a bug, is there a test or a concrete reproduction that demonstrates the bug *before* this change? A fix for a bug nobody can reproduce is slop.
   - **Test agent**: Does the change add or update a test that fails without the change and passes with it? Unverified behaviour change is slop. Check `gh pr checks` — is CI actually green, or merely pending/skipped?
   - **Assumptions agent**: Does the diff make incorrect assumptions about the codebase — calling functions that do not exist, mis-handling error/Result types, breaking an invariant, ignoring a precept the touched module is verified-by? Use `git grep` to verify call sites and contracts.
   - **Completeness agent**: Is the change partial or misleading — TODOs, half-handled cases, a description that overstates what the diff does, dead code, scope creep unrelated to the stated purpose?
4. Feed all findings into a filter agent. Discard weak or speculative objections. Keep only findings backed by a file:line citation or a reproducible command. Decide a verdict:
   - **request-changes** — any reproduction failure, missing test for a behaviour change, incorrect assumption, or material incompleteness
   - **comment** — non-blocking concerns only, change is fundamentally sound
   - **approve** — the claim is real, the fix is tested, assumptions check out, scope is clean
5. Post exactly one PR comment in the format below. Be direct and specific. Praise nothing that was not earned.

Notes (tell your agents this too):

- Use `gh` and read-only `git` only. No file edits, no other MCP servers, no web fetch.
- Make a todo list first.
- Every finding cites `path:line` or a runnable command. No vibes.
- This is review only — never push, never edit code, never change PR state. You comment; humans decide.
- If CI is not green, that alone is at least `comment` and usually `request-changes` — say so explicitly.
- Comment format (verbatim shape):

---

## Adversarial review — `<verdict>`

**Claim:** <one line: what the PR says it does>

**Reproduction:** <verified the bug exists on base / could NOT reproduce the bug / N/A — feature>
**Test coverage:** <fails-without + passes-with confirmed / NO test for the behaviour change / CI not green>

### Blocking
- `path:line` — <specific failure, with the command or evidence>

### Non-blocking
- `path:line` — <concern>

### What holds up
- <only genuinely verified-good things; omit this section if none>

🤖 adversarial-review

<!-- adversarial-review-bot:marker -->

---
