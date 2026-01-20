// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract VulnerableContract {
    mapping(address => uint256) public balances;
    address public owner;
    bytes32 public secretHash;
    
    // Test 1: Reentrancy - state change after external call
    function withdraw(uint256 amount) public {
        require(balances[msg.sender] >= amount);
        (bool success, ) = msg.sender.call{value: amount}("");
        balances[msg.sender] -= amount; // VULN: state change after call
    }
    
    // Test 2: Unchecked return value
    function unsafeSend(address payable recipient, uint256 amount) public {
        recipient.send(amount); // VULN: unchecked return
    }
    
    function unsafeCall(address target, bytes memory data) public {
        target.call(data); // VULN: unchecked return
    }
    
    // Test 3: Unprotected selfdestruct
    function destroy(address payable recipient) public {
        selfdestruct(recipient); // VULN: no access control
    }
    
    // Test 4: Strict balance equality
    function checkBalance() public view returns (bool) {
        if (address(this).balance == 1 ether) { // VULN: strict equality
            return true;
        }
        return false;
    }
    
    function isBalanceZero() public view returns (bool) {
        return address(this).balance == 0; // VULN: zero check
    }
    
    // Test 5: Front-running - hash comparison
    function revealSecret(string memory secret) public {
        if (keccak256(abi.encodePacked(secret)) == secretHash) { // VULN: front-runnable
            balances[msg.sender] += 100;
        }
    }
    
    // Test 6: Front-running - block randomness
    function weakRandom() public view returns (uint256) {
        return uint256(keccak256(abi.encodePacked(block.timestamp, block.number))); // VULN: predictable
    }
    
    // Test 7: Front-running - approve
    function approveToken(address token, address spender, uint256 amount) public {
        IERC20(token).approve(spender, amount); // VULN: front-runnable
    }
}

interface IERC20 {
    function approve(address spender, uint256 amount) external returns (bool);
}
