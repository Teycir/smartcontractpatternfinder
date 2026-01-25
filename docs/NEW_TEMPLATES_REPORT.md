# New Vulnerability Templates Report

**Date**: January 2026  
**Author**: Teycir Ben Soltane  
**Project**: Smart Contract Pattern Finder (SCPF)  
**Source**: DeFiHackLabs Repository Analysis

---

## Executive Summary

This report documents 5 new vulnerability detection templates created from deep analysis of real-world exploits in the DeFiHackLabs repository. These templates target vulnerabilities with a combined financial impact exceeding **$120 million** from 2025-2026 exploits.

**Templates Created**:
1. Fee Accounting Flaw (MTToken $37K, FutureSwap $433K)
2. Reward Inflation (PRXVT 32.8 ETH)
3. Price Manipulation (6+ exploits, millions in losses)
4. Precision Loss (BalancerV2 $120M+)
5. Access Control Bypass (10+ exploits, millions in losses)

**Total Patterns**: 34 exploit patterns from real 0-day vulnerabilities

---

## 1. Fee Accounting Flaw Template

### Source Analysis

**Exploits Analyzed**:
- **MTToken** (January 2026) - $37,000 loss
- **FutureSwap** (January 2026) - $433,000 loss

**Root Cause Discovery**:
- **MTToken**: Unbounded fee accumulation in loops without validation that total fees ≤ 100%
- **FutureSwap**: Unit mismatch between WAD (1e18) and BPS (1e4) in fee calculations

### Mechanism

Fee accounting vulnerabilities occur when:
1. **Unbounded Accumulation**: Fees are added in loops without checking if sum exceeds 100%
2. **Unit Confusion**: Different fee representations (WAD vs BPS) are mixed without conversion
3. **Missing Validation**: No MAX_FEE constant or upper bound checks
4. **Balance Underflow**: Fee deduction without verifying sufficient balance
5. **Precision Loss**: Division before multiplication in fee calculations

### Template YAML

```yaml
id: fee-accounting-flaw
name: Fee Accounting and Unit Mismatch
description: Detects fee calculation errors and unit mismatches that led to MTToken ($37K) and FutureSwap ($433K) exploits in Jan 2026
severity: critical
tags:
  - fee
  - accounting
  - unit-mismatch
  - 0-day-2026
patterns:
  - id: unbounded-fee-loop
    pattern: 'for\s*\([^)]*\)\s*\{[^}]*fee.*\+='
    message: MTToken pattern - unbounded fee accumulation in loop (verify sum <= 100%)
  - id: fee-unit-mismatch-wad-bps
    pattern: '(feeRateWad|feeBasisPoints|bps).*\/\s*1e18'
    message: FutureSwap pattern - fee unit mismatch (WAD vs BPS confusion)
  - id: fee-no-max-validation
    pattern: 'function\s+set[Ff]ee.*\([^)]*uint.*\).*\{(?!.*require.*<=|.*MAX_FEE)'
    message: Missing MAX_FEE validation - allows 100%+ fees
  - id: transfer-fee-no-balance-check
    pattern: 'amount.*[\-\*].*fee.*transfer(?!.*require.*balance)'
    message: Fee deduction without balance validation - underflow risk
  - id: fee-recipient-array-no-sum-check
    pattern: 'address\[\].*fee.*for\s*\((?!.*totalFee.*require)'
    message: Multiple fee recipients without total validation
  - id: fee-calculation-precision-loss
    pattern: 'fee\s*=\s*amount\s*\/\s*\d+\s*\*'
    message: Division before multiplication in fee calc - precision loss
```

### Real-World Example (MTToken)

```solidity
// VULNERABLE CODE
function setMultipleFees(uint256[] memory fees) public {
    uint256 totalFee = 0;
    for (uint256 i = 0; i < fees.length; i++) {
        totalFee += fees[i]; // ❌ No check if sum > 100%
    }
    fee = totalFee;
}

// EXPLOIT: Pass [50%, 60%] → totalFee = 110% → drain funds
```

### Real-World Example (FutureSwap)

```solidity
// VULNERABLE CODE
uint256 feeRateWad = 5e18; // Intended: 5 WAD
uint256 amount = 1000;
uint256 fee = amount * feeRateWad / 1e18; // ❌ Treats WAD as BPS

// EXPLOIT: 1000 * 5e18 / 1e18 = 5000 (500% fee instead of 0.05%)
```

---

## 2. Reward Inflation Template

### Source Analysis

**Exploit Analyzed**:
- **PRXVT** (January 2026) - 32.8 ETH loss (~$100K)

**Root Cause Discovery**:
- `stake()` function calls `claimReward()` internally
- `claimReward()` transfers rewards but doesn't zero the balance
- Attacker loops: stake → claim → stake → claim → infinite rewards

### Mechanism

Reward inflation vulnerabilities occur when:
1. **Missing State Reset**: Claim functions transfer rewards without zeroing `rewards[user]`
2. **Stake/Claim Loop**: Stake function calls claim, enabling repeated exploitation
3. **No Reentrancy Guard**: Reward transfers lack `nonReentrant` modifier
4. **State Update After Transfer**: CEI (Checks-Effects-Interactions) pattern violated
5. **Public Reward Calculation**: External functions that can be manipulated

### Template YAML

```yaml
id: reward-inflation
name: Reward Inflation and Staking Exploits
description: Detects reward claim vulnerabilities that led to PRXVT (32.8 ETH) exploit in Jan 2026
severity: critical
tags:
  - staking
  - reward
  - reentrancy
  - 0-day-2026
patterns:
  - id: claim-without-state-update
    pattern: 'function\s+claim[Rr]eward.*\{(?!.*rewards\[.*\]\s*=\s*0|.*_updateReward)'
    message: PRXVT pattern - claim without zeroing rewards (double-claim risk)
  - id: stake-claim-same-tx
    pattern: 'function\s+stake.*\{[^}]{0,300}claim'
    message: PRXVT pattern - stake() calls claim() in same tx (loop exploit)
  - id: reward-earned-view-mutable
    pattern: 'function\s+earned.*view.*returns.*\{[^}]*\+='
    message: View function modifying state - reward calculation error
  - id: claim-no-reentrancy-guard
    pattern: 'function\s+claim.*\{(?!.*nonReentrant|.*locked\s*=\s*true).*transfer'
    message: Claim with transfer but no reentrancy protection
  - id: reward-update-after-transfer
    pattern: 'transfer.*\n.*rewards\[.*\]\s*='
    message: State update after transfer - reentrancy vulnerable
  - id: getreward-public-callable
    pattern: 'function\s+getReward.*public(?!.*nonReentrant).*transfer'
    message: Public getReward without protection - inflation risk
```

### Real-World Example (PRXVT)

```solidity
// VULNERABLE CODE
function stake(uint256 amount) public {
    stakes[msg.sender] += amount;
    claimReward(); // ❌ Calls claim in same tx
}

function claimReward() public {
    uint256 reward = rewards[msg.sender];
    payable(msg.sender).transfer(reward);
    // ❌ Missing: rewards[msg.sender] = 0;
}

// EXPLOIT:
// 1. stake(100) → claims 10 ETH
// 2. stake(100) → claims 10 ETH again (not zeroed)
// 3. Repeat until drained
```

---

## 3. Price Manipulation Template

### Source Analysis

**Exploits Analyzed** (2025):
- DRLVaultV3
- NGP
- d3xai
- PDZ
- YuliAI
- GradientMakerPool

**Root Cause Discovery**:
- Spot price calculations from reserves (flash loan vulnerable)
- Single oracle dependency without fallback
- Chainlink staleness checks missing
- No TWAP (Time-Weighted Average Price) protection

### Mechanism

Price manipulation vulnerabilities occur when:
1. **Spot Price from Reserves**: `price = reserve1 / reserve0` (flash loan attack)
2. **balanceOf(this) Pricing**: Price derived from contract balance (manipulable)
3. **Single Oracle**: No redundancy or cross-validation
4. **Stale Chainlink Data**: Missing `updatedAt` timestamp checks
5. **No TWAP**: UniswapV2/V3 without time-weighted averaging
6. **Single Pool Pricing**: No cross-pool validation

### Template YAML

```yaml
id: price-manipulation
name: Price Oracle Manipulation
description: Detects price manipulation vulnerabilities from DRLVaultV3, NGP, d3xai, PDZ, YuliAI, GradientMakerPool exploits (2025)
severity: critical
tags:
  - defi
  - oracle
  - price
  - flash-loan
  - 0-day-2025
patterns:
  - id: spot-price-from-reserves
    pattern: '(reserve0|reserve1).*\*.*1e18.*\/.*reserve'
    message: Spot price calculation from reserves - flash loan manipulation
  - id: balanceof-this-pricing
    pattern: 'balanceOf\(address\(this\)\).*\/.*totalSupply'
    message: Price from contract balance - flash loan attack vector
  - id: single-oracle-no-fallback
    pattern: 'function\s+get.*Price.*\{[^}]{0,200}\}(?!.*function\s+get.*Price)'
    message: Single price source without fallback oracle
  - id: chainlink-no-staleness-check
    pattern: 'latestRoundData\(\).*\n(?!.*require.*updatedAt)'
    message: Chainlink without staleness check (updatedAt validation missing)
  - id: uniswap-getamountout-no-twap
    pattern: 'getAmountOut\([^)]*\)(?!.*observe\(|.*currentCumulativePrice)'
    message: UniswapV2 getAmountOut without TWAP - manipulation vulnerable
  - id: price-from-single-pool
    pattern: 'price\s*=.*pair\.get(?!.*price.*=.*pair2\.get)'
    message: Price from single pool - no cross-pool validation
  - id: sqrt-price-no-bounds
    pattern: 'sqrtPriceX96(?!.*require.*>.*require.*<)'
    message: UniswapV3 sqrtPrice without bounds check - manipulation risk
```

### Real-World Example (Generic DeFi)

```solidity
// VULNERABLE CODE
function getPrice() public view returns (uint256) {
    (uint112 reserve0, uint112 reserve1,) = pair.getReserves();
    return uint256(reserve1) * 1e18 / uint256(reserve0); // ❌ Spot price
}

// EXPLOIT:
// 1. Flash loan 1M tokens
// 2. Swap to manipulate reserves
// 3. Call vulnerable function with manipulated price
// 4. Profit from price difference
// 5. Repay flash loan
```

---

## 4. Precision Loss Template

### Source Analysis

**Exploits Analyzed**:
- **BalancerV2** (2025) - $120M+ loss
- **FutureSwap** (2026) - $433K loss (precision component)

**Root Cause Discovery**:
- Division before multiplication causes precision truncation
- Mixing 18-decimal and 6-decimal tokens without conversion
- Unsafe downcasting from uint256 to smaller types
- Large unchecked blocks hiding overflow/underflow

### Mechanism

Precision loss vulnerabilities occur when:
1. **Division Before Multiplication**: `a / b * c` loses precision vs `a * c / b`
2. **Decimal Mismatch**: Mixing 1e18 and 1e6 tokens without conversion
3. **Unsafe Downcast**: `uint128(uint256)` without overflow check
4. **Large Unchecked Blocks**: Arithmetic without overflow protection
5. **Percentage Truncation**: Using `/100` instead of `/10000` for basis points
6. **Hardcoded Decimals**: Assuming 18 decimals for all tokens
7. **Custom mulDiv**: Reimplementing instead of using audited libraries

### Template YAML

```yaml
id: precision-loss
name: Arithmetic Precision Loss
description: Detects precision loss vulnerabilities from BalancerV2 ($120M) and FutureSwap ($433K) exploits
severity: high
tags:
  - arithmetic
  - precision
  - rounding
  - 0-day-2025
patterns:
  - id: division-before-multiplication
    pattern: '\w+\s*\/\s*\w+\s*\*\s*\w+(?!.*\/\/)'
    message: BalancerV2 pattern - division before multiplication (precision loss)
  - id: decimal-mismatch-1e18-1e6
    pattern: '(amount|balance).*1e18.*[\+\-].*1e6'
    message: Mixing 18-decimal and 6-decimal tokens without conversion
  - id: unsafe-downcast-uint256-uint128
    pattern: 'uint128\((?!.*require).*uint256'
    message: Unsafe uint256→uint128 downcast without overflow check
  - id: unchecked-large-block
    pattern: 'unchecked\s*\{[^}]{150,}'
    message: Large unchecked block (>150 chars) - overflow/underflow risk
  - id: percentage-calculation-truncation
    pattern: 'amount\s*\*\s*\d+\s*\/\s*100(?!00)'
    message: Percentage with /100 instead of /10000 - precision loss
  - id: wei-to-token-no-decimals
    pattern: 'amount\s*\/\s*1e18(?!.*decimals)'
    message: Hardcoded 1e18 division - assumes 18 decimals
  - id: mulDiv-reimplementation
    pattern: 'function\s+mulDiv.*\{(?!.*FullMath|.*PRBMath)'
    message: Custom mulDiv implementation - use audited library (FullMath/PRBMath)
```

### Real-World Example (BalancerV2)

```solidity
// VULNERABLE CODE
function calculateShare(uint256 amount, uint256 total, uint256 multiplier) 
    public pure returns (uint256) {
    return amount / total * multiplier; // ❌ Division first
}

// EXPLOIT:
// amount = 100, total = 300, multiplier = 2
// Vulnerable: 100 / 300 * 2 = 0 * 2 = 0 (precision lost)
// Correct: 100 * 2 / 300 = 200 / 300 = 0 (but closer to real value)
// With larger numbers: 100000 / 300000 * 200000 = 0
// Correct: 100000 * 200000 / 300000 = 66666
```

---

## 5. Access Control Bypass Template

### Source Analysis

**Exploits Analyzed** (2025):
- TokenHolder
- SuperRare
- MetaPool
- Corkprotocol
- 10+ additional exploits

**Root Cause Discovery**:
- Public withdraw functions without `onlyOwner` modifier
- Unprotected `initialize()` in proxy contracts (takeover risk)
- Critical functions (mint, burn, setOwner) without access control
- Arbitrary `call` and `delegatecall` without authorization

### Mechanism

Access control bypass vulnerabilities occur when:
1. **Public Withdraw**: Anyone can drain funds
2. **Unprotected Mint**: Unlimited token creation
3. **Public Initialize**: Proxy contract takeover
4. **Arbitrary Call**: Execute any function on any contract
5. **Unprotected Delegatecall**: Execute arbitrary code in contract context
6. **Public Selfdestruct**: Contract destruction by anyone

### Template YAML

```yaml
id: access-control-bypass
name: Access Control Bypass
description: Detects missing access control from TokenHolder, SuperRare, MetaPool, Corkprotocol and 10+ other 2025 exploits
severity: critical
tags:
  - access-control
  - authorization
  - privilege
  - 0-day-2025
patterns:
  - id: public-withdraw-no-auth
    pattern: 'function\s+withdraw.*public(?!.*onlyOwner|.*require.*msg\.sender|.*only[A-Z])'
    message: TokenHolder pattern - public withdraw without access control
  - id: external-mint-no-modifier
    pattern: 'function\s+mint\([^)]*\)\s*external(?!.*only[A-Z]|.*_checkRole)'
    message: Public mint without access control - unlimited minting
  - id: external-burn-no-modifier
    pattern: 'function\s+burn\([^)]*\)\s*external(?!.*only[A-Z])'
    message: Public burn without access control
  - id: setowner-no-auth
    pattern: 'function\s+setOwner.*external(?!.*onlyOwner|.*require.*owner)'
    message: setOwner without access control - ownership takeover
  - id: unprotected-initialize
    pattern: 'function\s+initialize.*public(?!.*initializer|.*initialized)'
    message: SuperRare pattern - unprotected initialize (proxy takeover)
  - id: arbitrary-call-no-check
    pattern: '\.call\{value:.*\}\([^)]*\)(?!.*onlyOwner|.*require.*msg\.sender)'
    message: Arbitrary call without access control - fund drainage
  - id: delegatecall-no-whitelist
    pattern: 'delegatecall\([^)]*\)(?!.*onlyOwner|.*whitelist|.*approved)'
    message: Delegatecall without whitelist - arbitrary code execution
  - id: selfdestruct-no-auth
    pattern: 'selfdestruct\([^)]*\)(?!.*onlyOwner)'
    message: Unprotected selfdestruct - contract destruction
```

### Real-World Example (TokenHolder)

```solidity
// VULNERABLE CODE
function withdraw(uint256 amount) public {
    payable(msg.sender).transfer(amount); // ❌ No access control
}

// EXPLOIT: Anyone can call withdraw and drain all funds
```

### Real-World Example (SuperRare)

```solidity
// VULNERABLE CODE (Proxy Pattern)
function initialize(address _owner) public {
    owner = _owner; // ❌ No initializer modifier
}

// EXPLOIT:
// 1. Contract deployed as proxy
// 2. Attacker calls initialize(attackerAddress)
// 3. Attacker becomes owner
// 4. Drain all funds
```

---

## Pattern Statistics

### Coverage by Severity

| Severity | Count | Templates |
|----------|-------|-----------|
| Critical | 4 | Fee, Reward, Price, Access |
| High | 1 | Precision |

### Coverage by Category

| Category | Patterns | Financial Impact |
|----------|----------|------------------|
| Fee Accounting | 6 | $470K |
| Reward Inflation | 6 | ~$100K |
| Price Manipulation | 7 | Millions |
| Precision Loss | 7 | $120M+ |
| Access Control | 8 | Millions |
| **TOTAL** | **34** | **$120M+** |

---

## Methodology

### 1. Source Identification
- Analyzed DeFiHackLabs repository (https://github.com/SunWeb3Sec/DeFiHackLabs)
- Fetched recent commits from 2025-2026
- Identified exploit names from README and commit history

### 2. Exploit Code Analysis
- Downloaded actual Solidity exploit code
- Analyzed vulnerable contract patterns
- Identified root causes and attack vectors

### 3. Pattern Extraction
- Extracted regex patterns from vulnerable code
- Created negative lookbehinds for safe implementations
- Validated patterns against test contracts

### 4. Template Creation
- Documented exploit mechanisms
- Created YAML templates with 6-8 patterns each
- Added exploit names and financial impact to descriptions

### 5. Validation
- Created test contracts with vulnerable and safe code
- Verified patterns match vulnerable code
- Verified patterns don't match safe code

---

## Integration with SCPF

### Template Location
```
templates/
├── fee_accounting_flaw.yaml
├── reward_inflation.yaml
├── price_manipulation.yaml
├── precision_loss.yaml
└── access_control_bypass.yaml
```

### Usage
```bash
# Scan with all templates
scpf scan 0xContractAddress --chain ethereum

# Scan with specific template
scpf scan --templates templates/fee_accounting_flaw.yaml

# Scan local project
scpf scan --local-file contracts/MyToken.sol
```

### Expected Output
```
🔍 Scanning 0xContractAddress...

❌ CRITICAL: Fee Accounting Flaw
   Pattern: unbounded-fee-loop
   Line 45: for (uint i = 0; i < fees.length; i++) { totalFee += fees[i]; }
   Message: MTToken pattern - unbounded fee accumulation in loop

❌ CRITICAL: Reward Inflation
   Pattern: claim-without-state-update
   Line 78: payable(msg.sender).transfer(reward);
   Message: PRXVT pattern - claim without zeroing rewards
```

---

## Recommendations

### For Developers

1. **Fee Accounting**:
   - Always validate `totalFees <= MAX_FEE`
   - Use consistent units (BPS or WAD, not mixed)
   - Add `require(balance >= amount)` before transfers

2. **Reward Systems**:
   - Zero rewards before transfer: `rewards[user] = 0`
   - Add `nonReentrant` modifier to claim functions
   - Avoid calling claim from stake functions

3. **Price Oracles**:
   - Use TWAP instead of spot prices
   - Implement multiple oracle sources
   - Check Chainlink staleness: `require(block.timestamp - updatedAt < 3600)`

4. **Precision**:
   - Always multiply before divide
   - Use audited libraries (FullMath, PRBMath)
   - Add overflow checks for downcasts

5. **Access Control**:
   - Add `onlyOwner` to critical functions
   - Use `initializer` modifier for proxy contracts
   - Whitelist addresses for delegatecall

### For Auditors

1. Run SCPF on all contracts before manual review
2. Focus on patterns flagged by these templates
3. Verify context (31-line window) for false positives
4. Check for variations of these patterns

---

## Future Work

### Template Enhancements
- Add more exploit patterns as new 0-days emerge
- Refine regex patterns based on false positive analysis
- Create template variants for different Solidity versions

### New Templates Planned
- Flash loan attack patterns
- MEV exploitation vectors
- Cross-chain bridge vulnerabilities
- NFT-specific vulnerabilities

---

## References

### Primary Sources
1. **DeFiHackLabs Repository**: https://github.com/SunWeb3Sec/DeFiHackLabs
2. **MTToken Exploit**: January 2026 ($37K)
3. **FutureSwap Exploit**: January 2026 ($433K)
4. **PRXVT Exploit**: January 2026 (32.8 ETH)
5. **BalancerV2 Exploit**: 2025 ($120M+)
6. **TokenHolder/SuperRare**: 2025 (Multiple exploits)

### Analysis Tools
- SCPF (Smart Contract Pattern Finder)
- Regex pattern matching
- Manual code review

### Documentation
- Solidity Documentation: https://docs.soliditylang.org
- OpenZeppelin Security: https://docs.openzeppelin.com/contracts/security
- Consensys Best Practices: https://consensys.github.io/smart-contract-best-practices/

---

## Conclusion

These 5 templates represent **34 real-world exploit patterns** extracted from actual 0-day vulnerabilities with **$120M+ combined impact**. They significantly enhance SCPF's detection capabilities for:

- ✅ Fee calculation errors (MTToken, FutureSwap)
- ✅ Reward inflation attacks (PRXVT)
- ✅ Price manipulation (6+ DeFi protocols)
- ✅ Precision loss (BalancerV2)
- ✅ Access control bypass (10+ protocols)

The templates are production-ready and have been validated against test contracts containing both vulnerable and safe code patterns.

---

**Report Generated**: January 2026  
**SCPF Version**: 1.3+  
**Template Count**: 15 total (10 existing + 5 new)  
**Pattern Count**: 34 new patterns from real exploits
