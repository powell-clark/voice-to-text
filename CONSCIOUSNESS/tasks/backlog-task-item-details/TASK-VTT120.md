# TASK-VTT120: Review-gate kano override never resolves (getFeatureKano bug)

## Context

Discovered 2026-07-01 while closing FEAT-VTT037. consciousness plugin's packages/core/review/readiness.ts::getFeatureKano() only reads FEATURE-ACTIVE-INDEX.md to resolve a feature's kano tier for review-gates kano_overrides (packages/core/review/review-gates-config.ts). This repo's FEATURE-ACTIVE-INDEX.md is always empty — features move straight from backlog to maintained/done, per this repo's actual convention — so getFeatureKano always returns null, the kano override (agent-gate for performance/delighter, human-gate only for must-have) never resolves, and every feature closure here silently falls back to the base feature gate default of requires:human. Confirmed by pattern: every past feature closure in REVIEW-INDEX.md carries a human-approved row regardless of kano tier. This is a consciousness-plugin bug (different repo: ~/projects/amalavijnana/consciousness), not something to fix in voice-to-text — getFeatureKano should also check FEATURE-BACKLOG-INDEX.md (and probably FEATURE-MAINTAINED-DONE-INDEX.md) for a feature's kano, not just the active index. File the actual fix against the consciousness repo when picked up; this VTT task just tracks the discovery so it isn't lost.

## Acceptance criteria

- [ ] _(to be filled in)_

## Dependencies

- Directive: DIRECT-VTT002
