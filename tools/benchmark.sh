#!/bin/bash
# Voice to Text Model Benchmark
# Tests all models with sample audio and measures performance

set -e

SAMPLE_AUDIO="${1:-test_audio.wav}"
OUTPUT_FILE="${2:-benchmark_results.md}"

echo "=========================================="
echo "Voice to Text Model Benchmark"
echo "=========================================="
echo ""
echo "Sample audio: $SAMPLE_AUDIO"
echo "Output file: $OUTPUT_FILE"
echo ""

if [ ! -f "$SAMPLE_AUDIO" ]; then
    echo "ERROR: Sample audio file not found: $SAMPLE_AUDIO"
    echo ""
    echo "Usage: $0 <audio_file.wav> [output_file.md]"
    echo ""
    echo "To create a test file, record 10 seconds of speech:"
    echo "  arecord -d 10 -f S16_LE -r 16000 -c 1 test_audio.wav"
    exit 1
fi

# Get audio duration
DURATION=$(ffprobe -i "$SAMPLE_AUDIO" -show_entries format=duration -v quiet -of csv="p=0" 2>/dev/null || echo "unknown")
echo "Audio duration: ${DURATION}s"
echo ""

# Models to test
MODELS=(
    "CT2 tiny.en"
    "CT2 base.en"
    "CT2 small.en"
    "CT2 medium.en"
    "CT2 large-v3"
)

# Start benchmark results
cat > "$OUTPUT_FILE" << 'EOF'
# Voice to Text Benchmark Results

EOF

echo "Date: $(date)" >> "$OUTPUT_FILE"
echo "Audio duration: ${DURATION}s" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "| Model | Backend | Time (s) | Speed | Accuracy | Notes |" >> "$OUTPUT_FILE"
echo "|-------|---------|----------|-------|----------|-------|" >> "$OUTPUT_FILE"

# Run benchmarks
for model in "${MODELS[@]}"; do
    echo "Testing $model..."

    START=$(date +%s.%N)

    # Run transcription
    RESULT=$(python3 ../src/common/transcribe.py "$SAMPLE_AUDIO" "${model#CT2 }" "en" 2>&1 || echo "FAILED")

    END=$(date +%s.%N)
    ELAPSED=$(echo "$END - $START" | bc)
    SPEED=$(echo "scale=2; $DURATION / $ELAPSED" | bc)

    # Extract backend
    BACKEND=$(echo "$model" | grep -o "^[^ ]*")

    # Check for errors
    if echo "$RESULT" | grep -q "FAILED"; then
        echo "| $model | $BACKEND | - | - | - | ❌ Failed |" >> "$OUTPUT_FILE"
        echo "  FAILED"
    else
        # Truncate long results
        RESULT_SHORT=$(echo "$RESULT" | head -c 50)
        if [ ${#RESULT} -gt 50 ]; then
            RESULT_SHORT="${RESULT_SHORT}..."
        fi

        echo "| $model | $BACKEND | $ELAPSED | ${SPEED}x | ✓ | $RESULT_SHORT |" >> "$OUTPUT_FILE"
        echo "  OK (${ELAPSED}s, ${SPEED}x realtime)"
    fi

    sleep 1
done

echo "" >> "$OUTPUT_FILE"
echo "## System Info" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "- CPU: $(lscpu | grep 'Model name' | cut -d: -f2 | xargs)" >> "$OUTPUT_FILE"
echo "- RAM: $(free -h | grep Mem | awk '{print $2}')" >> "$OUTPUT_FILE"
echo "- GPU: $(lspci | grep -i vga | cut -d: -f3 | xargs || echo 'None')" >> "$OUTPUT_FILE"
echo "- CUDA: $(nvcc --version 2>/dev/null | grep release | awk '{print $5}' | tr -d ',' || echo 'Not installed')" >> "$OUTPUT_FILE"
echo "- Python: $(python3 --version)" >> "$OUTPUT_FILE"

echo ""
echo "=========================================="
echo "Benchmark complete!"
echo "Results written to: $OUTPUT_FILE"
echo "=========================================="

cat "$OUTPUT_FILE"
