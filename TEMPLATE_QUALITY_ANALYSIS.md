# Template Quality Crisis: Analysis & Questions for Claude Opus

**Date**: 2024-01-21  
**Context**: SCPF has 35 templates generating 30,321 false positives on 6 safe production contracts  
**Problem**: Templates match syntax, not vulnerabilities. Contextual filtering (0.6% reduction) can't fix this.

---

## 🔴 Current State: The Numbers

### Production Contract Results (Safe Contracts)

| Contract | Findings | Expected | False Positives |
|----------|----------|----------|-----------------|
| USDC | 1,147 | 0 | 1,147 (100%) |
| DAI | 1,713 | 0 | 1,713 (100%) |
| UNI | 3,802 | 0 | 3,802 (100%) |
| wstETH | 5,132 | 0 | 5,132 (100%) |
| Uniswap V2 Factory | 6,378 | 0 | 6,378 (100%) |
| Uniswap V2 Router | 12,149 | 0 | 12,149 (100%) |
| **TOTAL** | **30,321** | **0** | **30,321 (100%)** |

### Test Contract (70 lines, realistic DeFi)

| Template | Findings | Notes |
|----------|----------|-------|
| weak-randomness-prevrandao | 52 | Matches `block.` on every line |
| tx-origin-auth | 52 | Matches `tx.` on every line |
| signature-return-unchecked | 52 | Matches every line |
| **TOTAL** | **1,642** | **On 70-line contract** |

**Precision**: 0%  
**Recall**: Unknown (no vulnerable contracts tested)

---

## 📊 Template Categories

### 35 Templates Breakdown

1. **Semantic Templates** (15) - Use tree-sitter queries
   - Examples: `tx-origin-auth`, `weak-randomness-prevrandao`, `critical-function-no-auth`
   - Problem: Overly broad queries match everything
   
2. **Regex Templates** (20) - Use regex patterns
   - Examples: `reentrancy`, `delegatecall`, `unchecked-return`
   - Problem: Match syntax without context

---

## 🔍 Detailed Analysis: Good vs Bad Templates

### Example 1: BAD - weak-randomness-prevrandao

**Current Pattern** (Semantic):
```yaml
pattern: |\n  (member_expression\n    object: (identifier) @obj\n    property: (identifier) @prop)\n  (#eq? @obj \"block\")\n  (#eq? @prop \"prevrandao\")
```

**Problem**: Matches EVERY occurrence of `block.prevrandao`, even in comments or safe contexts.

**Test Result**: 52 matches in 70-line contract (matches every line with "block")

**What It Should Match**:
- `uint256 random = block.prevrandao % 100;` ✅ (used for randomness)
- `if (block.prevrandao > threshold) { ... }` ✅ (used for logic)

**What It Shouldn't Match**:
- `// block.prevrandao is not used` ❌ (comment)
- `uint256 timestamp = block.timestamp;` ❌ (different property)
- `require(block.number > lastBlock);` ❌ (different property)

**Why It's Broken**: Query matches `block.<anything>`, not just `block.prevrandao`.

---

### Example 2: BAD - tx-origin-auth

**Current Pattern** (Semantic):
```yaml
pattern: |\n  (binary_expression\n    (expression\n      (member_expression\n        (identifier) @_tx\n        (identifier) @_origin)))\n  (#eq? @_tx \"tx\")\n  (#eq? @_origin \"origin\")
```

**Problem**: Matches EVERY occurrence of `tx.origin`, even safe uses.

**Test Result**: 52 matches in 70-line contract

**What It Should Match**:
- `require(tx.origin == owner);` ✅ (authentication)
- `if (tx.origin == msg.sender) { ... }` ✅ (comparison)

**What It Shouldn't Match**:
- `// tx.origin should not be used` ❌ (comment)
- `address sender = msg.sender;` ❌ (no tx.origin)
- `require(msg.sender == owner);` ❌ (uses msg.sender, not tx.origin)

**Why It's Broken**: Query matches `tx.<anything>`, not specifically authentication patterns.

---

### Example 3: GOOD - delegatecall-user-input

**Current Pattern** (Regex):
```yaml
pattern: 'delegatecall\s*\([^)]*msg\.(sender|data)'
```

**Why It's Good**:
- Specific: Matches delegatecall with user input
- Contextual: Requires both delegatecall AND msg.sender/data
- Precise: Won't match safe delegatecall to hardcoded addresses

**Test Result**: 0 matches on safe contracts (expected)

---

### Example 4: MEDIUM - reentrancy-state-change

**Current Pattern** (Regex):
```yaml
pattern: '\.call\{value:'
```

**Problem**: Matches ALL call{value, even protected ones.

**What Contextual Filtering Does**:
- Filters if function has `nonReentrant` modifier ✅
- Filters if function has `onlyOwner` modifier ✅
- Still reports if no modifier ❌ (even if CEI pattern used)

**Improvement Needed**:
- Detect CEI pattern (state change before call)
- Detect return value checks
- Detect reentrancy locks (not just modifiers)

---

## ❓ Questions for Claude Opus

### 1. Semantic Query Precision

**Current Issue**: Semantic queries match too broadly.

**Example**: `weak-randomness-prevrandao` matches `block.<anything>` instead of just `block.prevrandao`.

**Questions**:
1. How do we make tree-sitter queries more precise?
2. Should we add negative constraints (NOT in comments, NOT in other contexts)?
3. Is there a way to validate the matched node is actually used (not just referenced)?
4. Should we combine semantic + regex for better precision?

**Specific Fix Needed**:
```yaml
# Current (matches everything)
(member_expression
  object: (identifier) @obj
  property: (identifier) @prop)
(#eq? @obj \"block\")
(#eq? @prop \"prevrandao\")

# Desired (matches only when used for randomness/logic)
# How do we add context that it's being USED, not just mentioned?
```

---

### 2. Context Requirements

**Current Issue**: Patterns match syntax without semantic context.

**Example**: `tx.origin == owner` is vulnerable, but `// tx.origin == owner` (comment) is not.

**Questions**:
1. How do we exclude comments from semantic queries?
2. How do we require the pattern to be in an active code path (not dead code)?
3. Should we check if the matched code is inside a function body?
4. How do we verify the pattern is actually used for authentication (not just comparison)?

---

### 3. False Positive vs False Negative Trade-off

**Current Approach**: Match everything, filter later (100% recall, 0% precision)

**Questions**:
1. What's acceptable precision/recall for a security scanner?
   - Industry standard: 80% precision, 90% recall?
   - Our target: 85% precision, 75% recall?
2. Should we be more conservative (fewer matches, higher confidence)?
3. How do we balance catching real vulnerabilities vs overwhelming users?

---

### 4. Template Validation Strategy

**Current Problem**: No way to validate templates before deployment.

**Questions**:
1. Should we create a test suite of:
   - Known vulnerable contracts (should match)
   - Known safe contracts (should NOT match)
2. How do we measure template quality?
   - Precision per template?
   - False positive rate per template?
3. Should we disable templates that have >50% false positive rate?
4. How do we version templates and track improvements?

---

### 5. Semantic vs Regex Trade-offs

**Observations**:
- Semantic queries are more powerful but harder to get right
- Regex patterns are simpler but less context-aware
- Best templates combine both

**Questions**:
1. When should we use semantic vs regex?
2. Should we have a hybrid approach (semantic for structure, regex for content)?
3. Can we use semantic queries to find candidates, then regex to validate?
4. Should we deprecate overly broad semantic queries?

---

### 6. Specific Template Fixes Needed

**Priority 1: Fix These Templates** (Highest false positive rate)

1. **weak-randomness-prevrandao**
   - Current: Matches every `block.` reference
   - Needed: Match only when used for randomness/logic
   - Question: How to detect "used for randomness"?

2. **tx-origin-auth**
   - Current: Matches every `tx.` reference
   - Needed: Match only when used for authentication
   - Question: How to detect "used for authentication"?

3. **signature-return-unchecked**
   - Current: Matches every line
   - Needed: Match only ECDSA.recover without return check
   - Question: How to verify return value is checked?

4. **critical-function-no-auth**
   - Current: Matches functions by name only
   - Needed: Verify no access control modifier
   - Question: How to detect custom access control patterns?

**Priority 2: Improve These Templates** (Medium false positive rate)

5. **reentrancy-state-change**
   - Current: Matches all `call{value:`
   - Needed: Detect CEI pattern, reentrancy locks
   - Question: How to detect state changes before/after call?

6. **unchecked-return-value**
   - Current: Matches all `.call(` without checking context
   - Needed: Verify return value is actually unchecked
   - Question: How to track return value usage?

---

### 7. Pattern-Based Protection Detection

**Current Issue**: Contextual filtering only detects known modifier names.

**Production Reality**: Contracts use custom implementations.

**Questions**:
1. How do we detect custom reentrancy guards?
   - Pattern: `require(!locked); locked = true; _; locked = false;`
   - Question: How to match this pattern in tree-sitter?

2. How do we detect custom access control?
   - Pattern: `require(msg.sender == owner);` at function start
   - Question: How to verify it's at function start, not nested?

3. How do we detect CEI pattern?
   - Pattern: State changes before external calls
   - Question: Requires CFG analysis or simpler heuristic?

4. How do we detect inherited modifiers?
   - Pattern: OpenZeppelin's `onlyOwner` from `Ownable`
   - Question: Do we need import/inheritance analysis?

---

### 8. Template Structure Improvements

**Current Structure**:
```yaml
id: template-id
name: Template Name
description: Description
severity: high
patterns:
  - id: pattern-id
    pattern: regex or semantic query
    message: Finding message
```

**Questions**:
1. Should we add `confidence` field (0.0-1.0)?
2. Should we add `requires_context` flag (needs contextual filtering)?
3. Should we add `test_cases` with expected matches/non-matches?
4. Should we add `known_false_positives` to document limitations?

**Proposed Structure**:
```yaml
id: template-id
name: Template Name
description: Description
severity: high
confidence: 0.85  # NEW: How confident are we in this pattern?
requires_context: true  # NEW: Needs contextual filtering?
patterns:
  - id: pattern-id
    pattern: regex or semantic query
    message: Finding message
    confidence: 0.90  # NEW: Pattern-specific confidence
test_cases:  # NEW: Validation
  should_match:
    - "require(tx.origin == owner);"
  should_not_match:
    - "require(msg.sender == owner);"
    - "// tx.origin should not be used"
```

---

### 9. Incremental Improvement Strategy

**Question**: What's the best approach to fix 35 templates?

**Option A: Fix All At Once**
- Pros: Comprehensive solution
- Cons: Takes weeks, high risk

**Option B: Fix Top 10 Worst**
- Pros: Quick wins, measurable impact
- Cons: Still have 25 broken templates

**Option C: Disable Bad, Keep Good**
- Pros: Immediate precision improvement
- Cons: Lose coverage

**Option D: Tiered Approach**
- Tier 1 (High Confidence): Enable by default
- Tier 2 (Medium Confidence): Enable with warning
- Tier 3 (Low Confidence): Disable by default
- Question: How to classify templates into tiers?

---

### 10. Production Contract Analysis

**Question**: Should we analyze production contracts to understand real patterns?

**Approach**:
1. Fetch source code from 100 production contracts
2. Analyze common patterns:
   - What modifiers do they actually use?
   - What reentrancy protection patterns?
   - What access control patterns?
3. Build templates based on real-world usage

**Questions**:
1. Is this worth the effort?
2. Would it improve template quality?
3. How do we avoid overfitting to specific contracts?

---

## 🎯 Specific Requests for Claude Opus

### Request 1: Fix Top 3 Worst Templates

Please provide fixed versions of:
1. `weak-randomness-prevrandao` - Currently matches every line
2. `tx-origin-auth` - Currently matches every line
3. `signature-return-unchecked` - Currently matches every line

**Requirements**:
- Precision >80% on safe contracts
- Recall >90% on vulnerable contracts
- Include test cases

---

### Request 2: Template Quality Framework

Please design a framework for:
1. Measuring template quality (precision, recall, F1)
2. Validating templates before deployment
3. Versioning and tracking improvements
4. Disabling low-quality templates

---

### Request 3: Pattern-Based Protection Detection

Please provide tree-sitter queries or regex patterns for:
1. Custom reentrancy guards (lock/unlock pattern)
2. Custom access control (require msg.sender checks)
3. CEI pattern detection (state changes before calls)
4. Return value checking patterns

---

### Request 4: Hybrid Semantic + Regex Approach

Please provide examples of combining semantic and regex:
1. Use semantic to find function definitions
2. Use regex to validate function content
3. Combine both for higher precision

---

## 📋 Summary

### The Core Problem
Templates match **syntax** (what code looks like) instead of **semantics** (what code does).

### Why Contextual Filtering Failed
- Built infrastructure before fixing templates
- 0.6% reduction on production contracts
- Can't fix fundamentally broken patterns

### What We Need
1. **Immediate**: Fix top 10 worst templates
2. **Short-term**: Pattern-based protection detection
3. **Long-term**: Template quality framework

### Success Criteria
- Precision >80% on safe contracts
- Recall >75% on vulnerable contracts
- <100 findings per safe contract (currently 1,000-12,000)

---

**Status**: Awaiting Claude Opus guidance on template fixes and quality framework
