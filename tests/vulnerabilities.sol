pragma solidity ^0.4.24;

contract VulnerableContract {
    mapping(address => uint256) public balances;
    address public owner;
    bool public paused;

    // 1. Reentrancy
    function withdraw(uint256 amount) public {
        require(balances[msg.sender] >= amount);
        msg.sender.call.value(amount)(""); // External call
        balances[msg.sender] -= amount; // State change after call
    }

    // 2. Unchecked Low-Level Call
    function unsafeCall(address target) public {
        target.call(abi.encodeWithSignature("doSomething()")); // Unchecked return
    }

    // 3. Delegatecall with User Input
    function delegate(address target, bytes memory data) public {
        target.delegatecall(data); // User controlled target/data
    }

    // 4. Missing Access Control
    function mint(uint256 amount) public { // Public mint
        balances[msg.sender] += amount;
    }
    
    function togglePause() public { // Public pause
        paused = !paused;
    }

    // 5. Timestamp Dependence
    function timelock() public {
        require(block.timestamp > 1234567890); // Timestamp in require
        if (block.timestamp % 2 == 0) { // Timestamp comparison
            // do something
        }
    }

    // 6. Unprotected Selfdestruct
    function destroy() public {
        selfdestruct(msg.sender); // Public selfdestruct
    }

    // 7. Integer Overflow (Pre-0.8.0)
    function batchTransfer(address[] receivers, uint256 value) public {
        uint256 total = receivers.length * value; // Multiply before checks could overflow
        require(balances[msg.sender] >= total);
        for (uint256 i = 0; i < receivers.length; i++) {
            balances[receivers[i]] += value; // Unchecked addition
            balances[msg.sender] -= value; // Unchecked subtraction
        }
    }

    // 8. DoS with Gas Limit
    address[] public users;
    function payAll() public {
        for (uint256 i = 0; i < users.length; i++) {
            users[i].transfer(1 ether); // External call in loop
        }
    }

    // 9. Front Running
    function buy() public payable {
        require(msg.value > 1 ether); // Price sensitive
    }

    // 10. Strict Balance Equality
    function checkBalance() public {
        require(address(this).balance == 10 ether); // Strict equality
    }

    // Bonus: tx.origin Authentication
    function authenticate() public {
        require(tx.origin == owner); // tx.origin auth
    }
}
