<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Merge conflict resolution

Conflicts in generated build output are resolved by recompiling the merged source, never by choosing or splicing a side; conflicts in append-only files are resolved by keeping both sides; and because a forge's web UI computes conflicts without reading .gitattributes, a repo that declares merge drivers is merged in a local clone with those drivers registered rather than through the forge.

## Trigger

when resolving a git merge conflict

## Narrative

Three facts, each measured in this federation, explain why conflict
resolution needs a rule rather than judgement.

The first is proportion. One session took 610 merge conflicts across two
pull requests, and 588 of them — 96% — were in committed build output. That
output is derived: the correct resolution is never either side, it is
"recompile the merged source". Hand-merging a bundle produces a file that
still parses, so the damage arrives at CI as behaviour rather than as a
syntax error. Separately, CONSCIOUSNESS/CHANGELOG.md was measured as the
single worst conflict site in the repository — the conflicting file on 7 of
12 conflicting pull requests — because every branch prepends a bullet to the
same list.

The second is that the forge lies about the conflict set. GitHub, and web
merge UIs generally, compute conflicts server-side without applying
.gitattributes. A repository whose drivers resolve twenty of twenty-one
files locally still shows all twenty-one as conflicts in the browser. An
operator who forwards that list to an agent has asked it to hand-merge files
git would have merged correctly and unattended. The fix is not a better
prompt; it is doing the merge where the drivers live.

The third is that the safe resolution differs by file class, and getting it
backwards is silent. For an append-only file — an event log, a changelog, an
index that only ever grows — taking one side destroys the other branch's
rows with no diagnostic, which is exactly the incident that cost six
evidence rows in one merge. The built-in union driver makes keeping both
sides automatic. But union is wrong wherever rows carry identifiers
allocated as "local maximum plus one": two branches offline both allocate the
same id, and union turns that loud conflict into a green merge over a
duplicate. This federation has deliberately split on that point — the
consciousness repo unions its task and review indexes because a CI
uniqueness test and a same-filename detail card both fail the merge, while
apgps refuses to union its indexes at all under ADR
pgps--index_merge_strategy, having no such backstop. Both are correct for
their repo. Neither is a default to copy without checking.

Registration is the last trap. union, ours and theirs are built into git and
need no setup, so a rule using them is live on every clone immediately. A
custom driver is a name in .gitattributes pointing at a command the local
git config must define, and until the repo's documented install step has run
that name resolves to nothing and git silently falls back to an ordinary
text merge. The fallback is the old behaviour rather than a new hazard, but
it means a driver can appear declared and be inert.

## Requires

- MUST resolve a conflict in generated or derived build output by recompiling the merged source, never by selecting or splicing either side
- MUST perform the merge in a local clone when the repository declares merge drivers in .gitattributes, because forge web-UI merges do not apply them
- MUST run the repository's documented driver-registration step before merging when .gitattributes names a custom driver — detect which step that repo uses rather than assuming one package manager or script name
- MUST keep both sides when resolving an append-only file, and verify after the merge that every row present on either side survives
- MUST confirm a duplicate-identifier backstop gates CI before applying union to any index whose rows allocate ids from a local maximum
- MUST name the file classes in a conflict set before resolving it — derived output, append-only, and hand-authored source each take a different resolution

## Forbids

- MUST NOT resolve an append-only file by taking one side — that discards the other branch's rows with no diagnostic
- MUST NOT hand-edit, splice, or manually reconcile generated build output during conflict resolution
- MUST NOT apply union to an index whose rows allocate identifiers from a local maximum unless a CI uniqueness check fails the merge on a duplicate
- MUST NOT treat a forge's conflict banner as the true conflict set for a repository that declares merge drivers
- MUST NOT widen a merge-driver glob to cover files that are rewritten in place — union concatenates two revisions of one record into corruption
- MUST NOT report a merge as resolved without re-running the repository's checks; a semantic conflict passes git merge and fails CI

## References

- precept:safety
- precept:evidence-discipline
- precept:precept_specification
- doc:.gitattributes
