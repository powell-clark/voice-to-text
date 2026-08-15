<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Changelog style

Release blocks are flat: no '### ' heading inside a block, one bullet per change, each opening with a verb from the closed set (Added, Changed, Deprecated, Fixed, Hardened, Improved, Reduced, Removed, Security, Updated) and stating the operator-visible effect in one sentence under 300 characters; rationale, rejected alternatives, reproduction detail and provenance beyond a single entity id belong on the task card, in the ADR, or in stream/commentary.jsonl.

## Trigger

when writing an entry to a CHANGELOG.md

## Narrative

Two defects motivate this, and the first is structural rather than aesthetic.

CONSCIOUSNESS/CHANGELOG.md is mapped to the `union` merge driver in
.gitattributes — the right call for the repo's worst conflict site, because
every branch prepends a bullet to the same list and keeping both sides is
always correct for an append-only file. Its documented cost is that two
branches each introducing the SAME heading leave two copies, and
bump-version.sh then promotes the block verbatim into the published GitHub
release body. Five duplicate `### Fixed` headings shipped that way across
v0.45.15, v0.45.19 and v0.45.24 (GitHub #1311). A flat block has no heading
for the driver to duplicate, so the failure becomes unrepresentable rather
than merely repaired.

The second is size. Measured over 4,839 lines and 1,846 bullets: 409 bullets
exceeded 300 characters, 211 exceeded 600, and 97 exceeded 1,200. Those are
not entries, they are postmortems — counting-polarity arguments, measurement
tables, rejected alternatives — published to readers who have no repo access
and no way to act on them. The precept_specification precept already mandates
exactly this relocation for precepts ("relocate postmortems … to
commentary.jsonl, and decision rationale to the relevant ADR"); nothing had
ever applied the same rule to the changelog.

Dropping that detail is not losing it. Every condensed bullet's original text
moves to the task card, the ADR, commentary.jsonl or an artifacts/ note in the
same change, so the reasoning stays greppable and the release body stays
readable. A changelog answers "what changed for me"; the diagnosis lives where
the next session will actually look for it.

The convention binds forward. Blocks released before it landed are frozen
history under the safety precept's append-only contract and are neither
linted nor rewritten.

## Requires

- MUST open every changelog bullet with one of Added, Changed, Deprecated, Fixed, Hardened, Improved, Reduced, Removed, Security, Updated — the verb carries the category that the '### ' heading layer used to carry
- MUST keep each bullet to one sentence stating the operator-visible effect, under 300 characters
- MUST relocate rationale, rejected alternatives, measurements and reproduction detail to the task card, the ADR, CONSCIOUSNESS/stream/commentary.jsonl or CONSCIOUSNESS/artifacts/ in the same change that condenses the bullet
- MUST write entries through the sanctioned append flow (add-entry-cli.js), which enforces this contract at the write path
- MUST run the changelog check before requesting review on any change that touches the changelog

## Forbids

- MUST NOT create a '### ' heading inside a release block — that is the line the union merge driver duplicates
- MUST NOT cite a PR number or a commit SHA in a bullet; git carries provenance and a published release body is read by people without repo access
- MUST NOT cite more than one entity id in a bullet
- MUST NOT rewrite or reformat a released block — history is append-only and predates the convention
- MUST NOT delete the detail a bullet is condensed out of; relocate it, and say where in the same change

## References

- precept:safety
- precept:precept_specification
- precept:naming-precision
- doc:CONSCIOUSNESS/CHANGELOG.md

## Verified by

packages/core/changelog/style.ts:lintUnreleased
