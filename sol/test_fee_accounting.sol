// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Test cases for fee_accounting_flaw.yaml - Based on MTToken & FutureSwap exploits

contract VulnerableFeeAccounting {
    uint256 public fee;
    uint256 public feeRateWad; // WAD = 1e18
    uint256 public feeBasisPoints; // BPS = 1e4
    uint256 public balance;
    address[] public feeRecipients;
    uint256[] public feePercentages;

    // SHOULD MATCH: unbounded-fee-loop (MTToken pattern)
    function setMultipleFees(uint256[] memory fees) public {
        uint256 totalFee = 0;
        for (uint256 i = 0; i < fees.length; i++) {
            totalFee += fees[i]; // No check if sum > 100%
        }
        fee = totalFee;
    }

    // SHOULD MATCH: fee-unit-mismatch-wad-bps (FutureSwap pattern)
    function calculateFeeWadToBps(uint256 amount) public view returns (uint256) {
        return amount * feeRateWad / 1e18; // WAD treated as BPS
    }

    // SHOULD MATCH: fee-no-max-validation
    function setFee(uint256 _fee) public {
        fee = _fee; // No MAX_FEE check - allows 100%+
    }

    // SHOULD MATCH: transfer-fee-no-balance-check
    function transferWithFee(address to, uint256 amount) public {
        uint256 feeAmount = amount * fee / 100;
        uint256 netAmount = amount - feeAmount; // No balance check
        payable(to).transfer(netAmount);
    }

    // SHOULD MATCH: fee-recipient-array-no-sum-check
    function distributeFees(uint256 amount) public {
        for (uint256 i = 0; i < feeRecipients.length; i++) {
            uint256 feeAmount = amount * feePercentages[i] / 100;
            // No totalFee validation
        }
    }

    // SHOULD MATCH: fee-calculation-precision-loss
    function calculateFeeLoss(uint256 amount) public pure returns (uint256) {
        uint256 fee = amount / 10000 * 25; // Division first
        return fee;
    }
}

contract SafeFeeAccounting {
    uint256 public constant MAX_FEE = 1000; // 10%
    uint256 public fee;

    function setFee(uint256 _fee) public {
        require(_fee <= MAX_FEE, "Fee exceeds maximum");
        fee = _fee;
    }

    function calculateFee(uint256 amount) public view returns (uint256) {
        require(fee <= MAX_FEE, "Invalid fee");
        return amount * fee / 10000; // Multiplication first
    }

    function transferWithFee(address to, uint256 amount) public {
        uint256 feeAmount = amount * fee / 10000;
        require(balance >= amount, "Insufficient balance");
        uint256 netAmount = amount - feeAmount;
        payable(to).transfer(netAmount);
    }
}
