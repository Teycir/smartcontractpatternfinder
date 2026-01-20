# Tree-Sitter Solidity Grammar Update Guide

## Problem

Current semantic patterns fail with: `Failed to compile tree-sitter query`

All 5 optimized templates use semantic patterns with incorrect node type names.

## Root Cause

Wrong node type names used:
- ❌ `binary_expression` → ✅ `binary_operation`
- ❌ `member_expression` → ✅ `member_access`
- ❌ `call_expression` → ✅ `function_call`
- ❌ `assignment_expression` → ✅ `assignment`
- ❌ Field-based matching (`function:`, `left:`) → ✅ Use positional children

## Actual Grammar Nodes

```bash
# Inspect AST
echo 'contract T { function f() { x.balance == 0; } }' > test.sol
tree-sitter parse test.sol
```

Actual nodes:
- `function_call` (not call_expression)
- `member_access` (not member_expression)
- `binary_operation` (not binary_expression)
- `assignment` (not assignment_expression)
- `identifier`, `number_literal` (same)

## Solution: Use Correct Node Names + Regex Fallbacks

### Corrected Semantic Patterns

**Balance Equality:**
```yaml
- id: balance-member-access
  kind: semantic
  pattern: |
    (member_access
      (_) @obj
      (identifier) @prop
      (#eq? @prop "balance")) @balance_ref
  message: "Balance access detected"
```

**External Call:**
```yaml
- id: external-call
  kind: semantic
  pattern: |
    (function_call
      (member_access
        (_) @target
        (identifier) @method)
      (#match? @method "^(call|delegatecall|send)$")) @ext_call
  message: "External call detected"
```

**Selfdestruct:**
```yaml
- id: selfdestruct-call
  kind: semantic
  pattern: |
    (function_call
      (identifier) @func
      (#match? @func "^(selfdestruct|suicide)$")) @destruct
  message: "Selfdestruct detected"
```

### Regex Fallbacks (Immediate Fix)

Replace semantic patterns with regex in all 5 templates:

```yaml
# Balance Equality
- id: balance-equality-regex
  kind: regex
  pattern: '\.balance\s*(==|!=)'
  message: "Strict balance equality"

# Reentrancy
- id: call-then-assign-regex
  kind: regex
  pattern: '\.(call|delegatecall)\s*[\({][^;]*;[^}]*\n\s*\w+\s*(\[[\w\.\[\]]+\])?\s*[-+*/]?='
  message: "State change after external call"

# Unchecked Return
- id: unchecked-call-regex
  kind: regex
  pattern: '^\s*[^=\n]*\.(call|delegatecall|send)\s*[\({]'
  message: "Unchecked low-level call"

# Selfdestruct
- id: selfdestruct-regex
  kind: regex
  pattern: '(selfdestruct|suicide)\s*\('
  message: "Selfdestruct usage"

# Front-running
- id: hash-comparison-regex
  kind: regex
  pattern: '(keccak256|sha256)\s*\([^)]+\)\s*=='
  message: "Hash comparison - front-runnable"
```

## Implementation Steps

1. **Update all 5 templates** with corrected node names:
   - `call_expression` → `function_call`
   - `member_expression` → `member_access`
   - Remove field-based matching (`function:`, `left:`, `right:`)
   - Use positional children or predicates only

2. **Add regex fallbacks** for every semantic pattern

3. **Test:**
   ```bash
   ./target/release/scpf scan test_vulnerable.sol -v 2>&1 | grep -c "Failed to compile"
   # Should output: 0
   ```

4. **Verify detections:**
   ```bash
   ./target/release/scpf scan test_vulnerable.sol | grep -E "(HIGH|CRITICAL)"
   ```

## Hybrid Approach (Recommended)

Use both semantic + regex in each template:

```yaml
patterns:
  # Semantic (precise, may fail on old grammar)
  - id: balance-access-semantic
    kind: semantic
    pattern: |
      (member_access
        (_) @obj
        (identifier) @prop
        (#eq? @prop "balance"))
    message: "Balance access (semantic)"
  
  # Regex (always works, less precise)
  - id: balance-equality-regex
    kind: regex
    pattern: '\.balance\s*(==|!=)'
    message: "Balance equality (regex)"
```

## Files to Update

1. `templates/reentrancy_state_change.yaml` - Fix `function_call`, `member_access`
2. `templates/unchecked_return_value.yaml` - Fix `function_call`, add regex
3. `templates/unprotected_selfdestruct_v2.yaml` - Fix `function_call`
4. `templates/strict_balance_equality_v2.yaml` - Fix `member_access`, add regex
5. `templates/front_running_v2.yaml` - Fix `function_call`, `member_access`

## Node Name Mapping

| Wrong Name | Correct Name | Usage |
|------------|--------------|-------|
| `call_expression` | `function_call` | `foo.call()`, `keccak256()` |
| `member_expression` | `member_access` | `msg.sender`, `x.balance` |
| `binary_expression` | `binary_operation` | `a == b`, `x != 0` |
| `assignment_expression` | `assignment` | `x = 5` |
| `augmented_assignment_expression` | `augmented_assignment` | `x += 5` |
| `function:` field | positional child | Use `(function_call ...)` |
| `left:`, `right:` fields | positional children | Use predicates instead |

## Expected Outcome

After fix:
- ✅ Zero "Failed to compile" warnings
- ✅ Semantic patterns work (when grammar supports)
- ✅ Regex fallbacks ensure coverage
- ✅ 700+ vulnerabilities detected in test files
