#!/bin/bash
# Stop Icarus Autonomous Mode
# Gracefully shuts down icarus-autonomous

# Find the process
PID=$(pgrep -f "icarus-autonomous")

if [ -z "$PID" ]; then
    echo "Icarus autonomous mode is not running"
    exit 0
fi

echo "Stopping Icarus autonomous mode (PID: $PID)..."

# Try graceful shutdown first (SIGTERM)
kill -TERM $PID

# Wait up to 10 seconds for graceful shutdown
for i in {1..10}; do
    if ! ps -p $PID > /dev/null 2>&1; then
        echo "✅ Icarus autonomous mode stopped gracefully"
        exit 0
    fi
    sleep 1
done

# If still running, force kill (SIGKILL)
if ps -p $PID > /dev/null 2>&1; then
    echo "Process did not stop gracefully, force killing..."
    kill -KILL $PID
    sleep 1

    if ps -p $PID > /dev/null 2>&1; then
        echo "❌ Failed to stop Icarus autonomous mode"
        exit 1
    else
        echo "✅ Icarus autonomous mode stopped (force kill)"
        exit 0
    fi
fi
