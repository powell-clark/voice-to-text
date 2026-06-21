---
allowed-tools: Bash(gh pr view:*), Bash(gh pr diff:*), Bash(gh search:*), Bash(gh issue list:*), Bash(gh issue view:*), Bash(gh api:*), Bash(gh pr comment:*), Bash(git diff:*), Bash(git log:*), Bash(git show:*), Bash(git status:*), Bash(git rev-parse:*), Bash(git ls-files:*), Bash(git grep:*), Bash(git branch:*), Bash(git remote:*)
description: Find GitHub issues that a PR might fix
---

# Find issues for PR command

Find open GitHub issues that a pull request might fix. Include all likely matches — do not artificially limit the number of results.

To do this, follow these steps precisely:

1. Use an agent to check if the PR (a) is closed/merged, or (b) already has a related-issues comment (check for the exact HTML marker `<!-- find-issues-bot:marker -->` in the PR comments — ignore other bot comments). If so, do not proceed.
2. Use an agent to view the PR title, body, and diff (`gh pr view`, `gh pr diff`) and return a summary of: what the PR changes, key technical terms / error messages / API / module names, and any issue numbers already referenced.
3. Launch 5 parallel agents to search GitHub for open issues this PR might fix, **scoped with `repo:owner/repo`**, each with a different strategy:
   - Agent 1: error messages or symptoms in the diff
   - Agent 2: feature/module names from changed files
   - Agent 3: API/function names that were modified
   - Agent 4: keywords from the PR title/description
   - Agent 5: broader terms related to the area of code changed
4. Feed Steps 2–3 into a filter agent to remove false positives. Exclude issues already referenced in the PR body (`fixes #`, `closes #`, `resolves #`). Keep only clearly-relevant open issues. If none remain, do not proceed.
5. Comment on the PR with all related open issues (no cap). Do not suggest closed issues.

Notes (tell your agents this too):

- Use `gh` and read-only `git` only. No file edits, no other MCP servers, no web fetch.
- Make a todo list first.
- Always scope searches with `repo:owner/repo`.
- Comment format (verbatim, example assumes 3 found). The fenced block lets the author copy-paste to auto-close on merge:

---

Found 3 issues this PR may fix:

1. <link to issue> - <one-line summary of why this PR is relevant>
2. <link to issue> - <one-line summary of why this PR is relevant>
3. <link to issue> - <one-line summary of why this PR is relevant>

> If this is helpful, copy the block below into the PR description to auto-close these issues on merge.

```
Fixes #<number>
Fixes #<number>
Fixes #<number>
```

🤖 find-issues

<!-- find-issues-bot:marker -->

---
