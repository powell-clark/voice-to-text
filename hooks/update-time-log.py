#!/usr/bin/env python3
"""
Auto-update time logs after user interactions.
Updates both project CONSCIOUSNESS/HUMAN-TIME-LOG.md and master ~/.claude/memory/time-tracking/HUMAN-TIME-LOG.md
"""

import sys
import json
import os
from datetime import datetime, timedelta


def find_project_root():
    """Find project root by looking for .claude directory"""
    current_dir = os.getcwd()
    while current_dir != "/":
        if os.path.exists(os.path.join(current_dir, ".claude")):
            return current_dir
        current_dir = os.path.dirname(current_dir)
    return os.getcwd()


def get_project_name():
    """Extract project name from path"""
    project_root = find_project_root()
    return os.path.basename(project_root)


def get_git_branch():
    """Get current git branch name (sanitized)"""
    project_root = find_project_root()
    try:
        import subprocess

        result = subprocess.run(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"],
            cwd=project_root,
            capture_output=True,
            text=True,
            timeout=2,
        )
        if result.returncode == 0:
            branch = result.stdout.strip()
            # Sanitize branch name - allow only alphanumeric, dash, underscore
            import re
            branch = re.sub(r'[^a-zA-Z0-9_-]', '', branch)
            if not branch:
                return None
            return branch
    except Exception:
        pass
    return None


def is_home_directory(project_root):
    """Check if we're in home directory (not a project repo)"""
    home = os.path.expanduser("~")
    return project_root == home


def get_session_id(hook_session_id):
    """Get session ID from hook stdin data (passed by Claude Code)"""
    project_root = find_project_root()

    # Use session_id from stdin (Claude Code provides this automatically)
    if not hook_session_id:
        return "unknown"

    # Truncate to first 8 chars for brevity
    conv_id = hook_session_id[:8]

    # Build session ID based on context
    if is_home_directory(project_root):
        # Home directory: just uuid8
        return conv_id
    else:
        # Project repo: project-branch-uuid8
        project_name = os.path.basename(project_root)
        branch = get_git_branch()

        if branch:
            return f"{project_name}-{branch}-{conv_id}"
        else:
            return f"{project_name}-{conv_id}"


def format_timestamp():
    """Format timestamp consistently"""
    return datetime.now().strftime("%Y-%m-%d %H:%M:%S GMT")


def get_current_block():
    """Get current 3-minute time block"""
    now = datetime.now()
    # Round down to nearest 3 minutes
    minutes = (now.minute // 3) * 3
    block_start = now.replace(minute=minutes, second=0, microsecond=0)
    block_end = block_start + timedelta(minutes=3)
    return f"{block_start.strftime('%H:%M')}-{block_end.strftime('%H:%M')}"


def get_last_block_from_log(log_file):
    """Get the last time block from log file"""
    if not os.path.exists(log_file):
        return None

    try:
        with open(log_file, "r") as f:
            lines = f.readlines()

        # Find last data row (skip headers, separators)
        for line in reversed(lines):
            if (
                "|" in line
                and not line.strip().startswith("|--")
                and "Block" not in line
            ):
                parts = [p.strip() for p in line.split("|")]
                if len(parts) >= 2 and parts[1]:
                    return parts[1]  # Return the block time

        return None
    except IOError:
        return None


def should_create_new_entry(log_file):
    """Check if we need a new time log entry"""
    current_block = get_current_block()
    last_block = get_last_block_from_log(log_file)

    # Create new entry if no previous entry or different block
    return last_block != current_block


def determine_activity(hook_data):
    """
    Determine activity description from hook data with maximum clarity.
    Generate rich narrative describing what Claude was doing with situational context.
    No length limit - comprehensive description preferred over brevity.
    """
    tool_name = hook_data.get("tool_name", "unknown")
    tool_input = hook_data.get("tool_input", {})

    if tool_name == "TodoWrite":
        # Show what todos were added/completed with full context
        todos = tool_input.get("todos", [])
        if todos:
            in_progress = [t["content"] for t in todos if t.get("status") == "in_progress"]
            completed = [t["content"] for t in todos if t.get("status") == "completed"]
            pending = [t["content"] for t in todos if t.get("status") == "pending"]

            desc = "Claude updated TODO.md task list via TodoWrite tool. "
            if completed:
                desc += f"Marked {len(completed)} task(s) as completed: {', '.join(completed)}. "
            if in_progress:
                desc += f"Currently working on: {', '.join(in_progress)}. "
            if pending:
                desc += f"{len(pending)} pending task(s) remaining: {', '.join(pending[:3])}."
            return desc
        return "Claude updated task list via TodoWrite tool"

    elif tool_name == "Bash":
        cmd = tool_input.get("command", "")
        description = tool_input.get("description", "")

        if description and cmd:
            # Combine human description with full command for complete context
            return f"Claude executed bash command: {description}. Full command: {cmd}"
        elif description:
            return f"Claude executed bash command: {description}"
        elif cmd:
            # Extract meaningful parts from command for clarity
            if "grep" in cmd.lower():
                return f"Claude searched files using grep to find patterns in codebase. Command: {cmd}"
            elif "git" in cmd.lower():
                return f"Claude ran git command to check repository status or history. Command: {cmd}"
            elif "date" in cmd.lower():
                return f"Claude got current timestamp to track time in session. Command: {cmd}"
            elif "find" in cmd.lower() or "fd" in cmd.lower():
                return f"Claude searched for files by name or pattern in directory tree. Command: {cmd}"
            else:
                return f"Claude executed bash command: {cmd}"
        return "Claude executed bash command without context"

    elif tool_name == "Read":
        filepath = tool_input.get("file_path", "")
        if filepath:
            filename = os.path.basename(filepath)
            rel_path = filepath.replace(os.path.expanduser("~"), "~")
            # Infer purpose from file type
            if filename.endswith(('.py', '.js', '.ts', '.tsx', '.jsx')):
                return f"Claude read source code file '{filename}' at {rel_path} to understand implementation details and logic"
            elif filename.endswith(('.md', '.txt', '.rst')):
                return f"Claude read documentation file '{filename}' at {rel_path} to gather context and understand requirements or instructions"
            elif filename.endswith(('.json', '.yaml', '.yml', '.toml')):
                return f"Claude read configuration file '{filename}' at {rel_path} to understand system settings and configuration"
            elif filename.endswith(('.sh', '.bash')):
                return f"Claude read shell script '{filename}' at {rel_path} to understand automation and command workflows"
            else:
                return f"Claude read file '{filename}' at {rel_path} to gather information for the current task"
        return "Claude read a file (path not provided)"

    elif tool_name == "Edit":
        filepath = tool_input.get("file_path", "")
        old_string = tool_input.get("old_string", "")
        new_string = tool_input.get("new_string", "")
        if filepath:
            filename = os.path.basename(filepath)
            rel_path = filepath.replace(os.path.expanduser("~"), "~")
            if old_string and new_string:
                # Show full context of the change
                old_preview = old_string[:200].replace("\n", " ").strip()
                new_preview = new_string[:200].replace("\n", " ").strip()
                return f"Claude edited '{filename}' at {rel_path}. Changed: '{old_preview}...' to: '{new_preview}...' to implement requested modifications"
            return f"Claude edited file '{filename}' at {rel_path} to make code changes"
        return "Claude edited a file (path not provided)"

    elif tool_name == "Write":
        filepath = tool_input.get("file_path", "")
        content = tool_input.get("content", "")
        if filepath:
            filename = os.path.basename(filepath)
            rel_path = filepath.replace(os.path.expanduser("~"), "~")
            if content:
                lines = len(content.split("\n"))
                chars = len(content)
                # Show first few lines for context
                first_lines = "\n".join(content.split("\n")[:3])[:200]
                return f"Claude created new file '{filename}' at {rel_path} with {lines} lines ({chars} chars). Content starts: {first_lines}..."
            return f"Claude created new file '{filename}' at {rel_path}"
        return "Claude wrote a new file (path not provided)"

    elif tool_name == "Grep":
        pattern = tool_input.get("pattern", "")
        path = tool_input.get("path", "")
        output_mode = tool_input.get("output_mode", "files_with_matches")

        search_type = "files matching" if output_mode == "files_with_matches" else "content matching"
        location = f"in {path}" if path else "in codebase"
        return f"Claude searched for {search_type} pattern '{pattern}' {location} using Grep tool to find relevant code or documentation."

    elif tool_name == "Glob":
        pattern = tool_input.get("pattern", "")
        path = tool_input.get("path", "")
        location = f"in {path}" if path else "in current directory"
        return f"Claude listed files matching glob pattern '{pattern}' {location} using Glob tool to discover files."

    elif tool_name == "Task":
        description = tool_input.get("description", "")
        subagent_type = tool_input.get("subagent_type", "")
        if description and subagent_type:
            return f"Claude launched {subagent_type} agent to handle task: {description[:120]}"
        return "Claude launched autonomous agent to handle complex task"

    else:
        # Generic fallback with tool name
        return f"Claude used {tool_name} tool to perform operation. Additional context not available."


def update_time_log(
    log_file, project_name, session_id, activity, include_project_column=False
):
    """Update a time log file"""
    if not should_create_new_entry(log_file):
        return False  # No update needed

    block = get_current_block()
    timestamp = format_timestamp()

    # DO NOT truncate - let terminal handle line wrapping
    # Long descriptive entries are valuable for time tracking

    # Read existing content
    if os.path.exists(log_file):
        with open(log_file, "r") as f:
            content = f.read()
    else:
        # Create new file with headers
        today = datetime.now().strftime("%Y-%m-%d")
        if include_project_column:
            content = f"# Claude Activity Log\n\nClaude's automatic tool usage tracking with full context.\n\n## {today}\n\nDate Time Window | Updated At | Project | Session ID | Activity\n-----------------|------------|---------|------------|----------\n"
        else:
            content = f"# Claude Activity Log\n\n## {today}\n\nDate Time Window | Updated At | Session ID | Activity\n-----------------|------------|------------|----------\n"

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
    # Sanitize activity to prevent TSV format corruption
    activity = activity.replace('|', '\\|').replace('\n', ' ').replace('\r', '')

    if include_project_column:
        new_entry = (
            f"{block} | {timestamp} | {project_name} | {session_id} | {activity}"
        )
    else:
        new_entry = f"{block} | {timestamp} | {session_id} | {activity}"

    # Append new entry at end (chronological order - oldest at top, newest at bottom)
    lines.append(new_entry)

    # Write back
    with open(log_file, "w") as f:
        f.write("\n".join(lines))

    return True


def main():
    try:
        # Read hook data from stdin
        hook_data = json.load(sys.stdin)

        project_root = find_project_root()
        project_name = get_project_name()
        # Get session_id from hook data (Claude Code provides this)
        hook_session_id = hook_data.get("session_id", "")
        session_id = get_session_id(hook_session_id)
        activity = determine_activity(hook_data)

        # Update CLAUDE's activity log (daily rotation)
        # Use daily file: CONSCIOUSNESS/time-logs/agent/YYYY-MM/YYYY-MM-DD.md
        today = datetime.now()
        year_month = today.strftime("%Y-%m")
        date_str = today.strftime("%Y-%m-%d")

        time_logs_dir = os.path.join(project_root, "CONSCIOUSNESS", "time-logs", "agent", year_month)
        os.makedirs(time_logs_dir, exist_ok=True)

        claude_log = os.path.join(time_logs_dir, f"{date_str}.md")

        claude_updated = update_time_log(
            claude_log, project_name, session_id, activity, include_project_column=False
        )

        if claude_updated:
            print(
                f"[CLAUDE ACTIVITY] Logged activity for block {get_current_block()}",
                file=sys.stderr,
            )

        return 0

    except Exception as e:
        print(f"Error updating Claude activity log: {e}", file=sys.stderr)
        import traceback

        traceback.print_exc(file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
