#!/bin/bash
# Time log viewer - TSV single-line format
CYAN='\033[0;36m'; GREEN='\033[0;32m'; DIM='\033[2m'; NC='\033[0m'

# Parse arguments
USE_COLOR=true; SHOW_AGENT=true; SHOW_HUMAN=true
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-color) USE_COLOR=false; shift ;;
        --agent-only) SHOW_HUMAN=false; shift ;;
        --human-only) SHOW_AGENT=false; shift ;;
        *) [ -z "$TIME_UNIT" ] && TIME_UNIT=$1 || TIME_VALUE=$1; shift ;;
    esac
done

[ "$USE_COLOR" = false ] && { CYAN=''; GREEN=''; DIM=''; NC=''; }

# Find logs
if [ -f "CONSCIOUSNESS/HUMAN-TIME-LOG.md" ]; then
    HUMAN_LOG="CONSCIOUSNESS/HUMAN-TIME-LOG.md"; AGENT_LOG="CONSCIOUSNESS/AGENT-TIME-LOG.md"
elif [ -f "$HOME/CONSCIOUSNESS/HUMAN-TIME-LOG.md" ]; then
    HUMAN_LOG="$HOME/CONSCIOUSNESS/HUMAN-TIME-LOG.md"; AGENT_LOG="$HOME/CONSCIOUSNESS/AGENT-TIME-LOG.md"
else
    echo "No time log found"; exit 1
fi

# Calculate entries
if [ "$TIME_UNIT" = "all" ]; then ENTRIES=999999
elif [ -z "$TIME_UNIT" ] || [ -z "$TIME_VALUE" ]; then
    echo "Usage: show-pretty.sh [m|h|d] [number]"; exit 1
else
    case $TIME_UNIT in
        m|min|minutes) ENTRIES=$((TIME_VALUE / 3 + 1)) ;;
        h|hr|hours) ENTRIES=$((TIME_VALUE * 20)) ;;
        d|day|days) ENTRIES=$((TIME_VALUE * 480)) ;;
        *) echo "Invalid unit"; exit 1 ;;
    esac
fi

# Agent log
if [ "$SHOW_AGENT" = true ] && [ -f "$AGENT_LOG" ]; then
    echo -e "${CYAN}AGENT:${NC}"
    awk -F"|" 'NR>4 {gsub(/^[ \t]+|[ \t]+$/,"",$1);gsub(/^[ \t]+|[ \t]+$/,"",$2);gsub(/^[ \t]+|[ \t]+$/,"",$3);gsub(/^[ \t]+|[ \t]+$/,"",$4);if(length($1)>0)print $1" | "$2" | "$3" | "$4}' "$AGENT_LOG" | tail -n "$ENTRIES"
fi

# Human log
if [ "$SHOW_HUMAN" = true ] && [ -f "$HUMAN_LOG" ]; then
    echo -e "${GREEN}HUMAN:${NC}"
    awk -F"|" 'NR>4 {gsub(/^[ \t]+|[ \t]+$/,"",$1);gsub(/^[ \t]+|[ \t]+$/,"",$2);gsub(/^[ \t]+|[ \t]+$/,"",$3);gsub(/^[ \t]+|[ \t]+$/,"",$4);if(length($1)>0)print $1" | "$2" | "$3" | "$4}' "$HUMAN_LOG" | tail -n "$ENTRIES"
fi
