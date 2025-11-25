#!/usr/bin/env python3
"""
Track activity context throughout session for meaningful time log entries.
Accumulates context each 3-minute block, generates summary at block end.
"""

import sys
import json
import os
from datetime import datetime, timedelta
from pathlib import Path


def get_current_block():
    """Get current 3-minute time block"""
    now = datetime.now()
    minutes = (now.minute // 3) * 3
    block_start = now.replace(minute=minutes, second=0, microsecond=0)
    block_end = block_start + timedelta(minutes=3)
    return f"{block_start.strftime('%H:%M')}-{block_end.strftime('%H:%M')}"


def get_context_file(session_id):
    """Get path to running context file for this session"""
    context_dir = Path.home() / ".claude" / "memory" / "time-tracking"
    context_dir.mkdir(parents=True, exist_ok=True)
    return context_dir / f"context-{session_id}.json"


def load_context(session_id):
    """Load running activity context"""
    context_file = get_context_file(session_id)
    if not context_file.exists():
        return {
            "current_block": get_current_block(),
            "block_start_time": datetime.now().isoformat(),
            "user_messages": [],
            "activities": [],
            "tools_used": set(),
            "state_changes": []
        }

    try:
        with open(context_file) as f:
            content = f.read().strip()
            if not content:  # File is empty
                return {
                    "current_block": get_current_block(),
                    "block_start_time": datetime.now().isoformat(),
                    "user_messages": [],
                    "activities": [],
                    "tools_used": set()
                }
            data = json.loads(content)
            data["tools_used"] = set(data.get("tools_used", []))
            data["state_changes"] = data.get("state_changes", [])
            return data
    except (json.JSONDecodeError, IOError):
        # Corrupted or locked file - return fresh context
        return {
            "current_block": get_current_block(),
            "block_start_time": datetime.now().isoformat(),
            "user_messages": [],
            "activities": [],
            "tools_used": set(),
            "state_changes": []
        }


def save_context(context, session_id):
    """Save running activity context"""
    context_file = get_context_file(session_id)
    data = dict(context)
    data["tools_used"] = list(data["tools_used"])
    
    with open(context_file, 'w') as f:
        json.dump(data, f, indent=2)


def format_timestamp():
    """Format timestamp consistently"""
    return datetime.now().strftime("%Y-%m-%d %H:%M:%S GMT")


def write_time_entry(log_file, block, session_id, activity, include_project=False, project_name=""):
    """Write time entry to log file (with deduplication)"""
    timestamp = format_timestamp()

    # Clean activity: remove newlines and normalize whitespace
    activity = " ".join(activity.split())

    # DO NOT truncate - let terminal handle line wrapping
    # Long descriptive entries are valuable for time tracking

    # Read existing content
    if log_file.exists():
        with open(log_file) as f:
            content = f.read()
    else:
        # Create new file with headers
        today = datetime.now().strftime("%Y-%m-%d")
        if include_project:
            content = f"# Master Time Log\n\nAll projects log here for master view of time across projects.\n\n## {today}\n\nDate Time Window | Updated At | Project | Session ID | Activity\n-----------------|------------|---------|------------|----------\n"
        else:
            content = f"# Time Log\n\n## {today}\n\nDate Time Window | Updated At | Session ID | Activity\n-----------------|------------|------------|----------\n"

    # Find insertion point (after header row, before first data row)
    lines = content.split("\n")
    insert_idx = None

    for i, line in enumerate(lines):
        if "|------" in line:  # Found separator row
            insert_idx = i + 1
            break

    if insert_idx is None:
        return False  # Couldn't find table structure

    # Create new entry
    if include_project:
        new_entry = f"{block} | {timestamp} | {project_name} | {session_id} | {activity}"
        # Check for duplicate: same block + project + session_id
        duplicate_pattern = f"{block} |"
        existing_entries_for_block = [l for l in lines[insert_idx:] if l.startswith(duplicate_pattern)]
        if existing_entries_for_block:
            # Check if exact block+session combo already exists
            for existing in existing_entries_for_block:
                parts = [p.strip() for p in existing.split("|")]
                if len(parts) >= 4 and parts[3] == session_id:  # Same session in this block
                    return True  # Skip duplicate
    else:
        new_entry = f"{block} | {timestamp} | {session_id} | {activity}"
        # Check for duplicate: same block + session_id
        duplicate_pattern = f"{block} |"
        existing_entries_for_block = [l for l in lines[insert_idx:] if l.startswith(duplicate_pattern)]
        if existing_entries_for_block:
            # Check if exact block+session combo already exists
            for existing in existing_entries_for_block:
                parts = [p.strip() for p in existing.split("|")]
                if len(parts) >= 3 and parts[2] == session_id:  # Same session in this block
                    return True  # Skip duplicate

    # Append new entry at end (chronological order - oldest at top, newest at bottom)
    lines.append(new_entry)

    # Write back
    with open(log_file, "w") as f:
        f.write("\n".join(lines))

    return True


def generate_block_summary(context):
    """Generate rich narrative summary of what Emmanuel was orchestrating during this block"""

    # Build comprehensive description from user messages and tool context
    user_messages = context.get("user_messages", [])
    tools = context.get("tools_used", set())
    activities = context.get("activities", [])

    # Start with what Emmanuel was asking/directing
    if user_messages and len(user_messages) > 0:
        # Combine all messages from this block for full context
        message_text = ". ".join(msg.strip() for msg in user_messages if msg.strip())

        # Skip single-word responses - try to get substantive content
        single_word_responses = ["yes", "y", "ok", "sure", "no", "n", "l2h", "pgps", "gps"]
        if message_text.lower() in single_word_responses and len(user_messages) > 1:
            message_text = ". ".join(msg.strip() for msg in user_messages[-2:] if msg.strip())

        # Add tool context to show what Claude was doing in response
        tool_context = ""
        if tools:
            tool_list = sorted(tools)
            if "TodoWrite" in tools:
                tool_context = " Claude updated TODO.md tracking progress on current story."
            elif "Edit" in tools or "Write" in tools:
                tool_context = f" Claude made code changes using {', '.join(tool_list[:3])}."
            elif "Read" in tools or "Grep" in tools or "Glob" in tools:
                tool_context = f" Claude investigated codebase using {', '.join(tool_list[:3])}."
            elif "Bash" in tools:
                tool_context = " Claude executed bash commands to gather information or run operations."
            else:
                tool_context = f" Claude used tools: {', '.join(tool_list[:4])}."

        # Combine message + tool context for full picture
        summary = f"{message_text}{tool_context}"

        # Capitalize first letter
        if summary and len(summary) > 0:
            summary = summary[0].upper() + summary[1:]

        return summary

    # Fallback: describe what tools Claude was using (when no user messages captured)
    if "TodoWrite" in tools:
        return "Reviewing and updating task progress via TodoWrite tool to track Claude's work on current story"
    elif "Edit" in tools or "Write" in tools:
        files = "configuration files" if "settings" in str(context) else "code files"
        return f"Directing Claude to modify {files} and reviewing the changes being made to the codebase"
    elif "Read" in tools or "Bash" in tools:
        return "Asking Claude to investigate codebase by reading files and running commands to understand system behaviour"
    elif len(tools) == 0:
        return "Idle session with no active tool usage, possibly reviewing output or planning next steps"
    else:
        return f"Working with Claude using multiple tools: {', '.join(sorted(tools)[:4])} to complete requested tasks"


def main():
    try:
        # Read hook data
        hook_data = json.load(sys.stdin)

        # Get session_id from stdin (Claude Code provides this automatically)
        hook_session_id = hook_data.get("session_id", "")
        if not hook_session_id:
            return 0  # Can't track without session ID

        # Truncate to first 8 chars for brevity
        session_id = hook_session_id[:8]

        # Load running context
        context = load_context(session_id)
        current_block = get_current_block()

        # Check if we've moved to a new block
        if context["current_block"] != current_block:
            # Generate summary for completed block
            summary = generate_block_summary(context)

            # Write to Emmanuel's time logs
            import subprocess
            from pathlib import Path

            project_root = Path.cwd()
            while not (project_root / ".claude").exists() and project_root != project_root.parent:
                project_root = project_root.parent

            project_name = project_root.name

            # Write to daily rotated log: CONSCIOUSNESS/time-logs/human/YYYY-MM/YYYY-MM-DD.md
            from datetime import datetime
            today = datetime.now()
            year_month = today.strftime("%Y-%m")
            date_str = today.strftime("%Y-%m-%d")

            time_logs_dir = project_root / "CONSCIOUSNESS" / "time-logs" / "human" / year_month
            time_logs_dir.mkdir(parents=True, exist_ok=True)

            project_log = time_logs_dir / f"{date_str}.md"
            write_time_entry(project_log, context['current_block'], session_id, summary, include_project=False)

            # Write to master EMMANUEL-MASTER-TIME-LOG.md
            master_log = Path.home() / ".claude" / "memory" / "time-tracking" / "EMMANUEL-MASTER-TIME-LOG.md"
            write_time_entry(master_log, context['current_block'], session_id, summary, include_project=True, project_name=project_name)

            print(f"[BLOCK COMPLETE] {context['current_block']}: {summary}", file=sys.stderr)
            
            # Reset context for new block
            context = {
                "current_block": current_block,
                "block_start_time": datetime.now().isoformat(),
                "user_messages": [],
                "activities": [],
                "tools_used": set(),
                "state_changes": []
            }
        
        # Track tool usage
        tool_name = hook_data.get("tool_name")
        if tool_name:
            context["tools_used"].add(tool_name)

        # Track session state changes
        from pathlib import Path
        project_root = Path.cwd()
        while not (project_root / ".claude").exists() and project_root != project_root.parent:
            project_root = project_root.parent

        state_file = project_root / ".claude" / ".session-state.json"
        if state_file.exists():
            try:
                with open(state_file) as f:
                    state_data = json.load(f)
                    state = state_data.get("state")
                    if state:
                        # Only log if this is a new state (not already in list)
                        if not context["state_changes"] or context["state_changes"][-1] != state:
                            context["state_changes"].append(state)
            except (IOError, json.JSONDecodeError):
                pass

        # Track user messages if available
        # (This would require UserPromptSubmit hook integration)

        # Save updated context
        save_context(context, session_id)
        
        return 0
        
    except Exception as e:
        print(f"Error tracking activity context: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc(file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
