// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Test cases for precision_loss.yaml - Based on BalancerV2 ($120M) exploit

contract VulnerablePrecisionLoss {
    // SHOULD MATCH: division-before-multiplication (BalancerV2 pattern)
    function calculateShare(uint256 amount, uint256 total, uint256 multiplier) public pure returns (uint256) {
        return amount / total * multiplier; // Wrong order - precision loss
    }

    // SHOULD MATCH: decimal-mismatch-1e18-1e6
    function convertTokens(uint256 amount18, uint256 amount6) public pure returns (uint256) {
        return amount18 + amount6; // Mixing 18 and 6 decimals
    }

    // SHOULD MATCH: unsafe-downcast-uint256-uint128
    function storeValue(uint256 largeValue) public pure returns (uint128) {
        return uint128(largeValue); // No overflow check
    }

    // SHOULD MATCH: unchecked-large-block
    function complexCalculation(uint256 a, uint256 b, uint256 c) public pure returns (uint256) {
        unchecked {
            uint256 result = a * b;
            result = result + c;
            result = result - a;
            result = result * 2;
            return result / b;
        }
    }

    // SHOULD MATCH: percentage-calculation-truncation
    function calculateFee(uint256 amount) public pure returns (uint256) {
        return amount * 25 / 100; // Should be /10000 for 0.25%
    }

    // SHOULD MATCH: wei-to-token-no-decimals
    function convertWeiToToken(uint256 weiAmount) public pure returns (uint256) {
        return weiAmount / 1e18; // Assumes 18 decimals
    }

    // SHOULD MATCH: mulDiv-reimplementation
    function mulDiv(uint256 a, uint256 b, uint256 c) public pure returns (uint256) {
        return a * b / c; // Custom implementation without overflow protection
    }
}

contract SafePrecisionLoss {
    // SHOULD NOT MATCH: correct order
    function calculateShare(uint256 amount, uint256 total, uint256 multiplier) public pure returns (uint256) {
        return amount * multiplier / total; // Multiplication first
    }

    // SHOULD NOT MATCH: proper decimal handling
    function convertTokens(uint256 amount18) public pure returns (uint256) {
        return amount18 * 1e6 / 1e18; // Explicit conversion
    }

    // SHOULD NOT MATCH: safe downcast
    function storeValue(uint256 largeValue) public pure returns (uint128) {
        require(largeValue <= type(uint128).max, "Overflow");
        return uint128(largeValue);
    }

    // SHOULD NOT MATCH: using audited library
    function mulDivSafe(uint256 a, uint256 b, uint256 c) public pure returns (uint256) {
        return FullMath.mulDiv(a, b, c);
    }
}

library FullMath {
    function mulDiv(uint256 a, uint256 b, uint256 c) internal pure returns (uint256) {
        return a * b / c;
    }
}
