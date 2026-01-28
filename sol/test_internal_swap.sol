// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract HoneypotToken {
    // This should NOT be flagged (internal function)
    function swap() internal {
        for (uint256 believers = 0; believers < 1; believers++) {
            // Fake swap logic
        }
    }
    
    // This should NOT be flagged (private function)
    function swap(uint256 amount) private {
        // Fake swap logic
    }
}

contract LegitimateAMM {
    // This SHOULD be flagged (public function without slippage)
    function swap(uint256 amountIn) public {
        // Missing slippage protection
        uint256 amountOut = amountIn * 2;
    }
    
    // This should NOT be flagged (has slippage protection)
    function swap(uint256 amountIn, uint256 amountOutMin) external {
        uint256 amountOut = amountIn * 2;
        require(amountOut >= amountOutMin, "Slippage");
    }
}
