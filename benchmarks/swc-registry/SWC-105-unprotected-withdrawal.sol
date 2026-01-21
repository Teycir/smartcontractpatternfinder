// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// SWC-105: Unprotected Ether Withdrawal
contract UnprotectedWithdrawal {
    address public owner;
    
    constructor() {
        owner = msg.sender;
    }
    
    // VULNERABLE: Missing access control
    function withdrawAll() public {
        // Should have: require(msg.sender == owner);
        payable(msg.sender).transfer(address(this).balance);
    }
    
    receive() external payable {}
}
