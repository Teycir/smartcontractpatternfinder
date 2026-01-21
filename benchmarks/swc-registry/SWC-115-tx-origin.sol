// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// SWC-115: Authorization through tx.origin
contract TxOriginVulnerable {
    address public owner;
    
    constructor() {
        owner = msg.sender;
    }
    
    // VULNERABLE: Using tx.origin for authentication
    function transferOwnership(address newOwner) public {
        require(tx.origin == owner, "Not owner");
        owner = newOwner;
    }
    
    // VULNERABLE: tx.origin check can be bypassed via phishing
    function withdraw() public {
        require(tx.origin == owner, "Not owner");
        payable(owner).transfer(address(this).balance);
    }
    
    receive() external payable {}
}
