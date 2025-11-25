#!/usr/bin/env python3
"""
Display last 2 hours of time log entries every 15 minutes.
Shows raw entries, not a summary - just the table view.
"""

import sys
import json
from datetime import datetime, timedelta
from pathlib import Path


def get_15min_marker():
    """Check if we're at a 15-minute mark (00, 15, 30, or 45)"""
    now = datetime.now()
    return now.minute in [0, 15, 30, 45] and now.second < 10  # Within first 10 seconds


def get_last_2_hours_entries(log_file):
    """Get time log entries from last 2 hours"""
    if not log_file.exists():
        return []
    
    now = datetime.now()
    two_hours_ago = now - timedelta(hours=2)
    
    entries = []
    with open(log_file) as f:
        lines = f.readlines()
    
    # Find the table data (skip headers)
    in_table = False
    for line in lines:
        if '|------' in line:
            in_table = True
            continue
        
        if in_table and '|' in line and line.strip():
            # Parse the timestamp from the entry
            parts = [p.strip() for p in line.split('|')]
            if len(parts) >= 3:
                try:
                    # Second column is "Updated At" timestamp
                    timestamp_str = parts[1]
                    entry_time = datetime.strptime(timestamp_str, "%Y-%m-%d %H:%M:%S GMT")
                    
                    if entry_time >= two_hours_ago:
                        entries.append(line.strip())
                except (ValueError, IndexError):
                    continue
    
    return entries


def main():
    try:
        # Only run at 15-minute marks
        if not get_15min_marker():
            return 0
        
        # Find project root
        project_root = Path.cwd()
        while not (project_root / ".claude").exists() and project_root != project_root.parent:
            project_root = project_root.parent
        
        # Get time log
        log_file = project_root / "CONSCIOUSNESS" / "HUMAN-TIME-LOG.md"
        
        if not log_file.exists():
            return 0
        
        entries = get_last_2_hours_entries(log_file)
        
        if not entries:
            return 0
        
        # Display the raw entries
        print("\n" + "="*80, file=sys.stderr)
        print(f"TIME LOG - Last 2 hours ({len(entries)} entries)", file=sys.stderr)
        print("="*80, file=sys.stderr)
        print("\nDate Time Window | Updated At | Session ID | Activity", file=sys.stderr)
        print("-----------------|------------|------------|----------", file=sys.stderr)
        for entry in entries:
            print(entry, file=sys.stderr)
        print("\n" + "="*80 + "\n", file=sys.stderr)
        
        return 0
        
    except Exception as e:
        # Fail silently - don't block tool execution
        return 0


if __name__ == "__main__":
    sys.exit(main())
