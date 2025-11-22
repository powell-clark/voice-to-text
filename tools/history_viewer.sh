#!/bin/bash
# Transcription History Viewer for Voice to Text
# Browse, search, and replay past recordings

set -e

RECORDINGS_DIR="${HOME}/.local/share/voice-to-text/recordings"
LOG_FILE="${HOME}/.local/share/voice-to-text/vtt.log"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m'

echo "=========================================="
echo "Voice to Text - History Viewer"
echo "=========================================="
echo ""

# Check if recordings directory exists
if [ ! -d "$RECORDINGS_DIR" ]; then
    echo -e "${RED}Error: Recordings directory not found${NC}"
    echo "Expected: $RECORDINGS_DIR"
    echo ""
    echo "No recordings have been made yet."
    exit 1
fi

# Count recordings
recording_count=$(find "$RECORDINGS_DIR" -name "*.wav" -type f 2>/dev/null | wc -l)

if [ "$recording_count" -eq 0 ]; then
    echo -e "${YELLOW}No recordings found in $RECORDINGS_DIR${NC}"
    exit 0
fi

echo -e "${GREEN}Found $recording_count recording(s)${NC}"
echo ""

# Function to format timestamp
format_timestamp() {
    local filename="$1"
    # Extract timestamp from filename: vtt_recording_YYYYMMDD_HHMMSS.wav
    if [[ "$filename" =~ vtt_recording_([0-9]{8})_([0-9]{6}) ]]; then
        local date="${BASH_REMATCH[1]}"
        local time="${BASH_REMATCH[2]}"

        # Format: YYYY-MM-DD HH:MM:SS
        echo "${date:0:4}-${date:4:2}-${date:6:2} ${time:0:2}:${time:2:2}:${time:4:2}"
    else
        echo "Unknown"
    fi
}

# Function to get file size in human-readable format
get_file_size() {
    local file="$1"
    du -h "$file" 2>/dev/null | cut -f1
}

# Function to get audio duration
get_duration() {
    local file="$1"
    if command -v soxi &> /dev/null; then
        soxi -D "$file" 2>/dev/null | awk '{printf "%.1fs", $1}'
    elif command -v ffprobe &> /dev/null; then
        ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 "$file" 2>/dev/null | awk '{printf "%.1fs", $1}'
    else
        echo "N/A"
    fi
}

# Function to find transcription in log
find_transcription() {
    local filename="$1"
    local basename=$(basename "$filename")

    if [ ! -f "$LOG_FILE" ]; then
        echo "(log file not found)"
        return
    fi

    # Search for transcription in log (look for lines near the filename)
    # This is a heuristic - may not always find the right transcription
    grep -A 20 "$basename" "$LOG_FILE" 2>/dev/null | grep "Transcription:" | head -1 | sed 's/^.*Transcription: //' || echo "(not found in log)"
}

# Function to play audio
play_audio() {
    local file="$1"
    echo -e "${BLUE}Playing: $(basename "$file")${NC}"

    if command -v aplay &> /dev/null; then
        aplay "$file" 2>/dev/null
    elif command -v paplay &> /dev/null; then
        paplay "$file" 2>/dev/null
    elif command -v ffplay &> /dev/null; then
        ffplay -nodisp -autoexit "$file" 2>/dev/null
    else
        echo -e "${RED}Error: No audio player found${NC}"
        echo "Install: sudo apt install alsa-utils"
        return 1
    fi
}

# List all recordings with details
list_recordings() {
    echo -e "${BOLD}Recording History:${NC}"
    echo ""
    printf "%-5s %-20s %-8s %-10s %s\n" "ID" "Date/Time" "Duration" "Size" "File"
    echo "--------------------------------------------------------------------------------"

    local i=1
    find "$RECORDINGS_DIR" -name "*.wav" -type f -printf "%T@ %p\n" 2>/dev/null | sort -rn | while read -r timestamp filepath; do
        local filename=$(basename "$filepath")
        local datetime=$(format_timestamp "$filename")
        local duration=$(get_duration "$filepath")
        local size=$(get_file_size "$filepath")

        printf "${GREEN}%-5d${NC} %-20s %-8s %-10s %s\n" "$i" "$datetime" "$duration" "$size" "$filename"
        i=$((i + 1))
    done
}

# Interactive mode
interactive_mode() {
    while true; do
        echo ""
        echo -e "${BOLD}=== Transcription History ===${NC}"
        echo "1) List all recordings"
        echo "2) Play recording by ID"
        echo "3) View transcription by ID"
        echo "4) Search recordings"
        echo "5) Delete recording by ID"
        echo "6) Export recordings"
        echo "q) Quit"
        echo ""
        read -p "Select option: " choice

        case "$choice" in
            1)
                list_recordings
                ;;
            2)
                read -p "Enter recording ID: " rec_id
                local filepath=$(find "$RECORDINGS_DIR" -name "*.wav" -type f -printf "%T@ %p\n" 2>/dev/null | sort -rn | sed -n "${rec_id}p" | cut -d' ' -f2)
                if [ -z "$filepath" ]; then
                    echo -e "${RED}Invalid ID${NC}"
                else
                    play_audio "$filepath"
                fi
                ;;
            3)
                read -p "Enter recording ID: " rec_id
                local filepath=$(find "$RECORDINGS_DIR" -name "*.wav" -type f -printf "%T@ %p\n" 2>/dev/null | sort -rn | sed -n "${rec_id}p" | cut -d' ' -f2)
                if [ -z "$filepath" ]; then
                    echo -e "${RED}Invalid ID${NC}"
                else
                    echo ""
                    echo -e "${BOLD}Transcription:${NC}"
                    find_transcription "$filepath"
                fi
                ;;
            4)
                read -p "Search term: " search_term
                echo ""
                echo -e "${BOLD}Search results for '$search_term':${NC}"
                grep -i "$search_term" "$LOG_FILE" 2>/dev/null | grep "Transcription:" || echo "No matches found"
                ;;
            5)
                read -p "Enter recording ID to delete: " rec_id
                local filepath=$(find "$RECORDINGS_DIR" -name "*.wav" -type f -printf "%T@ %p\n" 2>/dev/null | sort -rn | sed -n "${rec_id}p" | cut -d' ' -f2)
                if [ -z "$filepath" ]; then
                    echo -e "${RED}Invalid ID${NC}"
                else
                    echo "Delete: $(basename "$filepath")?"
                    read -p "Confirm (y/n): " confirm
                    if [[ "$confirm" =~ ^[Yy]$ ]]; then
                        rm "$filepath"
                        echo -e "${GREEN}Deleted${NC}"
                    fi
                fi
                ;;
            6)
                read -p "Export directory: " export_dir
                if [ -z "$export_dir" ]; then
                    export_dir="./exported_recordings"
                fi
                mkdir -p "$export_dir"
                cp "$RECORDINGS_DIR"/*.wav "$export_dir"/ 2>/dev/null
                echo -e "${GREEN}Exported to $export_dir${NC}"
                ;;
            q|Q)
                echo "Goodbye!"
                exit 0
                ;;
            *)
                echo -e "${RED}Invalid option${NC}"
                ;;
        esac
    done
}

# Main
if [ "$1" == "--list" ] || [ "$1" == "-l" ]; then
    list_recordings
elif [ "$1" == "--play" ] || [ "$1" == "-p" ]; then
    if [ -z "$2" ]; then
        echo "Usage: $0 --play <ID>"
        exit 1
    fi
    filepath=$(find "$RECORDINGS_DIR" -name "*.wav" -type f -printf "%T@ %p\n" 2>/dev/null | sort -rn | sed -n "${2}p" | cut -d' ' -f2)
    play_audio "$filepath"
elif [ "$1" == "--help" ] || [ "$1" == "-h" ]; then
    echo "Usage: $0 [option]"
    echo ""
    echo "Options:"
    echo "  (none)        Interactive mode"
    echo "  --list, -l    List all recordings"
    echo "  --play ID     Play recording by ID"
    echo "  --help, -h    Show this help"
    exit 0
else
    interactive_mode
fi
