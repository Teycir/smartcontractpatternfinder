// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// SWC-104: Unchecked Call Return Value
contract UncheckedCall {
    mapping(address => uint256) public balances;
    
    function deposit() public payable {
        balances[msg.sender] += msg.value;
    }
    
    // VULNERABLE: Ignoring return value of call
    function withdraw(uint256 amount) public {
        require(balances[msg.sender] >= amount);
        balances[msg.sender] -= amount;
        
        // VULNERABLE: Return value not checked
        payable(msg.sender).call{value: amount}("");
    }
    
    // VULNERABLE: Ignoring return value of send
    function withdrawSend(uint256 amount) public {
        require(balances[msg.sender] >= amount);
        balances[msg.sender] -= amount;
        
        // VULNERABLE: Return value not checked
        payable(msg.sender).send(amount);
    }
}
