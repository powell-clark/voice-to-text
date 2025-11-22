#!/bin/bash
# System Resource Monitor for Voice to Text
# Displays real-time stats in system tray or terminal

set -e

LOG_DIR="${HOME}/.local/share/voice-to-text"
RECORDINGS_DIR="$LOG_DIR/recordings"
LOG_FILE="$LOG_DIR/vtt.log"
PID_FILE="$LOG_DIR/vtt-linux.pid"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m'

# Get process info
get_process_info() {
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE" 2>/dev/null)
        if ps -p "$PID" > /dev/null 2>&1; then
            echo "running:$PID"
            return 0
        fi
    fi

    # Try to find by process name
    PID=$(pgrep -x vtt-linux | head -1)
    if [ -n "$PID" ]; then
        echo "running:$PID"
        return 0
    fi

    echo "stopped"
    return 1
}

# Get memory usage
get_memory_usage() {
    local pid="$1"
    if [ -z "$pid" ]; then
        echo "0"
        return
    fi

    ps -p "$pid" -o rss= 2>/dev/null | awk '{print int($1/1024)}' || echo "0"
}

# Get CPU usage
get_cpu_usage() {
    local pid="$1"
    if [ -z "$pid" ]; then
        echo "0.0"
        return
    fi

    ps -p "$pid" -o %cpu= 2>/dev/null | awk '{print $1}' || echo "0.0"
}

# Count recordings
count_recordings() {
    if [ -d "$RECORDINGS_DIR" ]; then
        find "$RECORDINGS_DIR" -name "*.wav" -type f 2>/dev/null | wc -l
    else
        echo "0"
    fi
}

# Calculate total recording time
get_total_recording_time() {
    if [ ! -d "$RECORDINGS_DIR" ]; then
        echo "0"
        return
    fi

    local total_seconds=0

    if command -v soxi &> /dev/null; then
        while IFS= read -r file; do
            duration=$(soxi -D "$file" 2>/dev/null | awk '{print int($1)}')
            if [ -n "$duration" ]; then
                total_seconds=$((total_seconds + duration))
            fi
        done < <(find "$RECORDINGS_DIR" -name "*.wav" -type f 2>/dev/null)
    fi

    # Convert to minutes
    echo "$((total_seconds / 60))"
}

# Count transcriptions
count_transcriptions() {
    if [ -f "$LOG_FILE" ]; then
        grep -c "Transcription:" "$LOG_FILE" 2>/dev/null || echo "0"
    else
        echo "0"
    fi
}

# Get disk usage
get_disk_usage() {
    if [ -d "$RECORDINGS_DIR" ]; then
        du -sh "$RECORDINGS_DIR" 2>/dev/null | cut -f1
    else
        echo "0B"
    fi
}

# Display stats
display_stats() {
    clear
    echo -e "${BOLD}=========================================="
    echo -e "Voice to Text - System Monitor"
    echo -e "==========================================${NC}"
    echo ""

    # Process status
    PROCESS_INFO=$(get_process_info)
    STATUS=$(echo "$PROCESS_INFO" | cut -d: -f1)
    PID=$(echo "$PROCESS_INFO" | cut -d: -f2)

    if [ "$STATUS" = "running" ]; then
        echo -e "${GREEN}● Running${NC} (PID: $PID)"

        # Resource usage
        MEM=$(get_memory_usage "$PID")
        CPU=$(get_cpu_usage "$PID")

        echo -e "  Memory: ${BLUE}${MEM} MB${NC}"
        echo -e "  CPU: ${BLUE}${CPU}%${NC}"
    else
        echo -e "${RED}○ Stopped${NC}"
    fi

    echo ""

    # Recording stats
    echo -e "${BOLD}Recording Statistics:${NC}"

    RECORDING_COUNT=$(count_recordings)
    TRANSCRIPTION_COUNT=$(count_transcriptions)
    TOTAL_TIME=$(get_total_recording_time)
    DISK_USAGE=$(get_disk_usage)

    echo -e "  Total recordings: ${GREEN}$RECORDING_COUNT${NC}"
    echo -e "  Total transcriptions: ${GREEN}$TRANSCRIPTION_COUNT${NC}"
    echo -e "  Total recording time: ${GREEN}${TOTAL_TIME} minutes${NC}"
    echo -e "  Disk usage: ${GREEN}${DISK_USAGE}${NC}"

    echo ""

    # Recent activity
    if [ -f "$LOG_FILE" ]; then
        echo -e "${BOLD}Recent Activity:${NC}"
        tail -5 "$LOG_FILE" | grep -E "(Recording|Transcription)" | sed 's/^/  /' || echo "  No recent activity"
    fi

    echo ""
    echo -e "${BOLD}Directories:${NC}"
    echo -e "  Logs: ${BLUE}$LOG_DIR${NC}"
    echo -e "  Recordings: ${BLUE}$RECORDINGS_DIR${NC}"
}

# Watch mode (auto-refresh)
watch_mode() {
    while true; do
        display_stats
        echo ""
        echo -e "${YELLOW}Press Ctrl+C to exit${NC}"
        sleep 2
    done
}

# Export stats to JSON
export_json() {
    PROCESS_INFO=$(get_process_info)
    STATUS=$(echo "$PROCESS_INFO" | cut -d: -f1)
    PID=$(echo "$PROCESS_INFO" | cut -d: -f2)

    MEM=$(get_memory_usage "$PID")
    CPU=$(get_cpu_usage "$PID")
    RECORDING_COUNT=$(count_recordings)
    TRANSCRIPTION_COUNT=$(count_transcriptions)
    TOTAL_TIME=$(get_total_recording_time)
    DISK_USAGE=$(get_disk_usage)

    cat <<EOF
{
  "status": "$STATUS",
  "pid": ${PID:-null},
  "memory_mb": $MEM,
  "cpu_percent": $CPU,
  "recordings": {
    "count": $RECORDING_COUNT,
    "total_minutes": $TOTAL_TIME,
    "disk_usage": "$DISK_USAGE"
  },
  "transcriptions": {
    "count": $TRANSCRIPTION_COUNT
  }
}
EOF
}

# Main
case "${1:-stats}" in
    stats|--stats|-s)
        display_stats
        ;;
    watch|--watch|-w)
        watch_mode
        ;;
    json|--json|-j)
        export_json
        ;;
    help|--help|-h)
        echo "Voice to Text System Monitor"
        echo ""
        echo "Usage: $0 [option]"
        echo ""
        echo "Options:"
        echo "  stats, -s     Show current stats (default)"
        echo "  watch, -w     Watch mode (auto-refresh every 2s)"
        echo "  json, -j      Export stats as JSON"
        echo "  help, -h      Show this help"
        ;;
    *)
        echo "Unknown option: $1"
        echo "Use --help for usage information"
        exit 1
        ;;
esac
