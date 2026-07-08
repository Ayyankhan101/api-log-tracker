#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

usage() {
    echo -e "${CYAN}Usage:${NC} ./deploy.sh <target> [options]"
    echo ""
    echo -e "${CYAN}Targets:${NC}"
    echo "  docker       Build and run locally with Docker Compose"
    echo "  fly          Deploy to Fly.io"
    echo "  railway      Deploy to Railway"
    echo "  render       Show Render deployment instructions"
    echo "  hf           Deploy to Hugging Face Spaces"
    echo "  local        Run locally without Docker"
    echo ""
    echo -e "${CYAN}Options:${NC}"
    echo "  --webhook <url>   Set webhook URL for anomaly reports"
    echo ""
    exit 1
}

ensure_csv_dir() {
    mkdir -p logs
    if [ ! -f logs/api_logs.csv ]; then
        echo "timestamp,id,source,method,endpoint,status_code,latency_ms,request_size,response_size,error" > logs/api_logs.csv
    fi
}

deploy_docker() {
    echo -e "${GREEN}[deploy] Building and starting with Docker Compose...${NC}"
    ensure_csv_dir
    docker compose up -d --build
    echo -e "${GREEN}[deploy] Daemon running at http://localhost:8080${NC}"
    echo -e "${CYAN}[deploy] Logs: docker compose logs -f${NC}"
    echo -e "${CYAN}[deploy] Stop:  docker compose down${NC}"
}

deploy_fly() {
    echo -e "${GREEN}[deploy] Deploying to Fly.io...${NC}"
    if ! command -v flyctl &>/dev/null; then
        echo -e "${RED}[deploy] flyctl not found. Install: https://fly.io/docs/handbook/installing-flyctl/${NC}"
        exit 1
    fi
    flyctl launch --copy-config --no-deploy
    flyctl deploy
    echo -e "${GREEN}[deploy] Deployed! Check: flyctl status${NC}"
}

deploy_railway() {
    echo -e "${GREEN}[deploy] Deploying to Railway...${NC}"
    if ! command -v railway &>/dev/null; then
        echo -e "${RED}[deploy] railway CLI not found. Install: npm i -g @railway/cli${NC}"
        exit 1
    fi
    railway login
    railway init
    railway up
    echo -e "${GREEN}[deploy] Deployed! Check: railway status${NC}"
}

deploy_render() {
    echo -e "${CYAN}[deploy] Render deployment:${NC}"
    echo "  1. Push this repo to GitHub"
    echo "  2. Go to https://dashboard.render.com/new"
    echo "  3. Select 'Blueprint' and connect your repo"
    echo "  4. Render will auto-detect render.yaml"
    echo "  5. Set environment variables in Render dashboard"
    echo "  6. Deploy!"
    echo ""
    echo -e "${CYAN}[deploy] render.yaml is already configured.${NC}"
}

deploy_hf() {
    echo -e "${CYAN}[deploy] Hugging Face Spaces deployment:${NC}"
    echo "  1. Create a new Space at https://huggingface.co/new-space"
    echo "  2. Select 'Docker' as the SDK"
    echo "  3. Push this repo to the Space's git remote"
    echo "  4. HF will auto-build the Dockerfile"
    echo "  5. Set secrets in Space Settings > Repository Secrets"
    echo ""
    echo -e "${CYAN}[deploy] The existing Dockerfile works as-is.${NC}"
}

deploy_local() {
    echo -e "${GREEN}[deploy] Running locally...${NC}"
    ensure_csv_dir
    cargo build --release
    echo -e "${GREEN}[deploy] Binary built at target/release/api_log_tracker${NC}"
    echo -e "${CYAN}[deploy] Run: ./target/release/api_log_tracker daemon${NC}"
    echo -e "${CYAN}[deploy] Or:  cargo run --release -- daemon${NC}"
}

WEBHOOK=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --webhook)
            WEBHOOK="$2"
            shift 2
            ;;
        *)
            TARGET="$1"
            shift
            ;;
    esac
done

TARGET="${TARGET:-}"
case "$TARGET" in
    docker)   deploy_docker ;;
    fly)      deploy_fly ;;
    railway)  deploy_railway ;;
    render)   deploy_render ;;
    hf)       deploy_hf ;;
    local)    deploy_local ;;
    *)        usage ;;
esac
