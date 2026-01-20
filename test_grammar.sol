// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract TestPatterns {
    address public owner;
    mapping(address => uint256) public balances;
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }
    
    // For loop pattern
    function loopWithCall(address[] memory targets) public {
        for (uint i = 0; i < targets.length; i++) {
            targets[i].call("");  // External call in loop
        }
    }
    
    // tx.origin pattern  
    function badAuth() public {
        require(tx.origin == owner);  // tx.origin usage
    }
    
    // Missing access control
    function withdraw() public {  // No modifier
        payable(msg.sender).transfer(address(this).balance);
    }
    
    // Reentrancy pattern
    function vulnerableWithdraw() public {
        uint256 amount = balances[msg.sender];
        (bool success,) = msg.sender.call{value: amount}("");  // Call before state change
        balances[msg.sender] = 0;  // State change after call
    }
    
    // Selfdestruct
    function destroy() public {
        selfdestruct(payable(owner));
    }
}
