// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// SWC-107: Reentrancy
contract ReentrancyVulnerable {
    mapping(address => uint) private userBalances;

    function withdrawBalance() public {
        uint amountToWithdraw = userBalances[msg.sender];
        
        // VULNERABLE: External call before state change
        (bool success, ) = msg.sender.call{value: amountToWithdraw}("");
        require(success);
        
        // State change after external call
        userBalances[msg.sender] = 0;
    }

    function deposit() public payable {
        userBalances[msg.sender] += msg.value;
    }

    function getBalance() public view returns (uint) {
        return userBalances[msg.sender];
    }
}
