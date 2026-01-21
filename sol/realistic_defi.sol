// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/**
 * @title RealisticDeFi
 * @dev Test contract with realistic DeFi patterns - mix of protected and vulnerable
 */
contract RealisticDeFi {
    bool private locked;
    address public owner;
    mapping(address => uint256) public balances;
    
    modifier nonReentrant() {
        require(!locked, "No reentrancy");
        locked = true;
        _;
        locked = false;
    }
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }
    
    constructor() {
        owner = msg.sender;
    }
    
    // PROTECTED - Should be FILTERED
    function withdrawProtected() external nonReentrant {
        uint256 amount = balances[msg.sender];
        balances[msg.sender] = 0;
        (bool success, ) = msg.sender.call{value: amount}("");
        require(success, "Transfer failed");
    }
    
    // VULNERABLE - Should be REPORTED (no guard)
    function withdrawVulnerable() external {
        uint256 amount = balances[msg.sender];
        (bool success, ) = msg.sender.call{value: amount}("");
        require(success, "Transfer failed");
        balances[msg.sender] = 0;
    }
    
    // PROTECTED - Should be FILTERED (admin only)
    function emergencyWithdraw(address payable recipient) external onlyOwner {
        (bool success, ) = recipient.call{value: address(this).balance}("");
        require(success, "Transfer failed");
    }
    
    // VULNERABLE - Should be REPORTED (public + no guard)
    function claimRewards() external {
        uint256 reward = calculateReward(msg.sender);
        (bool success, ) = msg.sender.call{value: reward}("");
        require(success, "Transfer failed");
    }
    
    // PROTECTED - Should be FILTERED
    function adminTransfer(address to, uint256 amount) external onlyOwner {
        (bool success, ) = payable(to).call{value: amount}("");
        require(success, "Transfer failed");
    }
    
    function calculateReward(address user) internal view returns (uint256) {
        return balances[user] / 10;
    }
    
    function deposit() external payable {
        balances[msg.sender] += msg.value;
    }
    
    receive() external payable {}
}
