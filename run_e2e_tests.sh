#!/bin/bash
# E2E Test Runner for Hanzo Net

set -e

echo "🚀 Hanzo Net E2E Test Suite"
echo "=========================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check dependencies
echo -e "${YELLOW}Checking dependencies...${NC}"
python3 -c "import qrcode" || pip install qrcode[pil]
python3 -c "import pytest" || pip install pytest pytest-asyncio
python3 -c "import playwright" || pip install playwright

# Install Playwright browsers if needed
if [ ! -d "$HOME/.cache/ms-playwright" ]; then
    echo -e "${YELLOW}Installing Playwright browsers...${NC}"
    playwright install
fi

# Start the server in background
echo -e "${YELLOW}Starting Hanzo Net server...${NC}"
python3 -m net.main --disable-tui &
SERVER_PID=$!
sleep 5

# Function to cleanup on exit
cleanup() {
    echo -e "${YELLOW}Cleaning up...${NC}"
    if [ ! -z "$SERVER_PID" ]; then
        kill $SERVER_PID 2>/dev/null || true
    fi
}
trap cleanup EXIT

# Run tests
echo -e "${GREEN}Running QR Code Tests...${NC}"
pytest tests/test_e2e_qr_mobile.py::TestQRCodeGeneration -v

echo -e "${GREEN}Running Mobile Detection Tests...${NC}"
pytest tests/test_e2e_qr_mobile.py::TestMobileDetection -v

echo -e "${GREEN}Running Network Integration Tests...${NC}"
pytest tests/test_e2e_qr_mobile.py::TestDistributedNetwork -v

echo -e "${GREEN}Running Browser E2E Tests...${NC}"
python3 tests/test_browser_e2e.py

echo -e "${GREEN}Running Playwright Mobile Tests...${NC}"
npx playwright test tests/mobile.spec.js --reporter=list

# Generate test report
echo -e "${GREEN}Generating Test Report...${NC}"
pytest tests/test_e2e_qr_mobile.py --html=test-report.html --self-contained-html || true

echo -e "${GREEN}✅ All E2E Tests Complete!${NC}"
echo ""
echo "Test Report: test-report.html"
echo "Server was running on: http://localhost:52415"