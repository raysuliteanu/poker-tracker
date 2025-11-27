#!/usr/bin/env bash

# Script to analyze Docker images with dive
# Provides image size and efficiency metrics

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Docker Image Analysis with dive ===${NC}\n"

# Set DOCKER_HOST to work around context issues
export DOCKER_HOST=unix:///var/run/docker.sock

# Check if dive is installed
if ! command -v dive &>/dev/null; then
    echo -e "${RED}Error: dive is not installed${NC}"
    echo "Install with: brew install dive"
    exit 1
fi

# Images to analyze
IMAGES=(
    "poker-tracker-backend"
    "poker-tracker-frontend"
)

# Function to run dive analysis
analyze_image() {
    local image=$1
    echo -e "${YELLOW}Analyzing: ${image}${NC}"
    echo "----------------------------------------"

    # Get image size
    local size=$(docker images ${image} --format "{{.Size}}")
    echo -e "${BLUE}Image size: ${size}${NC}"

    # Run dive CI analysis
    if dive "${image}" --ci 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}\n"
    else
        echo -e "${RED}✗ FAIL (see details above)${NC}\n"
    fi
}

# Analyze each image
for image in "${IMAGES[@]}"; do
    if docker images "${image}" --format "{{.Repository}}" | grep -q "${image}"; then
        analyze_image "${image}"
    else
        echo -e "${RED}Image not found: ${image}${NC}\n"
    fi
done

echo -e "${BLUE}=== Analysis Complete ===${NC}"
