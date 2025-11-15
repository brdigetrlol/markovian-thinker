#!/bin/bash
# watch-build.sh - Monitor Cargo build progress and update status line
# Usage: ./watch-build.sh [background_process_id]

BASH_ID="${1:-}"
BUILD_DIR="/mnt/c/Users/brdig/Documents/DriveSync/Software/repos-pxl/mcp/workspace/icarus-core"

# ANSI color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Status line update function
update_status() {
    local status="$1"
    echo -ne "\r${BLUE}[BUILD]${NC} ${status}                    "
}

# Parse compilation progress from cargo output
parse_progress() {
    local line="$1"

    # Extract "Compiling X/Y" pattern
    if [[ "$line" =~ Compiling.*\(([0-9]+)/([0-9]+)\) ]]; then
        local current="${BASH_REMATCH[1]}"
        local total="${BASH_REMATCH[2]}"
        local percent=$((current * 100 / total))
        echo "${percent}% (${current}/${total})"
        return 0
    fi

    # Extract package name being compiled
    if [[ "$line" =~ Compiling[[:space:]]+([^[:space:]]+) ]]; then
        echo "Compiling ${BASH_REMATCH[1]}"
        return 0
    fi

    # Check for completion
    if [[ "$line" =~ Finished ]]; then
        echo "Finished"
        return 0
    fi

    # Check for errors
    if [[ "$line" =~ error ]]; then
        echo "Error detected"
        return 1
    fi

    return 2
}

# Main monitoring loop
if [ -n "$BASH_ID" ]; then
    echo "Monitoring background process: $BASH_ID"

    # Poll the background process
    while true; do
        # Get latest output (this would require BashOutput tool in actual usage)
        # For standalone script, we'll tail the build log if available

        update_status "${YELLOW}Checking build status...${NC}"
        sleep 2
    done
else
    # Monitor live cargo build
    cd "$BUILD_DIR" || exit 1

    echo "Starting monitored build..."
    ~/.cargo/bin/cargo build --bin icarus-mcp --release 2>&1 | while IFS= read -r line; do
        progress=$(parse_progress "$line")
        status=$?

        if [ $status -eq 0 ]; then
            # Successfully parsed progress
            if [[ "$progress" == "Finished" ]]; then
                update_status "${GREEN}✓ Build complete!${NC}"
                echo ""
                break
            elif [[ "$progress" =~ ^[0-9]+% ]]; then
                update_status "${YELLOW}${progress}${NC}"
            else
                update_status "${BLUE}${progress}${NC}"
            fi
        elif [ $status -eq 1 ]; then
            # Error detected
            update_status "${RED}✗ Build failed${NC}"
            echo ""
            echo "$line"
        else
            # Other output, just show it
            echo "$line"
        fi
    done

    echo ""
    echo "Build monitoring complete."

    # Check if binary exists
    if [ -f "target/release/icarus-mcp" ]; then
        echo -e "${GREEN}✓ Binary created: target/release/icarus-mcp${NC}"
        ls -lh target/release/icarus-mcp
    else
        echo -e "${RED}✗ Binary not found${NC}"
    fi
fi
