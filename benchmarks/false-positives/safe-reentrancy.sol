// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// SAFE: Properly protected against reentrancy
contract SafeWithdrawal {
    mapping(address => uint) private userBalances;
    bool private locked;

    modifier noReentrancy() {
        require(!locked, "No reentrancy");
        locked = true;
        _;
        locked = false;
    }

    // SAFE: Checks-Effects-Interactions pattern
    function withdrawBalance() public noReentrancy {
        uint amountToWithdraw = userBalances[msg.sender];
        require(amountToWithdraw > 0, "No balance");
        
        // Effects: Update state BEFORE external call
        userBalances[msg.sender] = 0;
        
        // Interactions: External call AFTER state change
        (bool success, ) = msg.sender.call{value: amountToWithdraw}("");
        require(success, "Transfer failed");
    }

    function deposit() public payable {
        userBalances[msg.sender] += msg.value;
    }

    function getBalance() public view returns (uint) {
        return userBalances[msg.sender];
    }
}
