// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Based on Contract 17: L2OutputOracle (0x56a76bcC92...)
// Unchecked external calls affecting 9/20 contracts

contract VulnerableUncheckedReturn {
    // VULNERABLE: (bool success,) declared but not checked
    function transferWithUncheckedReturn(address _to) external payable {
        (bool success, ) = _to.call{value: msg.value}("");
        // success is declared but never checked - silent failure possible
        // Continue execution regardless of call result
    }

    // VULNERABLE: Return value completely ignored
    function sendEther(address payable recipient) external payable {
        recipient.call{value: msg.value}(""); // No assignment at all
    }

    // VULNERABLE: .send() unchecked
    function sendWithSend(address payable recipient) external payable {
        recipient.send(msg.value); // Return value ignored
    }

    // VULNERABLE: delegatecall unchecked
    function proxyCall(address target, bytes memory data) external {
        target.delegatecall(data); // Return value ignored
    }
}

contract SafeCheckedReturn {
    error TransferFailed();

    // SAFE: Return value checked with revert
    function transferWithCheck(address _to) external payable {
        (bool success, ) = _to.call{value: msg.value}("");
        if (!success) revert TransferFailed();
    }

    // SAFE: Return value checked with require
    function sendWithRequire(address payable recipient) external payable {
        (bool success, ) = recipient.call{value: msg.value}("");
        require(success, "Transfer failed");
    }

    // SAFE: Using assert
    function sendWithAssert(address payable recipient) external payable {
        (bool success, ) = recipient.call{value: msg.value}("");
        assert(success);
    }
}
