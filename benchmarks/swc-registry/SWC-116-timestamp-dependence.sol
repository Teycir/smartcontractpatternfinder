// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// SWC-116: Block values as a proxy for time
contract TimestampDependence {
    uint256 public lastWinner;
    
    // VULNERABLE: Using block.timestamp for randomness
    function play() public payable {
        require(msg.value == 1 ether);
        
        // VULNERABLE: Miners can manipulate timestamp
        if (block.timestamp % 2 == 0) {
            payable(msg.sender).transfer(2 ether);
            lastWinner = block.timestamp;
        }
    }
    
    // VULNERABLE: Using block.timestamp for critical logic
    function withdraw() public {
        // VULNERABLE: Timestamp can be manipulated within bounds
        require(block.timestamp > lastWinner + 1 days);
        payable(msg.sender).transfer(address(this).balance);
    }
    
    receive() external payable {}
}
