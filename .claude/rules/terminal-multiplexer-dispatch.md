<!-- GENERATED from CONSCIOUSNESS/precepts/ — do not edit manually -->
<!-- Source of truth: CONSCIOUSNESS/precepts/{name}.yaml -->
<!-- Regenerate: pnpm run generate:rules -->


# Terminal multiplexer dispatch

Use the active terminal multiplexer's CLI to start new sessions, open project panes, or send commands across panes; detect which multiplexer is running (wezterm via $WEZTERM_PANE, tmux via $TMUX, zellij via $ZELLIJ, kitty via $KITTY_WINDOW_ID, screen via $STY); when no multiplexer is active fall back to the host's native background-session manager if the host provides one (Claude Code exposes `claude agents` from v2.1.141), then to plain cd-and-claude only when neither is available; do not hardcode any single multiplexer and do not assume a native manager exists, because operators and hosts differ.

## Narrative

Issue #122 (BOOK project, 2026-04-19) demonstrated a failure mode when no IPC exists between Claude sessions: a Director monitoring an idle builder concluded the builder was finished and began implementing the same work in parallel. The root cause was twofold — idle session status was treated as 'task complete' without checking git, and the Director had no explicit constraint preventing it from doing builder work.

The fix encodes two hard rules: (1) idle ≠ finished — absence of a task claim does not imply the builder has committed all their work; check git status and git log before drawing conclusions; (2) Director is observe-only when no IPC channel connects sessions — the only valid escalation for a paused builder is to notify the operator. Doing the work yourself corrupts the division of responsibility and risks creating a duplicate implementation that the original builder overwrites on resume.

## Requires

- MUST detect the active multiplexer at session start or at the first cross-pane request
- MUST identify own pane ID at session start: WezTerm → $WEZTERM_PANE; tmux → $(tmux display-message -p '#{pane_id}'); kitty → $KITTY_WINDOW_ID — exclude own pane from all dispatch targets
- MUST dispatch new project panes via the detected multiplexer's CLI when the operator asks to open another project
- MUST use send-text (not copy-paste prompts) when forwarding instructions to another active session
- MUST list existing panes before dispatching to avoid duplicate sessions on the same target
- MUST surface the no-multiplexer condition and use the host's native background-session manager when one exists, falling back to cd-and-claude only when none does, rather than silently degrading
- MUST prefer the host's native background-session manager over plain cd-and-claude when no multiplexer is detected and the host provides one — Claude Code exposes `claude agents` from v2.1.141, which carries the session registry and worktree isolation that bare cd-and-claude discards
- MUST pass explicit session-config flags when dispatching via the native manager where the host supports them — Claude Code `claude agents` from v2.1.142 accepts --permission-mode, --model, --effort, --settings, --add-dir, --mcp-config, and --plugin-dir
- MUST cross-pane direct() to idle Builder sessions only — directing into mid-task work corrupts state
- MUST reference target tasks by ID and title together when dispatching (per naming-precision)
- MUST include 'read the detail card first' in every cross-pane dispatch instruction
- MUST log every cross-pane dispatch via director-context.jsonl per director-handoff
- MUST check git log AND git status in an actor's working area before concluding that actor has finished — idle status (no active task claim in active-sessions) does NOT imply committed work is complete
- MUST treat Director role as observe-only when no IPC channel connects sessions — the only valid escalation for a paused or idle builder is to notify the operator to wake them
- MUST verify via git status that no uncommitted work exists in a target area before instructing a builder to work there — an existing file may be another session's in-progress work, not an open artifact

## Forbids

- MUST NOT hardcode tmux or wezterm or any single multiplexer in dispatch logic
- MUST NOT direct() to a session shown as actively working on a task — only idle Builder sessions accept dispatch
- MUST NOT continue working on the same task after dispatching it cross-pane (avoid double-claim)
- MUST NOT use multiplexer CLI on the remote side of an SSH session — the multiplexer is local; fall back to plain shell remotely
- MUST NOT fall through to plain cd-and-claude when the host exposes a native background-session manager — that discards the host session registry and worktree isolation the manager provides
- MUST NOT send-text to a pane without first verifying the target pane_id does not equal own pane ID — self-targeting pollutes the director's own input box
- MUST NOT assume wezterm pane numbers (0, 1, 2…) map to pts/N TTY numbers — they are independent namespaces; verify via wezterm cli list
- MUST NOT treat an idle builder (current_task absent from active-sessions) as finished — absence of a task claim does not imply completion of uncommitted work; the builder may be paused, context-switched, or stalled
- MUST NOT write code, implement features, or build artifacts when operating in Director role — doing builder work from a Director session corrupts the division of responsibility and risks duplicating work the original builder will overwrite

## References

- precept:naming-precision
- precept:director-handoff
- doc:CONSCIOUSNESS/artifacts/MULTIPLEXER-CLI-REFERENCE.md

## Verified by

packages/core/session/dispatch.ts:getAssignment
