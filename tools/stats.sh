#!/bin/bash
# Usage Statistics Analyzer for Voice to Text
# Analyzes logs and recordings to provide insights

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

LOG_FILE="$HOME/.local/share/voice-to-text/vtt.log"
RECORDINGS_DIR="$HOME/.local/share/voice-to-text/recordings"

echo -e "${BOLD}========================================"
echo "Voice to Text - Usage Statistics"
echo "========================================${NC}"
echo ""

# Check if log exists
if [ ! -f "$LOG_FILE" ]; then
    echo -e "${RED}Error: Log file not found: $LOG_FILE${NC}"
    echo "No usage data available yet."
    exit 1
fi

# Get log file age
LOG_AGE_DAYS=$(( ($(date +%s) - $(stat -c %Y "$LOG_FILE")) / 86400 ))
if [ "$LOG_AGE_DAYS" -eq 0 ]; then
    LOG_AGE="today"
else
    LOG_AGE="$LOG_AGE_DAYS days ago"
fi

echo "Analyzing data since: $LOG_AGE"
echo "Log file: $LOG_FILE"
echo ""

# ===========================================
# Recording Statistics
# ===========================================
echo -e "${BOLD}=== Recording Statistics ===${NC}"
echo ""

# Count total recordings
TOTAL_RECORDINGS=$(grep -c "Recording saved:" "$LOG_FILE" 2>/dev/null || echo 0)
echo "Total recordings: ${GREEN}$TOTAL_RECORDINGS${NC}"

# Count transcriptions
TOTAL_TRANSCRIPTIONS=$(grep -c "Transcription:" "$LOG_FILE" 2>/dev/null || echo 0)
echo "Total transcriptions: ${GREEN}$TOTAL_TRANSCRIPTIONS${NC}"

# Calculate success rate
if [ "$TOTAL_RECORDINGS" -gt 0 ]; then
    SUCCESS_RATE=$((TOTAL_TRANSCRIPTIONS * 100 / TOTAL_RECORDINGS))
    echo "Success rate: ${GREEN}${SUCCESS_RATE}%${NC}"
fi

# Count rejections
REJECTED_SHORT=$(grep -c "REJECTED:TOO_SHORT" "$LOG_FILE" 2>/dev/null || echo 0)
REJECTED_QUIET=$(grep -c "REJECTED:TOO_QUIET" "$LOG_FILE" 2>/dev/null || echo 0)
TOTAL_REJECTED=$((REJECTED_SHORT + REJECTED_QUIET))

if [ "$TOTAL_REJECTED" -gt 0 ]; then
    echo ""
    echo "Rejected recordings: ${YELLOW}$TOTAL_REJECTED${NC}"
    [ "$REJECTED_SHORT" -gt 0 ] && echo "  - Too short: $REJECTED_SHORT"
    [ "$REJECTED_QUIET" -gt 0 ] && echo "  - Too quiet: $REJECTED_QUIET"
fi

# ===========================================
# Timing Statistics
# ===========================================
echo ""
echo -e "${BOLD}=== Performance Statistics ===${NC}"
echo ""

# Extract transcription times
TRANSCRIPTION_TIMES=$(grep "Processing:" "$LOG_FILE" 2>/dev/null | wc -l)

if [ "$TRANSCRIPTION_TIMES" -gt 0 ]; then
    # Try to calculate average (requires timing data in logs)
    # This is a simplified estimate
    echo "Transcriptions processed: $TRANSCRIPTION_TIMES"
else
    echo "No detailed timing data available"
fi

# Count timeout errors
TIMEOUT_ERRORS=$(grep -c "TRANSCRIPTION_TIMEOUT" "$LOG_FILE" 2>/dev/null || echo 0)
if [ "$TIMEOUT_ERRORS" -gt 0 ]; then
    echo -e "${YELLOW}⚠ Transcription timeouts: $TIMEOUT_ERRORS${NC}"
fi

# ===========================================
# Error Statistics
# ===========================================
TOTAL_ERRORS=$(grep -c "ERROR:" "$LOG_FILE" 2>/dev/null || echo 0)

if [ "$TOTAL_ERRORS" -gt 0 ]; then
    echo ""
    echo -e "${BOLD}=== Error Statistics ===${NC}"
    echo ""
    echo -e "Total errors: ${RED}$TOTAL_ERRORS${NC}"

    # Top error types
    echo ""
    echo "Top error types:"
    grep "ERROR:" "$LOG_FILE" | awk -F'ERROR:' '{print $2}' | sort | uniq -c | sort -rn | head -5 | while read count error; do
        echo "  $count × $error"
    done
fi

# ===========================================
# Storage Statistics
# ===========================================
echo ""
echo -e "${BOLD}=== Storage Statistics ===${NC}"
echo ""

if [ -d "$RECORDINGS_DIR" ]; then
    RECORDING_COUNT=$(find "$RECORDINGS_DIR" -name "*.wav" -type f 2>/dev/null | wc -l)
    DISK_USAGE=$(du -sh "$RECORDINGS_DIR" 2>/dev/null | cut -f1)

    echo "Stored recordings: $RECORDING_COUNT"
    echo "Disk usage: $DISK_USAGE"

    # Calculate total recording time if soxi is available
    if command -v soxi &> /dev/null; then
        TOTAL_SECONDS=0
        while IFS= read -r file; do
            duration=$(soxi -D "$file" 2>/dev/null | awk '{print int($1)}')
            [ -n "$duration" ] && TOTAL_SECONDS=$((TOTAL_SECONDS + duration))
        done < <(find "$RECORDINGS_DIR" -name "*.wav" -type f 2>/dev/null)

        TOTAL_MINUTES=$((TOTAL_SECONDS / 60))
        TOTAL_HOURS=$((TOTAL_MINUTES / 60))
        REMAINING_MINUTES=$((TOTAL_MINUTES % 60))

        if [ "$TOTAL_HOURS" -gt 0 ]; then
            echo "Total recording time: ${GREEN}${TOTAL_HOURS}h ${REMAINING_MINUTES}m${NC}"
        else
            echo "Total recording time: ${GREEN}${TOTAL_MINUTES}m${NC}"
        fi

        # Calculate average recording length
        if [ "$RECORDING_COUNT" -gt 0 ]; then
            AVG_SECONDS=$((TOTAL_SECONDS / RECORDING_COUNT))
            echo "Average recording: ${AVG_SECONDS}s"
        fi
    fi
else
    echo "No recordings directory found"
fi

# ===========================================
# Model Usage Statistics
# ===========================================
echo ""
echo -e "${BOLD}=== Model Usage ===${NC}"
echo ""

# Extract model switches from logs
MODELS_USED=$(grep "Model:" "$LOG_FILE" 2>/dev/null | awk '{print $NF}' | sort | uniq -c | sort -rn)

if [ -n "$MODELS_USED" ]; then
    echo "Model usage:"
    echo "$MODELS_USED" | while read count model; do
        echo "  $count × $model"
    done
else
    echo "No model usage data available"
fi

# ===========================================
# Activity Timeline
# ===========================================
echo ""
echo -e "${BOLD}=== Activity Timeline ===${NC}"
echo ""

# Count recordings per day (last 7 days)
echo "Last 7 days activity:"
for i in {0..6}; do
    date_str=$(date -d "$i days ago" +%Y-%m-%d 2>/dev/null || date -v-${i}d +%Y-%m-%d 2>/dev/null)
    count=$(grep "$date_str" "$LOG_FILE" 2>/dev/null | grep -c "Recording saved:" || echo 0)

    # Create a simple bar chart
    bar=""
    for ((j=0; j<count && j<50; j++)); do
        bar="${bar}█"
    done

    printf "  %s: %3d %s\n" "$date_str" "$count" "$bar"
done

# ===========================================
# Word Count Estimate
# ===========================================
echo ""
echo -e "${BOLD}=== Transcription Statistics ===${NC}"
echo ""

# Count total words transcribed
TOTAL_WORDS=0
while IFS= read -r line; do
    words=$(echo "$line" | wc -w)
    TOTAL_WORDS=$((TOTAL_WORDS + words))
done < <(grep "Transcription:" "$LOG_FILE" | sed 's/^.*Transcription: //')

if [ "$TOTAL_WORDS" -gt 0 ]; then
    echo "Total words transcribed: ${GREEN}${TOTAL_WORDS}${NC}"

    # Calculate average words per transcription
    if [ "$TOTAL_TRANSCRIPTIONS" -gt 0 ]; then
        AVG_WORDS=$((TOTAL_WORDS / TOTAL_TRANSCRIPTIONS))
        echo "Average words per transcription: $AVG_WORDS"
    fi

    # Estimate pages (250 words per page)
    PAGES=$((TOTAL_WORDS / 250))
    if [ "$PAGES" -gt 0 ]; then
        echo "Equivalent pages (250 words/page): $PAGES"
    fi
fi

# ===========================================
# Most Common Words
# ===========================================
echo ""
echo "Most common words:"
grep "Transcription:" "$LOG_FILE" | \
    sed 's/^.*Transcription: //' | \
    tr '[:upper:]' '[:lower:]' | \
    tr -s '[:punct:][:space:]' '\n' | \
    grep -v '^$' | \
    sort | uniq -c | sort -rn | head -10 | \
    awk '{printf "  %3d × %s\n", $1, $2}'

# ===========================================
# Recommendations
# ===========================================
echo ""
echo -e "${BOLD}=== Recommendations ===${NC}"
echo ""

# Analyze and provide recommendations
if [ "$REJECTED_SHORT" -gt "$((TOTAL_RECORDINGS / 10))" ]; then
    echo -e "${YELLOW}⚠ High number of rejections (too short)${NC}"
    echo "  → Try speaking for at least 1 second"
fi

if [ "$REJECTED_QUIET" -gt "$((TOTAL_RECORDINGS / 10))" ]; then
    echo -e "${YELLOW}⚠ High number of rejections (too quiet)${NC}"
    echo "  → Check microphone gain settings"
    echo "  → Speak louder or move microphone closer"
fi

if [ "$TIMEOUT_ERRORS" -gt 0 ]; then
    echo -e "${YELLOW}⚠ Transcription timeouts detected${NC}"
    echo "  → Consider using a smaller/faster model"
    echo "  → Check CPU/GPU availability"
fi

if [ "$TOTAL_ERRORS" -gt "$((TOTAL_RECORDINGS / 5))" ]; then
    echo -e "${YELLOW}⚠ High error rate${NC}"
    echo "  → Check logs: $LOG_FILE"
    echo "  → Consider running: ./tools/first_run_wizard.sh"
fi

# Storage recommendations
if [ -d "$RECORDINGS_DIR" ]; then
    DISK_USAGE_MB=$(du -sm "$RECORDINGS_DIR" 2>/dev/null | cut -f1)
    if [ "$DISK_USAGE_MB" -gt 1000 ]; then
        echo -e "${YELLOW}⚠ Recording storage is large (${DISK_USAGE})"
        echo "  → Consider backing up and cleaning old recordings"
        echo "  → Run: ./tools/backup.sh"
    fi
fi

echo ""
echo -e "${GREEN}✓ Statistics analysis complete${NC}"
echo ""
echo "For more details:"
echo "  - View logs: tail -f $LOG_FILE"
echo "  - System monitor: ./tools/system_monitor.sh watch"
echo "  - History viewer: ./tools/history_viewer.sh"
echo ""
