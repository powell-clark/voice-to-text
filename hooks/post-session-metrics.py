#!/usr/bin/env python3
"""
Session Metrics Finalization Hook

Finalizes and saves session metrics that were tracked in real-time.
Called automatically by SessionEnd hook.

Real-time tracking happens via session metrics file that's updated as work progresses.
This hook just finalizes the file and moves it to permanent storage.
"""

import json
import sys
from datetime import datetime
from pathlib import Path


def get_session_id():
    """Get current session ID from metadata"""
    metadata_file = Path(".claude/.session-metadata.json")
    if metadata_file.exists():
        with open(metadata_file) as f:
            return json.load(f).get("session_id", "unknown")
    return "unknown"


def get_git_branch():
    """Get current git branch"""
    try:
        import subprocess

        result = subprocess.run(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"],
            capture_output=True,
            text=True,
            check=True,
        )
        branch = result.stdout.strip()
        # Sanitize branch name - allow only alphanumeric, dash, underscore
        import re
        branch = re.sub(r'[^a-zA-Z0-9_-]', '', branch)
        return branch if branch else "unknown"
    except Exception:
        return "unknown"


def count_commits_since_start():
    """Count commits in this session (rough estimate from git log)"""
    try:
        import subprocess

        # Get commits from last 4 hours (approximate session duration)
        result = subprocess.run(
            ["git", "log", "--oneline", "--since=4 hours ago"],
            capture_output=True,
            text=True,
            check=True,
        )
        return len(result.stdout.strip().split("\n")) if result.stdout.strip() else 0
    except Exception:
        return 0


def finalize_session_metrics():
    """Finalize session metrics from running tracker file"""

    session_id = get_session_id()
    branch = get_git_branch()

    # Load running metrics file
    metrics_file = Path(f".claude/.session-metrics-{session_id}.json")
    if not metrics_file.exists():
        # No metrics tracked this session - create minimal record
        return {
            "session_id": session_id,
            "date": datetime.now().isoformat(),
            "branch": branch,
            "duration_minutes": 0,
            "speed": {},
            "value": {"tasks_completed": 0},
            "cost": {"commits": count_commits_since_start()},
        }

    with open(metrics_file) as f:
        running_metrics = json.load(f)

    # Calculate derived metrics
    start_time = datetime.fromisoformat(running_metrics["start_time"])
    end_time = datetime.now()
    duration_minutes = int((end_time - start_time).total_seconds() / 60)

    # Calculate test pass rate
    test_runs = running_metrics.get("test_runs", [])
    if test_runs:
        latest_test = test_runs[-1]
        total_tests = latest_test["passed"] + latest_test["failed"]
        test_pass_rate = latest_test["passed"] / total_tests if total_tests > 0 else 0
    else:
        test_pass_rate = 0

    # Calculate autonomous completion rate
    questions_asked = running_metrics.get("questions_asked", 0)
    autonomous_actions = running_metrics.get("autonomous_actions", 0)
    total_actions = questions_asked + autonomous_actions
    autonomous_rate = autonomous_actions / total_actions if total_actions > 0 else 1.0

    # Calculate iteration success rate
    iterations = running_metrics.get("iterations", {})
    total_iterations = iterations.get("total", 0)
    resolved_within_5 = iterations.get("resolved_within_5", 0)
    iteration_success_rate = (
        resolved_within_5 / total_iterations if total_iterations > 0 else 0
    )

    # Build final metrics
    metrics = {
        "session_id": session_id,
        "date": end_time.isoformat(),
        "branch": branch,
        "duration_minutes": duration_minutes,
        "speed": {
            "time_to_first_response_seconds": 0,  # Would need to track from session start
            "avg_iteration_minutes": 0,  # Could calculate from iteration timestamps
            "decision_latency_percent": (
                (questions_asked / total_actions * 100) if total_actions > 0 else 0
            ),
            "context_load_tokens": 0,  # From session start logs
        },
        "value": {
            "tasks_completed": running_metrics.get("tasks_completed", 0),
            "autonomous_completion_rate": autonomous_rate,
            "iteration_success_rate": iteration_success_rate,
            "test_pass_rate": test_pass_rate,
            "build_failures": 0,  # From pre-commit logs
            "memory_updates": 0,  # Git commits to .claude/memory/
        },
        "cost": {
            "tokens_used": 0,  # Would need API tracking
            "tokens_per_task": 0,
            "questions_asked": questions_asked,
            "autonomous_actions": autonomous_actions,
            "questions_ratio": (
                (questions_asked / autonomous_actions) if autonomous_actions > 0 else 0
            ),
            "commits": running_metrics.get("commits", 0),
            "commits_reverted": 0,
            "rework_rate": 0,
        },
        "tools_used": running_metrics.get("tools_used", {}),
        "skills_loaded": [],  # From session logs
        "questions_logged": running_metrics.get("questions_logged", 0),
        "memory_files_updated": [],  # Git diff .claude/memory/
    }

    return metrics


def save_metrics(metrics):
    """Save metrics to performance tracking directory"""
    perf_dir = Path(".claude/memory/performance/sessions")
    perf_dir.mkdir(parents=True, exist_ok=True)

    session_id = metrics["session_id"]
    output_file = perf_dir / f"{session_id}.json"

    with open(output_file, "w") as f:
        json.dump(metrics, f, indent=2)

    print(f"✓ Performance metrics saved: {output_file}")


def cleanup_running_metrics():
    """Remove running metrics file after finalization"""
    session_id = get_session_id()
    metrics_file = Path(f".claude/.session-metrics-{session_id}.json")
    if metrics_file.exists():
        metrics_file.unlink()


def main():
    """Main entry point"""
    try:
        metrics = finalize_session_metrics()
        save_metrics(metrics)
        cleanup_running_metrics()
    except Exception as e:
        print(f"Warning: Could not save performance metrics: {e}", file=sys.stderr)
        # Don't fail the session end - this is optional tracking


if __name__ == "__main__":
    main()
