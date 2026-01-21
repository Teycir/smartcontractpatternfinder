// SPDX-License-Identifier: MIT
pragma solidity ^0.7.6;

contract LegacyContract {
    uint256 public balance;
    
    function add(uint256 amount) public {
        balance += amount; // VULNERABLE in 0.7.x - no overflow protection
    }
}
