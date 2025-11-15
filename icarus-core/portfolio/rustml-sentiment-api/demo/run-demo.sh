#!/bin/bash

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                                                               ║"
echo "║     🦀 RustML Sentiment API - LIVE DEMO                      ║"
echo "║                                                               ║"
echo "║     Running the REAL Rust backend with interactive frontend  ║"
echo "║                                                               ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed!"
    echo "Please install Docker first: https://docs.docker.com/get-docker/"
    exit 1
fi

# Check if Docker Compose is available
if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo "❌ Docker Compose is not installed!"
    echo "Please install Docker Compose: https://docs.docker.com/compose/install/"
    exit 1
fi

echo "🔧 Starting the demo..."
echo ""

# Use docker-compose or docker compose based on what's available
if command -v docker-compose &> /dev/null; then
    COMPOSE_CMD="docker-compose"
else
    COMPOSE_CMD="docker compose"
fi

# Build and start services
echo "📦 Building Rust API (this may take a few minutes on first run)..."
$COMPOSE_CMD build

echo ""
echo "🚀 Starting services..."
$COMPOSE_CMD up -d

echo ""
echo "⏳ Waiting for API to be ready..."
sleep 5

# Wait for API to be healthy
MAX_RETRIES=30
RETRY_COUNT=0
while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -s http://localhost:3000/health > /dev/null 2>&1; then
        echo "✅ API is ready!"
        break
    fi
    echo "   Waiting for API... ($((RETRY_COUNT+1))/$MAX_RETRIES)"
    sleep 2
    RETRY_COUNT=$((RETRY_COUNT+1))
done

if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
    echo "❌ API failed to start. Check logs with: docker-compose logs"
    exit 1
fi

echo ""
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                                                               ║"
echo "║  ✅ DEMO IS RUNNING!                                         ║"
echo "║                                                               ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""
echo "🌐 Open your browser and visit:"
echo ""
echo "   👉 http://localhost:8080"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  API ENDPOINTS (Direct Access):"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  Health Check: http://localhost:3000/health"
echo "  Stats:        http://localhost:3000/stats"
echo "  API Docs:     http://localhost:3000/"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  DEMO FEATURES:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  ✓ Real Rust backend processing requests"
echo "  ✓ Live sentiment analysis with VADER ML"
echo "  ✓ Text classification into 11+ categories"
echo "  ✓ Real-time API responses"
echo "  ✓ Performance metrics and monitoring"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  COMMANDS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  View logs:    $COMPOSE_CMD logs -f"
echo "  Stop demo:    $COMPOSE_CMD down"
echo "  Restart:      $COMPOSE_CMD restart"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "💡 Tip: Leave this terminal open. Press Ctrl+C to stop the demo."
echo ""

# Follow logs
$COMPOSE_CMD logs -f
