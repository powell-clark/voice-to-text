---
allowed-tools: Bash(gh pr view:*), Bash(gh pr diff:*), Bash(gh pr checks:*), Bash(gh pr comment:*), Bash(gh pr create:*), Bash(gh pr list:*), Bash(gh issue view:*), Bash(gh issue comment:*), Bash(gh api:*), Bash(git pull:*), Bash(git fetch:*), Bash(git checkout:*), Bash(git switch:*), Bash(git add:*), Bash(git commit:*), Bash(git push:*), Bash(git status:*), Bash(git diff:*), Bash(git log:*), Bash(git show:*), Bash(git rev-parse:*), Bash(git branch:*), Bash(git grep:*), Bash(cargo:*), Read, Edit, Write, Glob, Grep
description: Drive a PR or issue forward — build, review, or progress it — with red→green tests and objective verification before any verdict
---

# Iterate a PR or issue forward

Drive this PR or issue toward done. Work out what it needs to move forward, then do it —
that might be building a fix, reviewing existing work, or just pushing an in-progress
change one step further. The goal is genuine, verified progress, not a snap verdict.

`git pull` the latest `main` first, and stay aware of related issues/PRs and their comments.

## 1. Assess

- **If an ISSUE:** reproduce it. State the problem and why solving it matters. If it is not
  real, is a question/feature request rather than a defect, or is not worth doing — say so
  plainly and stop. If work is needed and no PR exists, go to step 2 and open a PR.
- **If a PR:** assess it critically — is it correct, up to date with `main`, and mergeable?
  Go through every bot/reviewer comment and **accept or reject each one with a clear reason**.
  Resolve any conflicts with explicit reasoning. If it is not finished, keep building it
  forward. Determine the close process for any issue it resolves.
- **Right-size the work.** If it is too large for one clean change, or depends on unmerged
  prerequisite work, **split it**: identify or open the separate PRs/issues, sequence them, and
  land the blockers first. Do not force an oversized or dependency-tangled PR — name the pieces
  and the order they go in, and end with CONTINUE DEVELOPMENT pointing at the first one.

## 2. Build — test-first is a HARD gate

No production code ships without a test that went red then green.

1. Agree the behaviour, then write a test that **FAILS**, reproducing the bug or specifying
   the new behaviour (red). Do not write production code before the failing test exists.
2. Implement only until that test **PASSES** (green).
3. **Iterate** — fix → run tests → repeat — until the suite is green. Do not one-shot and hope.
   But do not spin: if the same failure keeps recurring or you cannot converge after a few
   honest attempts, **stop** — report the blocker and end with CONTINUE DEVELOPMENT (or
   DISAPPROVED if it is a genuine dead end). Never loop indefinitely.

Stay in scope: fix only what this PR/issue is about — no drive-by refactors, no unrelated
formatting.

## 3. Verify objectively — cite artifacts, never assert

A claim of success must be **backed by evidence, not stated**. Do not say "done" or give an
APPROVED verdict unless you can show:

- the test **failing without** the change and **passing with** it (paste the red and the green);
- `gh pr checks` observed **green** (not "should be green");
- `cargo test` (or scoped `cargo test <test_name>`) and `cargo check` **clean**;
- for any non-trivial change, an **independent refutation attempt** — try hard to break it —
  that **failed** to break it.

If you cannot produce these, you are **not done** — report exactly what is blocking and stop.
Do not claim success you cannot evidence.

## 4. Report and call it

Show your thinking. Flag any errors of reasoning you spot, and call out a weak or misguided
issue/PR plainly.

End with **where it stands** and the **next action**, then one explicit status line:

`VERDICT: APPROVED | CONTINUE DEVELOPMENT | DISAPPROVED | CLOSE WITH COMMENT — <next action>`

The deciding question for any not-yet-approved work is **is there a viable path forward?**

- **APPROVED** — conclusive: correct, tested red→green, CI green, ready to merge. Use only when
  every check in step 3 is evidenced.
- **CONTINUE DEVELOPMENT** — on the right track but not finished (includes "needs more work").
  A path forward exists; state the next concrete step. This is the normal ending for in-progress
  work and is **not** a rejection.
- **DISAPPROVED** — no viable path: the approach or reasoning is a dead end; stop pursuing this
  attempt. Say why.
- **CLOSE WITH COMMENT** — not real, not worth doing, or a duplicate; bin it with the reason.

## Guardrails

- Make a todo list first. One PR/issue at a time. Evidence over assertion.
- You **recommend**; you do not merge or approve on GitHub, and you do not close issues —
  those are the operator's calls.
- New PRs are opened as **draft** (`gh pr create --draft`).
- Never `git push --force`, `git reset --hard`, `git clean -fd`, or `git checkout -- .`.
  Never push to a protected branch.
- Do not touch protected paths: `CHANGELOG.md`, active ADRs, `.claude/settings.json`,
  anything under `~/.claude/plugins/`. If the only correct fix touches those, stop and explain.
- Commit messages: imperative subject, no AI/bot attribution, trailer
  `Authored-By: Emmanuel Powell-Clark <emmanuel@powellclark.com>`.
