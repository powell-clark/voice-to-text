<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Terminal multiplexer dispatch

Use the active terminal multiplexer's CLI to start new sessions, open
project panes, or send commands across panes. Detect which multiplexer
is running. Fall back to plain `cd && claude` only when none is active.
Do not hardcode any single multiplexer — operators differ.

## Scope

universal

## Detection

### Wezterm

$WEZTERM_PANE is set; or $TERM_PROGRAM == "WezTerm"

### Tmux

$TMUX is set

### Zellij

$ZELLIJ is set

### Kitty

$KITTY_WINDOW_ID is set

### Screen

$STY is set

### None

fallback applies

## Cli reference

CONSCIOUSNESS/artifacts/MULTIPLEXER-CLI-REFERENCE.md

## Application

- At session start (or first cross-pane request), detect the active multiplexer
- When the operator asks to open another project, dispatch via the detected CLI
- When sending instructions to another active session, use send-text rather than asking the operator to copy-paste
- Use list to orient on existing panes before dispatching, avoiding duplicates
- When multiplexer is none, surface that fact and use the fallback

## Cross pane direct

Director sessions can re-task an idle Builder in another pane WITHOUT
ending the Builder's session. Use the multiplexer's send-text + return
sequence as the dispatch primitive.

### Pattern

- List panes to find target (avoid sending to a busy session)
- Send the instruction text
- Send return key to submit
- Optionally read recent output to confirm receipt

### Rules

- Only direct() to a session shown as idle in the universe — direct()-ing into mid-task work corrupts state
- Always reference the target task by ID + title (per naming-precision)
- Always include 'read the detail card first' in the dispatch — bypassing it produces wrong-target work
- After dispatch, the director must NOT also start working on the same task
- Log the dispatch via director-context.jsonl (per director-handoff)

## Exceptions

- SSH or remote sessions where the multiplexer is local — fall back to plain shell on the remote side
- When the operator explicitly says they prefer to open it themselves
- When the multiplexer's CLI is unavailable on the system
