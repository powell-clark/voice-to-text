#!/usr/bin/env python3
"""
Error Logger - Centralized hook error tracking

Writes all hook errors to CONSCIOUSNESS/.errors.log for debugging.
Enables fixing issues in consciousness repo and propagating fixes to all projects.
"""

import os
import json
from datetime import datetime
from pathlib import Path


def find_project_root():
    """Find project root by looking for .claude directory"""
    current_dir = os.getcwd()
    while current_dir != "/":
        if os.path.exists(os.path.join(current_dir, ".claude")):
            return current_dir
        current_dir = os.path.dirname(current_dir)
    return os.getcwd()


def log_hook_error(hook_name, error, context=None):
    """
    Log hook error to CONSCIOUSNESS/.errors.log

    Args:
        hook_name: Name of the hook that failed (e.g., 'update-time-log.py')
        error: Exception object or error message
        context: Optional dict with additional context (tool, session, etc.)
    """
    try:
        project_root = find_project_root()
        error_log = os.path.join(project_root, "CONSCIOUSNESS", ".errors.log")

        # Ensure CONSCIOUSNESS directory exists
        os.makedirs(os.path.dirname(error_log), exist_ok=True)

        # Build error entry
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        project = os.path.basename(project_root)

        error_entry = {
            "timestamp": timestamp,
            "project": project,
            "hook": hook_name,
            "error": str(error),
            "error_type": type(error).__name__ if isinstance(error, Exception) else "Error",
            "context": context or {}
        }

        # Append to error log
        with open(error_log, "a") as f:
            f.write(json.dumps(error_entry) + "\n")

    except Exception as e:
        # If error logging fails, write to stderr but don't crash
        import sys
        print(f"Failed to log error: {e}", file=sys.stderr)


def get_recent_errors(limit=50):
    """
    Get recent errors from .errors.log

    Args:
        limit: Maximum number of errors to return (default 50)

    Returns:
        List of error dicts, most recent first
    """
    try:
        project_root = find_project_root()
        error_log = os.path.join(project_root, "CONSCIOUSNESS", ".errors.log")

        if not os.path.exists(error_log):
            return []

        errors = []
        with open(error_log, "r") as f:
            for line in f:
                try:
                    errors.append(json.loads(line.strip()))
                except json.JSONDecodeError:
                    continue

        # Return most recent first
        return errors[-limit:][::-1]

    except Exception as e:
        return []


def rotate_error_log(keep_days=7):
    """
    Rotate error log, keeping only last N days

    Args:
        keep_days: Number of days of history to keep (default 7)
    """
    try:
        from datetime import timedelta

        project_root = find_project_root()
        error_log = os.path.join(project_root, "CONSCIOUSNESS", ".errors.log")

        if not os.path.exists(error_log):
            return

        cutoff = datetime.now() - timedelta(days=keep_days)
        cutoff_str = cutoff.strftime("%Y-%m-%d")

        # Read and filter errors
        kept_errors = []
        with open(error_log, "r") as f:
            for line in f:
                try:
                    error = json.loads(line.strip())
                    if error.get("timestamp", "") >= cutoff_str:
                        kept_errors.append(line)
                except json.JSONDecodeError:
                    continue

        # Rewrite log with only recent errors
        with open(error_log, "w") as f:
            f.writelines(kept_errors)

    except Exception as e:
        pass  # Rotation failure shouldn't crash hooks
