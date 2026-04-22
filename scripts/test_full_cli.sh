#!/bin/bash
# Full CLI test for scan and 0-day import flows

set -e

echo "════════════════════════════════════════════════════════════════"
echo "  SCPF Full CLI Test - Scan + 0-Day Import"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Check if ETHERSCAN_API_KEY is set
if [ -z "$ETHERSCAN_API_KEY" ]; then
    echo -e "${YELLOW}⚠️  Warning: ETHERSCAN_API_KEY not set${NC}"
    echo "   Contract scanning will be limited to 0-day import only"
    echo ""
fi

# Build the project
echo -e "${CYAN}📦 Building SCPF...${NC}"
cargo build --release -p scpf-cli
echo -e "${GREEN}✓ Build complete${NC}"
echo ""

# Create test report directory
TIMESTAMP=$(date +%s)
TEST_REPORT_DIR="/tmp/scpf_test_$TIMESTAMP"
mkdir -p "$TEST_REPORT_DIR"
export SCPF_REPORT_DIR="$TEST_REPORT_DIR"

echo -e "${CYAN}📁 Test report directory: $TEST_REPORT_DIR${NC}"
echo ""

# Test 1: 0-Day Import Only (no API key needed)
echo "════════════════════════════════════════════════════════════════"
echo -e "${CYAN}TEST 1: 0-Day Import (30 days)${NC}"
echo "════════════════════════════════════════════════════════════════"
echo ""

./target/release/scpf scan \
    --fetch-zero-day 30 \
    --templates templates \
    --chains ethereum \
    --pages 1 \
    --concurrency 1

echo ""
echo -e "${GREEN}✓ 0-Day import complete${NC}"
echo ""

# Check if 0day_summary.md was created
if [ -f "$TEST_REPORT_DIR/0day_summary.md" ]; then
    echo -e "${GREEN}✓ 0day_summary.md created${NC}"
    echo -e "${CYAN}📄 File size: $(du -h "$TEST_REPORT_DIR/0day_summary.md" | cut -f1)${NC}"
    echo -e "${CYAN}📊 Preview (first 20 lines):${NC}"
    echo "────────────────────────────────────────────────────────────────"
    head -20 "$TEST_REPORT_DIR/0day_summary.md"
    echo "────────────────────────────────────────────────────────────────"
else
    echo -e "${RED}✗ 0day_summary.md NOT found${NC}"
    exit 1
fi

echo ""

# Test 2: Full Scan with Contract Addresses (requires API key)
if [ -n "$ETHERSCAN_API_KEY" ]; then
    echo "════════════════════════════════════════════════════════════════"
    echo -e "${CYAN}TEST 2: Full Scan with Contract Addresses${NC}"
    echo "════════════════════════════════════════════════════════════════"
    echo ""
    
    # Create new report directory for second test
    TIMESTAMP2=$(date +%s)
    TEST_REPORT_DIR2="/tmp/scpf_test_$TIMESTAMP2"
    mkdir -p "$TEST_REPORT_DIR2"
    export SCPF_REPORT_DIR="$TEST_REPORT_DIR2"
    
    echo -e "${CYAN}📁 Test report directory: $TEST_REPORT_DIR2${NC}"
    echo ""
    
    # Scan a known contract (USDT on Ethereum)
    ./target/release/scpf scan \
        0xdac17f958d2ee523a2206206994597c13d831ec7 \
        --templates templates \
        --chains ethereum \
        --pages 1 \
        --concurrency 2 \
        --min-severity high
    
    echo ""
    echo -e "${GREEN}✓ Contract scan complete${NC}"
    echo ""
    
    # Check if vuln_summary.md was created
    if [ -f "$TEST_REPORT_DIR2/vuln_summary.md" ]; then
        echo -e "${GREEN}✓ vuln_summary.md created${NC}"
        echo -e "${CYAN}📄 File size: $(du -h "$TEST_REPORT_DIR2/vuln_summary.md" | cut -f1)${NC}"
        echo -e "${CYAN}📊 Preview (first 30 lines):${NC}"
        echo "────────────────────────────────────────────────────────────────"
        head -30 "$TEST_REPORT_DIR2/vuln_summary.md"
        echo "────────────────────────────────────────────────────────────────"
    else
        echo -e "${RED}✗ vuln_summary.md NOT found${NC}"
        exit 1
    fi
    
    echo ""
    
    # List all files in report directory
    echo -e "${CYAN}📂 Report directory contents:${NC}"
    ls -lh "$TEST_REPORT_DIR2"
    echo ""
else
    echo "════════════════════════════════════════════════════════════════"
    echo -e "${YELLOW}TEST 2: Skipped (no ETHERSCAN_API_KEY)${NC}"
    echo "════════════════════════════════════════════════════════════════"
    echo ""
fi

# Test 3: Combined Flow (0-day + scan)
if [ -n "$ETHERSCAN_API_KEY" ]; then
    echo "════════════════════════════════════════════════════════════════"
    echo -e "${CYAN}TEST 3: Combined Flow (0-Day + Scan)${NC}"
    echo "════════════════════════════════════════════════════════════════"
    echo ""
    
    # Create new report directory for third test
    TIMESTAMP3=$(date +%s)
    TEST_REPORT_DIR3="/tmp/scpf_test_$TIMESTAMP3"
    mkdir -p "$TEST_REPORT_DIR3"
    export SCPF_REPORT_DIR="$TEST_REPORT_DIR3"
    
    echo -e "${CYAN}📁 Test report directory: $TEST_REPORT_DIR3${NC}"
    echo ""
    
    # Scan with both 0-day import and contract scanning
    ./target/release/scpf scan \
        0xdac17f958d2ee523a2206206994597c13d831ec7 \
        --fetch-zero-day 30 \
        --templates templates \
        --chains ethereum \
        --pages 1 \
        --concurrency 2 \
        --min-severity high
    
    echo ""
    echo -e "${GREEN}✓ Combined scan complete${NC}"
    echo ""
    
    # Check both files
    echo -e "${CYAN}📂 Checking report files:${NC}"
    
    if [ -f "$TEST_REPORT_DIR3/0day_summary.md" ]; then
        echo -e "${GREEN}✓ 0day_summary.md created ($(du -h "$TEST_REPORT_DIR3/0day_summary.md" | cut -f1))${NC}"
    else
        echo -e "${RED}✗ 0day_summary.md NOT found${NC}"
    fi
    
    if [ -f "$TEST_REPORT_DIR3/vuln_summary.md" ]; then
        echo -e "${GREEN}✓ vuln_summary.md created ($(du -h "$TEST_REPORT_DIR3/vuln_summary.md" | cut -f1))${NC}"
    else
        echo -e "${RED}✗ vuln_summary.md NOT found${NC}"
    fi
    
    echo ""
    echo -e "${CYAN}📂 Full report directory contents:${NC}"
    ls -lh "$TEST_REPORT_DIR3"
    echo ""
else
    echo "════════════════════════════════════════════════════════════════"
    echo -e "${YELLOW}TEST 3: Skipped (no ETHERSCAN_API_KEY)${NC}"
    echo "════════════════════════════════════════════════════════════════"
    echo ""
fi

# Summary
echo "════════════════════════════════════════════════════════════════"
echo -e "${GREEN}✓ All Tests Complete${NC}"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo -e "${CYAN}Test Results:${NC}"
echo "  • Test 1 (0-Day Only): $TEST_REPORT_DIR"
if [ -n "$ETHERSCAN_API_KEY" ]; then
    echo "  • Test 2 (Scan Only): $TEST_REPORT_DIR2"
    echo "  • Test 3 (Combined): $TEST_REPORT_DIR3"
else
    echo "  • Test 2 (Scan Only): Skipped (no API key)"
    echo "  • Test 3 (Combined): Skipped (no API key)"
fi
echo ""
echo -e "${CYAN}Cleanup:${NC}"
echo "  To remove test reports: rm -rf /tmp/scpf_test_*"
echo ""
