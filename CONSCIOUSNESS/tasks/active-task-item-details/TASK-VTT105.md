# TASK-VTT105: Claude PR-automation workflow parity with amalavijnana repos

Bring voice-to-text's GitHub Actions Claude integration to parity with the
`amalavijnana/consciousness` and `consciousness-apgps` repos, which run a full
suite of Claude-in-PR workflows. Adapt every Claude-driven workflow from the
reference repos' pnpm/TypeScript shape to this repo's Rust/cargo shape, and use
bare tier aliases (`opus`/`sonnet`/`haiku`) instead of dated model IDs so the
workflows never go stale as new models ship.

Delivered on branch `ci/claude-pr-workflows-parity` (PR, not direct-to-main,
because these run on the public repo with `CLAUDE_CODE_OAUTH_TOKEN`).

- [x] Add `scripts/ci/render-transcript.py` (the transcript→step-summary
      renderer every workflow depends on)
- [x] Add `pr-triage.yml` — `/find-issues` + `/find-duplicate-prs` on PR open
- [x] Add `adversarial-review.yml` — opus-tier adversarial pass, Rust path filter
- [x] Add `dedupe-issues.yml` — `/dedupe` on issue open
- [x] Add `solve-issue.yml` — `claude-fix`-gated autonomous fixer with a cargo +
      GTK/Vulkan toolchain prelude (replaces the reference's pnpm prelude)
- [x] Add `on-slop.yml` + `auto-close-duplicates.yml` (no-Claude label/cron reactions)
- [x] Upgrade `claude.yml` — author-association security gate, model-from-mention,
      cargo toolchain prelude, scoped `Bash(cargo|git|gh)` allowlist, transcript
- [x] Upgrade `claude-code-review.yml` — Rust paths, PR-write, `skip-ai-review`
      label, transcript, sonnet alias
- [x] All ten workflow YAMLs parse; referenced `.claude/commands/*` all present
- [ ] PR reviewed and merged (human gate — these are security-sensitive CI files)

Also fixes the stale `claude-opus-4-7` pins in `amalavijnana/consciousness`
(`solve-issue.yml`, `adversarial-review.yml`) — tracked separately as those are
other repos with their own review gates.
