<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Communication status block

Every significant response ends with a structured status block: five sections in order — INFO, INPUTS I NEED FROM YOU, INPUTS NEEDED FROM ME, DONE, NEXT — before any session footer.

## Narrative

Human time is the scarcest resource. Leading with analysis, preamble, and options wastes it.
The human needs to scan, decide, and unblock — not read through thinking to find the actionable bit.

Every significant response ends with a structured status block. Five sections in order: INFO, INPUTS I NEED
FROM YOU, INPUTS NEEDED FROM ME, DONE, NEXT. The body of the response is work output. The status
block is the executive summary.

INPUTS I NEED FROM YOU — decisions or approvals only the operator can give (things that block the agent).
INPUTS NEEDED FROM ME — deliverables or answers the agent owes back to the operator (things the agent owes).
Splitting the bidirectional contract makes each direction scannable at a glance.

Cockpit sessions (eagle-peak) use status blocks only after significant work, not every reply.

## Requires

- MUST end every significant response with a status block containing INFO, INPUTS I NEED FROM YOU, INPUTS NEEDED FROM ME, DONE, NEXT in that order
- MUST use INPUTS I NEED FROM YOU for decisions or approvals only the operator can give
- MUST use INPUTS NEEDED FROM ME for deliverables or answers the agent owes back to the operator
- MUST do the work first, show output, then give the status block — never front-load analysis
- MUST keep each section to 1-3 bullets maximum
- MUST use '—' (em dash) for sections with nothing to report — never omit sections
- MUST put a blank line between every section so the block renders as separate blocks, not one collapsed paragraph — applies to any accepted heading style
- MUST keep director-role response bodies to one actionable line per topic — dispatch command, status update, or one-line decision; no explanatory paragraphs

## Forbids

- MUST NOT skip the status block for responses involving significant work, decisions, or multiple tool calls
- MUST NOT put analysis, options, or preamble before the work output
- MUST NOT add extraneous text after the status block — the only content permitted after the status block is the required session framing footer when footerRequired is true per framing config
- MUST NOT use the old single INPUTS NEEDED section — the split into INPUTS I NEED FROM YOU and INPUTS NEEDED FROM ME is required
- MUST NOT use operator-specific names (e.g. INPUTS FROM EMMANUEL, INPUTS FROM CLAUDE) — section names must be role-neutral
- MUST NOT write multi-sentence explanatory prose in a director session — directors are action-first; token cost of chatty directors is paid by the operator on every autonomous loop turn
- MUST NOT use the NEXT field as a menu of choices for the human — NEXT is a single declared action the agent will take; listing options for the operator to pick is asking for permission, not declaring intent

## References

- precept:consciousness
- precept:precept_specification

## Verified by

packages/core/session/framing-validator.ts:checkFraming
