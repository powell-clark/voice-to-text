<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Data schema version discipline

Every change to packages/core/pgps/schema.json that affects content (column shape, enum values, entity types, frontmatter requirements, JSONL shape) MUST bump CONSCIOUSNESS/stream/schema-version atomically with the schema_version field inside schema.json; both files carry the same 17-digit timestamp value plus an optional underscore-separated description; drift between them is a structural integrity failure repaired before further schema work proceeds.

## Requires

- MUST bump CONSCIOUSNESS/stream/schema-version atomically with the schema_version field in packages/core/pgps/schema.json on any structural schema change
- MUST use the marker pattern ^[0-9]{17}(_[A-Za-z0-9_-]+)?$ — 17-digit YYYYMMDDHHMMSSmmm timestamp, optional underscore description suffix
- MUST author every breaking schema change as a TypeScript migration in packages/core/pgps/migrations/YYYYMMDDHHMMSSmmm_description.ts exporting `migration: Migration` (id, description, isApplied, up) and register it in packages/core/pgps/migrations/index.ts ALL_MIGRATIONS
- MUST treat the marker file as canonical for what-schema-this-repo-honours; schema.json field documents intent and is consumer-readable

## Forbids

- MUST NOT skip the bump for any change that adds, removes, or renames an INDEX column, changes an enum, renames an entity type, alters detail-card frontmatter requirements, or changes JSONL shape
- MUST NOT bypass the pre-commit gate via --no-verify unless the schema change is genuinely whitespace or comment-only
- MUST NOT use a marker value that fails the 17-digit pattern; freeform descriptive strings are not legal markers

## References

- precept:precept_specification
- doc:CONSCIOUSNESS/directives/active-directive-item-details/DIRECT-CCC029
- doc:packages/core/pgps/migrations/index.ts

## Verified by

packages/core/pgps/migrations/runner.ts:runMigrations
