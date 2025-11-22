#!/bin/bash
# Comprehensive Test Suite for Voice to Text
# Tests all major components and new features

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

# Test result tracking
declare -a FAILED_TESTS
declare -a SKIPPED_TESTS

# Print test header
print_test_header() {
    echo ""
    echo -e "${BOLD}=========================================="
    echo -e "$1"
    echo -e "==========================================${NC}"
    echo ""
}

# Run a test
run_test() {
    local test_name="$1"
    local test_command="$2"

    echo -n "Testing: $test_name... "

    if eval "$test_command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "${RED}✗ FAIL${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        FAILED_TESTS+=("$test_name")
        return 1
    fi
}

# Skip a test
skip_test() {
    local test_name="$1"
    local reason="$2"

    echo -e "Testing: $test_name... ${YELLOW}⊘ SKIP${NC} ($reason)"
    TESTS_SKIPPED=$((TESTS_SKIPPED + 1))
    SKIPPED_TESTS+=("$test_name")
}

# Test with expected output
run_test_with_output() {
    local test_name="$1"
    local test_command="$2"
    local expected_pattern="$3"

    echo -n "Testing: $test_name... "

    output=$(eval "$test_command" 2>&1)

    if echo "$output" | grep -q "$expected_pattern"; then
        echo -e "${GREEN}✓ PASS${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "${RED}✗ FAIL${NC}"
        echo "  Expected pattern: $expected_pattern"
        echo "  Got: $output"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        FAILED_TESTS+=("$test_name")
        return 1
    fi
}

echo -e "${BOLD}========================================"
echo "Voice to Text - Test Suite"
echo "========================================${NC}"
echo ""
echo "Starting comprehensive tests..."

# ===========================================
# Build System Tests
# ===========================================
print_test_header "Build System Tests"

run_test "Makefile.linux exists" "test -f Makefile.linux"
run_test "Source directory structure" "test -d src/linux && test -d src/common"

if command -v gcc &> /dev/null; then
    run_test "GCC available" "gcc --version"
else
    skip_test "GCC available" "gcc not installed"
fi

if command -v make &> /dev/null; then
    run_test "Make available" "make --version"
else
    skip_test "Make available" "make not installed"
fi

# ===========================================
# Wayland Support Tests
# ===========================================
print_test_header "Wayland Support Tests"

run_test "Wayland detection module exists" "test -f src/linux/wayland_detect.c"
run_test "Wayland detection header exists" "test -f src/linux/wayland_detect.h"
run_test "Wayland keyboard module exists" "test -f src/linux/keyboard_wayland.c"
run_test "Wayland typing module exists" "test -f src/linux/typing_wayland.c"
run_test "D-Bus keyboard module exists" "test -f src/linux/keyboard_wayland_dbus.c"
run_test "Wayland setup documentation exists" "test -f docs/WAYLAND_SETUP.md"

# ===========================================
# Crash Handler Tests
# ===========================================
print_test_header "Crash Handler Tests"

run_test "Crash handler source exists" "test -f src/common/crash_handler.c"
run_test "Crash handler header exists" "test -f src/common/crash_handler.h"
run_test "Crash test tool exists" "test -f tools/crash_test.c"

# ===========================================
# Error Handling Tests
# ===========================================
print_test_header "Error Handling Tests"

run_test "Error handler source exists" "test -f src/common/error_handler.c"
run_test "Error handler header exists" "test -f src/common/error_handler.h"

# ===========================================
# Recording Indicator Tests
# ===========================================
print_test_header "Recording Indicator Tests"

run_test "Recording indicator source exists" "test -f src/common/recording_indicator.c"
run_test "Recording indicator header exists" "test -f src/common/recording_indicator.h"

# Check if recording_indicator is in Makefile
run_test "Recording indicator in Makefile" "grep -q recording_indicator.c Makefile.linux"

# ===========================================
# Developer Tools Tests
# ===========================================
print_test_header "Developer Tools Tests"

run_test "Benchmark tool exists" "test -f tools/benchmark.sh && test -x tools/benchmark.sh"
run_test "First-run wizard exists" "test -f tools/first_run_wizard.sh && test -x tools/first_run_wizard.sh"
run_test "Profile tool exists" "test -f tools/profile.sh && test -x tools/profile.sh"
run_test "Leak detector exists" "test -f tools/leak_detector.sh && test -x tools/leak_detector.sh"
run_test "Crash test exists" "test -f tools/crash_test.c"
run_test "Model downloader exists" "test -f tools/download_model.sh && test -x tools/download_model.sh"
run_test "History viewer (CLI) exists" "test -f tools/history_viewer.sh && test -x tools/history_viewer.sh"
run_test "History viewer (GUI) exists" "test -f tools/history_viewer_gui.py && test -x tools/history_viewer_gui.py"
run_test "System monitor exists" "test -f tools/system_monitor.sh && test -x tools/system_monitor.sh"

# ===========================================
# Memory Leak Detection Tests
# ===========================================
print_test_header "Memory Leak Detection Tests"

run_test "Valgrind suppressions exist" "test -f tools/valgrind.supp"

if command -v valgrind &> /dev/null; then
    run_test "Valgrind available" "valgrind --version"
else
    skip_test "Valgrind available" "not installed"
fi

run_test "AddressSanitizer build target" "grep -q '^asan:' Makefile.linux"
run_test "Debug build target" "grep -q '^debug:' Makefile.linux"

# ===========================================
# Python Backend Tests
# ===========================================
print_test_header "Python Backend Tests"

if command -v python3 &> /dev/null; then
    run_test "Python 3 available" "python3 --version"

    run_test "Transcribe script exists" "test -f src/common/transcribe.py"

    # Check Python version
    PYTHON_VERSION=$(python3 --version 2>&1 | awk '{print $2}')
    PYTHON_MAJOR=$(echo "$PYTHON_VERSION" | cut -d. -f1)
    PYTHON_MINOR=$(echo "$PYTHON_VERSION" | cut -d. -f2)

    if [ "$PYTHON_MAJOR" -eq 3 ] && [ "$PYTHON_MINOR" -ge 10 ]; then
        run_test "Python version >= 3.10" "true"
    else
        run_test "Python version >= 3.10" "false"
    fi

    # Check for faster-whisper
    if python3 -c "import faster_whisper" 2>/dev/null; then
        run_test "faster-whisper installed" "true"
    else
        skip_test "faster-whisper installed" "not installed"
    fi
else
    skip_test "Python 3 available" "not installed"
fi

# ===========================================
# Transcription Timeout Tests
# ===========================================
print_test_header "Transcription Timeout Tests"

run_test "Timeout handler source exists" "test -f src/common/transcribe_timeout.c"
run_test "Timeout handler header exists" "test -f src/common/transcribe_timeout.h"

# ===========================================
# Configuration Tests
# ===========================================
print_test_header "Configuration Tests"

run_test "Settings module exists" "test -f src/common/settings.c"
run_test "Logging module exists" "test -f src/common/logging.c"

# ===========================================
# Documentation Tests
# ===========================================
print_test_header "Documentation Tests"

run_test "README exists" "test -f README.md"
run_test "CLAUDE.md exists" "test -f CLAUDE.md"
run_test "Wayland documentation exists" "test -f docs/WAYLAND_SETUP.md"

# ===========================================
# Git Tests
# ===========================================
print_test_header "Git Tests"

if [ -d .git ]; then
    run_test "Git repository" "true"
    run_test "Git status clean or working" "git status >/dev/null 2>&1"
else
    skip_test "Git repository" "not a git repo"
fi

# ===========================================
# Integration Tests
# ===========================================
print_test_header "Integration Tests"

# Check if build would work (compilation test)
if command -v gcc &> /dev/null && command -v make &> /dev/null; then
    if run_test "Clean build (dry-run)" "make -f Makefile.linux -n clean"; then
        # Try actual build if requested
        if [ "${RUN_BUILD_TEST:-0}" = "1" ]; then
            run_test "Full build" "make -f Makefile.linux clean && make -f Makefile.linux"
        else
            skip_test "Full build" "set RUN_BUILD_TEST=1 to enable"
        fi
    fi
else
    skip_test "Build tests" "gcc or make not available"
fi

# ===========================================
# Runtime Environment Tests
# ===========================================
print_test_header "Runtime Environment Tests"

run_test "Home directory writable" "test -w $HOME"

LOG_DIR="$HOME/.local/share/voice-to-text"
if [ -d "$LOG_DIR" ]; then
    run_test "Log directory exists" "true"
    run_test "Log directory writable" "test -w $LOG_DIR"
else
    skip_test "Log directory" "not created yet"
fi

# Check for required libraries
if pkg-config --exists portaudio-2.0; then
    run_test "PortAudio library available" "true"
else
    skip_test "PortAudio library available" "not installed"
fi

if pkg-config --exists gtk+-3.0; then
    run_test "GTK3 library available" "true"
else
    skip_test "GTK3 library available" "not installed"
fi

if pkg-config --exists libnotify; then
    run_test "libnotify available" "true"
else
    skip_test "libnotify available" "not installed"
fi

# ===========================================
# Summary
# ===========================================
echo ""
echo -e "${BOLD}=========================================="
echo "Test Summary"
echo "==========================================${NC}"
echo ""
echo -e "${GREEN}Passed:${NC}  $TESTS_PASSED"
echo -e "${RED}Failed:${NC}  $TESTS_FAILED"
echo -e "${YELLOW}Skipped:${NC} $TESTS_SKIPPED"
echo ""

TOTAL_TESTS=$((TESTS_PASSED + TESTS_FAILED + TESTS_SKIPPED))
if [ $TOTAL_TESTS -gt 0 ]; then
    PASS_RATE=$((TESTS_PASSED * 100 / TOTAL_TESTS))
    echo -e "Pass rate: ${BOLD}${PASS_RATE}%${NC}"
fi

# Show failed tests
if [ $TESTS_FAILED -gt 0 ]; then
    echo ""
    echo -e "${RED}Failed tests:${NC}"
    for test in "${FAILED_TESTS[@]}"; do
        echo "  - $test"
    done
fi

# Show skipped tests
if [ $TESTS_SKIPPED -gt 0 ] && [ "${SHOW_SKIPPED:-0}" = "1" ]; then
    echo ""
    echo -e "${YELLOW}Skipped tests:${NC}"
    for test in "${SKIPPED_TESTS[@]}"; do
        echo "  - $test"
    done
fi

echo ""

# Exit with appropriate code
if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
