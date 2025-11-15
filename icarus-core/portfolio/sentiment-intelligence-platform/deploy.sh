#!/bin/bash

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                                                               ║"
echo "║     🚀 Deploying Sentiment Intelligence Platform            ║"
echo "║                                                               ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

# Function to check command exists
command_exists() {
    command -v "$1" &> /dev/null
}

# Check Docker
if ! command_exists docker; then
    echo "❌ Docker is not installed!"
    echo "Please install Docker: https://docs.docker.com/get-docker/"
    exit 1
fi

# Check Docker Compose
if ! command_exists docker-compose && ! docker compose version &> /dev/null; then
    echo "❌ Docker Compose is not installed!"
    echo "Please install Docker Compose: https://docs.docker.com/compose/install/"
    exit 1
fi

# Determine compose command
if command_exists docker-compose; then
    COMPOSE_CMD="docker-compose"
else
    COMPOSE_CMD="docker compose"
fi

echo "📋 Deployment Options:"
echo ""
echo "  1) Frontend only (demo mode)"
echo "  2) Full stack (frontend + Rust API)"
echo "  3) Production build (optimized)"
echo "  4) Development mode (with hot reload)"
echo ""
read -p "Select option (1-4): " option

case $option in
    1)
        echo ""
        echo "🎨 Deploying frontend only (demo mode)..."
        $COMPOSE_CMD up -d frontend
        ;;
    2)
        echo ""
        echo "🔧 Deploying full stack (frontend + API)..."
        $COMPOSE_CMD --profile with-api up -d
        ;;
    3)
        echo ""
        echo "⚡ Building production-optimized images..."
        $COMPOSE_CMD build --no-cache
        $COMPOSE_CMD up -d frontend
        ;;
    4)
        echo ""
        echo "🛠️  Starting development mode..."
        echo "   This will run containers in foreground with logs"
        $COMPOSE_CMD up frontend
        ;;
    *)
        echo "❌ Invalid option"
        exit 1
        ;;
esac

if [ $? -eq 0 ]; then
    echo ""
    echo "╔═══════════════════════════════════════════════════════════════╗"
    echo "║                                                               ║"
    echo "║  ✅ DEPLOYMENT SUCCESSFUL!                                   ║"
    echo "║                                                               ║"
    echo "╚═══════════════════════════════════════════════════════════════╝"
    echo ""
    echo "🌐 Application is running at:"
    echo ""
    echo "   👉 Frontend: http://localhost:8080"

    if [ "$option" == "2" ]; then
        echo "   👉 API: http://localhost:3000"
        echo "   👉 API Health: http://localhost:3000/health"
    fi

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  USEFUL COMMANDS:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "  View logs:        $COMPOSE_CMD logs -f"
    echo "  Stop services:    $COMPOSE_CMD down"
    echo "  Restart:          $COMPOSE_CMD restart"
    echo "  View status:      $COMPOSE_CMD ps"
    echo "  Rebuild:          $COMPOSE_CMD build"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
else
    echo ""
    echo "❌ Deployment failed! Check the errors above."
    exit 1
fi
