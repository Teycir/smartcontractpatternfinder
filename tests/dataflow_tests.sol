// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract DataFlowTests {
    mapping(address => uint) public balances;
    mapping(address => bool) public whitelist;
    address public owner;
    
    // Test 1: Direct taint to sink (should detect)
    function bad1(address target) public {
        target.delegatecall("");
    }
    
    // Test 2: Indirect taint (should detect)
    function bad2(address target) public {
        address impl = target;
        impl.delegatecall("");
    }
    
    // Test 3: Sanitized (should NOT detect)
    function good1(address target) public {
        require(whitelist[target], "not allowed");
        target.delegatecall("");
    }
    
    // Test 4: CEI violation (should detect)
    function bad3() public {
        msg.sender.call{value: balances[msg.sender]}("");
        balances[msg.sender] = 0;
    }
    
    // Test 5: CEI correct (should NOT detect)
    function good2() public {
        uint amt = balances[msg.sender];
        balances[msg.sender] = 0;
        msg.sender.call{value: amt}("");
    }
    
    // Test 6: Modifier protection (should NOT detect)
    modifier nonReentrant() {
        _;
    }
    
    function good3() public nonReentrant {
        msg.sender.call{value: balances[msg.sender]}("");
        balances[msg.sender] = 0;
    }
    
    // Test 7: After require (should NOT detect)
    function good4() public {
        require(balances[msg.sender] > 0);
        msg.sender.call{value: balances[msg.sender]}("");
        balances[msg.sender] = 0;
    }
    
    // Test 8: tx.origin to selfdestruct (should detect CRITICAL)
    function bad4() public {
        if (tx.origin == owner) {
            selfdestruct(payable(msg.sender));
        }
    }
}
