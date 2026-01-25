// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Based on Contract 9: ChannelImplementation (0x9e32b13ce7...)
// Signature replay and MEV vulnerabilities

contract VulnerableSignature {
    // VULNERABLE: No replay protection (missing nonce)
    function withdraw(uint256 amount, bytes memory signature) external {
        bytes32 messageHash = keccak256(abi.encodePacked(msg.sender, amount));
        address signer = recoverSigner(messageHash, signature);
        require(signer == msg.sender, "Invalid signature");
        // withdraw logic - can be replayed!
    }

    // VULNERABLE: ecrecover without address(0) check
    function recoverSigner(bytes32 hash, bytes memory sig) internal pure returns (address) {
        (uint8 v, bytes32 r, bytes32 s) = splitSignature(sig);
        return ecrecover(hash, v, r, s); // No check for address(0)
    }

    // VULNERABLE: ECDSA.recover without validation
    function verifySignatureECDSA(bytes32 hash, bytes memory signature) public pure returns (address) {
        return ECDSA.recover(hash, signature); // No validation
    }

    function splitSignature(bytes memory sig) internal pure returns (uint8, bytes32, bytes32) {
        require(sig.length == 65);
        bytes32 r;
        bytes32 s;
        uint8 v;
        assembly {
            r := mload(add(sig, 32))
            s := mload(add(sig, 64))
            v := byte(0, mload(add(sig, 96)))
        }
        return (v, r, s);
    }
}

contract SafeSignature {
    mapping(address => uint256) public nonces;

    // SAFE: Has replay protection with nonce
    function withdraw(uint256 amount, uint256 nonce, bytes memory signature) external {
        require(nonce == nonces[msg.sender], "Invalid nonce");
        bytes32 messageHash = keccak256(abi.encodePacked(msg.sender, amount, nonce));
        address signer = recoverSigner(messageHash, signature);
        require(signer != address(0), "Invalid signature");
        require(signer == msg.sender, "Unauthorized");
        nonces[msg.sender]++;
        // withdraw logic
    }

    function recoverSigner(bytes32 hash, bytes memory sig) internal pure returns (address) {
        (uint8 v, bytes32 r, bytes32 s) = splitSignature(sig);
        address recovered = ecrecover(hash, v, r, s);
        require(recovered != address(0), "Invalid signature");
        return recovered;
    }

    function splitSignature(bytes memory sig) internal pure returns (uint8, bytes32, bytes32) {
        require(sig.length == 65);
        bytes32 r;
        bytes32 s;
        uint8 v;
        assembly {
            r := mload(add(sig, 32))
            s := mload(add(sig, 64))
            v := byte(0, mload(add(sig, 96)))
        }
        return (v, r, s);
    }
}

library ECDSA {
    function recover(bytes32 hash, bytes memory signature) internal pure returns (address) {
        // Mock implementation
        return address(0);
    }
}
