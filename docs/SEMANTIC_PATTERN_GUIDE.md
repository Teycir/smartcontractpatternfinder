# Tree-Sitter Semantic Pattern Guide for SCPF

## Context
We use tree-sitter-solidity v1.2.13 for semantic analysis. Patterns must be precise to avoid false positives.

## Grammar Reference
Node types: https://github.com/JoranHonig/tree-sitter-solidity/blob/master/src/node-types.json

## Key Node Types
- `function_definition` - Function declarations
- `function_body` - Function body (NOT `block`)
- `block_statement` - Block statements
- `call_expression` - Function calls
- `member_expression` - Property access (e.g., `msg.sender`, `address.call`)
- `identifier` - Variable/function names
- `assignment_expression` - Assignments
- `binary_expression` - Binary operations
- `for_statement` - For loops
- `modifier_definition` - Modifiers

## Pattern Requirements

### ✅ GOOD: Specific, Low False Positives
```yaml
# Detect reentrancy: external call followed by state change
- id: reentrancy-pattern
  kind: semantic
  pattern: |
    (function_body
      (expression_statement
        (call_expression
          function: (member_expression
            property: (identifier) @method (#match? @method "^(call|send|transfer)$"))))
      (expression_statement
        (assignment_expression)))
  message: State change after external call - reentrancy risk
```

### ✅ GOOD: Precise Member Expression
```yaml
# Detect tx.origin usage
- id: tx-origin
  kind: semantic
  pattern: |
    (member_expression
      object: (identifier) @tx (#eq? @tx "tx")
      property: (identifier) @origin (#eq? @origin "origin"))
  message: tx.origin usage - use msg.sender instead
```

### ❌ BAD: Too Broad (False Positives)
```yaml
# This matches EVERY call expression
- id: call-expression
  kind: semantic
  pattern: '(call_expression) @call'
  message: External call detected
```

### ❌ BAD: Too Broad (False Positives)
```yaml
# This matches EVERY assignment
- id: assignment
  kind: semantic
  pattern: '(assignment_expression) @assign'
  message: State change detected
```

## Vulnerability Patterns Needed

### 1. Reentrancy (High Priority)
**Goal:** Detect external call followed by state change in same function
**Pattern:** Look for `call/send/transfer` followed by assignment in `function_body`

### 2. Unchecked Low-Level Call
**Goal:** Detect `call/delegatecall/send` without checking return value
**Pattern:** `call_expression` with low-level call NOT wrapped in require/if

### 3. Delegatecall with User Input
**Goal:** Detect delegatecall where argument comes from msg.data/calldata
**Pattern:** `delegatecall` with argument matching user input patterns

### 4. Missing Access Control
**Goal:** Detect critical functions (withdraw, mint, burn) without modifiers
**Pattern:** `function_definition` with specific names, no `modifier_invocation`

### 5. Timestamp Dependence
**Goal:** Detect block.timestamp in conditional logic
**Pattern:** `block.timestamp` inside `if_statement` or comparison

### 6. Unprotected Selfdestruct
**Goal:** Detect selfdestruct without access control
**Pattern:** `selfdestruct` in function without `onlyOwner` modifier

### 7. Integer Overflow (Pre-0.8.0)
**Goal:** Detect multiplication before division
**Pattern:** `binary_expression` with `*` nested in `/` operation

### 8. DoS with Block Gas Limit
**Goal:** Detect unbounded loops with external calls
**Pattern:** `for_statement` containing `call_expression`

### 9. Front-Running Vulnerable
**Goal:** Detect public functions using msg.value without protection
**Pattern:** `function_definition` (public/external) with `msg.value` in body

### 10. Strict Balance Equality
**Goal:** Detect `== balance` comparisons
**Pattern:** `binary_expression` with `==` and `balance` property

## Pattern Template

```yaml
id: vulnerability-name
name: Human Readable Name
description: What this detects and why it matters
severity: high  # critical, high, medium, low, info
tags:
  - security
  - semantic
patterns:
  - id: pattern-id
    kind: semantic
    pattern: |
      (node_type
        field: (child_node
          property: (identifier) @var (#eq? @var "value")))
    message: Clear description of the issue and fix
```

## Testing Patterns

Test against this vulnerable contract:
```solidity
contract Vulnerable {
    mapping(address => uint) public balances;
    
    // Reentrancy
    function withdraw() public {
        uint amount = balances[msg.sender];
        msg.sender.call{value: amount}("");  // External call
        balances[msg.sender] = 0;  // State change after
    }
    
    // Unchecked call
    function unsafeCall(address target) public {
        target.call("");  // Return value not checked
    }
    
    // tx.origin
    function authenticate() public {
        require(tx.origin == owner);  // Should use msg.sender
    }
    
    // Timestamp dependence
    function timelock() public {
        require(block.timestamp > unlockTime);
    }
    
    // Unprotected selfdestruct
    function destroy() public {
        selfdestruct(payable(msg.sender));  // No access control
    }
}
```

## Success Criteria
- ✅ Detects real vulnerabilities
- ✅ Minimal false positives (<10%)
- ✅ Clear, actionable messages
- ✅ Patterns compile without errors

## Request to AI
Please generate tree-sitter semantic patterns for the 10 vulnerabilities listed above. Each pattern must:
1. Use correct node types from tree-sitter-solidity v1.2.13
2. Be specific enough to avoid false positives
3. Include clear messages explaining the issue
4. Follow the YAML template format
5. Be tested against the vulnerable contract above
