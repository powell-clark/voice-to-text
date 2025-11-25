#!/usr/bin/env python3
"""
Capture user messages for activity tracking context.
Runs on UserPromptSubmit to record what Emmanuel asked Claude to do.
"""

import sys
import json
from pathlib import Path
from datetime import datetime


def get_context_file(session_id):
    """Get path to running context file for this session"""
    context_dir = Path.home() / ".claude" / "memory" / "time-tracking"
    context_dir.mkdir(parents=True, exist_ok=True)
    return context_dir / f"context-{session_id}.json"


def load_context(session_id):
    """Load running activity context"""
    context_file = get_context_file(session_id)
    if not context_file.exists():
        return {
            "current_block": "",
            "block_start_time": datetime.now().isoformat(),
            "user_messages": [],
            "activities": [],
            "tools_used": [],
            "state_changes": []
        }
    
    try:
        with open(context_file) as f:
            content = f.read().strip()
            if not content:
                return {
                    "current_block": "",
                    "block_start_time": datetime.now().isoformat(),
                    "user_messages": [],
                    "activities": [],
                    "tools_used": [],
                    "state_changes": []
                }
            return json.loads(content)
    except (json.JSONDecodeError, IOError):
        return {
            "current_block": "",
            "block_start_time": datetime.now().isoformat(),
            "user_messages": [],
            "activities": [],
            "tools_used": [],
            "state_changes": []
        }


def save_context(context, session_id):
    """Save running activity context"""
    context_file = get_context_file(session_id)
    with open(context_file, 'w') as f:
        json.dump(context, f, indent=2)


def main():
    try:
        # Read hook data from stdin
        hook_data = json.load(sys.stdin)
        
        # Get session ID
        session_id = hook_data.get("session_id", "")[:8]
        if not session_id:
            return 0
        
        # Get user message
        user_message = hook_data.get("user_message", "").strip()
        if not user_message:
            return 0

        # Filter out Claude Code UI hook output from message
        lines = user_message.split("\n")
        cleaned_lines = []
        for line in lines:
            # Skip hook success/failure messages
            if "⎿" in line or "hook succeeded" in line.lower() or "hook failed" in line.lower():
                continue
            # Skip empty lines
            if not line.strip():
                continue
            cleaned_lines.append(line)

        user_message = "\n".join(cleaned_lines).strip()

        # Skip system messages and very short messages
        if len(user_message) < 3:
            return 0
        
        # Load context
        context = load_context(session_id)
        
        # Add user message (keep last 5 messages for context)
        if "user_messages" not in context:
            context["user_messages"] = []
        
        context["user_messages"].append(user_message)
        context["user_messages"] = context["user_messages"][-5:]  # Keep last 5
        
        # Save updated context
        save_context(context, session_id)
        
        return 0
        
    except Exception as e:
        print(f"Error capturing user message: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
