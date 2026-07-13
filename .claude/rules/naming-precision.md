<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Naming precision

Names should be 2-6 words that convey meaning.
One word is too vague. Seven words is a sentence, not a name.

This applies to all human-visible identifiers:
- Variable and function names (2-3 words)
- File and directory names (2-4 words)
- Plan titles and branch names (3-6 words)
- Commit message subjects (3-8 words)
- Roadmap item titles (3-6 words)

Exception: single-word names are acceptable when clarity is equal
(e.g., `count` is fine, `item_count` adds nothing).

## Scope

universal

## Rationale

Random or cryptic names (e.g., memoized-finding-dragon) tell you nothing.
Descriptive names (e.g., finish-building-consciousness) let you scan and
recognise without opening the file.

The sweet spot is precision without verbosity:
- Too short: "fix" — fix what?
- Right: "fix-error-tracking" — clear
- Too long: "fix-the-error-tracking-telemetry-system-for-hooks" — a sentence

## Examples

### Good

- cleanup-stale-sessions
- universe-orchestration-plan
- fix-lock-race-condition
- session_id
- error_count

### Bad

- memoized-finding-dragon
- plan1
- x
- the-plan-for-fixing-all-the-things-that-are-broken

## Roadmap items

ALWAYS reference roadmap items with both ID AND title.
IDs alone are meaningless to humans.

### Good

- TASK-CCC2970 (Telemetry audit)
- STORY-CCC146 (Branch-aware schema migrations)

### Bad

- TASK-CCC2970
- the telemetry task

## Coordination terminology

This is an operating group of responsible individuals carrying out a
mission and vision together. They have dialogue, argue rationally, use
evidence and past history and data and documents and artifacts. They
communicate well, understand each other's theory of mind, apply the
ten factors, and remain objective. The system is Buddhist.

There are leaders with positions and responsibilities. Hierarchy exists
where it serves the mission. Use coordination science terminology —
the discipline shared across medicine, aviation, ecology, organisational
behaviour, and the military. Military terms are fine when they're
genuinely better. The goal is precision, not avoidance.

### Group nouns

#### Good

- team
- unit
- force
- crew

#### Avoid

- swarm
- hive
- army
- platoon

### Prefer

- role (not rank)
- enter/start (not spawn — see philosophy.yaml responsible_terminology)
- actor (not soldier, operative, combatant)
- protocol (not doctrine — unless referring to Buddhist philosophy)
- steering (not orders, commands)
- span of control (organisational science since 1933, Graicunas)
- running estimate (coordination science, used across all domains)
- closed-loop communication (universal in safety-critical domains)
- intent-based delegation (acceptable; Auftragstaktik also acceptable in technical context)
