// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Test cases for access_control_bypass.yaml - Based on TokenHolder/SuperRare exploits

contract VulnerableAccessControl {
    address public owner;
    bool public initialized;
    mapping(address => uint256) public balances;

    // SHOULD MATCH: public-withdraw-no-auth (TokenHolder pattern)
    function withdraw(uint256 amount) public {
        payable(msg.sender).transfer(amount);
    }

    function withdrawFunds(address to, uint256 amount) public {
        payable(to).transfer(amount);
    }

    // SHOULD MATCH: external-mint-no-modifier
    function mint(address to, uint256 amount) external {
        balances[to] += amount;
    }

    // SHOULD MATCH: external-burn-no-modifier
    function burn(uint256 amount) external {
        balances[msg.sender] -= amount;
    }

    // SHOULD MATCH: setowner-no-auth
    function setOwner(address newOwner) external {
        owner = newOwner;
    }

    // SHOULD MATCH: unprotected-initialize (SuperRare pattern)
    function initialize(address _owner) public {
        owner = _owner;
    }

    // SHOULD MATCH: arbitrary-call-no-check
    function executeCall(address target, bytes memory data) public payable {
        target.call{value: msg.value}(data);
    }

    // SHOULD MATCH: delegatecall-no-whitelist
    function executeDelegate(address target, bytes memory data) public {
        target.delegatecall(data);
    }

    // SHOULD MATCH: selfdestruct-no-auth
    function destroy(address payable recipient) public {
        selfdestruct(recipient);
    }
}

contract SafeAccessControl {
    address public owner;
    bool public initialized;

    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }

    modifier initializer() {
        require(!initialized, "Already initialized");
        _;
        initialized = true;
    }

    function withdraw(uint256 amount) public onlyOwner {
        payable(msg.sender).transfer(amount);
    }

    function mint(address to, uint256 amount) external onlyOwner {
        // Protected
    }

    function initialize(address _owner) public initializer {
        require(!initialized, "Already initialized");
        owner = _owner;
    }

    function executeCall(address target, bytes memory data) public onlyOwner {
        require(msg.sender == owner, "Unauthorized");
        target.call{value: 0}(data);
    }

    function executeDelegate(address target, bytes memory data) public onlyOwner {
        target.delegatecall(data);
    }

    function destroy(address payable recipient) public onlyOwner {
        selfdestruct(recipient);
    }
}
