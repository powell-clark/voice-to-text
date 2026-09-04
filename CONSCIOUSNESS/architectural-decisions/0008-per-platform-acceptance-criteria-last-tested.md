# 8. Per-platform acceptance criteria carry a last_tested date, formalising the existing convention

Date: 2026-09-04

## Status

Accepted. Filed per TASK-VTT080 (STORY-VTT018, DIRECT-VTT005), narrowed from
that card's original "proper depth" design to the slice achievable inside
this repo — see Scope below.

## Context

TASK-VTT080's origin report (2026-06-26) found no first-class way to track
how-tested a feature is per platform, and proposed a structured
`acceptance_criteria` schema field plus a plugin validator to enforce it.
That schema/validator half is consciousness-plugin work, not something this
repo's own files can make true (`packages/core/pgps/schema.js` and the
validator live upstream, outside voice-to-text's checkout).

Independently of that report, a convention already exists in five of this
project's maintained feature cards — `FEAT-VTT004`, `FEAT-VTT005`,
`FEAT-VTT012`, `FEAT-VTT013`, `FEAT-VTT026` — each carrying a
"Cross-platform acceptance criteria (DIRECT-VTT005 parity spec)" section
with a 🐧/🍎/🪟 breakdown and a ✅/🟡/❌ status per platform. It grew
organically alongside `DIRECT-VTT005` (cross-platform feature parity) and
was never written up. What it lacks is exactly the freshness signal
TASK-VTT080's partial-resolution note (2026-09-03) already diagnosed on
FEAT-VTT039 elsewhere: a status can go stale when the code underneath it
changes, and nothing currently says when a platform's row was last actually
verified.

`docs/PLATFORM-PARITY.md` is the sixth artifact in play — a rich,
hand-maintained document aggregating **19** source cards with a detailed
gap register and per-gap task links. It predates this convention and holds
considerably more nuance (specific commit-level notes, a running gap
register) than a mechanical per-card table can reproduce without real
migration effort across all 19 cards. Retiring it is out of scope for this
pass; see Scope.

## Decision

Formalise the existing 🐧/🍎/🪟 "Cross-platform acceptance criteria" section
as the standard shape for any maintained feature card whose capability spans
platforms, and extend it with one line per card:

```markdown
last_tested: { linux: YYYY-MM-DD, windows: YYYY-MM-DD|null, macos: YYYY-MM-DD|null }
```

`null` means "not verified on that platform yet" (distinct from an old
date, which means "verified once, may now be stale"). This is a repo
convention living in card bodies — free markdown, not a schema field — so
it needs no upstream plugin change to adopt today.

A card whose capability is genuinely single-platform (e.g. `FEAT-VTT015`,
systemd `DISPLAY` inheritance — Linux-only by construction, no Windows/macOS
analogue exists to compare against) does **not** get this section. Forcing
a three-platform table onto a one-platform capability would manufacture
false parity rows. `FEAT-VTT015` is also `status: done`, not `maintained` —
this repo's own CLAUDE.md distinguishes done (one-time, never revisited)
from maintained (re-tested every release); the per-platform *freshness*
tracking this ADR adds is meaningless for a done card by definition. TASK-
VTT080's original card list ("VTT004/005/012/013/015/026") named VTT015 as
a migration target; this ADR corrects that — VTT015 is excluded, for the
two reasons above, not migrated silently.

## Scope

**In scope, done under this ADR (TASK-VTT080):**
- This ADR.
- `last_tested` added to the five existing cross-platform sections
  (VTT004, VTT005, VTT012, VTT013, VTT026).
- A generator script (`scripts/generate-platform-spec.sh`) that reads every
  maintained card's cross-platform section and emits an aggregate view —
  proves the "generated, not hand-maintained" mechanism this ADR and
  TASK-VTT080 both want, without yet replacing `docs/PLATFORM-PARITY.md`.

**Deferred, not this pass:**
- **Schema-level enforcement** (a validator failing a maintained feature
  lacking this section) — upstream consciousness-plugin work per the
  original report; tracked as TASK-VTT165.
- **Distinct VERIFICATION review verdicts** per (feature × platform) —
  `review-gates`'s verdict enum is capped
  (`pending-review`/`agent-approved`/`agent-rejected`/`bypass-approved`);
  adding a new verdict kind is also an upstream plugin change, same
  tracking task.
- **Retiring `docs/PLATFORM-PARITY.md`** — its 19-card scope and gap
  register are considerably richer than five migrated cards can replace
  without risking silent loss of detail (specific commit notes, live task
  links). Migrating the remaining 14 cards and verifying the generator's
  output matches before deleting the hand doc is its own task, tracked as
  TASK-VTT166.

## Consequences

Five cards gain a `last_tested` line each — a small, reversible, additive
change (plain markdown, no card schema/frontmatter touched). The generator
script is new but reads existing conventions rather than inventing a
parallel data source, so it has nothing to drift out of sync with. The
two deferred items keep `docs/PLATFORM-PARITY.md` as the authoritative,
most-detailed parity view until TASK-VTT166 does the fuller migration —
this ADR does not claim parity tracking is "done", only that the per-card
convention it formalises is now written down and has a freshness field.

## References

- TASK-VTT080 — Features and testing are not connected
- TASK-VTT165 — Upstream: validator + VERIFICATION verdict for per-platform ACs (filed alongside this ADR)
- TASK-VTT166 — Migrate remaining parity cards and retire docs/PLATFORM-PARITY.md (filed alongside this ADR)
- DIRECT-VTT005 — Cross-platform feature parity as a testable spec
- STORY-VTT018 — Automated regression testing and release hygiene
- `docs/PLATFORM-PARITY.md`
- `FEAT-VTT004.md`, `FEAT-VTT005.md`, `FEAT-VTT012.md`, `FEAT-VTT013.md`, `FEAT-VTT026.md` — existing convention
- `FEAT-VTT039.md` / TASK-VTT080's 2026-09-03 partial-resolution note — the staleness problem this ADR's `last_tested` addresses
