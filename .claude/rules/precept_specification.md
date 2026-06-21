<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Precept specification

Every yaml in CONSCIOUSNESS/precepts/ MUST conform to precept_specification.schema.json and bind via predicates rather than prose or implementation detail.

## Requires

- MUST locate enacting prose inside the precept's narrative field rather than in external orientation files
- MUST relocate postmortems and bump-history to commentary.jsonl, and decision rationale to the relevant ADR
- MUST reference other precepts, ADRs, STEER events, and docs by id rather than duplicating their content

## Forbids

- MUST NOT include prose paragraphs of three or more sentences in rule, requires, or forbids — long-form orienting prose belongs in the narrative field
- MUST NOT use the narrative field to carry binding rules — narrative orients, while rule and requires and forbids bind
- MUST NOT embed source paths from src/**, packages/**, scripts/**, or dist/** outside the verified_by and narrative fields
- MUST NOT bypass the linter via --no-verify except for whitespace or comment-only edits

## References

- adr:precepts--content_schema
- adr:plugin--precepts_system
- adr:precepts--delivery_mechanism

## Verified by

packages/core/precepts/lint.ts:lintPrecept
