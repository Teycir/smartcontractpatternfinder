// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Test cases for precision_loss.yaml template

contract VulnerablePrecisionLoss {
    // SHOULD MATCH: division-before-multiplication
    function calculateShare(uint256 amount, uint256 total, uint256 multiplier) public pure returns (uint256) {
        return amount / total * multiplier; // Wrong order
    }

    // SHOULD MATCH: decimal-mismatch
    function convertTokens(uint256 amount) public pure returns (uint256) {
        // token1 has 18 decimals, token2 has 6 decimals
        return amount; // No conversion
    }

    // SHOULD MATCH: unsafe-downcast
    function storeValue(uint256 largeValue) public pure returns (uint128) {
        return uint128(largeValue); // No overflow check
    }

    // SHOULD MATCH: truncation-in-calculation
    function calculateFee(uint256 amount) public pure returns (uint256) {
        uint256 rate = 25; // 0.25%
        return amount * rate / 10000; // Truncates small amounts
    }

    // SHOULD MATCH: integer-division-loss
    function distribute(uint256 total, uint256 recipients) public pure returns (uint256) {
        return total / recipients; // Loses remainder
    }
}

contract SafePrecisionLoss {
    // SHOULD NOT MATCH: correct order
    function calculateShare(uint256 amount, uint256 total, uint256 multiplier) public pure returns (uint256) {
        return amount * multiplier / total;
    }

    // SHOULD NOT MATCH: proper decimal handling
    function convertTokens(uint256 amount) public pure returns (uint256) {
        return amount * 1e6 / 1e18; // Explicit conversion
    }

    // SHOULD NOT MATCH: safe downcast
    function storeValue(uint256 largeValue) public pure returns (uint128) {
        require(largeValue <= type(uint128).max, "Overflow");
        return uint128(largeValue);
    }
}
