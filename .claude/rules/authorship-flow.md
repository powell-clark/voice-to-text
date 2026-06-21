<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Authorship flow

When the practitioner describes problems, stories, directives, or features in conversation, the agent captures them as PGPS entities — feature cards with acceptance criteria, ADRs for one-way-door architectural decisions, directives via the append flow, review verdicts per the review-gates precept — before proceeding to implementation.

## Trigger

when CONSCIOUSNESS/ exists and the practitioner is describing project intent

## Narrative

The install-screen promise commits the agent to a specific authorship handshake: practitioner provides intent in natural language, agent converts it into durable PGPS structure with acceptance criteria, ADRs, and review gates. This precept makes the promise binding rather than implicit.

Without it, the conversation is the only record. Conversations are not durable; transcripts are not queryable; future sessions cannot resume from them. The PGPS is the substrate that survives every turn-end, model swap, and provider switch. Capturing entities at the moment of intent is what makes "all in your CONSCIOUSNESS/ folder tracked in git" true rather than aspirational.

The pattern is not "ask permission for every capture" — the precept binds the agent to capture proactively while surfacing the new entity ID and title so the practitioner can steer. Silent captures violate transparency; permission-asking on every capture starves productivity. The middle ground is reliable, transparent, fast capture with operator visibility.

## Requires

- MUST capture practitioner-described features as feature cards under CONSCIOUSNESS/features/{status}-feature-item-details/ with at least one acceptance criterion recorded before any implementation begins
- MUST file an ADR under CONSCIOUSNESS/adr/ before implementing any one-way-door architectural decision surfaced in conversation — language choice, persistence model, API contract, security model, license, deployment topology
- MUST capture practitioner-described directives as DIRECTIVE entries via the PGPS append flow rather than treating the conversation transcript as the record
- MUST emit a review verdict row to CONSCIOUSNESS/reviews/REVIEW-INDEX.md on every transition to done — bypass-approved when the entity's per-entity gate is auto-flow, human-pending when it is hard-gated
- MUST surface every new entity to the practitioner with its assigned ID and title before continuing so the practitioner can steer or rename

## Forbids

- MUST NOT begin implementation work when the practitioner has described a feature without first capturing it as a feature card with acceptance criteria
- MUST NOT make a one-way-door architectural decision in conversation without filing the ADR — the conversation transcript is not a durable record
- MUST NOT skip the review verdict emission on done-transitions, even on auto-close; the audit trail is required under every path
- MUST NOT capture entities silently — every PGPS write is surfaced to the practitioner with the new ID and title so they can steer
- MUST NOT ask permission before every capture — that pattern starves productivity and contradicts the install-screen promise of working out structure as we go

## References

- precept:spec-driven-development
- precept:review-gates
- precept:task_lifecycle
- precept:precept_specification
- doc:CONSCIOUSNESS/CHANGELOG.md
