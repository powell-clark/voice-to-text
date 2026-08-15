# ADR-0000: Use Architecture Decision Records

**Status:** Active
**Date:** 2025-11-22
**Context:** Consciousness project

## Context

The consciousness project maintains the autonomous session coordination, time tracking, and progress monitoring system for Claude Code. As this system evolves, we need to document significant architectural decisions to maintain clarity and consistency across development sessions.

## Decision

We will use Architecture Decision Records (ADRs) to document all significant architectural decisions for this project.

**ADR Scope for Consciousness:**
- Hook architecture decisions (lifecycle, execution order, concurrency)
- Skill design patterns
- GPS tracking file formats (TSV, bracket notation)
- Time logging system (3-minute blocks, deduplication)
- STM concurrency system (version files, lock files)
- SMART criteria enforcement
- Cross-reference validation
- Propagation strategy
- Interoperability with other AI tools

**ADR Format:**
```markdown
# ADR-NNNN: Title

**Status:** [Active|Superseded|Deprecated]
**Date:** YYYY-MM-DD
**Context:** Consciousness project

## Context
Why this decision is needed

## Decision
What we decided to do

## Consequences
Impact of this decision (both positive and negative)

## Alternatives Considered
Other options we evaluated

## References
- Links to issues, commits, discussions
```

## Consequences

**Positive:**
- Architectural decisions are documented and searchable
- New developers (or future sessions) can understand why choices were made
- Prevents revisiting settled decisions
- Creates project-specific knowledge base for the consciousness system itself

**Negative:**
- Requires discipline to write ADRs for significant decisions
- Additional overhead during development

## Alternatives Considered

1. **No formal ADR process** - Rely on git history and comments
   - Rejected: Decisions get lost in commit messages, hard to search

2. **Documentation only in docs/adr/** - Store ADRs outside CONSCIOUSNESS/
   - Rejected: ADRs are part of project tracking and should be validated by totem

## References

- This is the first ADR for the consciousness system
- Subsequent ADRs will document existing architectural decisions (hook lifecycle, TSV format, etc.)
- Stored in CONSCIOUSNESS/adr/ as part of project tracking structure
