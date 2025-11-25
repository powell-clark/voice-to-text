#!/usr/bin/env python3
"""
Read session ID from .session-metadata.json for multi-session TODO management.
No longer creates .current-session-id file (broken for parallel sessions).
"""

import os
import sys
import json


def find_project_root():
    """Find project root by looking for .claude directory"""
    current_dir = os.getcwd()
    while current_dir != "/":
        if os.path.exists(os.path.join(current_dir, ".claude")):
            return current_dir
        current_dir = os.path.dirname(current_dir)
    return os.getcwd()


def get_session_id():
    """Read session ID from .session-metadata.json (created by session-start.sh hook)"""
    project_root = find_project_root()
    metadata_file = os.path.join(project_root, ".claude", ".session-metadata.json")

    # Read from metadata file
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


def main():
    session_id = get_session_id()
    print(f"[TODO] Session ID: {session_id}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
