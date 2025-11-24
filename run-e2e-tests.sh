#!/usr/bin/env bash
set -euo pipefail

# Script to run Playwright e2e tests against the full stack
# Uses docker-compose.e2e.yml for isolated e2e testing environment

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting e2e test environment...${NC}"
docker compose -f docker-compose.e2e.yml up --build --abort-on-container-exit --exit-code-from playwright

TEST_EXIT_CODE=$?

# Tear down environment
echo -e "${YELLOW}Tearing down e2e test environment...${NC}"
docker compose -f docker-compose.e2e.yml down

if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}E2E tests completed successfully!${NC}"
    echo -e "${YELLOW}Test reports available at:${NC}"
    echo -e "  - frontend/playwright-report/"
    echo -e "  - frontend/test-results/"
else
    echo -e "${RED}E2E tests failed with exit code $TEST_EXIT_CODE${NC}"
    echo -e "${YELLOW}Check test reports for details:${NC}"
    echo -e "  - frontend/playwright-report/"
    echo -e "  - frontend/test-results/"
    exit $TEST_EXIT_CODE
fi
