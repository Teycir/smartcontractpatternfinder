# Template System Architecture

## Overview
SCPF uses a **dual pattern matching system** with two complementary approaches:

## 1. Regex Patterns (Simple Text Matching)

### Purpose
Fast, lightweight pattern matching for simple vulnerability detection.

### How It Works
- Uses regular expressions to search source code as plain text
- No parsing or AST analysis required
- Fast but less precise

### Example: `templates/reentrancy.yaml`
```yaml
patterns:
  - id: external-call-with-value
    pattern: '\.call\{value:'  # Regex pattern
    message: External call with value transfer detected
```

### Use Cases
- Quick scans
- Simple string patterns
- Performance-critical scenarios
- Basic vulnerability detection

## 2. Semantic Patterns (Tree-Sitter AST)

### Purpose
Deep, accurate analysis using Abstract Syntax Tree (AST) parsing.

### How It Works
- Uses tree-sitter to parse Solidity code into AST
- Queries AST structure using tree-sitter query language
- Understands code semantics, not just text
- More accurate, fewer false positives

### Example: `templates/semantic_working.yaml`
```yaml
patterns:
  - id: tx-origin
    kind: semantic  # Specifies semantic analysis
    pattern: |
      (member_expression
        object: (identifier) @tx (#eq? @tx "tx")
        property: (identifier) @origin (#eq? @origin "origin"))
    message: tx.origin usage detected
```

### Use Cases
- Precise vulnerability detection
- Context-aware analysis
- Complex pattern matching
- Reducing false positives

## Pattern Types in Code

### PatternKind Enum
```rust
pub enum PatternKind {
    Regex,    // Default: Simple text matching
    Semantic, // Tree-sitter AST analysis
}
```

## Why Both?

### Regex Patterns
✅ **Pros:**
- Fast execution
- Simple to write
- No parsing overhead
- Good for obvious patterns

❌ **Cons:**
- False positives
- No context awareness
- Can't understand code structure

### Semantic Patterns
✅ **Pros:**
- Highly accurate
- Context-aware
- Understands code structure
- Fewer false positives

❌ **Cons:**
- Slower execution
- More complex to write
- Requires tree-sitter grammar

## Template Structure

### Regex Template
```yaml
id: simple-check
name: Simple Vulnerability Check
severity: medium
patterns:
  - id: pattern-1
    pattern: 'regex_here'  # No 'kind' = defaults to Regex
    message: Issue found
```

### Semantic Template
```yaml
id: semantic-check
name: Semantic Vulnerability Check
severity: high
patterns:
  - id: pattern-1
    kind: semantic  # Explicitly semantic
    pattern: '(tree_sitter_query) @node'
    message: Issue found
```

## Scanner Behavior

The scanner automatically:
1. Checks `pattern.kind` field
2. If `Semantic`: Uses tree-sitter AST analysis
3. If `Regex` (default): Uses regex matching
4. Both can coexist in the same template

## Best Practices

### Use Regex When:
- Pattern is simple string matching
- Performance is critical
- Pattern is unambiguous (e.g., `selfdestruct`)

### Use Semantic When:
- Need to understand code context
- Avoiding false positives is critical
- Pattern involves code structure (e.g., function calls with specific arguments)

## Example: Combining Both

```yaml
id: comprehensive-check
name: Comprehensive Security Check
severity: high
patterns:
  # Fast regex check
  - id: quick-check
    pattern: '\.call\('
    message: External call detected
  
  # Precise semantic check
  - id: precise-check
    kind: semantic
    pattern: |
      (call_expression
        function: (member_expression
          property: (identifier) @method (#eq? @method "call")))
    message: External call with context
```

## Performance Comparison

| Aspect | Regex | Semantic |
|--------|-------|----------|
| Speed | ⚡ Very Fast | 🐢 Slower |
| Accuracy | ⚠️ Moderate | ✅ High |
| False Positives | ❌ Higher | ✅ Lower |
| Complexity | ✅ Simple | ⚠️ Complex |
| Context Awareness | ❌ No | ✅ Yes |

## Conclusion

The dual system provides:
- **Flexibility**: Choose the right tool for each pattern
- **Performance**: Use regex for simple checks
- **Accuracy**: Use semantic for complex analysis
- **Scalability**: Mix both approaches in templates
