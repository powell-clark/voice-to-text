#!/usr/bin/env python3
"""
Protect Active ADRs from modification.

PreToolUse hook that blocks Write/Edit operations on ADR files marked as "Active".
Proposed ADRs can be edited freely.
Active ADRs require a new superseding ADR instead of direct modification.
"""

import json
import os
import re
import sys


def get_adr_status(file_path):
    """Read ADR file and extract status."""
    try:
        with open(file_path, 'r') as f:
            content = f.read()
            # Look for **Status:** line
            match = re.search(r'\*\*Status:\*\*\s*(\w+)', content)
            if match:
                return match.group(1).strip()
    except Exception:
        pass
    return None


def main():
    # Read hook input from stdin
    hook_input = json.loads(sys.stdin.read())

    tool_name = hook_input.get("toolName", "")
    params = hook_input.get("parameters", {})

    # Only check Write and Edit tools
    if tool_name not in ["Write", "Edit"]:
        sys.exit(0)

    # Get file path being modified
    file_path = params.get("file_path", "")

    # Check if this is an ADR file
    if not re.search(r'/CONSCIOUSNESS/adr/\d{4}-.*\.md$', file_path):
        sys.exit(0)

    # If file doesn't exist yet, allow creation (new ADR)
    if not os.path.exists(file_path):
        sys.exit(0)

    # Check if ADR is Active
    status = get_adr_status(file_path)

    if status == "Active":
        print(f"\n❌ Cannot modify Active ADR: {os.path.basename(file_path)}")
        print(f"Status: {status}")
        print("\nActive ADRs are immutable historical records.")
        print("To change this decision:")
        print("  1. Create a new ADR that supersedes this one")
        print("  2. Add '**Supersedes:** ADR-NNNN' to the new ADR")
        print("  3. Optionally update this ADR's status to 'Superseded'")
        print("\nTo edit draft ADRs:")
        print("  - Change status to 'Proposed' before editing")
        print("  - Or create the ADR with 'Proposed' status initially")
        sys.exit(1)

    # Allow modifications to Proposed, Superseded, Deprecated ADRs
    sys.exit(0)


if __name__ == "__main__":
    main()
