# All Failing Templates - Complete Reference

**Summary:** 14 templates, 87 failing patterns, 28.7% success rate

**Goal:** Fix all patterns to achieve 100% success rate (122/122 passing)

---

## Common Errors to Fix:
1. **Field names**: Remove `body:`, `left:`, `right:`, `operator:`, `function:`, `property:`, `name:`, `type:`, `parameters:`, `arguments:`
2. **Structure errors**: Incorrect nesting of `function_body`, `block_statement`
3. **NodeType errors**: Using non-existent node types

---

## Working Pattern Reference (from advanced_audit_fixed.yaml):

```yaml
- id: tx-origin-auth-fixed
  kind: semantic
  pattern: |
    (call_expression
      (expression
        (identifier) @func)
      (call_argument
        (expression
          (binary_expression
            (expression
              (member_expression
                (identifier) @obj
                (identifier) @prop))))))
    (#eq? @func "require")
    (#eq? @obj "tx")
    (#eq? @prop "origin")
```

---
id: zero-day-emerging
name: Zero-Day and Emerging Vulnerabilities
description: Latest exploit patterns from recent hacks and security research (Updated Monthly)
severity: critical
tags:
  - zero-day
  - emerging
  - cutting-edge
  - 2024-2025
patterns:
  # Read-only reentrancy (Curve Finance hack 2023)
  - id: readonly-reentrancy
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (modifier_invocation
          (identifier) @mod (#match? @mod "^(view|pure)$"))
        (function_body
          (expression_statement
            (call_expression
              (member_expression)))))
    message: View function with external call - vulnerable to read-only reentrancy (Curve-style)
    required: true
    fallback: regex
    on_error: skip

  # ERC4626 inflation attack (2024)
  - id: erc4626-inflation
    kind: semantic
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(deposit|mint)$")
        (function_body
          (expression_statement
            (binary_expression
              (call_expression
                (member_expression
                  (identifier) @supply (#eq? @supply "totalSupply")))))))
    message: ERC4626 vault without virtual shares - vulnerable to inflation attack

  # Permit front-running (2023-2024)
  - id: permit-frontrun
    kind: semantic
    pattern: |
      (function_definition
        (parameter_list
          (parameter
            (identifier) @sig (#match? @sig "^(v|r|s|signature|permit)$")))
        (function_body
          (expression_statement
            (call_expression
              (member_expression
                (identifier) @permit (#eq? @permit "permit"))))))
    message: Permit signature without deadline check - vulnerable to front-running

  # Cross-contract reentrancy (Uniswap V3 style)
  - id: cross-contract-reentrancy
    kind: semantic
    pattern: |
      (function_definition
        (function_body
          (expression_statement
            (call_expression
              (member_expression
                (identifier) @callback (#match? @callback "^(callback|hook|onFlash)$"))))
          (expression_statement
            (member_expression))))
    message: Callback with state read after - vulnerable to cross-contract reentrancy

  # ERC777 reentrancy hooks
  - id: erc777-hook-reentrancy
    kind: semantic
    pattern: |
      (function_definition
        (function_body
          (expression_statement
            (call_expression
              (member_expression
                (identifier) @method (#match? @method "^(send|transfer)$"))
              (call_arguments
                (identifier) @token)))))
    message: Token transfer without reentrancy guard - ERC777 hooks enable reentrancy

  # Vyper reentrancy lock bypass (2023)
  - id: vyper-lock-bypass
    kind: semantic
    pattern: |
      (function_definition
        (function_body
          (expression_statement
            (call_expression
              (member_expression
                (identifier) @raw (#match? @raw "^(raw_call|create_forwarder_to)$"))))
          (expression_statement
            (assignment_expression))))
    message: Raw call with state change - Vyper lock bypass pattern detected

  # Arbitrum sequencer downtime (2024)
  - id: arbitrum-sequencer-check
    kind: semantic
    pattern: |
      (call_expression
        (member_expression
          (identifier) @method (#match? @method "^(latestRoundData|latestAnswer)$")))
    message: Chainlink oracle without sequencer uptime check - critical on L2s (Arbitrum/Optimism)

  # Blast yield manipulation (2024)
  - id: blast-yield-exploit
    kind: semantic
    pattern: |
      (function_definition
        (function_body
          (expression_statement
            (call_expression
              (member_expression
                (identifier) @claim (#match? @claim "^(claim|claimYield|claimGas)$"))))))
    message: Blast yield claim without access control - yield can be stolen

  # EIP-4844 blob data manipulation
  - id: blob-data-validation
    kind: semantic
    pattern: |
      (function_definition
        (parameter_list
          (parameter
            (type_name) @type (#eq? @type "bytes")
            (identifier) @blob (#match? @blob "^(blob|blobData)$"))))
    message: Blob data parameter without validation - EIP-4844 data integrity risk

  # Account abstraction (ERC-4337) validation
  - id: aa-validation-bypass
    kind: semantic
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(validateUserOp|validatePaymasterUserOp)$")
        (function_body
          (return_statement)))
    message: AA validation without proper checks - can bypass signature verification

  # Multicall reentrancy (2024)
  - id: multicall-reentrancy
    kind: semantic
    pattern: |
      (function_definition
        (identifier) @name (#eq? @name "multicall")
        (function_body
          (for_statement
            (block_statement
              (expression_statement
                (call_expression
                  (member_expression
                    (identifier) @delegatecall (#eq? @delegatecall "delegatecall"))))))))
    message: Multicall with delegatecall in loop - enables complex reentrancy attacks

  # Uniswap V4 hook manipulation
  - id: uniswap-v4-hook-exploit
    kind: semantic
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(beforeSwap|afterSwap|beforeModifyPosition|afterModifyPosition)$")
        (function_body
          (expression_statement
            (call_expression))))
    message: Uniswap V4 hook with external call - can manipulate pool state

  # Eigenlayer restaking risks (2024)
  - id: restaking-slashing-risk
    kind: semantic
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(stake|delegate|restake)$")
        (function_body
          (expression_statement
            (call_expression
              (member_expression
                (identifier) @approve (#eq? @approve "approve"))))))
    message: Restaking without slashing protection - funds at risk from operator misbehavior

  # Pendle PT/YT manipulation (2024)
  - id: pendle-pt-yt-exploit
    kind: semantic
    pattern: |
      (function_definition
        (function_body
          (expression_statement
            (call_expression
              (member_expression
                (identifier) @method (#match? @method "^(mintPY|redeemPY|swapExactPtForYt)$"))))))
    message: Pendle PT/YT operation - verify oracle manipulation protection

  # Balancer composable pool reentrancy (2023)
  - id: balancer-composable-reentrancy
    kind: semantic
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(onJoinPool|onExitPool)$")
        (function_body
          (expression_statement
            (call_expression
              (member_expression)))))
    message: Balancer pool hook with external call - composable pool reentrancy risk

  # Aave V3 e-mode manipulation
  - id: aave-emode-exploit
    kind: semantic
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(setUserEMode|borrow)$")
        (function_body
          (expression_statement
            (call_expression))))
    message: Aave e-mode operation - verify category manipulation protection

  # Compound V3 absorb exploit
  - id: compound-v3-absorb
    kind: semantic
    pattern: |
      (function_definition
        (identifier) @name (#eq? @name "absorb")
        (function_body
          (expression_statement
            (call_expression))))
    message: Compound V3 absorb without validation - liquidation manipulation risk

  # GMX V2 price impact manipulation
  - id: gmx-v2-price-impact
    kind: semantic
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(createOrder|executeOrder)$")
        (function_body
          (expression_statement
            (binary_expression))))
    message: GMX V2 order execution - verify price impact calculation protection

  # Morpho optimizer rate manipulation
  - id: morpho-rate-manipulation
    kind: semantic
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(supply|borrow)$")
        (function_body
          (expression_statement
            (call_expression
              (member_expression
                (identifier) @rate (#match? @rate "^(supplyRate|borrowRate)$"))))))
    message: Morpho rate-dependent logic - verify manipulation protection

  # Frax V3 AMO exploit
  - id: frax-amo-exploit
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(mintFrax|burnFrax|addAMO)$")
        (visibility) @vis (#match? @vis "^(public|external)$"))
    message: Frax AMO operation without proper access control - collateral manipulation risk
    required: true
    fallback: regex
    on_error: skip


---


## FILE: templates/advanced_audit.yaml
id: advanced-audit-checks
name: Advanced Audit-Level Vulnerability Detection
description: |
  Deep semantic analysis to catch vulnerabilities auditors commonly miss:
  - Reentrancy patterns (state changes after external calls)
  - Unchecked return values from low-level calls
  - Access control issues on critical functions
  - Oracle manipulation and flash loan vulnerabilities
  - Timestamp dependence and front-running risks
severity: high
tags:
  - security
  - audit
  - advanced
  - semantic
patterns:
  # State changes after external calls (reentrancy)
  - id: state-after-call
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (function_body
          (expression_statement
            (call_expression
              (member_expression
                (identifier) @call (#match? @call "^(call|transfer|send)$"))))
          (expression_statement
            (assignment_expression))))
    message: State change after external call - potential reentrancy vulnerability
    required: true
    fallback: regex
    on_error: skip

  # Unchecked return values
  - id: unchecked-call-return
    kind: semantic
    validate: true
    pattern: |
      (expression_statement
        (call_expression
          (member_expression
            (identifier) @method (#match? @method "^(call|delegatecall|send)$"))))
    message: Unchecked low-level call return value - may silently fail
    required: true
    fallback: regex
    on_error: skip

  # Unsafe delegatecall with user input
  - id: delegatecall-user-input
    kind: semantic
    pattern: |
      (call_expression
        (member_expression
          (identifier) @dc (#eq? @dc "delegatecall"))
        (call_arguments
          (identifier) @arg (#match? @arg "^(msg|_|data|input|calldata)")))
    message: Delegatecall with potential user-controlled input - critical vulnerability

  # Missing zero address check
  - id: missing-zero-check
    kind: semantic
    pattern: |
      (function_definition
        (parameter_list
          (parameter
            (type_name) @type (#eq? @type "address")))
        (function_body
          (expression_statement
            (assignment_expression
              (member_expression)
              (identifier)))))
    message: Address parameter assigned without zero-address validation

  # Integer overflow in multiplication before division
  - id: overflow-mul-div
    kind: semantic
    pattern: |
      (binary_expression
        (binary_expression) @mul
        (#match? @mul "\\*"))
    message: Multiplication before division - potential overflow, use SafeMath or Solidity 0.8+

  # Timestamp dependence
  - id: timestamp-dependence
    kind: semantic
    pattern: |
      (binary_expression
        (member_expression
          (identifier) @block (#eq? @block "block")
          (identifier) @prop (#eq? @prop "timestamp")))
    message: Block timestamp used in logic - miners can manipulate within ~15 seconds

  # Unprotected selfdestruct
  - id: unprotected-selfdestruct
    kind: semantic
    pattern: |
      (function_definition
        (function_body
          (expression_statement
            (call_expression
              (identifier) @sd (#eq? @sd "selfdestruct")))))
    message: Selfdestruct without access control - verify proper protection

  # Ether locked forever (no withdraw function)
  - id: payable-no-withdraw
    kind: semantic
    pattern: |
      (function_definition
        (modifier_invocation
          (identifier) @mod (#eq? @mod "payable")))
    message: Payable function detected - ensure withdrawal mechanism exists

  # Front-running vulnerable patterns
  - id: frontrun-vulnerable
    kind: semantic
    pattern: |
      (function_definition
        (visibility) @vis (#match? @vis "^(public|external)$")
        (function_body
          (expression_statement
            (assignment_expression
              (member_expression)
              (member_expression
                (identifier) @msg (#eq? @msg "msg")
                (identifier) @val (#eq? @val "value"))))))
    message: Public function with msg.value assignment - vulnerable to front-running

  # Dangerous strict equality with balance
  - id: strict-balance-equality
    kind: semantic
    pattern: |
      (binary_expression
        (member_expression
          (identifier) @bal (#eq? @bal "balance")))
    message: Strict equality with balance - use >= instead, balance can be forced

  # Missing event emission
  - id: state-change-no-event
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (visibility) @vis (#match? @vis "^(public|external)$")
        (function_body
          (expression_statement
            (assignment_expression
              (member_expression)))))
    message: State change in public function without event emission
    required: false
    fallback: regex
    on_error: skip
    cache: true

  # Shadowing state variables
  - id: variable-shadowing
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (parameter_list
          (parameter
            (identifier) @param)))
    message: Parameter may shadow state variable - verify no naming conflicts
    required: false
    fallback: regex
    on_error: skip
    cache: true

  # Uninitialized storage pointer
  - id: uninitialized-storage
    kind: semantic
    pattern: |
      (variable_declaration
        (type_name)
        (identifier) @loc (#eq? @loc "storage")
        (identifier))
    message: Storage pointer without initialization - points to slot 0

  # Reentrancy via modifier
  - id: modifier-external-call
    kind: semantic
    pattern: |
      (modifier_definition
        (function_body
          (expression_statement
            (call_expression
              (member_expression
                (identifier) @call (#match? @call "^(call|transfer|send)$"))))))
    message: External call in modifier - can enable reentrancy attacks

  # DoS with block gas limit
  - id: unbounded-loop
    kind: semantic
    pattern: |
      (for_statement
        (block_statement
          (expression_statement
            (call_expression))))
    message: Loop with external calls - vulnerable to DoS via block gas limit

  # Access control on critical functions
  - id: missing-access-control
    kind: semantic
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(withdraw|transfer|mint|burn|destroy|kill|pause|unpause|upgrade)$")
        (visibility) @vis (#match? @vis "^(public|external)$"))
    message: Critical function without visible access control modifier

  # Signature replay attacks
  - id: missing-nonce
    kind: semantic
    pattern: |
      (function_definition
        (parameter_list
          (parameter
            (type_name) @type (#match? @type "^(bytes|bytes32)$")
            (identifier) @name (#match? @name "^(sig|signature)$"))))
    message: Signature parameter without nonce - vulnerable to replay attacks

  # Price oracle manipulation
  - id: single-source-price
    kind: semantic
    pattern: |
      (call_expression
        (member_expression
          (identifier) @method (#match? @method "^(getPrice|price|getReserves)$")))
    message: Single price source - vulnerable to oracle manipulation, use TWAP or multiple sources

  # Flash loan attack surface
  - id: flash-loan-vulnerable
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (visibility) @vis (#match? @vis "^(public|external)$")
        (function_body
          (expression_statement
            (binary_expression
              (member_expression
                (identifier) @bal (#eq? @bal "balance"))))))
    message: Balance-dependent logic in public function - vulnerable to flash loan attacks
    required: true
    fallback: regex
    on_error: skip

  # Centralization risk - detects single point of control via owner/admin modifiers
  - id: owner-single-point
    kind: semantic
    validate: true
    pattern: |
      (modifier_invocation
        (identifier) @mod (#match? @mod "^(onlyOwner|onlyAdmin)$"))
    message: Single owner/admin control - consider multi-sig or timelock for decentralization
    required: false
    fallback: regex
    on_error: skip


---


## FILE: templates/defi_vulnerabilities.yaml
id: defi-vulnerabilities
name: DeFi Protocol Vulnerability Detection
description: |
  Specialized patterns for DeFi protocols including:
  - AMM (Automated Market Maker) vulnerabilities
  - Lending protocol risks
  - Staking mechanism exploits
  - Oracle manipulation
  - MEV and sandwich attacks
severity: critical
tags:
  - defi
  - amm
  - lending
  - security
  - semantic
patterns:
  # AMM: K value manipulation
  - id: amm-k-manipulation
    kind: semantic
    validate: true
    pattern: |
      (binary_expression
        operator: "*"
        left: (identifier) @reserve1
        right: (identifier) @reserve2)
    message: Reserve multiplication detected - verify K value protection against manipulation
    required: false
    fallback: regex
    on_error: skip

  # Slippage protection missing
  - id: missing-slippage-check
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(swap|exchange|trade)$")
        (parameter_list
          (parameter
            (identifier) @amount)))
    message: Swap function without visible slippage parameter - users vulnerable to sandwich attacks
    required: true
    fallback: regex
    on_error: skip

  # Price impact not calculated
  - id: no-price-impact
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(swap|buy|sell)$")
        (function_body
          (expression_statement
            (assignment_expression))))
    message: Trade function without price impact calculation - large trades can be exploited
    required: false
    fallback: regex
    on_error: skip

  # Reentrancy in LP token operations
  - id: lp-reentrancy
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(mint|burn|addLiquidity|removeLiquidity)$")
        (function_body
          (expression_statement
            (call_expression
              (member_expression
                (identifier) @call (#eq? @call "call"))))))
    message: External call in liquidity function - reentrancy risk for LP tokens
    required: true
    fallback: regex
    on_error: skip

  # Flash loan callback without validation
  - id: unvalidated-flash-callback
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(onFlashLoan|executeOperation|flashLoanCallback)$")
        (visibility) @vis (#eq? @vis "external"))
    message: Flash loan callback without sender validation - anyone can trigger
    required: true
    fallback: regex
    on_error: skip

  # Lending: Collateral ratio manipulation
  - id: collateral-ratio-unsafe
    kind: semantic
    validate: true
    pattern: |
      (binary_expression
        operator: "/"
        left: (identifier) @collateral
        right: (identifier) @debt)
    message: Collateral ratio calculation - verify oracle manipulation protection
    required: false
    fallback: regex
    on_error: skip

  # Liquidation without price check
  - id: liquidation-no-price-check
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(liquidate|seize)$")
        (function_body
          (expression_statement
            (call_expression
              (member_expression
                (identifier) @transfer (#match? @transfer "^(transfer|transferFrom)$"))))))
    message: Liquidation function - verify current price oracle check before execution
    required: true
    fallback: regex
    on_error: skip

  # Interest rate manipulation
  - id: interest-rate-exploit
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(borrow|withdraw)$")
        (function_body
          (expression_statement
            (binary_expression
              (identifier) @rate))))
    message: Interest rate calculation in borrow/withdraw - verify manipulation protection
    required: false
    fallback: regex
    on_error: skip

  # Staking: Reward calculation overflow
  - id: reward-overflow
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(getReward|claimReward|harvest)$")
        (function_body
          (expression_statement
            (binary_expression))))
    message: Reward multiplication - verify overflow protection and precision loss
    required: false
    fallback: regex
    on_error: skip

  # Staking: Withdrawal delay bypass
  - id: withdrawal-delay-bypass
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(withdraw|unstake)$")
        (function_body
          (expression_statement
            (call_expression
              (member_expression
                (identifier) @transfer (#eq? @transfer "transfer"))))))
    message: Withdrawal without timelock check - verify delay mechanism cannot be bypassed
    required: false
    fallback: regex
    on_error: skip

  # Governance: Vote manipulation
  - id: vote-weight-manipulation
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(vote|castVote)$")
        (function_body
          (expression_statement
            (call_expression
              (member_expression
                (identifier) @balanceOf (#eq? @balanceOf "balanceOf"))))))
    message: Vote using current balance - vulnerable to flash loan vote manipulation
    required: true
    fallback: regex
    on_error: skip

  # Governance: Proposal execution delay
  - id: no-timelock-execution
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(execute|executeProposal)$")
        (visibility) @vis (#match? @vis "^(public|external)$"))
    message: Proposal execution without visible timelock - users cannot react to malicious proposals
    required: false
    fallback: regex
    on_error: skip

  # Oracle: Stale price data
  - id: no-staleness-check
    kind: semantic
    validate: true
    pattern: |
      (call_expression
        (member_expression
          (identifier) @method (#match? @method "^(latestAnswer|getPrice|latestRoundData)$")))
    message: Oracle price fetch without staleness check - may use outdated prices
    required: true
    fallback: regex
    on_error: skip

  # Oracle: Single point of failure
  - id: single-oracle-dependency
    kind: semantic
    validate: true
    pattern: |
      (variable_declaration
        type: (user_defined_type_name) @type (#match? @type "^(AggregatorV3Interface|IOracle|IPriceOracle)$"))
    message: Single oracle interface - consider multiple oracle sources for redundancy
    required: false
    fallback: regex
    on_error: skip

  # MEV: Sandwich attack surface
  - id: mev-sandwich-vulnerable
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(swap|buy|sell)$")
        (visibility) @vis (#eq? @vis "public")
        (function_body
          (expression_statement
            (call_expression
              (member_expression
                (identifier) @transfer (#eq? @transfer "transfer"))))))
    message: Public swap without MEV protection - vulnerable to sandwich attacks
    required: true
    fallback: regex
    on_error: skip

  # Yield farming: Reward dilution
  - id: reward-dilution
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(stake|deposit)$")
        (function_body
          (expression_statement
            (assignment_expression
              (member_expression
                (identifier) @totalStaked (#eq? @totalStaked "totalStaked"))))))
    message: Total staked update - verify reward distribution prevents dilution attacks
    required: false
    fallback: regex
    on_error: skip

  # Cross-chain: Bridge validation
  - id: bridge-no-validation
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(bridge|mint|unlock)$")
        (parameter_list
          (parameter
            (type_name) @type (#eq? @type "bytes"))))
    message: Bridge function with bytes parameter - verify signature and merkle proof validation
    required: true
    fallback: regex
    on_error: skip

  # NFT: Reentrancy in transfers
  - id: nft-transfer-reentrancy
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(safeTransferFrom|transferFrom)$")
        (function_body
          (expression_statement
            (call_expression))))
    message: NFT transfer with external call - verify reentrancy protection (CEI pattern)
    required: true
    fallback: regex
    on_error: skip

  # Vault: Share price manipulation
  - id: vault-share-manipulation
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(deposit|mint)$")
        (function_body
          (expression_statement
            (binary_expression
              (identifier) @assets
              (identifier) @totalSupply))))
    message: Share calculation using totalSupply - vulnerable to first depositor attack
    required: true
    fallback: regex
    on_error: skip

  # Auction: Bid front-running
  - id: auction-frontrun
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        (identifier) @name (#match? @name "^(bid|placeBid)$")
        (visibility) @vis (#eq? @vis "public"))
    message: Public bid function - vulnerable to front-running, consider commit-reveal scheme
    required: false
    fallback: regex
    on_error: skip


---


## FILE: templates/logic_bugs_gas.yaml
id: logic-bugs-gas-optimization
name: Logic Bugs and Gas Optimization
description: Catch subtle logic errors and expensive gas patterns auditors flag
severity: medium
tags:
  - logic
  - gas
  - optimization
  - semantic
patterns:
  # Logic: Off-by-one errors
  - id: off-by-one-loop
    kind: semantic
    pattern: |
      (for_statement
        condition: (binary_expression
          operator: "<="
          right: (member_expression
            property: (identifier) @length (#eq? @length "length"))))
    message: Loop with <= length - off-by-one error, arrays are 0-indexed

  # Logic: Division before multiplication
  - id: division-before-multiplication
    kind: semantic
    pattern: |
      (binary_expression
        operator: "*"
        left: (binary_expression
          operator: "/"))
    message: Division before multiplication - precision loss, reorder operations

  # Logic: Incorrect comparison operator
  - id: assignment-in-condition
    kind: semantic
    pattern: |
      (if_statement
        condition: (assignment_expression))
    message: Assignment in condition instead of comparison - likely bug

  # Gas: Storage read in loop
  - id: storage-read-loop
    kind: semantic
    pattern: |
      (for_statement
        body: (function_body
          (expression_statement
            (member_expression))))
    message: Storage variable accessed in loop - cache in memory to save gas

  # Gas: Redundant storage writes
  - id: redundant-storage-write
    kind: semantic
    pattern: |
      (expression_statement
        (assignment_expression
          left: (member_expression)
          right: (member_expression)))
    message: Storage-to-storage assignment - verify not redundant write

  # Gas: String comparison
  - id: string-comparison-expensive
    kind: semantic
    pattern: |
      (binary_expression
        operator: "=="
        left: (call_expression
          function: (member_expression
            property: (identifier) @keccak (#eq? @keccak "keccak256")))
        right: (call_expression
          function: (member_expression
            property: (identifier) @keccak2 (#eq? @keccak2 "keccak256"))))
    message: String comparison via keccak256 - consider using bytes32 for gas savings

  # Gas: Unnecessary SafeMath in 0.8+
  - id: unnecessary-safemath
    kind: semantic
    pattern: |
      (call_expression
        function: (member_expression
          property: (identifier) @method (#match? @method "^(add|sub|mul|div)$")))
    message: SafeMath usage detected - unnecessary in Solidity 0.8+, use native operators

  # Gas: Public function not called internally
  - id: public-should-external
    kind: semantic
    pattern: |
      (function_definition
        visibility: (visibility) @vis (#eq? @vis "public"))
    message: Public function - if not called internally, use external to save gas

  # Gas: Array length in loop condition
  - id: array-length-not-cached
    kind: semantic
    pattern: |
      (for_statement
        condition: (binary_expression
          right: (member_expression
            property: (identifier) @length (#eq? @length "length"))))
    message: Array length read every iteration - cache in variable to save gas

  # Logic: Unhandled return value
  - id: ignored-return-value
    kind: semantic
    pattern: |
      (expression_statement
        (call_expression
          function: (member_expression
            property: (identifier) @method (#match? @method "^(approve|transfer|transferFrom)$"))))
    message: ERC20 return value ignored - some tokens return false instead of reverting

  # Logic: Missing input validation
  - id: missing-zero-amount-check
    kind: semantic
    pattern: |
      (function_definition
        parameters: (parameter_list
          (parameter
            type: (type_name) @type (#match? @type "^(uint|uint256)$")
            name: (identifier) @name (#match? @name "^(amount|value|quantity)$")))
        body: (function_body
          (expression_statement
            (call_expression))))
    message: Amount parameter without zero check - validate inputs to prevent waste

  # Logic: Incorrect event parameter
  - id: event-wrong-indexed
    kind: semantic
    pattern: |
      (event_definition
        parameters: (event_parameter_list
          (event_parameter
            type: (type_name) @type (#match? @type "^(string|bytes)$"))))
    message: String/bytes in event - cannot be indexed, use bytes32 hash instead

  # Logic: Floating pragma
  - id: floating-pragma
    kind: regex
    validate: true
    pattern: '^\s*pragma\s+solidity\s+\^'
    message: Floating pragma - lock to specific version for production
    required: false
    on_error: skip

  # Gas: Emit in loop
  - id: emit-in-loop
    kind: semantic
    pattern: |
      (for_statement
        body: (function_body
          (emit_statement)))
    message: Event emission in loop - expensive gas cost, consider batching

  # Logic: Shadowing built-in
  - id: shadowing-builtin
    kind: semantic
    pattern: |
      (variable_declaration
        name: (identifier) @name (#match? @name "^(now|assert|require|revert)$"))
    message: Variable shadows built-in symbol - confusing and error-prone

  # Gas: Unnecessary initialization
  - id: unnecessary-zero-init
    kind: semantic
    pattern: |
      (variable_declaration
        name: (identifier)
        value: (number_literal) @zero (#eq? @zero "0"))
    message: Explicit zero initialization - unnecessary, default is zero

  # Logic: Incorrect inheritance order
  - id: inheritance-order
    kind: semantic
    pattern: |
      (contract_declaration
        (inheritance_specifier)
        (inheritance_specifier))
    message: Multiple inheritance - verify C3 linearization order is correct

  # Gas: Struct packing inefficient
  - id: struct-packing-inefficient
    kind: semantic
    validate: true
    pattern: |
      (struct_declaration
        body: (struct_member
          type: (type_name) @type1 (#match? @type1 "^uint256$"))
        body: (struct_member
          type: (type_name) @type2 (#match? @type2 "^(uint8|uint16|uint32|uint64|uint128|bool|address)$")))
    message: Struct packing inefficient - group smaller types together to save storage slots
    required: false
    fallback: regex
    on_error: skip

  # Logic: Timestamp overflow
  - id: timestamp-uint32-overflow
    kind: semantic
    validate: true
    pattern: |
      (variable_declaration
        type: (type_name) @type (#match? @type "^uint32$")
        name: (identifier) @name (#match? @name "^(timestamp|time|deadline)$"))
    message: uint32 for timestamp - overflows in 2106, use uint256
    required: false
    fallback: regex
    on_error: skip

  # Logic: Missing constructor
  - id: missing-constructor-init
    kind: semantic
    validate: true
    pattern: |
      (contract_declaration
        (state_variable_declaration
          type: (user_defined_type_name)))
    message: Contract reference without constructor initialization - verify proper setup
    required: false
    fallback: regex
    on_error: skip


---


## FILE: templates/front_running_v2.yaml
id: front-running-v2
name: Front-Running Vulnerabilities (v2)
description: |
  Detects patterns vulnerable to front-running (MEV) attacks:
  - Hash reveals without commit phase
  - Block-based randomness
  - Unprotected approve() calls
  - First-come-first-served rewards
  
  Mitigations: commit-reveal, Flashbots, private mempools, slippage checks.
severity: high
tags:
  - security
  - front-running
  - SWC-114
  - MEV

patterns:
  # Semantic: Hash function call
  - id: hash-function-call
    kind: semantic
    validate: true
    pattern: |
      (function_call
        (identifier) @hash_fn
        (#match? @hash_fn "^(keccak256|sha256|sha3|ripemd160)$")) @hash_call
    message: "Hash function '@hash_fn' used. If comparing user input, implement commit-reveal to prevent front-running."
    required: true
    fallback: regex
    on_error: skip

  # Semantic: Block property access
  - id: block-property-access
    kind: semantic
    validate: true
    pattern: |
      (member_access
        (identifier) @obj
        (#eq? @obj "block")
        (identifier) @prop
        (#match? @prop "^(timestamp|number|difficulty|prevrandao|coinbase)$")) @block_access
    message: "block.@prop used. If for randomness, miners can manipulate. Use Chainlink VRF."
    required: true
    fallback: regex
    on_error: skip

  # Regex: Hash comparison
  - id: hash-comparison-regex
    kind: regex
    validate: true
    pattern: '(keccak256|sha256|sha3)\s*\([^)]+\)\s*==\s*\S+'
    message: "Hash comparison detected. Vulnerable to front-running if input is user-provided."
    required: true
    on_error: skip

  # Regex: Block-based randomness
  - id: block-randomness-regex
    kind: regex
    validate: true
    pattern: '(keccak256|sha256)\s*\(\s*(abi\.)?(encode|encodePacked)\s*\([^)]*block\.(timestamp|number|difficulty|prevrandao)'
    message: "Using block properties for randomness is predictable. Use verifiable randomness (VRF)."
    required: true
    on_error: skip

  # Regex: ERC20 approve
  - id: approve-frontrun-regex
    kind: regex
    validate: true
    pattern: '\.approve\s*\(\s*\w+\s*,\s*[^0][^)]*\)'
    message: "ERC20 approve() with non-zero value. Vulnerable to front-running - use increaseAllowance() or set to 0 first."
    required: true
    on_error: skip

  # Regex: Conditional reward pattern
  - id: conditional-transfer-regex
    kind: regex
    validate: true
    pattern: 'if\s*\([^)]*==\s*[^)]+\)\s*\{[^}]*\.(transfer|send|call)\s*[\({]'
    message: "Conditional transfer on equality check. Review for front-running if condition involves secrets."
    required: true
    on_error: skip


---


## FILE: templates/reentrancy_state_change.yaml
id: reentrancy-state-change-v4
name: Reentrancy - State Change After External Call
description: |
  Detects potential reentrancy vulnerabilities where state variables may be
  modified after external calls. Follows the Checks-Effects-Interactions pattern
  violation detection.
  
  Limitations:
  - Cannot distinguish storage vs memory variables
  - Cannot detect reentrancy guards (nonReentrant modifier)
  - Manual review required to confirm exploitability
severity: high
tags:
  - security
  - reentrancy
  - SWC-107
  - CEI-pattern

patterns:
  # Semantic: External call detection
  - id: external-call-detected
    kind: semantic
    pattern: |
      (function_call
        (member_access
          (identifier) @target
          (identifier) @method)
        (#match? @method "^(call|delegatecall|staticcall|transfer|send)$")) @ext_call
    message: "External call via '@method' detected. Verify state changes occur BEFORE this call (CEI pattern)."

  # Semantic: Assignment in function body
  - id: state-assignment-in-function
    kind: semantic
    pattern: |
      (assignment
        (identifier) @var
        (_)) @assign
    message: "State assignment to '@var'. If this follows an external call, potential reentrancy."

  # Regex: call followed by assignment
  - id: call-then-assign-regex
    kind: regex
    validate: true
    pattern: '\.(call|delegatecall|staticcall|transfer|send)\s*[\({][^;]*;[^}]*\n\s*\w+\s*(\[[\w\.\[\]]+\])?\s*[-+*/]?='
    message: "Pattern suggests state change after external call. Review for reentrancy vulnerability."
    required: true
    on_error: skip
    cache: true

  # Regex: Classic reentrancy pattern
  - id: classic-reentrancy-regex
    kind: regex
    validate: true
    pattern: '\.call\{[^}]*value[^}]*\}[^;]*;[\s\S]{0,200}(balances|balance|amounts?|deposits?)\s*\[[^\]]+\]\s*[-=]'
    message: "High-risk: balance mapping modified after value transfer. Classic reentrancy pattern."
    required: true
    on_error: skip
    cache: true


---


## FILE: templates/semantic_vulnerabilities.yaml
id: semantic-vulnerabilities
name: Semantic Vulnerability Detection
description: AST-based detection of smart contract vulnerabilities using tree-sitter
severity: high
tags:
  - security
  - semantic
  - ast-analysis
patterns:
  - id: external-call-detection
    kind: semantic
    pattern: |
      (call_expression
        function: (member_expression
          property: (identifier) @call (#eq? @call "call")))
    message: External call detected - potential reentrancy risk

  - id: delegatecall-usage
    kind: semantic
    pattern: |
      (call_expression
        function: (member_expression
          property: (identifier) @delegatecall (#eq? @delegatecall "delegatecall")))
    message: Delegatecall usage detected - verify target contract safety

  - id: tx-origin-auth
    kind: semantic
    pattern: |
      (member_expression
        object: (identifier) @tx (#eq? @tx "tx")
        property: (identifier) @origin (#eq? @origin "origin"))
    message: tx.origin used for authentication - use msg.sender instead

  - id: selfdestruct-usage
    kind: semantic
    pattern: |
      (call_expression
        function: (identifier) @selfdestruct (#eq? @selfdestruct "selfdestruct"))
    message: selfdestruct usage detected - ensure proper access control

  # Detects low-level calls without return value checks
  - id: unchecked-low-level-call
    kind: semantic
    validate: true
    pattern: |
      (expression_statement
        (call_expression
          function: (member_expression
            property: (identifier) @call (#match? @call "^(call|delegatecall|staticcall)$"))))
    message: Unchecked low-level call - verify return value is checked
    required: true
    fallback: regex
    on_error: skip


---


## FILE: templates/tx_origin_auth.yaml
id: tx-origin-authentication
name: tx.origin Used for Authentication
description: |
  Using tx.origin for authentication is vulnerable to phishing attacks.
  An attacker can trick the owner into calling a malicious contract.
severity: high
tags:
  - security
  - tx-origin
  - SWC-115
patterns:
  - id: tx-origin-equals
    kind: semantic
    validate: true
    pattern: |
      (binary_expression
        (member_expression
          object: (identifier) @_tx (#eq? @_tx "tx")
          property: (identifier) @_origin (#eq? @_origin "origin"))
        operator: "==")
    message: "tx.origin used for authentication. Use msg.sender instead to prevent phishing."
    required: true
    fallback: regex
    on_error: skip

  - id: require-tx-origin
    kind: semantic
    pattern: |
      (call_expression
        function: (identifier) @_func (#eq? @_func "require")
        arguments: (call_argument
          (binary_expression
            (member_expression
              object: (identifier) @_tx (#eq? @_tx "tx")
              property: (identifier) @_origin (#eq? @_origin "origin"))
            operator: "==")))
    message: "require(tx.origin == ...) is vulnerable to phishing. Use msg.sender for auth."

  - id: tx-origin-owner-check
    kind: semantic
    pattern: |
      (binary_expression
        (member_expression
          object: (identifier) @_tx (#eq? @_tx "tx")
          property: (identifier) @_origin (#eq? @_origin "origin"))
        operator: "=="
        (identifier) @_owner (#match? @_owner "^(owner|admin|_owner)$"))
    message: "tx.origin owner check is insecure. Replace with msg.sender == owner."


---


## FILE: templates/unchecked_return_value.yaml
id: unchecked-return-value-v4
name: Unchecked Low-Level Call Return Value
description: |
  Detects low-level calls (call, delegatecall, staticcall, send) where the
  boolean return value is discarded. These functions return false on failure
  rather than reverting.
  
  Safe patterns NOT flagged:
  - (bool success, ) = addr.call{...}(...);
  - require(addr.send(amount), "failed");
  - if (addr.call(...)) { ... }
  
  Note: .transfer() reverts on failure and is not flagged.
severity: high
tags:
  - security
  - unchecked-return
  - SWC-104
  - error-handling

patterns:
  # Semantic: Low-level call detection
  - id: low-level-call
    kind: semantic
    pattern: |
      (expression_statement
        (function_call
          (member_access
            (_) @target
            (identifier) @method)
          (#match? @method "^(call|delegatecall|staticcall|send)$"))) @unchecked
    message: "Low-level '@method' call return value not captured. Use: (bool success,) = @target.@method(...); require(success);"

  # Regex: Expression statement with low-level call (no assignment)
  - id: unchecked-call-regex
    kind: regex
    validate: true
    pattern: '^\s*[^=\n]*\.(call|delegatecall|staticcall|send)\s*[\({]'
    message: "Unchecked low-level call detected. Return value should be verified."
    required: true
    on_error: skip

  # Regex: send without assignment or require
  - id: unchecked-send-regex
    kind: regex
    validate: true
    pattern: '^\s*\w+\.send\([^)]+\)\s*;'
    message: "'.send()' return not checked. Use require(addr.send(amount)) or .transfer()."
    required: true
    on_error: skip

  # Regex: Properly checked call (info only)
  - id: checked-call-info
    kind: regex
    validate: true
    pattern: '\(\s*bool\s+\w+\s*,?\s*\)\s*=\s*\w+\.(call|delegatecall)'
    message: "[INFO] Properly checked low-level call found. Good practice."
    required: false
    on_error: skip


---


## FILE: templates/missing_access_control.yaml
id: missing-access-control
name: Missing Access Control for Critical Functions
description: |
  Critical functions (like withdraw, mint, owner change) should have access control modifiers.
severity: high
tags:
  - security
  - access-control
  - SWC-105
patterns:
  # Detects critical functions without access control modifiers
  - id: critical-function-no-auth
    kind: semantic
    validate: true
    pattern: |
      (function_definition
        name: (identifier) @func_name (#match? @func_name "^(withdraw|mint|burn|setOwner|transferOwnership)$")
        (visibility) @vis (#match? @vis "^(public|external)$")
        !modifier_invocation)
    message: "Critical function found without modifiers. Ensure access control is implemented (e.g. onlyOwner)."
    required: true
    fallback: regex
    on_error: skip


---


## FILE: templates/strict_balance_equality_v2.yaml
id: strict-balance-equality-v2
name: Strict Balance Equality
description: |
  Detects strict equality (== or !=) comparisons involving .balance.
  
  Contract balance can be forcibly increased via:
  - selfdestruct(contractAddress)
  - Coinbase rewards
  - Pre-funded CREATE2 addresses
  
  Recommendation: Use >= or <= instead of == for balance checks.
severity: medium
tags:
  - security
  - balance-manipulation
  - SWC-132
  - logic-error

patterns:
  # Semantic: member access to .balance
  - id: balance-access
    kind: semantic
    validate: true
    pattern: |
      (member_access
        (_) @obj
        (identifier) @prop
        (#eq? @prop "balance")) @balance_ref
    message: "Access to '@obj.balance'. If used in equality check, vulnerable to manipulation."
    required: false
    fallback: regex
    on_error: skip

  # Regex: Universal balance equality (covers all cases)
  - id: balance-equality-regex
    kind: regex
    validate: true
    pattern: '\.balance\s*(==|!=)\s*\S+'
    message: "Strict equality with .balance detected. Balance can be force-changed - use >= or <= instead."
    required: false
    fallback: regex
    on_error: skip

  # Regex: Reversed operand
  - id: balance-equality-reversed-regex
    kind: regex
    validate: true
    pattern: '\S+\s*(==|!=)\s*[^.]*\.balance'
    message: "Comparing value to .balance with strict equality. Vulnerable to forced deposits."
    required: false
    fallback: regex
    on_error: skip

  # Regex: Specific zero check
  - id: balance-zero-regex
    kind: regex
    validate: true
    pattern: '\.balance\s*==\s*0\b'
    message: "Checking balance == 0 is bypassable. Attacker can force-send ETH via selfdestruct."
    required: false
    fallback: regex
    on_error: skip

  # Regex: address(this).balance pattern
  - id: this-balance-regex
    kind: regex
    validate: true
    pattern: 'address\s*\(\s*this\s*\)\s*\.balance\s*(==|!=)'
    message: "Contract balance strict equality. Can be manipulated by external actors."
    required: false
    fallback: regex
    on_error: skip


---


## FILE: templates/unprotected_selfdestruct_v2.yaml
id: unprotected-selfdestruct-v2
name: Selfdestruct Usage Detection
description: |
  Detects all uses of selfdestruct/suicide. This operation:
  - Permanently destroys the contract
  - Sends all ETH to specified recipient
  - Cannot be undone
  
  CRITICAL: Verify access control exists (onlyOwner, multi-sig, etc.)
  
  Note: selfdestruct is deprecated (EIP-6049) and behavior may change
  in future hard forks.
severity: critical
tags:
  - security
  - selfdestruct
  - SWC-106
  - access-control

patterns:
  # Semantic: Direct selfdestruct call
  - id: selfdestruct-call
    kind: semantic
    pattern: |
      (function_call
        (identifier) @func
        (#match? @func "^(selfdestruct|suicide)$")) @destruct
    message: "selfdestruct detected. CRITICAL: Verify onlyOwner or equivalent access control."

  # Semantic: selfdestruct with payable cast
  - id: selfdestruct-payable
    kind: semantic
    pattern: |
      (function_call
        (identifier) @func
        (#match? @func "^(selfdestruct|suicide)$")
        (function_call_arguments
          (function_call
            (identifier) @cast
            (#eq? @cast "payable")))) @destruct
    message: "selfdestruct(payable(...)) detected. Ensure protected by access modifier."

  # Regex: Any selfdestruct
  - id: selfdestruct-regex
    kind: regex
    pattern: '(selfdestruct|suicide)\s*\('
    message: "selfdestruct/suicide usage detected. Contract will be permanently destroyed."
    cache: true

  # Regex: selfdestruct in public function
  - id: selfdestruct-public-regex
    kind: regex
    pattern: 'function\s+\w+\s*\([^)]*\)\s*(public|external)[^{]*\{[^}]*(selfdestruct|suicide)\s*\('
    message: "CRITICAL: selfdestruct in public/external function. Verify access control modifier."
    cache: true


---


## FILE: templates/advanced_audit_fixed.yaml
id: advanced-audit-checks-fixed
name: Advanced Audit-Level Vulnerability Detection (Fixed)
description: Fixed semantic patterns using correct tree-sitter-solidity node types
severity: high
tags:
  - security
  - audit
  - semantic
  - fixed

patterns:
  # Unbounded loop with external calls
  - id: unbounded-loop-fixed
    kind: semantic
    pattern: |
      (for_statement
        (block_statement
          (statement
            (expression_statement
              (expression
                (call_expression))))))
    message: Loop with external calls - vulnerable to DoS via block gas limit

  # tx.origin authentication
  - id: tx-origin-auth-fixed
    kind: semantic
    pattern: |
      (call_expression
        (expression
          (identifier) @func)
        (call_argument
          (expression
            (binary_expression
              (expression
                (member_expression
                  (identifier) @obj
                  (identifier) @prop))))))
      (#eq? @func "require")
      (#eq? @obj "tx")
      (#eq? @prop "origin")
    message: tx.origin used for authentication - vulnerable to phishing

  # Missing access control on critical functions
  - id: missing-access-control-fixed
    kind: semantic
    pattern: |
      (function_definition
        (identifier) @name
        (visibility) @vis
        (function_body))
      (#match? @name "^(withdraw|transfer|destroy|kill)$")
      (#eq? @vis "public")
    message: Critical function without access control modifier

  # Reentrancy: external call before state change
  - id: reentrancy-pattern-fixed
    kind: semantic
    pattern: |
      (function_body
        (statement
          (expression_statement
            (expression
              (call_expression
                (expression
                  (member_expression
                    (identifier)
                    (identifier) @method))))))
        (statement
          (expression_statement
            (expression
              (assignment_expression)))))
      (#match? @method "^(call|transfer|send)$")
    message: External call before state modification - reentrancy risk

  # Unprotected selfdestruct
  - id: unprotected-selfdestruct-fixed
    kind: semantic
    pattern: |
      (function_definition
        (visibility) @vis
        (function_body
          (statement
            (expression_statement
              (expression
                (call_expression
                  (expression
                    (identifier) @func)))))))
      (#eq? @vis "public")
      (#eq? @func "selfdestruct")
    message: Unprotected selfdestruct - contract can be destroyed by anyone

  # Unchecked low-level call
  - id: unchecked-call-fixed
    kind: semantic
    pattern: |
      (expression_statement
        (expression
          (call_expression
            (expression
              (member_expression
                (identifier)
                (identifier) @method)))))
      (#match? @method "^(call|delegatecall|send)$")
    message: Unchecked low-level call return value - may silently fail

  # Timestamp dependence
  - id: timestamp-dependence-fixed
    kind: semantic
    pattern: |
      (binary_expression
        (expression
          (member_expression
            (identifier) @obj
            (identifier) @prop)))
      (#eq? @obj "block")
      (#eq? @prop "timestamp")
    message: Block timestamp used in logic - miners can manipulate

  # Strict balance equality
  - id: strict-balance-equality-fixed
    kind: semantic
    pattern: |
      (binary_expression
        (expression
          (member_expression
            (identifier)
            (identifier) @prop)))
      (#eq? @prop "balance")
    message: Strict equality with balance - use >= instead


---


## FILE: templates/zero_day_live.yaml
id: zero-day-live
name: Live 0-Day Detection
description: 'Auto-generated from security feeds (Updated: 2026-01-20)'
severity: critical
tags:
- zero-day
- live
patterns:
- id: flash_loan_20260119
  pattern: |-
    (function_definition
      (block_statement
        (expression_statement
          (binary_expression
            (member_expression
              (identifier) @balance (#eq? @balance "balance"))))))
  message: Makina - defillama (2026-01-19)
  kind: semantic


---



---

## Validation Command

Run after each fix:
```bash
./target/release/validate_all
```

## Files Summary:
1. zero_day_emerging.yaml - 19 failures
2. advanced_audit.yaml - 17 failures
3. defi_vulnerabilities.yaml - 17 failures
4. logic_bugs_gas.yaml - 16 failures
5. front_running_v2.yaml - 2 failures
6. reentrancy_state_change.yaml - 2 failures
7. semantic_vulnerabilities.yaml - 4 failures
8. tx_origin_auth.yaml - 3 failures
9. unchecked_return_value.yaml - 3 failures
10. missing_access_control.yaml - 1 failure
11. strict_balance_equality_v2.yaml - 1 failure
12. unprotected_selfdestruct_v2.yaml - 1 failure
13. advanced_audit_fixed.yaml - 1 failure
14. zero_day_live.yaml - 1 failure

