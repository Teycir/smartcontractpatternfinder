// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// SWC-106: Unprotected SELFDESTRUCT Instruction
contract UnprotectedSelfdestruct {
    address public owner;
    
    constructor() {
        owner = msg.sender;
    }
    
    // VULNERABLE: No access control on selfdestruct
    function kill(address payable recipient) public {
        // Should have: require(msg.sender == owner);
        selfdestruct(recipient);
    }
    
    receive() external payable {}
}
