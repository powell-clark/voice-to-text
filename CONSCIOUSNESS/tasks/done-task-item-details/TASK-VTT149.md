# TASK-VTT149: Timing recorder drops fields on an in-flight build

## Context

Introduced by TASK-VTT134 and caught on its first real use, the v2.3.11
release. The recorder ran immediately after upload, when the build had not
finished, and produced:

    date: invalid date 'Currently building'
      2.3.11 noble/amd64: queued 2s, built -16s, published after pending

`record-ppa-times.sh` reads jq's `@tsv` output with `IFS=$'\t'`. Tab is IFS
whitespace, so bash `read` collapses consecutive tabs rather than treating the
gap between them as an empty field. A build still in progress has a null
`datebuilt`, which `// ""` turned into an empty field, so every later field
shifted one place left: `buildstate` landed in the `build_finish` variable,
`date -d` was handed "Currently building", and the row recorded a negative
build time.

The row written to the tracked log was wrong in the same way — the state
string sat in the `build_finish_utc` column and the `state` column was empty.

This is the same class of defect as the one this file already warns about for
the feature index: a shifted column is silent, and the data looks plausible
until something tries to parse it.

## Acceptance criteria

- [x] No field is ever emitted empty — the jq projection substitutes "pending"
      for a null `date_started`, `datebuilt`, or missing publication, so the
      tab-collapsing behaviour of `read` has nothing to collapse
- [x] `delta()` only subtracts values that look like ISO-8601 instants;
      anything else ("pending", a build state, an empty string) yields empty
      and renders as "pending" rather than reaching `date -d`
- [x] Verified against the exact failing input — the same in-flight noble
      build now records `built pending` with no `date:` error, and the row has
      all 12 columns with "Currently building" in the state column
- [x] Re-running replaces the bad row rather than appending beside it, so the
      tracked log is corrected in place
- [x] Release step counter corrected — the timings step made the script print
      "[10/9]"; the total is now 1 build + 3 per distro + tag + archive +
      timings

## Notes

Two defences rather than one, because either alone would have prevented this
and the failure is silent: the projection stops empty fields existing, and
`delta()` refuses to compute on anything that is not a timestamp even if a
field does shift for some other reason.

## Dependencies

- Story: STORY-VTT018
- Directive: DIRECT-VTT002
