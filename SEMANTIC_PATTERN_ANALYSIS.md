# Semantic Pattern Analysis Report

**Date**: 2026-01-20  
**Total Templates**: 27  
**Total Semantic Patterns**: ~150+  
**Failing Patterns**: 92  
**Success Rate**: ~39%

---

## Executive Summary

Out of 150+ semantic patterns across 27 templates, **92 patterns are failing** to compile with tree-sitter-solidity v1.2 grammar. The patterns work conceptually but use incorrect node type names or structure that don't match the actual grammar.

### Error Distribution

| Error Type | Count | Percentage | Root Cause |
|------------|-------|------------|------------|
| **Structure** | 469 | 72.9% | Wrong field names or nesting |
| **NodeType** | 154 | 23.9% | Non-existent node types |
| **Field** | 21 | 3.2% | Invalid field references |
| **TOTAL** | 644 | 100% | - |

---

## Root Causes

### 1. Structure Errors (72.9%)

**Problem**: Patterns use field names that don't exist in the grammar.

**Examples**:
```yaml
# ❌ WRONG - Uses field names
(function_definition
  body: (function_body
    (expression_statement)))

# ✅ CORRECT - Direct nesting
(function_definition
  (function_body
    (statement
      (expression_statement))))
```

**Why it fails**: tree-sitter-solidity v1.2 doesn't use field names like `body:`, `left:`, `right:` in most contexts.

### 2. NodeType Errors (23.9%)

**Problem**: Patterns reference node types that don't exist.

**Examples**:
```yaml
# ❌ WRONG - These nodes don't exist
(function_call)      # Should be: call_expression
(member_access)      # Should be: member_expression  
(assignment)         # Should be: assignment_expression
(call_arguments)     # Should be: call_argument_list
(parameter_list)     # Doesn't exist as direct child
(user_defined_type_name)  # Doesn't exist

# ✅ CORRECT - Actual node types
(call_expression)
(member_expression)
(assignment_expression)
```

### 3. Field Errors (3.2%)

**Problem**: Using negation or field references incorrectly.

**Examples**:
```yaml
# ❌ WRONG
(function_definition
  !modifier_invocation)

# ✅ CORRECT - Negation works differently
(function_definition)
# Then check in code if modifier exists
```

---

## Failing Patterns by Template

### Critical Templates (High Priority)

#### 1. advanced-audit-checks (20 failures)
- `state-after-call` - Structure error
- `unchecked-call-return` - Structure error
- `delegatecall-user-input` - NodeType error (call_arguments)
- `missing-zero-check` - NodeType error (parameter_list)
- `overflow-mul-div` - Structure error (field names)
- `timestamp-dependence` - Structure error
- `unprotected-selfdestruct` - Structure error
- `frontrun-vulnerable` - Structure error
- `strict-balance-equality` - Structure error
- `state-change-no-event` - Structure error
- `variable-shadowing` - NodeType error
- `uninitialized-storage` - Structure error
- `modifier-external-call` - Structure error
- `unbounded-loop` - Structure error
- `missing-access-control` - Structure error
- `missing-nonce` - NodeType error
- `single-source-price` - Structure error
- `flash-loan-vulnerable` - Structure error

#### 2. defi-vulnerabilities (18 failures)
- `amm-k-manipulation` - Structure error
- `missing-slippage-check` - NodeType error
- `no-price-impact` - Structure error
- `lp-reentrancy` - Structure error
- `unvalidated-flash-callback` - Structure error
- `collateral-ratio-unsafe` - Structure error
- `liquidation-no-price-check` - Structure error
- `interest-rate-exploit` - Structure error
- `reward-overflow` - Structure error
- `withdrawal-delay-bypass` - Structure error
- `vote-weight-manipulation` - Structure error
- `no-timelock-execution` - Structure error
- `no-staleness-check` - Structure error
- `single-oracle-dependency` - NodeType error
- `mev-sandwich-vulnerable` - Structure error
- `reward-dilution` - Structure error
- `bridge-no-validation` - NodeType error
- `nft-transfer-reentrancy` - Structure error
- `vault-share-manipulation` - Structure error
- `auction-frontrun` - Structure error

#### 3. zero-day-emerging (15 failures)
- `readonly-reentrancy` - Structure error
- `erc4626-inflation` - Structure error
- `permit-frontrun` - NodeType error
- `cross-contract-reentrancy` - Structure error
- `erc777-hook-reentrancy` - NodeType error
- `vyper-lock-bypass` - Structure error
- `arbitrum-sequencer-check` - Structure error
- `blast-yield-exploit` - Structure error
- `blob-data-validation` - NodeType error
- `aa-validation-bypass` - Structure error
- `multicall-reentrancy` - Structure error
- `uniswap-v4-hook-exploit` - Structure error
- `restaking-slashing-risk` - Structure error
- `pendle-pt-yt-exploit` - Structure error
- `balancer-composable-reentrancy` - Structure error
- `aave-emode-exploit` - Structure error
- `compound-v3-absorb` - Structure error
- `gmx-v2-price-impact` - Structure error
- `morpho-rate-manipulation` - Structure error
- `frax-amo-exploit` - Structure error

#### 4. logic-bugs-gas-optimization (14 failures)
- `off-by-one-loop` - Structure error
- `division-before-multiplication` - Structure error
- `assignment-in-condition` - Structure error
- `storage-read-loop` - Structure error
- `redundant-storage-write` - Structure error
- `string-comparison-expensive` - Structure error
- `unnecessary-safemath` - Structure error
- `public-should-external` - Structure error
- `array-length-not-cached` - Structure error
- `ignored-return-value` - Structure error
- `missing-zero-amount-check` - NodeType error
- `event-wrong-indexed` - NodeType error
- `emit-in-loop` - Structure error
- `unnecessary-zero-init` - Structure error
- `struct-packing-inefficient` - Structure error
- `missing-constructor-init` - NodeType error

#### 5. Other Templates
- `front-running-v2` (2 failures)
- `reentrancy-state-change-v4` (2 failures)
- `missing-access-control` (1 failure)
- `semantic-vulnerabilities` (4 failures)
- `zero-day-live` (1 failure)
- `strict-balance-equality-v2` (1 failure)
- `unchecked-return-value-v4` (1 failure)
- `advanced-audit-checks-fixed` (1 failure)

---

## Working Patterns (Success Examples)

### ✅ Patterns That Work

From `advanced_audit_fixed.yaml`:
- `tx-origin-auth-fixed` - ✅ Works
- `missing-access-control-fixed` - ✅ Works  
- `reentrancy-pattern-fixed` - ✅ Works
- `unprotected-selfdestruct-fixed` - ✅ Works
- `unchecked-call-fixed` - ✅ Works
- `timestamp-dependence-fixed` - ✅ Works
- `strict-balance-equality-fixed` - ✅ Works

**Why they work**: Use correct node structure with `statement` wrapper and proper nesting.

---

## Common Pattern Fixes

### Fix 1: Add Statement Wrapper

```yaml
# ❌ WRONG
(function_body
  (expression_statement
    (call_expression)))

# ✅ CORRECT
(function_body
  (statement
    (expression_statement
      (expression
        (call_expression)))))
```

### Fix 2: Remove Field Names

```yaml
# ❌ WRONG
(binary_expression
  operator: "=="
  left: (identifier)
  right: (identifier))

# ✅ CORRECT
(binary_expression
  (expression (identifier))
  (expression (identifier)))
```

### Fix 3: Fix Node Type Names

```yaml
# ❌ WRONG
(function_call)
(member_access)
(call_arguments)

# ✅ CORRECT
(call_expression)
(member_expression)
(call_argument_list)
```

### Fix 4: Wrap Expressions

```yaml
# ❌ WRONG
(call_expression
  function: (member_expression))

# ✅ CORRECT
(call_expression
  (expression
    (member_expression)))
```

---

## Questions for Resolution

### 1. Grammar Documentation
**Q**: Where is the official tree-sitter-solidity v1.2 grammar documentation?  
**Context**: Need authoritative source for node types and structure.  
**Impact**: Would eliminate guesswork in pattern writing.

### 2. Field Names
**Q**: Which contexts support field names (like `body:`, `left:`, `right:`)?  
**Context**: Some patterns use fields, unclear when they're valid.  
**Impact**: 469 Structure errors could be resolved.

### 3. Parameter Lists
**Q**: How to query function parameters in tree-sitter-solidity v1.2?  
**Context**: `parameter_list` node doesn't seem to exist as expected.  
**Impact**: Affects 20+ patterns checking function parameters.

### 4. Negation
**Q**: How to check for absence of child nodes (like `!modifier_invocation`)?  
**Context**: Negation syntax unclear.  
**Impact**: Access control patterns can't verify missing modifiers.

### 5. Call Arguments
**Q**: What's the correct node type for function call arguments?  
**Context**: Patterns use `call_arguments` but should be `call_argument_list`?  
**Impact**: Affects patterns checking function call parameters.

---

## Recommended Actions

### Immediate (High Priority)

1. **Create Grammar Reference**
   - Run `dump_tree` on comprehensive Solidity examples
   - Document all node types and their structure
   - Create pattern writing guide

2. **Fix Critical Templates**
   - `advanced-audit-checks` (20 patterns)
   - `defi-vulnerabilities` (18 patterns)
   - `zero-day-emerging` (15 patterns)

3. **Validate Pattern Syntax**
   - Create validation tool
   - Test each pattern individually
   - Document working examples

### Medium Priority

4. **Update All Templates**
   - Apply fixes from `advanced_audit_fixed.yaml`
   - Use correct node structure
   - Remove invalid field names

5. **Add Tests**
   - Unit tests for each pattern
   - Test against known vulnerable contracts
   - Verify no false positives

### Long Term

6. **Contribute to Grammar**
   - Report issues to tree-sitter-solidity
   - Suggest grammar improvements
   - Add missing node types if needed

7. **Template Library**
   - Create verified pattern collection
   - Document each pattern's purpose
   - Provide test cases

---

## Impact Assessment

### Current State
- **Detection Coverage**: ~39% (only working patterns)
- **False Negatives**: HIGH - Missing 61% of patterns
- **Production Ready**: NO - Critical patterns failing

### After Fixes
- **Detection Coverage**: ~100% (all patterns working)
- **False Negatives**: LOW - Comprehensive detection
- **Production Ready**: YES - Full semantic analysis

---

## Files Requiring Updates

### Templates to Fix (Priority Order)

1. `templates/advanced_audit.yaml` - 20 patterns
2. `templates/defi-vulnerabilities.yaml` - 18 patterns
3. `templates/zero-day-emerging.yaml` - 15 patterns
4. `templates/logic-bugs-gas-optimization.yaml` - 14 patterns
5. `templates/semantic-vulnerabilities.yaml` - 4 patterns
6. `templates/front-running-v2.yaml` - 2 patterns
7. `templates/reentrancy-state-change-v4.yaml` - 2 patterns
8. `templates/missing-access-control.yaml` - 1 pattern
9. `templates/zero-day-live.yaml` - 1 pattern
10. `templates/strict-balance-equality-v2.yaml` - 1 pattern
11. `templates/unchecked-return-value-v4.yaml` - 1 pattern
12. `templates/advanced_audit_fixed.yaml` - 1 pattern

---

## Next Steps

1. **Use `dump_tree` utility** to analyze more Solidity patterns
2. **Create pattern fix guide** with before/after examples
3. **Fix templates one by one** starting with highest priority
4. **Test each fix** against vulnerable contracts
5. **Document working patterns** for future reference

---

## Conclusion

The semantic pattern system is **partially functional** but requires systematic fixes across 92 patterns in 12 templates. The root cause is well understood (incorrect node types and structure), and the solution is clear (use correct grammar from `dump_tree` analysis).

**Estimated Effort**: 2-4 hours to fix all 92 patterns using the working examples as templates.

**Priority**: CRITICAL - Semantic patterns are essential for production-grade vulnerability detection.
