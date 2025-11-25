#!/usr/bin/env python3
"""
Orchestrator-integrated PostToolUse hook.
Triggers orchestrator on timestamp calls (date command) which coordinates all scheduled tasks.
"""

import sys
import json
import subprocess
from pathlib import Path


def main():
    """Main hook entry point"""
    try:
        # Read hook data from stdin
        hook_data = json.loads(sys.stdin.read())

        # Extract tool information
        tool_name = hook_data.get("tool", "")
        command = hook_data.get("parameters", {}).get("command", "")

        # Only trigger orchestrator on timestamp calls (date command)
        if tool_name == "Bash" and "date" in command and "+%Y-%m-%d %H:%M:%S" in command:
            # Call orchestrator
            orchestrator_path = Path.home() / ".claude/orchestrator/orchestrator.py"

            if orchestrator_path.exists():
                result = subprocess.run(
                    ["python3", str(orchestrator_path)],
                    capture_output=True,
                    text=True,
                    timeout=30
                )

                if result.returncode == 0:
                    # Parse orchestrator output
                    try:
                        output = json.loads(result.stdout)
                        actions = []

                        for check_name, check_result in output.items():
                            if check_result.get("ran"):
                                actions.append(check_name.replace("_", " ").title())

                        if actions:
                            print(f"Orchestrator: {', '.join(actions)}", file=sys.stderr)
                        else:
                            # Silent when nothing due
                            pass
                    except json.JSONDecodeError:
                        # Orchestrator output not JSON, skip
                        pass
                else:
                    print(f"Orchestrator error: {result.stderr}", file=sys.stderr)

        # Exit successfully
        sys.exit(0)

    except Exception as e:
        print(f"Hook error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
