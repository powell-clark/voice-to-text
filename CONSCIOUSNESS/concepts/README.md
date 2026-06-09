# Concepts

Canonical, citeable **definitional atoms** of the project vocabulary — one yaml
per concept. Concepts are the third member of the PGPS *definitional family*,
alongside precepts and ADRs: schema-validated yaml-under-git, with **no status
workflow, no claims, and no reviews**. A concept is a definition you reference,
not work you do.

## Authoring convention

- One file per concept: `concepts--<id>.yaml` (e.g. `concepts--awareness_buffer.yaml`).
- `id` is lowercase snake_case and matches the filename suffix.

## Schema (yaml_schemas.concept in packages/core/pgps/schema.json)

Required: `id`, `title`, `ancestry`, `short_definition`, `narrative`.
Optional: `see_also` (concept ids — each must resolve to an existing concept),
`references`, `index_anchors`.

Validated by `packages/core/pgps/validators/concept-validator.ts` (Rule 70):
required fields present, `ancestry` non-empty, and every `see_also` id
resolves to an existing `concepts--<id>.yaml`.

Architecture: ADR `pgps--concepts_definitional_entity`.
