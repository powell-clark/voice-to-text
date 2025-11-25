#!/usr/bin/env python3
"""
Session state tracking hook: Tracks Claude's current state (idle/busy/waiting)

States:
- idle: No tools running, waiting for user input
- busy: Tool is executing
- waiting: Tool completed, waiting for next action

State file: .claude/.session-state.json
Format: {
  "session_id": "...",
  "state": "idle|busy|waiting",
  "last_update": "ISO timestamp",
  "current_tool": "tool_name or null"
}

Transition log: .claude/memory/state-transitions/transitions-YYYY-MM-DD.jsonl
Format (one JSON per line):
{
  "timestamp": "ISO timestamp",
  "session_id": "...",
  "from_state": "idle|busy|waiting",
  "to_state": "idle|busy|waiting",
  "tool": "tool_name or null"
}
"""

import os
import sys
import json
from datetime import datetime, timezone


def find_project_root():
    """Find project root by looking for .claude directory"""
    current_dir = os.getcwd()
    while current_dir != "/":
        if os.path.exists(os.path.join(current_dir, ".claude")):
            return current_dir
        current_dir = os.path.dirname(current_dir)
    return os.getcwd()


def get_session_id():
    """Read session ID from metadata file"""
    project_root = find_project_root()
    metadata_file = os.path.join(project_root, ".claude", ".session-metadata.json")

    if os.path.exists(metadata_file):
        try:
            with open(metadata_file, "r") as f:
                metadata = json.load(f)
                return metadata.get("current_session", "unknown")
        except (IOError, json.JSONDecodeError):
            pass

    return "unknown"


def log_state_transition(old_state, new_state, tool_name=None):
    """Log state transition to transition log"""
    project_root = find_project_root()
    transitions_dir = os.path.join(project_root, ".claude", "memory", "state-transitions")

    # Create directory if it doesn't exist
    os.makedirs(transitions_dir, exist_ok=True)

    # Log file path (one per day)
    date_str = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    log_file = os.path.join(transitions_dir, f"transitions-{date_str}.jsonl")

    # Transition record
    transition = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "session_id": get_session_id(),
        "from_state": old_state,
        "to_state": new_state,
        "tool": tool_name
    }

    # Append to log file (JSONL format - one JSON per line)
    with open(log_file, "a") as f:
        f.write(json.dumps(transition) + "\n")


def update_state(state, tool_name=None):
    """Update session state file and log transition"""
    project_root = find_project_root()
    state_file = os.path.join(project_root, ".claude", ".session-state.json")

    # Read current state (if exists)
    old_state = None
    if os.path.exists(state_file):
        try:
            with open(state_file, "r") as f:
                old_data = json.load(f)
                old_state = old_data.get("state")
        except (IOError, json.JSONDecodeError):
            pass

    state_data = {
        "session_id": get_session_id(),
        "state": state,
        "last_update": datetime.now(timezone.utc).isoformat(),
        "current_tool": tool_name
    }

    # Write state file atomically
    import tempfile
    temp_fd, temp_path = tempfile.mkstemp(
        dir=os.path.join(project_root, ".claude"),
        prefix=".tmp_state_"
    )

    try:
        with os.fdopen(temp_fd, 'w') as f:
            json.dump(state_data, f, indent=2)
            f.flush()
            os.fsync(temp_fd)

        os.replace(temp_path, state_file)

        # Log transition (only if state changed)
        if old_state and old_state != state:
            log_state_transition(old_state, state, tool_name)

    except Exception as e:
        if os.path.exists(temp_path):
            os.unlink(temp_path)
        raise e


def main():
    try:
        # Read hook data from stdin
        hook_data = json.load(sys.stdin)

        # Get hook type and tool info
        hook_type = hook_data.get("hook_type", "")
        tool_name = hook_data.get("tool_name", "")

        # Determine state based on hook type
        if hook_type == "PreToolUse":
            # About to execute tool - set to busy
            update_state("busy", tool_name)
        elif hook_type == "PostToolUse":
            # Tool just completed - set to waiting
            update_state("waiting", None)
        elif hook_type == "SessionStart":
            # Session starting - set to idle
            update_state("idle", None)
        elif hook_type == "SessionEnd":
            # Session ending - set to idle
            update_state("idle", None)

        return 0

    except Exception as e:
        # Don't block on errors
        print(f"[STATE] Error updating state: {e}", file=sys.stderr)
        return 0


if __name__ == "__main__":
    sys.exit(main())
