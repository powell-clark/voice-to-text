<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Spec driven development

Feature detail cards are the product specification. Each feature's XML
file in CONSCIOUSNESS/features/{status}-feature-item-details/ defines
what must be built, how it must behave, and how to verify it works.

When building a feature, the detail card is the authoritative reference.
When reviewing a feature, the acceptance criteria in the card determine
pass or fail. When composing a product specification, the set of all
active feature cards IS the specification.

## Scope

universal

## Principles

### Single source

The feature detail card is the single source of truth for what a
feature does. Requirements are not scattered across ADRs, stories,
and conversations — they are consolidated in the card.

### Living document

Feature cards are updated as understanding improves. Acceptance
criteria are added, refined, and marked done as implementation
progresses. The card reflects current state, not initial intent.

### Acceptance as spec

Each <criterion> in the acceptance_criteria block is a testable
specification line. "Done" means the criterion is verifiable in
the codebase. "Pending" means it is specified but not yet built.

### Composability

The product specification at any point is the union of all active
feature detail cards. Reading all active cards in priority order
gives a complete picture of what the product does and will do.

## Card structure

### Format

XML in CONSCIOUSNESS/features/{status}-feature-item-details/FEAT-CCC###.xml

### Required elements

- feature[@id, @status, @priority, @kano]
- title
- description
- acceptance_criteria with at least one criterion
- stories (linked story IDs)
- tasks (linked task IDs)

### Criterion statuses

- done — implemented and verifiable
- pending — specified but not yet built
- deferred — intentionally postponed

## Workflow

### Before building

Read the feature detail card. If no card exists, create one before
writing any code. The card defines what you are building.

### During building

Mark criteria as done when their implementation is verifiable.
Add new criteria discovered during implementation.

### Before review

All non-deferred criteria should be done. The card is the checklist
the reviewer uses. Partial completion is noted, not hidden.

## Composite specification

### Status

planned (TASK-CCC28711)

### Concept

A /pgps specs view would read all active feature cards and render
a composite product specification. This is the full picture of what
the product does — on demand, never committed, always current.
