#!/bin/bash
# Start Icarus Autonomous Mode
# Launches icarus-autonomous with CUDA support in the background

# CUDA environment setup
export PATH=/usr/local/cuda-12.6/bin:$PATH
export LD_LIBRARY_PATH=/usr/local/cuda-12.6/lib64:$LD_LIBRARY_PATH

# Binary location
BINARY_PATH="$(dirname "$0")/target/release/icarus-autonomous"
LOG_FILE="/tmp/icarus-autonomous.log"

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: icarus-autonomous binary not found at $BINARY_PATH"
    echo "Please build with: cargo build --release --features cuda"
    exit 1
fi

# Check if already running
if pgrep -f "icarus-autonomous" > /dev/null; then
    echo "Icarus autonomous mode is already running (PID: $(pgrep -f icarus-autonomous))"
    exit 1
fi

# Start in background with logging
echo "Starting Icarus autonomous mode..."
echo "Logs will be written to: $LOG_FILE"

nohup "$BINARY_PATH" >> "$LOG_FILE" 2>&1 &
PID=$!

# Wait a moment to check if it started successfully
sleep 2

if ps -p $PID > /dev/null; then
    echo "✅ Icarus autonomous mode started successfully (PID: $PID)"
    echo "To view logs: tail -f $LOG_FILE"
    echo "To stop: ./kill-icarus.sh or kill $PID"
else
    echo "❌ Failed to start Icarus autonomous mode"
    echo "Check logs at: $LOG_FILE"
    exit 1
fi
