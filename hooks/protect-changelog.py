#!/usr/bin/env python3
"""
Safety hook: Protects against dangerous operations

Blocks:
1. CHANGE-LOG.md writes (append-only file)
2. Dangerous bash commands (when safe mode enabled):
   - System: rm -rf /, sudo rm, dd, mkfs, fork bombs
   - Filesystem: chmod -R 777, rm -rf $HOME
   - Git: push --force, reset --hard, clean -f, rm -rf .git

Safe Mode:
- Enabled by default (CLAUDE_SAFE_MODE=1)
- Disable: export CLAUDE_SAFE_MODE=0
- When disabled, only CHANGE-LOG.md protection remains active

Returns JSON: {"decision": "allow"|"block", "reason": "explanation"}
"""

import sys
import json
import re
import os


def check_dangerous_command(command):
    """Check if bash command contains dangerous patterns"""
    dangerous_patterns = [
        (r'rm\s+.*-rf\s+/', "Destructive recursive delete of root directory"),
        (r'sudo\s+rm', "Sudo with rm command - potential system damage"),
        (r'\bdd\b.*if=/dev/', "dd with block device - can destroy data"),
        (r'mkfs\.', "Filesystem formatting - destroys data"),
        (r':\(\)\{.*:\|:.*\};:', "Fork bomb - will crash system"),
        (r'rm\s+.*-rf.*\$HOME', "Recursive delete of home directory"),
        (r'chmod\s+-R\s+777\s+/', "Recursive 777 permissions on root - security risk"),
        (r'git\s+push\s+.*--force', "Force push can overwrite remote history"),
        (r'git\s+reset\s+--hard\s+HEAD~', "Hard reset loses commits permanently"),
        (r'git\s+clean\s+-[dfx]*f', "Git clean -f deletes untracked files permanently"),
        (r'rm\s+-rf\s+\.git', "Deleting .git directory loses all version history"),
    ]

    for pattern, reason in dangerous_patterns:
        if re.search(pattern, command, re.IGNORECASE):
            return True, reason

    return False, None


def is_safe_mode_enabled():
    """Check if safe mode is enabled (default: True)"""
    # Check environment variable
    safe_mode = os.getenv("CLAUDE_SAFE_MODE", "1")
    return safe_mode != "0"


def main():
    try:
        # Read hook data from stdin
        hook_data = json.load(sys.stdin)

        # Get tool information
        tool_name = hook_data.get("tool_name", "")
        tool_input = hook_data.get("tool_input", {})

        # Check if safe mode is enabled
        safe_mode = is_safe_mode_enabled()

        # Check for dangerous Bash commands
        if tool_name == "Bash" and safe_mode:
            command = tool_input.get("command", "")
            is_dangerous, danger_reason = check_dangerous_command(command)

            if is_dangerous:
                print("=" * 70)
                print("❌ BLOCKED: Dangerous command detected")
                print("=" * 70)
                print("")
                print(f"Reason: {danger_reason}")
                print("")
                print(f"Command: {command[:100]}")
                print("")
                print("This command has been blocked for safety.")
                print("If you need to run this, do it manually outside Claude.")
                print("=" * 70)

                response = {
                    "decision": "block",
                    "reason": f"Dangerous command blocked: {danger_reason}"
                }
                print(json.dumps(response), file=sys.stderr)
                return 1

        # Check if targeting CHANGE-LOG.md
        file_path = tool_input.get("file_path", "")

        if "CHANGE-LOG.md" in file_path or "change-log.md" in file_path.lower():

            if tool_name == "Write":
                print("=" * 70)
                print("❌ BLOCKED: Cannot use Write tool on CHANGE-LOG.md")
                print("=" * 70)
                print("")
                print("CHANGE-LOG.md is append-only to preserve history.")
                print("")
                print("Use instead:")
                print("  python3 .claude/scripts/changelog.py add <entry-file>")
                print("")
                print("This ensures safe prepending without losing existing entries.")
                print("=" * 70)

                # Return JSON response
                response = {
                    "decision": "block",
                    "reason": "CHANGE-LOG.md is append-only; use changelog.py script instead of Write tool"
                }
                print(json.dumps(response), file=sys.stderr)
                return 1

            elif tool_name == "Edit":
                print("=" * 70)
                print("[WARNING] WARNING: Editing CHANGE-LOG.md directly")
                print("=" * 70)
                print("")
                print("You're about to edit CHANGE-LOG.md manually.")
                print("Make sure you're not accidentally overwriting entries!")
                print("")
                print("Recommended: Use changelog.py script instead:")
                print("  python3 .claude/scripts/changelog.py add <entry-file>")
                print("")
                print("Proceeding with Edit (use carefully)...")
                print("=" * 70)

                # Return JSON response (allow with warning)
                response = {
                    "decision": "allow",
                    "reason": "Edit allowed with warning - CHANGE-LOG.md should use changelog.py script"
                }
                print(json.dumps(response), file=sys.stderr)
                return 0

        # Default allow
        response = {
            "decision": "allow",
            "reason": "No CHANGE-LOG.md protection needed"
        }
        print(json.dumps(response), file=sys.stderr)
        return 0

    except Exception as e:
        # Error in hook - allow operation but log error
        response = {
            "decision": "allow",
            "reason": f"Hook error (allowing by default): {e}"
        }
        print(json.dumps(response), file=sys.stderr)
        return 0


if __name__ == "__main__":
    sys.exit(main())
