#!/usr/bin/env python3
"""
Post-session hook: cleanup and commit changes.

Runs when session ends (rarely fires - usually only on explicit /exit).
- Closes Chrome DevTools browser instances
- Commits any uncommitted changes to tracking files
"""
import subprocess
import sys
import time
import os
from pathlib import Path


def find_project_root():
    """Find project root by looking for .claude directory"""
    current_dir = os.getcwd()
    while current_dir != "/":
        if os.path.exists(os.path.join(current_dir, ".claude")):
            return current_dir
        current_dir = os.path.dirname(current_dir)
    return os.getcwd()


def commit_session_changes():
    """Commit any uncommitted changes to tracking files"""
    try:
        project_root = find_project_root()

        # Check if there are uncommitted changes
        result = subprocess.run(
            ["git", "status", "--porcelain"],
            cwd=project_root,
            capture_output=True,
            text=True,
            timeout=5
        )

        if not result.stdout.strip():
            # No changes to commit
            return

        # Stage tracking files
        tracking_files = ["TASKS.md", "STORY.md", "EPICS.md", "ROADMAP.md", "TODO.md"]
        for file in tracking_files:
            file_path = os.path.join(project_root, "CONSCIOUSNESS", file)
            if os.path.exists(file_path):
                subprocess.run(
                    ["git", "add", file_path],
                    cwd=project_root,
                    capture_output=True,
                    timeout=5
                )

        # Commit with session end marker
        subprocess.run(
            ["git", "commit", "-m", "auto: Session end - commit tracking files\n\n[SessionEnd hook]"],
            cwd=project_root,
            capture_output=True,
            timeout=5
        )

        print("[SESSION END] Committed tracking file changes", file=sys.stderr)

    except Exception as e:
        print(f"[SESSION END] Commit failed: {e}", file=sys.stderr)


def cleanup_chrome_devtools():
    """Kill Chrome DevTools MCP browser instances."""
    try:
        # Kill Chrome DevTools processes
        result = subprocess.run(
            ["pkill", "-f", "chrome.*chrome-devtools-mcp"],
            capture_output=True,
            timeout=5,
        )

        # Give processes time to terminate gracefully
        time.sleep(1)

        # Check if any processes were killed
        if result.returncode == 0:
            print("[DONE] Closed Chrome DevTools browser", file=sys.stderr)
        else:
            # Exit code 1 means no processes matched (which is fine)
            pass

    except subprocess.TimeoutExpired:
        print("[WARNING] Timeout while closing Chrome DevTools", file=sys.stderr)
    except Exception as e:
        print(f"[WARNING] Error closing Chrome DevTools: {e}", file=sys.stderr)


if __name__ == "__main__":
    commit_session_changes()
    cleanup_chrome_devtools()
