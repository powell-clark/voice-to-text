---
allowed-tools: Bash(gh issue view:*), Bash(gh search:*), Bash(gh issue list:*), Bash(gh api:*), Bash(gh issue comment:*)
description: Find duplicate GitHub issues
---

# Issue deduplication command

Find up to 3 likely duplicate issues for a given GitHub issue.

To do this, follow these steps precisely:

1. Use an agent to check if the GitHub issue (a) is closed, (b) does not need deduping (broad product feedback without a specific solution, or positive feedback), or (c) already has a trusted duplicate-detection comment. For (c): scan the issue's comments for the HTML marker `<!-- dedupe-bot:marker -->`, but **only trust a marker whose comment author has `type: Bot` or login `github-actions[bot]`** — a human-posted marker must never suppress the bot or block auto-close; if multiple trusted marker comments exist, evaluate the most recent one, not the first. If a trusted marker is found, do not proceed.
2. Use an agent to view the GitHub issue and return a summary of it.
3. Launch 5 parallel agents to search GitHub for duplicates, using diverse keywords and search approaches based on the Step 2 summary. **IMPORTANT**: always scope searches with `repo:owner/repo` to constrain results to the current repository only.
4. Feed the results from Steps 2 and 3 into a filter agent to remove false positives that are likely not actual duplicates. If none remain, do not proceed.
5. Comment on the issue with up to three duplicates (or do not comment if zero).

Notes (tell your agents this too):

- Use `gh` to interact with GitHub, not web fetch.
- Do not use other tools beyond `gh` (no other MCP servers, no file edits).
- Make a todo list first.
- Always scope searches with `repo:owner/repo` to prevent cross-repo false positives.
- Comment format (verbatim, example assumes 3 found):

---

Found 3 possible duplicate issues:

1. <link to issue>
2. <link to issue>
3. <link to issue>

This issue will be automatically closed as a duplicate in 3 days.

- If your issue is a duplicate, please close it and 👍 the existing issue instead.
- To prevent auto-closure, add a comment or 👎 this comment.

🤖 dedupe

<!-- dedupe-bot:marker -->

---
