// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract VulnerableContract {
    mapping(address => uint256) public balances;
    address public owner;

    constructor() {
        owner = msg.sender;
    }

    // VULNERABILITY: Reentrancy - external call before state update
    function withdraw() public {
        uint256 amount = balances[msg.sender];
        require(amount > 0, "No balance");
        
        // External call before state update
        (bool success, ) = msg.sender.call{value: amount}("");
        require(success, "Transfer failed");
        
        // State update after external call - VULNERABLE!
        balances[msg.sender] = 0;
    }

    // VULNERABILITY: tx.origin for authentication
    function authWithTxOrigin() public {
        require(tx.origin == owner, "Not owner");
        // Dangerous: tx.origin can be exploited via phishing
    }

    // VULNERABILITY: Unchecked delegatecall
    function proxyCall(address target, bytes memory data) public {
        (bool success, ) = target.delegatecall(data);
        // No check on success - VULNERABLE!
    }

    // VULNERABILITY: Unchecked low-level call
    function unsafeTransfer(address payable recipient, uint256 amount) public {
        recipient.call{value: amount}("");
        // Return value not checked - VULNERABLE!
    }

    // VULNERABILITY: selfdestruct without proper access control
    function destroy() public {
        selfdestruct(payable(owner));
        // Should have proper access control
    }

    // SAFE: Proper checks-effects-interactions pattern
    function safeWithdraw() public {
        uint256 amount = balances[msg.sender];
        require(amount > 0, "No balance");
        
        // State update BEFORE external call
        balances[msg.sender] = 0;
        
        // External call after state update - SAFE
        (bool success, ) = msg.sender.call{value: amount}("");
        require(success, "Transfer failed");
    }

    // SAFE: Using msg.sender for authentication
    function authWithMsgSender() public {
        require(msg.sender == owner, "Not owner");
        // Safe authentication
    }

    receive() external payable {
        balances[msg.sender] += msg.value;
    }
}
