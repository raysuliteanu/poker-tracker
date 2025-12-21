#!/usr/bin/env bash
set -euo pipefail

# Script to run k6 performance tests against the backend API
# Uses docker-compose.perf.yml for isolated performance testing environment

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting performance test environment...${NC}"
docker compose -f docker-compose.yml -f docker-compose.perf.yml up -d --build

# Wait for services to be ready
echo -e "${YELLOW}Waiting for backend to be healthy...${NC}"
timeout 60 bash -c 'until wget -qO /dev/null http://localhost:8080/api/health 2>/dev/null; do sleep 2; done' || {
    echo -e "${RED}Backend failed to start. Checking logs:${NC}"
    docker compose -f docker-compose.yml -f docker-compose.perf.yml logs backend
    exit 1
}

VUS=${1:-100}
echo -e "${GREEN}Backend is ready. Running k6 performance tests with ${VUS} VUs...${NC}"
k6 run -e K6_VUS=${VUS} performance-test.js

TEST_EXIT_CODE=$?

# Tear down environment
echo -e "${YELLOW}Tearing down performance test environment...${NC}"
docker compose -f docker-compose.yml -f docker-compose.perf.yml down

if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}Performance tests completed successfully!${NC}"
else
    echo -e "${RED}Performance tests failed with exit code $TEST_EXIT_CODE${NC}"
    exit $TEST_EXIT_CODE
fi
