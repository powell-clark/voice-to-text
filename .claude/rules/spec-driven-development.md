<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Spec driven development

Feature detail cards are the capability specification — each card defines capability scope (description), how it must behave (acceptance criteria), and how to verify it works; the set of all active feature cards IS the capability specification; ADRs own architectural decisions, feature cards own capability acceptance, precepts own validation rules.

## Requires

- MUST read the relevant feature card AND any architecture ADR it implements before building
- MUST create a feature card before writing code when no card exists for the capability being built
- MUST mark acceptance criteria done as they become verifiable and add criteria discovered during build
- MUST resolve every non-deferred criterion to done before requesting review
- MUST keep card frontmatter complete: id, status, priority, kano, title, description, acceptance_criteria with at least one criterion, stories linked, tasks linked
- MUST file each card under CONSCIOUSNESS/features/{status}-feature-item-details/ matching its lifecycle status

## Forbids

- MUST NOT place architectural decisions inside feature cards — those belong in ADRs with consequences and considered-alternatives shape
- MUST NOT place capability acceptance inside ADRs — those belong on feature cards
- MUST NOT request review when an acceptance criterion is in pending state without an explicit deferred annotation
- MUST NOT ship code that has no feature card or no acceptance criteria — invisible work

## References

- adr:adrs--serve_as_specifications
- precept:precept_specification
- doc:CONSCIOUSNESS/features

## Verified by

packages/core/pgps/card-validator.ts:validateCard
