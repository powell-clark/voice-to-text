#!/usr/bin/env python3
"""
Multi-session TODO sync - handles concurrent Claude sessions.
Each session has own section, tasks can be completed cross-session.

Uses optimistic locking to prevent lost completions when multiple
sessions update TODO.md simultaneously.
"""

import sys
import json
import os
from datetime import datetime, timedelta
from pathlib import Path

# Import version manager for optimistic locking
try:
    # Try relative import from utils/
    sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))
    from utils.version_manager import VersionManager
except ImportError:
    # Fallback: optimistic locking not available
    VersionManager = None


def find_project_root():
    """Find project root by looking for .claude directory"""
    current_dir = os.getcwd()
    while current_dir != "/":
        if os.path.exists(os.path.join(current_dir, ".claude")):
            return current_dir
        current_dir = os.path.dirname(current_dir)
    return os.getcwd()


def get_claude_conversation_id():
    """Get Claude Code conversation UUID from debug logs"""
    debug_dir = os.path.expanduser("~/.claude/debug")
    if not os.path.exists(debug_dir):
        return None

    try:
        # Find most recent debug log (filename is the conversation UUID)
        import glob

        debug_files = glob.glob(os.path.join(debug_dir, "*.txt"))
        if not debug_files:
            return None

        latest_file = max(debug_files, key=os.path.getmtime)
        # Extract UUID from filename
        uuid = os.path.basename(latest_file).replace(".txt", "")
        # Return first 8 chars
        return uuid[:8]
    except Exception:
        return None


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
            # Sanitize branch name for session ID - allow only alphanumeric, dash, underscore
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


def get_session_id():
    """Get session ID using real Claude conversation UUID"""
    project_root = find_project_root()

    # Get Claude conversation UUID (required)
    conv_id = get_claude_conversation_id()
    if not conv_id:
        return "unknown"

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


def parse_multi_session_todo_content(content):
    """Parse TODO.md content with multiple session sections"""
    sessions = {}
    completed = []
    current_session = None

    if not content:
        return sessions, completed

    try:
        lines = content.splitlines(keepends=True)

        i = 0
        while i < len(lines):
            line = lines[i].strip()

            # Session header
            if line.startswith("## Session:"):
                session_id = line.split("Session:")[1].strip().split()[0]
                current_session = session_id
                sessions[session_id] = {
                    "tasks": [],
                    "context": "",
                    "started": "",
                    "last_active": "",
                }
                i += 1
                continue

            # Parse session metadata
            if current_session and line.startswith("**"):
                if "Started:" in line:
                    sessions[current_session]["started"] = (
                        line.split("Started:")[1].strip().strip("**").strip()
                    )
                elif "Last Active:" in line:
                    sessions[current_session]["last_active"] = (
                        line.split("Last Active:")[1].strip().strip("**").strip()
                    )
                elif "Working On:" in line:
                    sessions[current_session]["context"] = (
                        line.split("Working On:")[1].strip().strip("**").strip()
                    )
                i += 1
                continue

            # Tasks under current session
            if current_session and line.startswith("-"):
                task = line.lstrip("-").strip()
                if task.startswith("[IN_PROGRESS]"):
                    task = task[13:].strip()  # len("[IN_PROGRESS]") = 13
                if task and task != "None":
                    sessions[current_session]["tasks"].append(task)
                i += 1
                continue

            # Completed tasks section
            if line.startswith("## Recently Completed"):
                i += 1
                while i < len(lines):
                    line = lines[i].strip()
                    if line.startswith("---") or line.startswith("##"):
                        break
                    if line.startswith("-") and "[DONE]" in line:
                        completed.append(line.lstrip("-").strip())
                    i += 1
                continue

            i += 1

    except Exception:
        pass

    return sessions, completed


def parse_multi_session_todo(todo_file):
    """Parse TODO.md file with multiple session sections"""
    if not os.path.exists(todo_file):
        return {}, []

    try:
        with open(todo_file, "r") as f:
            content = f.read()
        return parse_multi_session_todo_content(content)
    except IOError:
        return {}, []


def format_timestamp(dt=None):
    """Format timestamp consistently"""
    if dt is None:
        dt = datetime.now()
    return dt.strftime("%Y-%m-%d %H:%M:%S GMT")


def is_stale_session(last_active_str, hours=2):
    """Check if session is stale (inactive for >hours)"""
    try:
        # Parse the timestamp
        last_active = datetime.strptime(last_active_str, "%Y-%m-%d %H:%M:%S GMT")
        age = datetime.now() - last_active
        return age > timedelta(hours=hours)
    except (ValueError, TypeError):
        return False


def parse_task_with_criteria(task_content):
    """Parse task content into description, story, and acceptance criteria.

    Format: "Task description | Story: STORY-### | Success: acceptance criteria"
    Returns: (description, story, criteria) tuple
    """
    story = None
    criteria = None
    description = task_content.strip()

    # Check for Story reference
    if " | Story: " in task_content:
        parts = task_content.split(" | Story: ", 1)
        description = parts[0].strip()
        remaining = parts[1]

        # Extract story and check for Success
        if " | Success: " in remaining:
            story_parts = remaining.split(" | Success: ", 1)
            story = story_parts[0].strip()
            criteria = story_parts[1].strip()
        else:
            story = remaining.strip()
    elif " | Success: " in task_content:
        # Legacy format without story
        parts = task_content.split(" | Success: ", 1)
        description = parts[0].strip()
        criteria = parts[1].strip()

    return description, story, criteria


def extract_context_from_tasks(tasks):
    """Extract high-level context from task list"""
    if not tasks:
        return "No active tasks"

    # Use first task as context (usually the main goal)
    first_task = tasks[0].replace("[IN_PROGRESS]", "").strip()
    # Remove story and acceptance criteria from context display
    desc, _, _ = parse_task_with_criteria(first_task)
    if len(desc) > 50:
        return desc[:47] + "..."
    return desc


def build_todo_content(sessions, completed_tasks):
    """Build TODO.md content from sessions and completed tasks"""
    content = "# TODO\n\n"

    # Separate active and stale sessions
    active_sessions = []
    stale_sessions = []

    for sid, sess in sessions.items():
        if is_stale_session(sess.get("last_active", "")):
            stale_sessions.append((sid, sess))
        else:
            active_sessions.append((sid, sess))

    # Write active sessions first
    for sid, sess in active_sessions:
        # Determine status for this session
        is_current = sess.get("is_current", False)
        status = "Active (This Session)" if is_current else "Active"

        # Extract time from last_active
        last_active_parts = sess.get("last_active", "").split()
        last_update = last_active_parts[1] if len(last_active_parts) > 1 else sess.get("last_active", "")

        content += f"## Session: {sid} ({status} - {last_update})\n"
        content += f"**Started:** {sess.get('started', '')}\n"
        content += f"**Last Active:** {sess.get('last_active', '')}\n"
        content += f"**Working On:** {sess.get('context', 'No active tasks')}\n\n"

        if sess.get("tasks"):
            content += "### Tasks\n"
            for task in sess["tasks"]:
                desc, story, criteria = parse_task_with_criteria(task)
                content += f"- {desc}\n"
                if story:
                    content += f"  **Story:** {story}\n"
                if criteria:
                    content += f"  **Success:** {criteria}\n"
        else:
            content += "### Tasks\n- None\n"

        content += "\n---\n\n"

    # Write stale sessions
    for sid, sess in stale_sessions:
        content += f"## Session: {sid} (Stale - inactive {sess.get('last_active', '')})\n"
        content += f"**Working On:** {sess.get('context', 'No active tasks')}\n\n"

        if sess.get("tasks"):
            content += "### Tasks\n"
            for task in sess["tasks"]:
                desc, story, criteria = parse_task_with_criteria(task)
                content += f"- ⚠️ {desc} (Session expired - review before continuing)\n"
                if story:
                    content += f"  **Story:** {story}\n"
                if criteria:
                    content += f"  **Success:** {criteria}\n"
        else:
            content += "### Tasks\n- None\n"

        content += "\n---\n\n"

    # Completed tasks section
    content += "## Recently Completed (Last 24h)\n"
    if completed_tasks:
        for task in completed_tasks:
            content += f"- {task}\n"
    else:
        content += "- None\n"

    content += f"\n---\n\n**Last Updated:** {format_timestamp()}\n"

    return content


def main():
    try:
        # Read hook data from stdin
        hook_data = json.load(sys.stdin)

        # Extract todos from tool_input
        if "tool_input" not in hook_data or "todos" not in hook_data["tool_input"]:
            print("No todos found in hook data", file=sys.stderr)
            return 0

        todos = hook_data["tool_input"]["todos"]

        # Get project root and files
        project_root = find_project_root()

        # Standard TODO filename (Claude Code convention)
        if is_home_directory(project_root):
            # Home directory: CONSCIOUSNESS/TODO-MASTER.md (semantic sense)
            consciousness_todo = os.path.join(project_root, "CONSCIOUSNESS", "TODO-MASTER.md")
            old_todo = os.path.join(project_root, "TODO-MASTER.md")
            # Use CONSCIOUSNESS path if it exists, otherwise fall back to old path
            todo_file = consciousness_todo if os.path.exists(consciousness_todo) else old_todo
        else:
            # Project directory: TODO.md at root (TodoWrite native location)
            todo_file = os.path.join(project_root, "TODO.md")

        # Get current session ID
        session_id = get_session_id()

        # Prepare task lists for this session
        pending = [t for t in todos if t["status"] == "pending"]
        in_progress = [t for t in todos if t["status"] == "in_progress"]
        newly_completed = [t for t in todos if t["status"] == "completed"]

        # Build current session's task list
        current_tasks = []
        for todo in pending:
            current_tasks.append(todo["content"])
        for todo in in_progress:
            current_tasks.append(f"[IN_PROGRESS] {todo['content']}")

        # Try optimistic locking if available
        use_optimistic = False
        if VersionManager and os.path.exists(Path(project_root) / "CONSCIOUSNESS"):
            vm = VersionManager(Path(project_root) / "CONSCIOUSNESS")

            def update_fn(current_content):
                """Update function for optimistic locking"""
                # Parse existing TODO.md content
                sessions, completed_tasks = parse_multi_session_todo_content(current_content)

                # Add newly completed tasks to global completed list
                now_str = datetime.now().strftime("%H:%M")
                for task in newly_completed:
                    completed_entry = f"[DONE] {task['content']} ({session_id} @ {now_str})"
                    if completed_entry not in completed_tasks:
                        completed_tasks.insert(0, completed_entry)  # Add at top

                # Keep only last 20 completed tasks
                completed_tasks = completed_tasks[:20]

                # Update or create current session
                if session_id not in sessions:
                    sessions[session_id] = {
                        "started": format_timestamp(),
                        "last_active": format_timestamp(),
                        "context": extract_context_from_tasks(current_tasks),
                        "tasks": current_tasks,
                        "is_current": True,
                    }
                else:
                    sessions[session_id]["last_active"] = format_timestamp()
                    sessions[session_id]["context"] = extract_context_from_tasks(current_tasks)
                    sessions[session_id]["tasks"] = current_tasks
                    sessions[session_id]["is_current"] = True

                # Mark all other sessions as not current
                for sid in sessions:
                    if sid != session_id:
                        sessions[sid]["is_current"] = False

                # Build new content
                return build_todo_content(sessions, completed_tasks)

            # Use optimistic update
            try:
                vm.optimistic_update("TODO", update_fn)
                print(f"[DONE] Synced session {session_id}: {len(current_tasks)} tasks (optimistic)", file=sys.stderr)
                use_optimistic = True
            except Exception as e:
                print(f"[WARNING] Optimistic update failed: {e}, falling back to direct write", file=sys.stderr)
                # Fall through to legacy approach

        # Legacy approach (no optimistic locking - direct file write)
        if not use_optimistic:
            # Parse existing TODO.md
            sessions, completed_tasks = parse_multi_session_todo(todo_file)

            # Add newly completed tasks to global completed list
            now_str = datetime.now().strftime("%H:%M")
            for task in newly_completed:
                completed_entry = f"[DONE] {task['content']} ({session_id} @ {now_str})"
                if completed_entry not in completed_tasks:
                    completed_tasks.insert(0, completed_entry)

            # Keep only last 20 completed tasks
            completed_tasks = completed_tasks[:20]

            # Update or create current session
            if session_id not in sessions:
                sessions[session_id] = {
                    "started": format_timestamp(),
                    "last_active": format_timestamp(),
                    "context": extract_context_from_tasks(current_tasks),
                    "tasks": current_tasks,
                    "is_current": True,
                }
            else:
                sessions[session_id]["last_active"] = format_timestamp()
                sessions[session_id]["context"] = extract_context_from_tasks(current_tasks)
                sessions[session_id]["tasks"] = current_tasks
                sessions[session_id]["is_current"] = True

            # Mark all other sessions as not current
            for sid in sessions:
                if sid != session_id:
                    sessions[sid]["is_current"] = False

            # Build TODO.md content
            content = build_todo_content(sessions, completed_tasks)

            # Write to TODO.md directly (no optimistic locking)
            with open(todo_file, "w") as f:
                f.write(content)

            print(f"[DONE] Synced session {session_id}: {len(current_tasks)} tasks (direct write)", file=sys.stderr)

        # Update sync time marker
        marker_file = os.path.join(project_root, ".claude", ".todo-sync-time")
        try:
            todo_mtime = os.path.getmtime(todo_file)
            with open(marker_file, "w") as f:
                f.write(str(todo_mtime))
        except (OSError, IOError):
            pass

        return 0

    except Exception as e:
        print(f"Error syncing todos: {e}", file=sys.stderr)
        import traceback

        traceback.print_exc(file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
