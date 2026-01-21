// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract ModernContract {
    uint256 public balance;
    
    function add(uint256 amount) public {
        balance += amount; // Safe in 0.8.x - has overflow protection
    }
    
    function subtract(uint256 amount) public {
        balance -= amount; // Safe in 0.8.x - has underflow protection
    }
}
