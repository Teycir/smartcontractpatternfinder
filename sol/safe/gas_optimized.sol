// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract GasOptimized {
    // Safe: Intentional gas optimization with bounds
    function optimizedMath(uint256 x, uint256 y, uint256 z) public pure returns (uint256) {
        require(x < 1e18 && y > 0 && z < 1e18, "Bounds check");
        return x / y * z; // Intentional for gas, bounds checked
    }
    
    // Safe: Comment example
    function example() public pure {
        // uint256 fee = amount / 100 * rate; // Example of vulnerable pattern
        uint256 fee = 0; // Actual safe implementation
    }
}
