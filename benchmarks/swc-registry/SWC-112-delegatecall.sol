// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// SWC-112: Delegatecall to Untrusted Callee
contract DelegatecallVulnerable {
    address public owner;
    uint public value;
    
    constructor() {
        owner = msg.sender;
    }
    
    // VULNERABLE: Delegatecall to user-provided address
    function forward(address callee, bytes memory data) public {
        (bool success, ) = callee.delegatecall(data);
        require(success, "Delegatecall failed");
    }
    
    function setValue(uint _value) public {
        require(msg.sender == owner);
        value = _value;
    }
}
