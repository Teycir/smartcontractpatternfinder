// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Test cases for reward_inflation.yaml - Based on PRXVT exploit (32.8 ETH)

contract VulnerableRewardInflation {
    mapping(address => uint256) public rewards;
    mapping(address => uint256) public stakes;
    mapping(address => uint256) public lastClaim;

    // SHOULD MATCH: claim-without-state-update (PRXVT pattern)
    function claimReward() public {
        uint256 reward = rewards[msg.sender];
        payable(msg.sender).transfer(reward);
        // Missing: rewards[msg.sender] = 0;
    }

    // SHOULD MATCH: stake-claim-same-tx (PRXVT loop exploit)
    function stake(uint256 amount) public {
        stakes[msg.sender] += amount;
        claimReward(); // Calls claim in same tx - loop possible
    }

    // SHOULD MATCH: reward-earned-view-mutable
    function earned(address user) public view returns (uint256) {
        // View function shouldn't modify state but pattern checks for +=
        return stakes[user] * 10;
    }

    // SHOULD MATCH: claim-no-reentrancy-guard
    function claimRewards() public {
        uint256 amount = rewards[msg.sender];
        payable(msg.sender).transfer(amount); // No nonReentrant
        rewards[msg.sender] = 0;
    }

    // SHOULD MATCH: reward-update-after-transfer
    function getRewardUnsafe() public {
        uint256 reward = rewards[msg.sender];
        payable(msg.sender).transfer(reward);
        rewards[msg.sender] = 0; // State update after transfer
    }

    // SHOULD MATCH: getreward-public-callable
    function getReward() public {
        uint256 reward = calculateReward(msg.sender);
        payable(msg.sender).transfer(reward); // No nonReentrant
    }

    function calculateReward(address user) internal view returns (uint256) {
        return stakes[user] * 5;
    }
}

contract SafeRewardInflation {
    mapping(address => uint256) public rewards;
    bool private locked;

    modifier nonReentrant() {
        require(!locked, "Reentrant call");
        locked = true;
        _;
        locked = false;
    }

    function claimReward() public nonReentrant {
        uint256 reward = rewards[msg.sender];
        rewards[msg.sender] = 0; // Update before transfer
        payable(msg.sender).transfer(reward);
    }

    function stake(uint256 amount) public {
        // No claim call in stake
        _updateReward(msg.sender);
    }

    function _updateReward(address user) internal {
        // Proper state management
    }
}
