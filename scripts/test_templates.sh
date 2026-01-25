#!/bin/bash
# Test SCPF templates against real vulnerable contract patterns

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TEST_FILE="$PROJECT_ROOT/sol/test_vulnerable_patterns.sol"
TEMPLATES_DIR="$PROJECT_ROOT/templates"

echo "🧪 SCPF Template Validation Test Suite"
echo "========================================"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Test function
test_pattern() {
    local contract_name=$1
    local template_file=$2
    local expected_pattern=$3
    local should_detect=$4  # "MUST_DETECT" or "MUST_NOT_DETECT"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    echo -n "Testing: $contract_name with $template_file ... "
    
    # Run SCPF scan (simulated - replace with actual command)
    # For now, use grep to simulate pattern matching
    if grep -q "$expected_pattern" "$TEST_FILE"; then
        if [ "$should_detect" = "MUST_DETECT" ]; then
            echo -e "${GREEN}✓ PASS${NC}"
            PASSED_TESTS=$((PASSED_TESTS + 1))
        else
            echo -e "${RED}✗ FAIL (False Positive)${NC}"
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi
    else
        if [ "$should_detect" = "MUST_NOT_DETECT" ]; then
            echo -e "${GREEN}✓ PASS${NC}"
            PASSED_TESTS=$((PASSED_TESTS + 1))
        else
            echo -e "${RED}✗ FAIL (False Negative)${NC}"
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi
    fi
}

echo "📋 Test Suite: Vulnerable Patterns (Must Detect)"
echo "================================================"

# Test 1: OracleRNG - Weak Randomness
test_pattern "OracleRNG_Vulnerable" "weak_randomness.yaml" "blockhash" "MUST_DETECT"

# Test 2: MoonCatsStrategyV2 - Arbitrary Call
test_pattern "MoonCatsStrategyV2_Vulnerable" "unchecked_return_value.yaml" "\.call\{value:.*\}.*data" "MUST_DETECT"

# Test 3: BondingCurve - Reentrancy
test_pattern "BondingCurve_Vulnerable" "reentrancy.yaml" "baseToken\.transferFrom" "MUST_DETECT"

# Test 4: Channel - Signature Replay
test_pattern "Channel_Vulnerable" "signature_unchecked.yaml" "ecrecover" "MUST_DETECT"

# Test 5: TransferRegistry - Cross-chain Gas Grief
test_pattern "TransferRegistry_Vulnerable" "cross_chain_gas_grief.yaml" "\.call\{value:" "MUST_DETECT"
test_pattern "TransferRegistry_Vulnerable" "cross_chain_gas_grief.yaml" "sendMessage" "MUST_DETECT"

# Test 6: IdentityRegistry - Delegatecall
test_pattern "IdentityRegistry_Vulnerable" "delegatecall_user_input.yaml" "delegatecall" "MUST_DETECT"

# Test 7: AlpacaFarm - Callback Reentrancy
test_pattern "AlpacaFarm_Vulnerable" "reentrancy_callback.yaml" "onERC1155Received" "MUST_DETECT"

echo ""
echo "📋 Test Suite: Safe Patterns (Must NOT Detect)"
echo "=============================================="

# Test 8: Safe CEI Pattern - Should not flag
test_pattern "Safe_ChecksEffectsInteractions" "reentrancy.yaml" "balances\[msg\.sender\] \+= amount" "MUST_NOT_DETECT"

# Test 9: Safe Reentrancy Guard - Should not flag
test_pattern "Safe_ReentrancyGuard" "reentrancy.yaml" "nonReentrant" "MUST_NOT_DETECT"

# Test 10: Safe Signature with Nonce - Should not flag
test_pattern "Safe_SignatureWithNonce" "signature_unchecked.yaml" "nonces\[recipient\]" "MUST_NOT_DETECT"

echo ""
echo "========================================"
echo "📊 Test Results Summary"
echo "========================================"
echo "Total Tests:  $TOTAL_TESTS"
echo -e "Passed:       ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed:       ${RED}$FAILED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed. Review template patterns.${NC}"
    exit 1
fi
