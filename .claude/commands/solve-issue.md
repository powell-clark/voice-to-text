---
allowed-tools: Bash(gh issue view:*), Bash(gh issue comment:*), Bash(gh pr create:*), Bash(gh pr list:*), Bash(gh api:*), Bash(git checkout:*), Bash(git switch:*), Bash(git add:*), Bash(git commit:*), Bash(git push:*), Bash(git status:*), Bash(git diff:*), Bash(git log:*), Bash(git rev-parse:*), Bash(git branch:*), Bash(git grep:*), Bash(cargo:*), Read, Edit, Write, Glob, Grep
description: Reproduce a reported issue, fix it, test it, and open a draft PR
---

# Autonomous issue-solver command

Attempt a fix for a reported GitHub issue. The bar is **gensho — actual proof**: a fix that does not demonstrably work is worse than no fix. You open a **draft** PR for human review. You never merge, never close the issue, never push to a protected branch.

To do this, follow these steps precisely:

1. **Idempotency / eligibility.** Use an agent to check whether the issue (a) is closed, (b) already has a solve-issue comment (exact HTML marker `<!-- solve-issue-bot:marker -->`), or (c) already has an open PR linked to it. If any, stop — do not proceed.
2. **Understand.** Read the issue fully (`gh issue view --comments`). Summarise: the claimed problem, expected vs actual behaviour, and the smallest reproduction it describes. If the issue is a feature request, vague, or a question rather than a defect, **stop and post the "not auto-solvable" comment** (format below) — do not attempt speculative feature work.
3. **Reproduce first.** On a fresh branch `claude/solve-issue-<number>`, write a failing test (or a minimal runnable reproduction) that demonstrates the bug on the current code. If you **cannot** reproduce it, stop and post the "could not reproduce" comment. A fix for a bug you cannot reproduce is slop — do not submit it.
4. **Fix.** Make the smallest change that makes the reproduction pass. Stay in scope — fix this issue only, no drive-by refactors, no unrelated formatting. Do not touch protected paths (`CHANGELOG.md`, active ADRs, `.claude/settings.json`, anything under `~/.claude/plugins/`). If the only correct fix would touch those, stop and explain in the comment.
5. **Prove.** Run `cargo test` (or scoped `cargo test <test_name>`), then `cargo check`. The reproduction from Step 3 must now pass and must have failed before. If tests do not pass, do not open a PR — post the "attempted, not verified" comment with what you found.
6. **Adversarially self-review** using the same standard as `/adversarial-review`: did the bug really exist? does the test fail without the fix and pass with it? any incorrect assumptions, partial handling, or scope creep? If your own review would request changes, fix it before opening the PR, or downgrade to the "attempted, not verified" comment.
7. **Open a draft PR** (`gh pr create --draft`). Title: concise, imperative, 3–8 words. Body: what the bug was, the reproduction, the fix, the test evidence (commands + before/after), and `Fixes #<number>`. Then post the issue comment linking the PR.

Notes:

- Make a todo list first.
- Commit message: imperative subject, no AI/bot attribution lines, `Authored-By: Emmanuel Powell-Clark <emmanuel@powellclark.com>`.
- PR is always `--draft`. Never `gh pr merge`, never `gh issue close`, never push to `main`.
- Never use `git reset --hard`, `git push --force`, `git clean`, or `git checkout -- .`.
- One issue at a time. Stay in scope. Evidence over assertion.
- Issue comment format (verbatim) when a draft PR was opened:

---

I attempted a fix for this issue: <link to draft PR>

**Reproduction:** <how the bug was reproduced — test path or command>
**Fix:** <one-line summary of the change>
**Verification:** reproduction fails on base, passes on the branch; `cargo check` clean.

This is a **draft** PR for human review — it has not been merged and this issue remains open.

🤖 solve-issue

<!-- solve-issue-bot:marker -->

---

- Comment format (verbatim) when NOT auto-solvable / could-not-reproduce / attempted-not-verified — pick the one line that applies:

---

I looked at this but did not open a PR:

- <one of: this is a feature request or question, not an auto-solvable defect / I could not reproduce the described behaviour on the current code / I attempted a fix but could not verify it with a passing test>

Leaving this for a human. No code was changed.

🤖 solve-issue

<!-- solve-issue-bot:marker -->

---
