// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

contract SafePatterns is Ownable, ReentrancyGuard {
    mapping(address => uint256) public rewards;
    
    // Safe: Has onlyOwner
    function withdraw(uint256 amount) public onlyOwner {
        payable(msg.sender).transfer(amount);
    }
    
    // Safe: Has nonReentrant
    function claimReward() public nonReentrant {
        uint256 reward = rewards[msg.sender];
        rewards[msg.sender] = 0;
        payable(msg.sender).transfer(reward);
    }
    
    // Safe: Multiplication before division
    function calculateShare(uint256 amount, uint256 total, uint256 multiplier) 
        public pure returns (uint256) {
        return amount * multiplier / total;
    }
}
