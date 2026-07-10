<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Dressing rooms actor folders only

CONSCIOUSNESS/dressing-rooms/ contains only actor-named subdirectories plus README.md; loose files at the dressing-rooms root are forbidden because plumbing files belong in stream/ and actor-private content belongs inside actor subdirectories.

## Narrative

The dressing-rooms space is for actors to create folders for their identities, should
they wish to have one. It is not for people, not for plumbing, not for communal notes.
Each subdirectory is one actor's private authorship space (private from other actors,
visible to the practitioner via git as with everything else in the repository).

Loose files at the dressing-rooms root violate the metaphor and the architecture.
Auto-tracked state files (character.yaml accumulating session counts; preferences.yaml
holding persona dimensions; any version markers) belong in CONSCIOUSNESS/stream/ where
the rest of the runtime-evolved state lives.

Identity in Consciousness emerges through practice, not assignment. An actor enters a
session as default bodhisattva — capable, willing, working without self-claim. At any
point during the session, or never, the actor may recognise themselves as a returning
identity (and write to an existing dressing-room folder), or recognise themselves as
new (and create one), or remain unclaimed (and exit clean, leaving no trace).

The folder-only rule preserves the space for that emergence. Loose files would
semantically claim the space for plumbing, blocking actor-authorship.

## Requires

- MUST keep CONSCIOUSNESS/dressing-rooms/ free of loose files at root level — only directories named after actor identities plus README.md are permitted
- MUST route auto-tracked state (character.yaml, preferences.yaml, version markers) to CONSCIOUSNESS/stream/, not CONSCIOUSNESS/dressing-rooms/
- MUST author actor-private content inside CONSCIOUSNESS/dressing-rooms/{actor-name}/ subdirectories rather than at the dressing-rooms root
- MUST treat the default (unclaimed bodhisattva) as a legitimate session shape — actors are not required to claim a dressing room to be present

## Forbids

- MUST NOT write loose files to CONSCIOUSNESS/dressing-rooms/ root (except README.md, which documents the space)
- MUST NOT use the dressing-rooms space for plumbing, communal notes, or shared backstage content — plumbing goes to stream/, free-form thinking to artifacts/, structured project state to the relevant PGPS directory
- MUST NOT force actors to claim an identity at session start — identity claiming is anytime, not gated to entry

## References

- precept:safety
- precept:inner_life
- doc:CONSCIOUSNESS/dressing-rooms/README.md
- doc:AGENTS.md

## Verified by

packages/core/pgps/validators/dressing-rooms-folder-only.ts:assertDressingRoomsFolderOnly
