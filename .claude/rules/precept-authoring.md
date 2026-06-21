<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Precept authoring

Every precept authored without a trigger field fires for all operators, all actor types, and all operating environments — authors must verify universal applicability, document propagation timing accurately, and obtain operator review before including the precept in a plugin release.

## Narrative

A trigger-absent precept is not scoped to a project, an operator, or an actor type — it fires for every
actor (Claude Code, Codex CLI, and future adapters such as Gemini CLI) in every project that installs the
plugin. This is a very high blast radius. The actor authoring a new precept typically knows their own
environment well and may unknowingly embed local assumptions into a universally applied rule.

Three failure modes documented from the 2026-04-25 terminal-multiplexer-dispatch authoring incident:

1. TOOL HARDCODING: The first draft named the precept after a specific multiplexer (WezTerm) and baked in
   WezTerm-specific CLI syntax. Operators with tmux, zellij, kitty, screen, or no multiplexer would have
   received a rule that does not match their environment. The fix: use detection logic and enumerate
   per-tool commands, never assume one tool.

2. PROPAGATION TIMING: The authoring agent claimed the new precept would auto-propagate to consumer projects
   at the next SessionStart. Dev-tree edits do not propagate — only released plugin content does. Under
   universal inheritance (ADR precepts--universal_inheritance, TASK-CCC29297), consumer repos carry no
   universal precept yamls; their .claude/rules/ render from the released plugin's precepts at session
   start. So a precept change reaches consumers on the plugin release that carries it, at each consumer's
   next session start — never directly from the dev tree. A consumer-local yaml sharing a plugin precept's
   filename does not override content; only `enabled: false` is honoured (per-project disable).

3. ACTOR PORTABILITY: A precept that references Claude Code-specific paths (~/.claude/), Codex CLI-specific
   config locations, or IDE-specific behaviours must either narrow its trigger or enumerate conditions per
   actor. Universal scope means correct behaviour under Claude Code, Codex CLI, and any future wired adapter.

## Requires

- MUST verify that every trigger-absent precept's requires and forbids rules apply correctly under all wired actor contexts (Claude Code, Codex CLI) before marking the precept ready for release
- MUST use detection-based logic when a precept's subject varies by tool or environment — enumerate per-tool commands rather than assuming a specific multiplexer, shell, package manager, or IDE
- MUST state precept propagation timing accurately — dev-tree edits reach consumer projects only on plugin release; consumers render the released plugin's precepts into .claude/rules/ at their next session start
- MUST obtain operator review before shipping any new or materially changed trigger-absent precept in a plugin release
- MUST add a precept to lint-allowlist.txt with an inline tracking-task reference when known schema violations must ship temporarily

## Forbids

- MUST NOT hardcode a specific terminal multiplexer, package manager, IDE, or OS-specific file path in a trigger-absent precept's requires or forbids rules — reference the class of tool and require detection
- MUST NOT claim that a dev-tree precept edit propagates to consumer projects before a plugin release ships — consumer rules render from the released plugin's precepts, never from the dev tree
- MUST NOT ship a trigger-absent precept without verifying its rules are correct under all wired actor contexts
- MUST NOT skip operator review when adding a trigger-absent precept to a plugin release

## References

- precept:precept_specification
- precept:terminal-multiplexer-dispatch
- doc:CONSCIOUSNESS/precepts/lint-allowlist.txt
- doc:packages/core/precepts/inherit.ts
