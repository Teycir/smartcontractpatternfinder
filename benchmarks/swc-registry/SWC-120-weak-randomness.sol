// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// SWC-120: Weak Sources of Randomness from Chain Attributes
contract WeakRandomness {
    address public lastWinner;
    
    // VULNERABLE: Using block attributes for randomness
    function lottery() public payable {
        require(msg.value == 1 ether);
        
        // VULNERABLE: Predictable randomness
        uint256 random = uint256(keccak256(abi.encodePacked(
            block.timestamp,
            block.difficulty,
            msg.sender
        )));
        
        if (random % 2 == 0) {
            payable(msg.sender).transfer(address(this).balance);
            lastWinner = msg.sender;
        }
    }
    
    // VULNERABLE: Using blockhash for randomness
    function guess(uint256 number) public payable {
        require(msg.value == 1 ether);
        
        // VULNERABLE: Blockhash can be predicted
        uint256 answer = uint256(blockhash(block.number - 1)) % 10;
        
        if (number == answer) {
            payable(msg.sender).transfer(address(this).balance);
        }
    }
    
    receive() external payable {}
}
