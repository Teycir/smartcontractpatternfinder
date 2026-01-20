# Why So Few Regex-Only Templates?

## Current Template Distribution

### By Pattern Type (25 templates total):

| Type | Count | Templates |
|------|-------|-----------|
| **Regex-only** | 7 | ERC compliance (3), Vyper (4) |
| **Semantic-only** | 11 | Most security vulnerabilities |
| **Mixed (both)** | 6 | Complex vulnerabilities |
| **Default (unspecified)** | 1 | Basic reentrancy |

## Why Regex Templates Are Fewer?

### 1. **Evolution of the Project**

The project evolved from simple regex matching to sophisticated semantic analysis:

```
Phase 1: Regex-only (simple)
  ↓
Phase 2: Added tree-sitter (semantic)
  ↓
Phase 3: Migrated most patterns to semantic (current)
```

### 2. **Semantic Patterns Are Superior**

For most vulnerability detection, semantic analysis is better:

| Vulnerability | Why Semantic Wins |
|---------------|-------------------|
| Reentrancy | Needs to track call order + state changes |
| Access Control | Needs to understand function modifiers |
| Integer Overflow | Needs to understand arithmetic context |
| tx.origin | Needs to distinguish from comments/strings |

### 3. **Regex Is Only Used When Appropriate**

Regex templates exist for specific use cases:

#### **ERC Compliance** (3 templates)
- `erc20_compliance.yaml`
- `erc721_compliance.yaml`
- `erc1155_compliance.yaml`

**Why regex?** Simple interface checking:
```yaml
- pattern: 'function transfer\('
- pattern: 'function balanceOf\('
```
Just checking if functions exist, no complex logic needed.

#### **Vyper Language** (4 templates)
- `vyper_access_control.yaml`
- `vyper_integer_overflow.yaml`
- `vyper_reentrancy.yaml`
- `vyper_timestamp.yaml`

**Why regex?** Tree-sitter grammar is Solidity-focused. Vyper has different syntax, so regex is simpler for basic checks.

### 4. **Mixed Templates Show the Strategy**

6 templates use **both** regex and semantic:

```yaml
# Example: front_running_v2.yaml
patterns:
  # Semantic for complex analysis
  - kind: semantic
    pattern: '(call_expression ...) @call'
  
  # Regex for simple string checks
  - kind: regex
    pattern: 'block\.timestamp'
```

**Strategy**: Use semantic for hard stuff, regex for simple stuff.

## The Real Answer

**Regex templates are few because:**

1. ✅ **Semantic is better** for most vulnerabilities
2. ✅ **Project matured** from regex to semantic
3. ✅ **Regex kept only where appropriate**:
   - Simple interface checks (ERC compliance)
   - Non-Solidity languages (Vyper)
   - Quick string matching in mixed templates

## Should We Add More Regex Templates?

### ❌ No, because:
- Semantic analysis is more accurate
- Fewer false positives
- Better context awareness
- Tree-sitter is fast enough

### ✅ Yes, only if:
- Checking simple string presence
- Performance is absolutely critical
- Pattern is unambiguous (e.g., exact function names)
- Supporting languages without tree-sitter grammar

## Recommendation

**Current distribution is optimal:**
- Regex: 7 templates (28%) - appropriate use cases
- Semantic: 11 templates (44%) - complex analysis
- Mixed: 6 templates (24%) - best of both
- Default: 1 template (4%) - legacy

The low number of regex-only templates reflects **good engineering**, not a limitation.
