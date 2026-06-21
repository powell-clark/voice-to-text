---
allowed-tools: Bash(gh pr view:*), Bash(gh pr diff:*), Bash(gh search:*), Bash(gh pr list:*), Bash(gh api:*), Bash(gh pr comment:*), Bash(git diff:*), Bash(git log:*), Bash(git show:*), Bash(git status:*), Bash(git rev-parse:*), Bash(git ls-files:*), Bash(git grep:*), Bash(git branch:*), Bash(git remote:*)
description: Find open PRs that may duplicate a given PR
---

# Find duplicate PRs command

Find other open, unmerged pull requests that may be duplicates of (or substantially overlap with) a given PR.

To do this, follow these steps precisely:

1. Use an agent to check if the PR (a) is closed/merged, or (b) already has a duplicate-PR comment (check for the exact HTML marker `<!-- find-duplicate-prs-bot:marker -->` in the PR comments — ignore other bot comments). If so, do not proceed.
2. Use an agent to view the PR title, body, and diff (`gh pr view`, `gh pr diff`) and return a summary of: what it changes (files, functions, features), key technical terms, and any issue numbers referenced (two PRs fixing the same issue are likely duplicates).
3. Launch 3 parallel agents to search for other **open, unmerged** PRs that may duplicate this one, **scoped with `repo:owner/repo is:pr is:open is:unmerged`**, each with a different strategy:
   - Agent 1: PR title keywords + referenced issue numbers
   - Agent 2: `gh pr list` filtered by the same files/paths touched in the diff
   - Agent 3: feature/API/function names from the changed code
4. Feed Steps 2–3 into a filter agent to remove false positives. Exclude the PR itself. Keep only PRs that change the same code area for the same purpose, or fix the same referenced issue. **If none remain, do not comment at all — silently exit.**
5. If and only if at least one likely duplicate was found, comment on the PR.

Notes (tell your agents this too):

- Use `gh` and read-only `git` only. No file edits, no other MCP servers, no web fetch.
- Make a todo list first.
- Always scope searches with `repo:owner/repo`.
- Only match **open, unmerged** PRs — never closed, merged, or draft. Never include the input PR.
- **Do not post a comment if zero duplicates are found.**
- Comment format (verbatim, example assumes 2 found):

---

This PR may be a duplicate of:

1. <link to PR> - <one-line summary of why it overlaps>
2. <link to PR> - <one-line summary of why it overlaps>

🤖 find-duplicate-prs

<!-- find-duplicate-prs-bot:marker -->

---
