<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Review gates

Items cannot advance from in_review to done without a qualifying
approval. The review process is layered: agent review prepares,
human review approves.

## Scope

universal

## Status flow

### Standard

pending → in_progress → in_review → done

### Agent boundary

in_review — agent cannot cross this to done

### Human boundary

done — only human can set this status

### Reference

STEER-CC004

## Agent review

### What agents do

- Run test suites and verify passing
- Check acceptance criteria against implementation
- Verify code coverage thresholds are met
- Validate PGPS integrity after changes
- Check for security issues (OWASP top 10)

### What agents cannot do

- Mark items as done
- Override human rejection
- Skip review for convenience

## Human review

### Required for

- Moving items to done status
- Accepting or rejecting stories
- Approving features for release
- Closing directives

### Tools

- approve CLI: node core/dist/review/approve.js <entity-id>
- REVIEW-INDEX.md tracks all verdicts (agent and human)
- Prose PGPS review (future: FEAT-CCC96)

## Review record

### Format

PSV in CONSCIOUSNESS/reviews/REVIEW-INDEX.md

### Columns

id|target_type|target_id|reviewer_type|reviewer_id|verdict|iteration|reviewed_at|notes

### Verdicts

#### Agent

pending-review, agent-approved, agent-rejected

#### Human

human-approved, human-rejected

## Trust scoring

### Status

planned (FEAT-CCC94)

### Concept

When a human approves or rejects an item, their verdict is compared
against prior agent reviewers. Agents whose verdicts align with the
human earn trust. Agents whose verdicts diverge lose trust. Over time,
trusted agents may earn delegation authority.

## Exceptions

- Autonomous task claiming (backlog → in_progress) does not require review
- Bug fixes in conscious mode: create task → fix → in_review (agent cannot skip to done)
