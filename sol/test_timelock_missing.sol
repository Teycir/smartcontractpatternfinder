// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Based on Contracts 12-14, 18, 20: T-REX Proxies
// Lack of Timelock affects 16/20 contracts analyzed

contract VulnerableNoTimelock {
    address public owner;
    address public implementation;

    modifier onlyOwner() {
        require(msg.sender == owner);
        _;
    }

    // VULNERABLE: No timelock protection
    function setOwner(address newOwner) external onlyOwner {
        owner = newOwner;
    }

    // VULNERABLE: Upgrade without timelock
    function upgradeTo(address newImplementation) external onlyOwner {
        implementation = newImplementation;
    }

    // VULNERABLE: setImplementation without timelock
    function setImplementation(address newImpl) external onlyOwner {
        implementation = newImpl;
    }

    // VULNERABLE: Critical parameter change without timelock
    function setFeeRate(uint256 newRate) external onlyOwner {
        // fee rate change
    }
}

contract SafeWithTimelock {
    address public owner;
    uint256 public constant TIMELOCK_DELAY = 2 days;

    modifier onlyOwner() {
        require(msg.sender == owner);
        _;
    }

    modifier timelock() {
        // timelock implementation
        _;
    }

    // SAFE: Has timelock protection
    function upgradeTo(address newImplementation) external onlyOwner timelock {
        // upgrade logic
    }
}
