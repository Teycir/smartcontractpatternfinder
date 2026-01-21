// SPDX-License-Identifier: MIT
pragma solidity ^0.4.18;

// SWC-101: Integer Overflow and Underflow
contract IntegerOverflow {
    mapping(address => uint256) public balances;
    
    function deposit() public payable {
        // VULNERABLE: No overflow check
        balances[msg.sender] += msg.value;
    }
    
    function withdraw(uint256 amount) public {
        // VULNERABLE: No underflow check
        balances[msg.sender] -= amount;
        msg.sender.transfer(amount);
    }
    
    function batchTransfer(address[] recipients, uint256 value) public {
        uint256 total = recipients.length * value;
        require(balances[msg.sender] >= total);
        
        // VULNERABLE: Overflow in multiplication
        for (uint256 i = 0; i < recipients.length; i++) {
            balances[recipients[i]] += value;
        }
        balances[msg.sender] -= total;
    }
}
