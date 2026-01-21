// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract TestProtected {
    bool private locked;
    address public owner;
    
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
    
    // Should NOT be flagged (has nonReentrant)
    function withdrawProtected() external nonReentrant {
        (bool success, ) = msg.sender.call{value: address(this).balance}("");
        require(success);
    }
    
    // SHOULD be flagged (no protection)
    function withdrawVulnerable() external {
        (bool success, ) = msg.sender.call{value: address(this).balance}("");
        require(success);
    }
    
    // Should NOT be flagged (has onlyOwner)
    function adminWithdraw() external onlyOwner {
        (bool success, ) = msg.sender.call{value: address(this).balance}("");
        require(success);
    }
}
