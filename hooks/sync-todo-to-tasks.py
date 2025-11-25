#!/usr/bin/env python3
"""
Sync TodoWrite completions to TASKS.md.

Rules:
1. TodoWrite completion + Story reference + task doesn't exist → CREATE task as done
2. TodoWrite completion + matching task exists → MARK task done
3. TodoWrite in-progress + task exists → MARK task in-progress
4. NEVER delete tasks (preserve strategic plan)

Hook runs AFTER TodoWrite, not TodoWrite directly (prevents file corruption).
"""

import sys
import json
import os
import re


def find_project_root():
    """Find project root by looking for .claude directory"""
    current_dir = os.getcwd()
    while current_dir != "/":
        if os.path.exists(os.path.join(current_dir, ".claude")):
            return current_dir
        current_dir = os.path.dirname(current_dir)
    return os.getcwd()


def parse_todo_story_reference(todo_content):
    """Extract story reference from TodoWrite format: 'description | Story: STORY-XXX | Success: ...'"""
    match = re.search(r'\|\s*Story:\s*(STORY-[A-Z0-9]+)', todo_content)
    if match:
        return match.group(1)
    return None


def find_matching_task(tasks, story_id, todo_content):
    """Find task in TASKS.md matching this todo's story and description"""
    # Extract just the description part (before first |)
    desc_match = re.match(r'^([^|]+)', todo_content)
    if not desc_match:
        return None

    description = desc_match.group(1).strip().lower()

    # Find tasks with matching story
    for task in tasks:
        if task['story'] == story_id:
            # Check if description matches (fuzzy match on first 50 chars)
            task_title_start = task['title'][:50].lower()
            desc_start = description[:50]

            # If significant overlap, consider it a match
            if desc_start in task_title_start or task_title_start in desc_start:
                return task

    return None


def parse_tasks_md(tasks_file):
    """Parse TASKS.md into list of task dicts"""
    if not os.path.exists(tasks_file):
        return []

    tasks = []
    with open(tasks_file, 'r') as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith('#') or '|' not in line:
                continue

            # Skip header row
            if 'id|story|status|title' in line.lower():
                continue

            parts = [p.strip() for p in line.split('|')]
            if len(parts) >= 4:
                tasks.append({
                    'id': parts[0],
                    'story': parts[1],
                    'status': parts[2],
                    'title': parts[3],
                    'original_line': line
                })

    return tasks


def get_next_task_id():
    """Get next TASK-### ID using atomic counter"""
    import subprocess

    project_root = find_project_root()
    id_generator = os.path.join(project_root, "utils", "id-generator.py")

    if not os.path.exists(id_generator):
        # Fallback to old method if generator doesn't exist
        return None

    try:
        result = subprocess.run(
            ["python3", id_generator, "task"],
            capture_output=True,
            text=True,
            timeout=5
        )

        if result.returncode == 0:
            return result.stdout.strip()
        else:
            return None

    except Exception:
        return None


def create_task(tasks_file, story_id, title):
    """Create new task in TASKS.md with status=done"""
    # Get next ID using atomic counter
    task_id = get_next_task_id()

    if not task_id:
        # Fallback: parse existing tasks to get next ID with project scope
        tasks = parse_tasks_md(tasks_file)

        # Determine project scope from story ID (STORY-CC002 → CC)
        scope_match = re.match(r'STORY-([A-Z]+)\d+', story_id)
        scope = scope_match.group(1) if scope_match else 'CC'

        max_id = 0
        for task in tasks:
            # Match TASK-CC### format
            match = re.match(r'TASK-' + scope + r'(\d+)', task['id'])
            if match:
                task_num = int(match.group(1))
                if task_num > max_id:
                    max_id = task_num
        task_id = f"TASK-{scope}{max_id + 1:03d}"

    # Extract story number (CC002 from STORY-CC002)
    story_num = story_id.replace('STORY-', '')

    # Create new task line
    new_task = f"{task_id}|{story_num}|done|{title}"

    # Append to file
    with open(tasks_file, 'a') as f:
        f.write(new_task + '\n')

    return task_id


def update_task_status(tasks_file, task_id, new_status):
    """Update task status in TASKS.md"""
    if not os.path.exists(tasks_file):
        return False

    with open(tasks_file, 'r') as f:
        lines = f.readlines()

    updated = False
    for i, line in enumerate(lines):
        if line.strip().startswith(task_id + '|'):
            parts = [p.strip() for p in line.split('|')]
            if len(parts) >= 4:
                # Update status (3rd column)
                parts[2] = new_status
                lines[i] = '|'.join(parts) + '\n'
                updated = True
                break

    if updated:
        with open(tasks_file, 'w') as f:
            f.writelines(lines)

    return updated


def auto_commit_tasks(project_root, changes_description):
    """Auto-commit TASKS.md after changes"""
    import subprocess

    try:
        tasks_file = os.path.join(project_root, "CONSCIOUSNESS", "TASKS.md")

        # Stage TASKS.md
        subprocess.run(
            ["git", "add", tasks_file],
            cwd=project_root,
            capture_output=True,
            timeout=5
        )

        # Commit with structured message
        commit_msg = f"auto: {changes_description}\n\n[TodoWrite → TASKS.md sync]"
        subprocess.run(
            ["git", "commit", "-m", commit_msg],
            cwd=project_root,
            capture_output=True,
            timeout=5
        )

        print(f"[AUTO-COMMIT] {changes_description}", file=sys.stderr)
        return True

    except Exception as e:
        print(f"[AUTO-COMMIT FAILED] {e}", file=sys.stderr)
        return False


def main():
    try:
        # Read hook data from stdin
        hook_data = json.load(sys.stdin)

        # Extract todos from tool_input
        if "tool_input" not in hook_data or "todos" not in hook_data["tool_input"]:
            return 0

        todos = hook_data["tool_input"]["todos"]

        # Get project root
        project_root = find_project_root()
        tasks_file = os.path.join(project_root, "CONSCIOUSNESS", "TASKS.md")

        if not os.path.exists(tasks_file):
            # No TASKS.md, nothing to sync
            return 0

        # Parse existing tasks
        tasks = parse_tasks_md(tasks_file)

        # Track changes for commit message
        changes = []

        # Process completed todos with story references
        for todo in todos:
            if todo.get('status') != 'completed':
                continue

            content = todo.get('content', '')
            story_id = parse_todo_story_reference(content)

            if not story_id:
                continue  # No story reference, can't match to task

            # Extract title (description before first |)
            desc_match = re.match(r'^([^|]+)', content)
            if not desc_match:
                continue

            title = desc_match.group(1).strip()

            # Find matching task
            matching_task = find_matching_task(tasks, story_id, content)

            if matching_task:
                # Update existing task to done
                if matching_task['status'] != 'done':
                    updated = update_task_status(tasks_file, matching_task['id'], 'done')
                    if updated:
                        print(f"[TASK COMPLETED] {matching_task['id']}: {matching_task['title']}", file=sys.stderr)
                        changes.append(f"Completed {matching_task['id']}")
            else:
                # Create new task as done
                task_id = create_task(tasks_file, story_id, title)
                print(f"[TASK CREATED] {task_id}: {title}", file=sys.stderr)
                changes.append(f"Created {task_id}")
                # Re-parse tasks to include newly created one
                tasks = parse_tasks_md(tasks_file)

        # Process in-progress todos (mark tasks as in-progress)
        for todo in todos:
            if todo.get('status') != 'in_progress':
                continue

            content = todo.get('content', '')
            story_id = parse_todo_story_reference(content)

            if not story_id:
                continue

            matching_task = find_matching_task(tasks, story_id, content)

            if matching_task:
                if matching_task['status'] == 'planned':
                    updated = update_task_status(tasks_file, matching_task['id'], 'in-progress')
                    if updated:
                        print(f"[TASK STARTED] {matching_task['id']}: {matching_task['title']}", file=sys.stderr)
                        changes.append(f"Started {matching_task['id']}")

        # Auto-commit if any changes were made
        if changes:
            changes_desc = ", ".join(changes)
            auto_commit_tasks(project_root, changes_desc)

        return 0

    except Exception as e:
        print(f"Error syncing todos to tasks: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc(file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
