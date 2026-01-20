# Semantic Search

SCPF supports two pattern matching modes:

## Pattern Types

### 1. Regex Patterns (Default)
Fast text-based matching using regular expressions.

```yaml
patterns:
  - id: simple-call
    pattern: '\.call\{value:'
    message: External call with value
```

### 2. Semantic Patterns (AST-based)
Precise code structure matching using tree-sitter AST queries.

```yaml
patterns:
  - id: ast-call
    kind: semantic
    pattern: |
      (call_expression
        function: (member_expression
          property: (identifier) @call (#eq? @call "call")))
    message: External call detected
```

## Benefits of Semantic Search

- **Precision**: Matches code structure, not just text
- **Context-aware**: Understands Solidity syntax
- **Fewer false positives**: Ignores comments and strings
- **AST-level analysis**: Detects patterns regex can't

## Tree-sitter Query Syntax

### Basic Patterns

```scheme
; Match any function call
(call_expression) @call

; Match specific identifier
(identifier) @name (#eq? @name "transfer")

; Match member expression
(member_expression
  object: (identifier) @obj
  property: (identifier) @prop)
```

### Predicates

```scheme
; Equality check
(#eq? @var "value")

; Regex match
(#match? @var "^(call|delegatecall)$")

; Not equal
(#not-eq? @var "safe")
```

## Example Templates

### Reentrancy Detection

```yaml
id: semantic-reentrancy
name: Semantic Reentrancy Detection
severity: high
patterns:
  - id: external-call
    kind: semantic
    pattern: |
      (call_expression
        function: (member_expression
          property: (identifier) @call (#eq? @call "call"))) @usage
    message: External call - check for reentrancy
```

### tx.origin Usage

```yaml
id: tx-origin-semantic
name: tx.origin Detection
severity: high
patterns:
  - id: tx-origin
    kind: semantic
    pattern: |
      (member_expression
        object: (identifier) @tx (#eq? @tx "tx")
        property: (identifier) @origin (#eq? @origin "origin"))
    message: Avoid tx.origin for authentication
```

## Performance

- **Regex**: ~1ms per contract
- **Semantic**: ~5-10ms per contract (includes parsing)

Use regex for simple patterns, semantic for complex structure matching.

## Limitations

- Requires valid Solidity syntax
- Slower than regex patterns
- Query syntax learning curve

## Resources

- [Tree-sitter Documentation](https://tree-sitter.github.io/tree-sitter/)
- [Solidity Grammar](https://github.com/JoranHonig/tree-sitter-solidity)
- [Query Syntax](https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries)
