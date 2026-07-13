<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Naming precision

Names convey meaning in 2-6 words: variables and functions take 2-3 words, files and directories 2-4, plan titles and branch names 3-6, commit subjects 3-8, roadmap item titles 3-6; single-word names are acceptable when clarity is equal.

## Narrative

**DIRECTIVE — not "Epic" — is the top-of-hierarchy PGPS term.**

AI models trained on PM documentation (Jira, SAFe, Agile, Linear) have a
strong prior toward "Epic" as the top-level entity. That training bias is the
root cause of this precept's most commonly broken rule. Counteract it
consciously: whenever the top-of-hierarchy entity arises, the correct term is
DIRECTIVE, not Epic, not theme, not initiative.

The vocabulary is load-bearing, not cosmetic. "The Director issues directives"
encodes the theatrical framing and the authority relationship in a single word.
"Epic" erases both and imports a Jira mental model that conflicts with the
Consciousness architecture.

The fixed hierarchy is: DIRECTIVE (prefix DIRECT-XX###) → STORY
(STORY-XX###) → TASK (TASK-XX###). Features (FEAT-XX###) hang off stories;
reviews (REVIEW-XX###) hang off any item. Every entity reference pairs ID with
title: "DIRECT-CCC027 (Migrate epics to directives)", never a bare ID and
never an EPIC-XX### prefix.

## Requires

- MUST keep variable, function, file, directory, branch, commit subject, and roadmap title names within the 2-6 word band per identifier kind
- MUST reference every roadmap item with both ID AND title together — for example 'TASK-CCC2970 (Telemetry audit)' not 'TASK-CCC2970' alone
- MUST use DIRECTIVE as the top-of-hierarchy term with ID prefix DIRECT-XX###; the chain is directive then story then task; features hang off stories; reviews hang off any item
- MUST default to one descriptive word when an identifier needs no qualifier (count, total) and clarity is equal

## Forbids

- MUST NOT name identifiers in seven or more words — that is a sentence not a name
- MUST NOT use one-word names that erase meaning when context demands more (plan1, fix, x)
- MUST NOT use the term EPIC for top-of-hierarchy items; only DIRECTIVE
- MUST NOT reference roadmap items by ID without title (or by title without ID) in operator-facing prose

## References

- precept:precept_specification
- doc:CONSCIOUSNESS/adr/adrs--one_decision_per_file.yaml

## Verified by

packages/core/pgps/validators/semantic-validator/relationship-rules.ts:checkStoryFulfilmentReadiness
