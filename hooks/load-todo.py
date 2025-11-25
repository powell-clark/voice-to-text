#!/usr/bin/env python3
"""
Load TODO.md at session start and populate TodoWrite with multi-session support.
Auto-summarizes other active sessions and detects potential conflicts.
"""

import os
import sys
import json
from datetime import datetime, timedelta


def find_project_root():
    """Find project root by looking for .claude directory"""
    current_dir = os.getcwd()
    while current_dir != "/":
        if os.path.exists(os.path.join(current_dir, ".claude")):
            return current_dir
        current_dir = os.path.dirname(current_dir)
    return os.getcwd()


def get_session_id():
    """Get current session ID from .session-metadata.json"""
    project_root = find_project_root()
    metadata_file = os.path.join(project_root, ".claude", ".session-metadata.json")

    if os.path.exists(metadata_file):
        try:
            with open(metadata_file, "r") as f:
                metadata = json.load(f)
                session_id = metadata.get("current_session")
                if session_id:
                    return session_id
        except (IOError, json.JSONDecodeError):
            pass

    # Fallback if metadata doesn't exist (shouldn't happen with SessionStart hook)
    return "unknown-session"


def is_stale_session(last_active_str, hours=2):
    """Check if session is stale (inactive for >hours)"""
    try:
        last_active = datetime.strptime(last_active_str, "%Y-%m-%d %H:%M:%S GMT")
        age = datetime.now() - last_active
        return age > timedelta(hours=hours)
    except (ValueError, TypeError):
        return False


def parse_multi_session_todo(todo_file):
    """Parse TODO.md with multiple session sections"""
    sessions = {}
    completed = []
    current_session = None

    if not os.path.exists(todo_file):
        return sessions, completed

    try:
        with open(todo_file, "r") as f:
            lines = f.readlines()

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
                    "is_stale": False,
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
                    last_active = (
                        line.split("Last Active:")[1].strip().strip("**").strip()
                    )
                    sessions[current_session]["last_active"] = last_active
                    sessions[current_session]["is_stale"] = is_stale_session(
                        last_active
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
                is_in_progress = False

                if task.startswith("[IN_PROGRESS]"):
                    is_in_progress = True
                    task = task[13:].strip()  # len("[IN_PROGRESS]") = 13

                # Skip stale session warning prefix
                if task.startswith("[WARNING]"):
                    task = task[9:].strip()  # len("[WARNING]") = 9
                    if task.startswith("(Session expired"):
                        # Skip the warning suffix
                        continue

                if task and task != "None" and not task.startswith("(Session expired"):
                    sessions[current_session]["tasks"].append(
                        {
                            "content": task,
                            "status": "in_progress" if is_in_progress else "pending",
                        }
                    )
                i += 1
                continue

            # Completed tasks section
            if line.startswith("## Recently Completed"):
                i += 1
                while i < len(lines):
                    line = lines[i].strip()
                    if line.startswith("---") or line.startswith("##"):
                        break
                    if line.startswith("-") and "✅" in line:
                        completed.append(line.lstrip("-").strip())
                    i += 1
                continue

            i += 1

    except IOError:
        pass

    return sessions, completed


def is_home_directory(project_root):
    """Check if we're in home directory (not a project repo)"""
    home = os.path.expanduser("~")
    return project_root == home


def find_todo_file(project_root):
    """Find TODO file using standard Claude Code conventions"""
    if is_home_directory(project_root):
        # Home directory: TODO-MASTER.md (semantic sense)
        return os.path.join(project_root, "TODO-MASTER.md")
    else:
        # Project directory: TODO.md (standard)
        return os.path.join(project_root, "TODO.md")


def main():
    project_root = find_project_root()
    todo_file = find_todo_file(project_root)
    session_id = get_session_id()

    sessions, completed = parse_multi_session_todo(todo_file)

    # Get current session's tasks
    current_tasks = sessions.get(session_id, {}).get("tasks", [])

    # Get other active sessions (non-stale)
    other_active = {
        sid: data
        for sid, data in sessions.items()
        if sid != session_id and not data.get("is_stale", False)
    }

    # Output summary
    if current_tasks:
        print(f"\n{'=' * 60}")
        print(f"[TODO] Session: {session_id}")
        print(f"{'=' * 60}\n")

        print(f"Found {len(current_tasks)} task(s) to load:\n")
        for task in current_tasks:
            status_icon = "[IN_PROGRESS]" if task["status"] == "in_progress" else "-"
            print(f"  {status_icon} {task['content']}")
        print()
    else:
        print(f"\n[TODO] Session {session_id} - starting fresh (no tasks)\n")

    # Auto-summarize other active sessions (if 2+ sessions exist)
    if len(other_active) > 0:
        print(f"{'=' * 60}")
        print(f"👥 Other Active Sessions ({len(other_active)})")
        print(f"{'=' * 60}\n")

        for sid, data in other_active.items():
            task_count = len(data["tasks"])
            context = data["context"] or "No tasks"
            last_active = (
                data["last_active"].split()[1] if data["last_active"] else "Unknown"
            )

            print(f"  - {sid} ({task_count} tasks, last active: {last_active})")
            print(f"    Working on: {context}\n")

        print(f"{'=' * 60}\n")

    # Show recent completions
    if completed:
        print(f"[DONE] Recently Completed ({len(completed)}):\n")
        for task in completed[:5]:  # Show last 5
            print(f"  - {task}")
        if len(completed) > 5:
            print(f"  ... and {len(completed) - 5} more")
        print()

    return 0


if __name__ == "__main__":
    sys.exit(main())
