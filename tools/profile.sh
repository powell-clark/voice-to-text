#!/bin/bash
# Performance profiler for Voice to Text
# Monitors CPU, memory, and GPU usage during transcription

set -e

SAMPLE_AUDIO="${1:-test_audio.wav}"
MODEL="${2:-small.en}"
OUTPUT_DIR="${3:-./profiling_results}"

echo "=========================================="
echo "Voice to Text Performance Profiler"
echo "=========================================="
echo ""
echo "Audio file: $SAMPLE_AUDIO"
echo "Model: $MODEL"
echo "Output: $OUTPUT_DIR"
echo ""

if [ ! -f "$SAMPLE_AUDIO" ]; then
    echo "ERROR: Audio file not found: $SAMPLE_AUDIO"
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

# Start monitoring
MONITOR_PID=""

monitor_resources() {
    local output_file="$1"
    echo "timestamp,cpu_percent,mem_mb,gpu_util,gpu_mem_mb" > "$output_file"

    while true; do
        timestamp=$(date +%s.%N)

        # CPU usage (top)
        cpu=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)

        # Memory usage
        mem=$(free -m | grep Mem | awk '{print $3}')

        # GPU usage (if NVIDIA)
        if command -v nvidia-smi &> /dev/null; then
            gpu_util=$(nvidia-smi --query-gpu=utilization.gpu --format=csv,noheader,nounits | head -1)
            gpu_mem=$(nvidia-smi --query-gpu=memory.used --format=csv,noheader,nounits | head -1)
        else
            gpu_util=0
            gpu_mem=0
        fi

        echo "$timestamp,$cpu,$mem,$gpu_util,$gpu_mem" >> "$output_file"
        sleep 0.1
    done
}

# Start background monitoring
monitor_resources "$OUTPUT_DIR/metrics.csv" &
MONITOR_PID=$!

echo "Monitoring started (PID: $MONITOR_PID)"
echo "Running transcription..."
echo ""

# Run transcription with timing
START=$(date +%s.%N)

python3 ../src/common/transcribe.py "$SAMPLE_AUDIO" "$MODEL" "en" > "$OUTPUT_DIR/transcription.txt" 2>&1

END=$(date +%s.%N)
ELAPSED=$(echo "$END - $START" | bc)

# Stop monitoring
kill $MONITOR_PID 2>/dev/null || true
wait $MONITOR_PID 2>/dev/null || true

echo ""
echo "Transcription complete in ${ELAPSED}s"
echo ""

# Analyze results
echo "Analyzing performance metrics..."

if command -v python3 &> /dev/null; then
    python3 << 'EOF'
import sys
import csv

metrics_file = sys.argv[1] if len(sys.argv) > 1 else "./profiling_results/metrics.csv"

with open(metrics_file) as f:
    reader = csv.DictReader(f)
    data = list(reader)

if not data:
    print("No metrics collected")
    sys.exit(1)

cpu_vals = [float(row['cpu_percent']) for row in data if row['cpu_percent']]
mem_vals = [float(row['mem_mb']) for row in data if row['mem_mb']]
gpu_vals = [float(row['gpu_util']) for row in data if row['gpu_util'] and float(row['gpu_util']) > 0]
gpu_mem_vals = [float(row['gpu_mem_mb']) for row in data if row['gpu_mem_mb'] and float(row['gpu_mem_mb']) > 0]

print("\n=== Performance Summary ===\n")
print(f"CPU Usage:")
print(f"  Average: {sum(cpu_vals)/len(cpu_vals):.1f}%")
print(f"  Peak:    {max(cpu_vals):.1f}%")
print()
print(f"Memory Usage:")
print(f"  Average: {sum(mem_vals)/len(mem_vals):.0f} MB")
print(f"  Peak:    {max(mem_vals):.0f} MB")
print()

if gpu_vals:
    print(f"GPU Usage:")
    print(f"  Average: {sum(gpu_vals)/len(gpu_vals):.1f}%")
    print(f"  Peak:    {max(gpu_vals):.1f}%")
    print()
    print(f"GPU Memory:")
    print(f"  Average: {sum(gpu_mem_vals)/len(gpu_mem_vals):.0f} MB")
    print(f"  Peak:    {max(gpu_mem_vals):.0f} MB")
else:
    print("GPU: Not used (CPU-only mode)")

print()
print(f"Samples collected: {len(data)}")
print(f"Sampling rate: ~{len(data)/float(sys.argv[2]):.1f} Hz" if len(sys.argv) > 2 else "")

EOF
python3 -c "import sys; print()" "$OUTPUT_DIR/metrics.csv" "$ELAPSED"
fi

echo ""
echo "Results saved to: $OUTPUT_DIR/"
echo "  - metrics.csv: Raw performance data"
echo "  - transcription.txt: Transcription output"
echo ""
echo "To visualize:"
echo "  python3 -c 'import pandas as pd; import matplotlib.pyplot as plt; df=pd.read_csv(\"$OUTPUT_DIR/metrics.csv\"); df.plot(); plt.show()'"
