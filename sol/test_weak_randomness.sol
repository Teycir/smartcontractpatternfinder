// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Based on Contract 6: OracleRNG (0x403e967a4d...)
// Risk Score: 215 - EXPLOITABLE

contract VulnerableRandomness {
    // VULNERABLE: blockhash() is miner-exploitable
    function generateRandomBlockhash() public view returns (uint256) {
        return uint256(blockhash(block.number - 1)) % 100;
    }

    // VULNERABLE: block.timestamp % is predictable
    function generateRandomTimestamp() public view returns (uint256) {
        return block.timestamp % 100;
    }

    // VULNERABLE: block.number % is predictable
    function generateRandomBlockNumber() public view returns (uint256) {
        return block.number % 100;
    }

    // VULNERABLE: Combined weak randomness
    function mintRandomNFT(address to) public {
        uint256 tokenId = uint256(blockhash(block.number - 1)) % 10000;
        // mint logic
    }
}

contract SafeRandomness {
    // SAFE: Using Chainlink VRF (recommended)
    function requestRandomWords() external {
        // Chainlink VRF implementation
    }
}
