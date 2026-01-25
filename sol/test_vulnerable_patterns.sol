// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/**
 * Test file containing REAL vulnerable patterns from analyzed contracts
 * Used to validate SCPF template detection accuracy
 */

// ============================================================================
// Contract 6: OracleRNG (0x41dF754132756ED64BfE0eEBf007Dc1F90101cAF)
// Template: weak_randomness.yaml
// Expected: MUST detect blockhash usage
// ============================================================================
contract OracleRNG_Vulnerable {
    function predictBattle(uint256 battleId) external view returns (uint256) {
        // VULNERABLE: Weak randomness using blockhash
        bytes32 hash = blockhash(block.number - 1);
        return uint256(hash) % 100;
    }
}

// ============================================================================
// Contract 15: MoonCatsStrategyV2 (0x341D67a2353a56AF978DC228b305470756b63C41)
// Template: unchecked_return_value.yaml (pattern: arbitrary-call-with-data)
// Expected: MUST detect arbitrary external call with data
// ============================================================================
contract MoonCatsStrategyV2_Vulnerable {
    function buyTargetNFT(
        uint256 value,
        bytes calldata data,
        uint256 expectedId,
        address target
    ) external {
        // VULNERABLE: Arbitrary external call with user-controlled data
        (bool success, bytes memory returnData) = target.call{value: value}(data);
        require(success, "Call failed");
    }
}

// ============================================================================
// Contract 16: BondingCurve (0xbCCdd5d884125F545c2714Feb87E3536aF06A4a8)
// Template: reentrancy.yaml (pattern: erc20-transferfrom-before-state)
// Expected: MUST detect transferFrom before state change
// ============================================================================
interface IERC20 {
    function transferFrom(address from, address to, uint256 amount) external returns (bool);
}

contract BondingCurve_Vulnerable {
    IERC20 public baseToken;
    mapping(address => uint256) public balances;

    function buy(uint256 baseIn, uint256 minOut, address to) external returns (uint256 out) {
        require(baseIn > 0, "zero");
        out = baseIn * 2; // simplified
        require(out >= minOut, "slippage");
        
        // VULNERABLE: External call before state change
        require(baseToken.transferFrom(msg.sender, address(this), baseIn));
        balances[to] += out; // State change AFTER external call
    }
}

// ============================================================================
// Contract 9: Channel (0x527B1dedD4C254ce134c2D8C505a68325f6ACdfe)
// Template: signature_unchecked.yaml
// Expected: MUST detect ecrecover without replay protection
// ============================================================================
contract Channel_Vulnerable {
    function settleWithSignature(
        address recipient,
        uint256 amount,
        bytes32 messageHash,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external {
        // VULNERABLE: No nonce/timestamp replay protection
        address signer = ecrecover(messageHash, v, r, s);
        require(signer != address(0), "Invalid signature");
        payable(recipient).transfer(amount);
    }
}

// ============================================================================
// Contract 17: TransferRegistry (0xDAB785F7719108390A26ff8d167e40aE4789F8D7)
// Template: cross_chain_gas_grief.yaml
// Expected: MUST detect .call{value:} before sendMessage
// ============================================================================
interface IL1Messenger {
    function sendMessage(address target, bytes calldata message, uint32 gasLimit) external;
}

interface IFactRegistry {
    function registerFact(bytes32 fact) external;
}

contract TransferRegistry_Vulnerable {
    IL1Messenger public l1Messenger;
    IFactRegistry public factRegistry;
    uint32 public minGasLimit = 100000;

    function transfer(address _to, bytes32 _salt) external payable {
        require(_to != address(0), "Zero address");
        bytes32 fact = keccak256(abi.encode(_to, msg.value, _salt));
        
        // VULNERABLE: ETH sent before cross-chain message
        (bool success, ) = _to.call{value: msg.value}("");
        require(success, "Transfer failed");
        
        // If _to consumes all gas, this fails but ETH is already sent
        l1Messenger.sendMessage(
            address(factRegistry),
            abi.encodeWithSelector(IFactRegistry.registerFact.selector, fact),
            minGasLimit
        );
    }
}

// ============================================================================
// Contract 18: IdentityRegistry (0xcd6B0b4D31fB143F24946172D26137aa83d702E8)
// Template: delegatecall_user_input.yaml
// Expected: MUST detect delegatecall usage
// ============================================================================
interface ITREXImplementationAuthority {
    function getIRImplementation() external view returns (address);
}

contract IdentityRegistry_Vulnerable {
    address public implementationAuthority;

    function getImplementationAuthority() public view returns (address) {
        return implementationAuthority;
    }

    fallback() external payable {
        // VULNERABLE: Delegatecall to potentially malicious implementation
        address logic = ITREXImplementationAuthority(getImplementationAuthority()).getIRImplementation();
        
        assembly {
            calldatacopy(0x0, 0x0, calldatasize())
            let success := delegatecall(sub(gas(), 10000), logic, 0x0, calldatasize(), 0, 0)
            returndatacopy(0, 0, returndatasize())
            switch success
            case 0 { revert(0, returndatasize()) }
            default { return(0, returndatasize()) }
        }
    }
}

// ============================================================================
// Contract 19: AlpacaFarm (0x054F3832AaC0eB98f82Ba9E3f1447Ab373308B8B)
// Template: reentrancy_callback.yaml
// Expected: MUST detect onERC1155Received callback
// ============================================================================
interface IERC1155Receiver {
    function onERC1155Received(
        address operator,
        address from,
        uint256 id,
        uint256 value,
        bytes calldata data
    ) external returns (bytes4);
}

contract AlpacaFarm_Vulnerable is IERC1155Receiver {
    struct UserInfo {
        uint256 amount;
        uint256 alpacaID;
        uint256 alpacaEnergy;
    }
    
    mapping(address => UserInfo) public userInfo;

    function onERC1155Received(
        address,
        address _from,
        uint256 _id,
        uint256,
        bytes calldata
    ) external override returns (bytes4) {
        // VULNERABLE: State changes during callback
        UserInfo storage user = userInfo[_from];
        
        if (user.amount > 0) {
            // External calls during callback
            _safeAlpaTransfer(_from, 100);
        }
        
        // State change after external call
        user.alpacaID = _id;
        user.alpacaEnergy = 1000;
        
        return this.onERC1155Received.selector;
    }
    
    function _safeAlpaTransfer(address to, uint256 amount) internal {
        // Simplified
    }
}

// ============================================================================
// SAFE CONTRACTS - Should NOT trigger false positives
// ============================================================================

contract Safe_ChecksEffectsInteractions {
    IERC20 public token;
    mapping(address => uint256) public balances;

    function deposit(uint256 amount) external {
        // SAFE: State change before external call
        balances[msg.sender] += amount;
        require(token.transferFrom(msg.sender, address(this), amount));
    }
}

contract Safe_ReentrancyGuard {
    bool private locked;
    
    modifier nonReentrant() {
        require(!locked, "Reentrant call");
        locked = true;
        _;
        locked = false;
    }

    function withdraw(uint256 amount) external nonReentrant {
        // SAFE: Has reentrancy guard
        (bool success, ) = msg.sender.call{value: amount}("");
        require(success);
    }
}

contract Safe_SignatureWithNonce {
    mapping(address => uint256) public nonces;

    function executeWithSignature(
        address recipient,
        uint256 amount,
        uint256 nonce,
        bytes32 messageHash,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external {
        // SAFE: Has nonce-based replay protection
        require(nonces[recipient] == nonce, "Invalid nonce");
        nonces[recipient]++;
        
        address signer = ecrecover(messageHash, v, r, s);
        require(signer != address(0), "Invalid signature");
        payable(recipient).transfer(amount);
    }
}
