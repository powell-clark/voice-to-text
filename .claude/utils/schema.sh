#!/usr/bin/env bash
#
# Schema helper for Bash scripts
# Sources schema values from precepts/schema.json
#
# Usage:
#   source "$(dirname "$0")/../utils/schema.sh"
#   header=$(get_header "STORY.md")
#   columns=$(get_columns "STORY.md")
#   msg=$(get_message "STORY.md" "empty")
#

# Find schema.json
find_schema() {
    local dir="$PWD"
    while [ "$dir" != "/" ]; do
        if [ -f "$dir/precepts/schema.json" ]; then
            echo "$dir/precepts/schema.json"
            return 0
        fi
        dir=$(dirname "$dir")
    done

    # Try home directory
    if [ -f "$HOME/.claude/precepts/schema.json" ]; then
        echo "$HOME/.claude/precepts/schema.json"
        return 0
    fi

    return 1
}

# Find config.json
find_config() {
    local dir="$PWD"
    while [ "$dir" != "/" ]; do
        if [ -f "$dir/CONSCIOUSNESS/config.json" ]; then
            echo "$dir/CONSCIOUSNESS/config.json"
            return 0
        fi
        dir=$(dirname "$dir")
    done
    return 1
}

SCHEMA_FILE=$(find_schema)
CONFIG_FILE=$(find_config)

# Get header for a file
# Usage: get_header "STORY.md"
get_header() {
    local filename="$1"
    if [ -z "$SCHEMA_FILE" ]; then
        echo ""
        return 1
    fi
    python3 -c "
import json
with open('$SCHEMA_FILE') as f:
    schema = json.load(f)
header = schema.get('files', {}).get('$filename', {}).get('header', '')
print(header if header else '')
"
}

# Get column count for a file
# Usage: get_columns "STORY.md"
get_columns() {
    local filename="$1"
    if [ -z "$SCHEMA_FILE" ]; then
        echo ""
        return 1
    fi
    python3 -c "
import json
with open('$SCHEMA_FILE') as f:
    schema = json.load(f)
cols = schema.get('files', {}).get('$filename', {}).get('columns')
print(cols if cols else '')
"
}

# Get display message for a file state
# Usage: get_message "STORY.md" "empty"  # or "missing" or "corrupted"
get_message() {
    local filename="$1"
    local msg_type="$2"
    if [ -z "$SCHEMA_FILE" ]; then
        echo "[$msg_type]"
        return 1
    fi
    python3 -c "
import json
with open('$SCHEMA_FILE') as f:
    schema = json.load(f)
key = '${msg_type}_message'
msg = schema.get('files', {}).get('$filename', {}).get(key, '[$msg_type]')
print(msg)
"
}

# Get project prefix from config
# Usage: prefix=$(get_prefix)
get_prefix() {
    if [ -z "$CONFIG_FILE" ]; then
        echo ""
        return 1
    fi
    python3 -c "
import json
with open('$CONFIG_FILE') as f:
    config = json.load(f)
prefix = config.get('project', {}).get('prefix', '')
print(prefix)
"
}

# Get valid statuses
# Usage: statuses=$(get_statuses)
get_statuses() {
    if [ -z "$SCHEMA_FILE" ]; then
        echo "planned in-progress in-review blocked done"
        return 1
    fi
    python3 -c "
import json
with open('$SCHEMA_FILE') as f:
    schema = json.load(f)
statuses = schema.get('statuses', {}).get('work', [])
print(' '.join(statuses))
"
}

# Get file template
# Usage: template=$(get_template "STORY.md")
get_template() {
    local filename="$1"
    if [ -z "$SCHEMA_FILE" ]; then
        echo ""
        return 1
    fi
    python3 -c "
import json
with open('$SCHEMA_FILE') as f:
    schema = json.load(f)
template = schema.get('templates', {}).get('$filename', '')
print(template)
"
}

# Check if file is required
# Usage: if is_required "STORY.md"; then ...
is_required() {
    local filename="$1"
    if [ -z "$SCHEMA_FILE" ]; then
        return 1
    fi
    python3 -c "
import json
import sys
with open('$SCHEMA_FILE') as f:
    schema = json.load(f)
required = schema.get('files', {}).get('$filename', {}).get('required', False)
sys.exit(0 if required else 1)
"
}

# Validate file against schema
# Usage: check_file_schema "CONSCIOUSNESS/stories/STORY.md" "STORY.md"
# Returns: 0=valid, 1=missing, 2=corrupted
check_file_schema() {
    local filepath="$1"
    local filename="$2"

    if [ ! -f "$filepath" ]; then
        return 1  # missing
    fi

    local expected_header=$(get_header "$filename")
    if [ -z "$expected_header" ]; then
        return 0  # no header requirement
    fi

    local actual_header=$(grep -v '^#' "$filepath" | grep -v '^$' | grep '|' | head -1)

    if [ -z "$actual_header" ]; then
        return 2  # corrupted (no header found)
    fi

    if [ "$actual_header" != "$expected_header" ]; then
        return 2  # corrupted (wrong header)
    fi

    local expected_cols=$(get_columns "$filename")
    if [ -n "$expected_cols" ]; then
        local data_lines=$(grep -v '^#' "$filepath" | grep -v '^$' | grep '|' | tail -n +2)
        if [ -n "$data_lines" ]; then
            local first_row=$(echo "$data_lines" | head -1)
            local col_count=$(echo "$first_row" | tr -cd '|' | wc -c)
            col_count=$((col_count + 1))

            if [ "$col_count" -ne "$expected_cols" ]; then
                return 2  # corrupted (wrong column count)
            fi
        fi
    fi

    return 0  # valid
}

# Get all tracked files from schema
# Usage: get_all_files
get_all_files() {
    if [ -z "$SCHEMA_FILE" ]; then
        echo ""
        return 1
    fi
    python3 -c "
import json
with open('$SCHEMA_FILE') as f:
    schema = json.load(f)
for filename in schema.get('files', {}).keys():
    print(filename)
"
}

# Get file path from schema
# Usage: get_file_path "STORY.md"
get_file_path() {
    local filename="$1"
    if [ -z "$SCHEMA_FILE" ]; then
        echo ""
        return 1
    fi
    python3 -c "
import json
with open('$SCHEMA_FILE') as f:
    schema = json.load(f)
path = schema.get('files', {}).get('$filename', {}).get('path', '')
print(path)
"
}

# Print file status with appropriate message
# Usage: print_file_status "CONSCIOUSNESS/stories/STORY.md" "STORY.md" "STORY.$PROJECT"
print_file_status() {
    local filepath="$1"
    local filename="$2"
    local section_name="$3"

    check_file_schema "$filepath" "$filename"
    local status=$?

    case $status in
        0)
            # Valid - check for content
            local content=$(grep -v '^#' "$filepath" 2>/dev/null | grep -v '^$' | grep '|' | tail -n +2)
            if [ -n "$content" ]; then
                return 0  # has content
            else
                echo "  $(get_message "$filename" "empty")"
                return 3  # empty but valid
            fi
            ;;
        1)
            echo "  $(get_message "$filename" "missing")"
            return 1
            ;;
        2)
            echo "  $(get_message "$filename" "corrupted")"
            return 2
            ;;
    esac
}
