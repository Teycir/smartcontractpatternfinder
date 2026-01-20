// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract VulnerableBank {
    mapping(address => uint256) public balances;
    
    // Reentrancy vulnerability
    function withdraw(uint256 amount) public {
        require(balances[msg.sender] >= amount, "Insufficient balance");
        
        // External call before state update (VULNERABLE!)
        (bool success, ) = msg.sender.call{value: amount}("");
        require(success, "Transfer failed");
        
        // State update after external call (TOO LATE!)
        balances[msg.sender] -= amount;
    }
    
    // tx.origin vulnerability
    function transferOwnership(address newOwner) public {
        require(tx.origin == owner, "Not owner"); // VULNERABLE!
        owner = newOwner;
    }
    
    address public owner;
    
    // Unchecked call
    function sendEther(address payable recipient, uint256 amount) public {
        recipient.call{value: amount}(""); // Return value not checked!
    }
    
    // Delegatecall vulnerability
    function execute(address target, bytes memory data) public {
        target.delegatecall(data); // DANGEROUS!
    }
    
    receive() external payable {
        balances[msg.sender] += msg.value;
    }
}
