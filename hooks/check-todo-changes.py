#!/usr/bin/env python3
"""
Check if TODO.md has been modified since last sync
Runs before each user prompt is processed (UserPromptSubmit hook)
"""

import os


def find_project_root():
    """Find project root by looking for .claude directory"""
    current_dir = os.getcwd()
    while current_dir != "/":
        if os.path.exists(os.path.join(current_dir, ".claude")):
            return current_dir
        current_dir = os.path.dirname(current_dir)
    return os.getcwd()


def get_last_sync_time():
    """Read last sync time from sync marker file"""
    project_root = find_project_root()
    marker_file = os.path.join(project_root, ".claude", ".todo-sync-time")

    if os.path.exists(marker_file):
        try:
            with open(marker_file, "r") as f:
                return float(f.read().strip())
        except (ValueError, IOError):
            return 0
    return 0


def update_sync_time(mtime):
    """Update sync marker file with new time"""
    project_root = find_project_root()
    marker_file = os.path.join(project_root, ".claude", ".todo-sync-time")

    try:
        with open(marker_file, "w") as f:
            f.write(str(mtime))
    except IOError:
        pass


def main():
    project_root = find_project_root()
    todo_file = os.path.join(project_root, "TODO.md")

    if not os.path.exists(todo_file):
        return 0

    # Get TODO.md modification time
    todo_mtime = os.path.getmtime(todo_file)
    last_sync = get_last_sync_time()

    # If TODO.md is newer than last sync, warn Claude
    if todo_mtime > last_sync:
        # Parse TODO.md to see what tasks exist
        tasks = []
        try:
            with open(todo_file, "r") as f:
                in_pending_section = False
                for line in f:
                    if "## Pending Tasks" in line:
                        in_pending_section = True
                        continue
                    if in_pending_section and line.startswith("---"):
                        break
                    if in_pending_section and line.strip().startswith("-"):
                        task = line.strip().lstrip("-").strip()
                        if task.startswith(""):
                            task = task[2:].strip()
                        if task and task != "None":
                            tasks.append(task)
        except IOError:
            pass

        if tasks:
            print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
            print("[TODO] TODO.md CHANGE DETECTED")
            print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
            print(f"TODO.md has {len(tasks)} task(s) and was modified since last sync.")
            print("\nCurrent tasks in TODO.md:")
            for i, task in enumerate(tasks, 1):
                print(f"  {i}. {task}")
            print(
                "\n[WARNING] REMINDER: Load these tasks into TodoWrite to keep in sync"
            )
            print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n")

        # Update sync time marker
        update_sync_time(todo_mtime)

    return 0


if __name__ == "__main__":
    exit(main())
