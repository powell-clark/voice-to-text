#!/bin/bash
# Memory leak detector for Voice to Text
# Uses valgrind and AddressSanitizer to detect memory leaks

set -e

OUTPUT_DIR="${1:-./leak_results}"
TEST_DURATION="${2:-30}"

echo "=========================================="
echo "Voice to Text - Memory Leak Detector"
echo "=========================================="
echo ""
echo "Output: $OUTPUT_DIR"
echo "Test duration: ${TEST_DURATION}s"
echo ""

mkdir -p "$OUTPUT_DIR"

# Check for valgrind
if ! command -v valgrind &> /dev/null; then
    echo "ERROR: valgrind not found"
    echo "Install with: sudo apt install valgrind"
    exit 1
fi

# Check if binary exists
if [ ! -f "./vtt-linux" ]; then
    echo "ERROR: vtt-linux binary not found"
    echo "Build first with: make -f Makefile.linux"
    exit 1
fi

echo "=== Running Valgrind Leak Check ==="
echo ""
echo "This will run the application for ${TEST_DURATION}s under valgrind"
echo "Press Ctrl+C after testing to stop early"
echo ""
echo "Starting in 3 seconds..."
sleep 3

# Run valgrind with leak checking
VALGRIND_OPTS="--leak-check=full --show-leak-kinds=all --track-origins=yes --verbose --log-file=$OUTPUT_DIR/valgrind.log"

echo "Running: valgrind $VALGRIND_OPTS ./vtt-linux"
echo ""

# Run with timeout
timeout ${TEST_DURATION}s valgrind $VALGRIND_OPTS ./vtt-linux 2>&1 | tee "$OUTPUT_DIR/valgrind_output.txt" || true

echo ""
echo "=== Analyzing Results ==="
echo ""

# Parse valgrind output
if [ -f "$OUTPUT_DIR/valgrind.log" ]; then
    echo "Valgrind leak summary:"
    echo ""

    # Extract leak summary
    grep -A 10 "LEAK SUMMARY" "$OUTPUT_DIR/valgrind.log" || echo "No leak summary found"

    echo ""
    echo "Heap summary:"
    grep -A 5 "HEAP SUMMARY" "$OUTPUT_DIR/valgrind.log" || echo "No heap summary found"

    echo ""
    echo "Error summary:"
    grep "ERROR SUMMARY" "$OUTPUT_DIR/valgrind.log" || echo "No error summary found"

    # Count different leak types
    definitely_lost=$(grep "definitely lost:" "$OUTPUT_DIR/valgrind.log" | awk '{print $4}' | sed 's/,//g')
    indirectly_lost=$(grep "indirectly lost:" "$OUTPUT_DIR/valgrind.log" | awk '{print $4}' | sed 's/,//g')
    possibly_lost=$(grep "possibly lost:" "$OUTPUT_DIR/valgrind.log" | awk '{print $4}' | sed 's/,//g')
    still_reachable=$(grep "still reachable:" "$OUTPUT_DIR/valgrind.log" | awk '{print $4}' | sed 's/,//g')

    echo ""
    echo "=== Memory Leak Analysis ==="
    echo ""

    if [ -n "$definitely_lost" ] && [ "$definitely_lost" -gt 0 ]; then
        echo "⚠️  CRITICAL: $definitely_lost bytes definitely lost"
        echo "   These are real memory leaks that must be fixed"
    else
        echo "✓ No definite memory leaks detected"
    fi

    if [ -n "$indirectly_lost" ] && [ "$indirectly_lost" -gt 0 ]; then
        echo "⚠️  WARNING: $indirectly_lost bytes indirectly lost"
        echo "   These are caused by definitely lost blocks"
    fi

    if [ -n "$possibly_lost" ] && [ "$possibly_lost" -gt 0 ]; then
        echo "⚠️  POSSIBLE: $possibly_lost bytes possibly lost"
        echo "   These may be false positives (internal pointers)"
    fi

    if [ -n "$still_reachable" ] && [ "$still_reachable" -gt 0 ]; then
        echo "ℹ️  INFO: $still_reachable bytes still reachable"
        echo "   These are global/static allocations (usually OK)"
    fi

    echo ""
    echo "Full report saved to:"
    echo "  - $OUTPUT_DIR/valgrind.log (detailed leak report)"
    echo "  - $OUTPUT_DIR/valgrind_output.txt (application output)"

else
    echo "ERROR: Valgrind log not generated"
    exit 1
fi

echo ""
echo "=== Recommendations ==="
echo ""
echo "1. Fix all 'definitely lost' leaks immediately"
echo "2. Review 'possibly lost' - often false positives in GTK/GLib"
echo "3. 'Still reachable' is usually OK for global allocations"
echo "4. Run multiple times to confirm reproducibility"
echo ""
echo "To build with AddressSanitizer (faster alternative):"
echo "  make -f Makefile.linux clean"
echo "  make -f Makefile.linux CFLAGS='-fsanitize=address -g' LDFLAGS='-fsanitize=address'"
echo "  ./vtt-linux"
echo ""
echo "To suppress known GTK/GLib leaks, create a suppressions file:"
echo "  valgrind --leak-check=full --gen-suppressions=all ./vtt-linux 2>&1 | grep -A 5 '^{'"
echo ""
