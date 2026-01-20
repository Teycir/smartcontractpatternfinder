# Semantic Pattern Failure Analysis

## Context

**Project**: Smart Contract Pattern Finder (SCPF)
**Issue**: Semantic patterns failing to compile with tree-sitter-solidity grammar
**Severity**: CRITICAL - Semantic patterns are the primary detection method

## Current State

### What Works
- ✅ Regex patterns compile and execute
- ✅ Scanner infrastructure functional
- ✅ Template loading system working
- ✅ Report generation working
- ✅ Multi-chain scanning working
- ✅ 0-day template updates working

### What Fails
- ❌ ALL semantic patterns fail with "Failed to compile tree-sitter query"
- ❌ 40+ semantic patterns skipped during scan
- ❌ Critical vulnerability detection compromised

## Technical Details

### Dependencies
```toml
tree-sitter = "0.26"
tree-sitter-solidity = "1.2"  # Latest available on crates.io
tree-sitter-language = "0.1"
```

### Error Pattern
```
WARN scpf_core::scanner: Skipping semantic pattern 'unbounded-loop' in template 'advanced-audit-checks': Failed to compile tree-sitter query
WARN scpf_core::scanner: Skipping semantic pattern 'missing-access-control' in template 'advanced-audit-checks': Failed to compile tree-sitter query
[... 40+ similar warnings ...]
```

### Example Failing Query
```yaml
# Template: advanced_audit.yaml
- id: unbounded-loop
  kind: semantic
  pattern: |
    (for_statement
      body: (function_body
        (expression_statement
          (call_expression))))
  message: Loop with external calls - vulnerable to DoS via block gas limit
```

### Scanner Implementation
**File**: `crates/scpf-core/src/semantic.rs`

```rust
pub fn scan_with_tree(
    &mut self,
    source: &str,
    tree: &Tree,
    pattern: &Pattern,
    template_id: &str,
    severity: Severity,
    file_path: PathBuf,
) -> Result<Vec<Match>> {
    let language = tree_sitter_solidity::LANGUAGE.into();
    let query = Query::new(&language, &pattern.pattern)
        .context("Failed to compile tree-sitter query")?;  // FAILS HERE
    // ...
}
```

## Test Results

### Unit Tests
```bash
cargo test test_reentrancy_detection
# Result: PASS ✅
```

**Working test query**:
```rust
pattern: r#"(call_expression) @call"#
```

### Production Scan
```bash
./scanners/full-audit-with-reports.sh
# Result: 40+ semantic patterns FAIL ❌
```

## Affected Templates

1. `advanced_audit.yaml` - 20+ patterns
2. `front-running-v2.yaml` - 2 patterns
3. `reentrancy-state-change-v4.yaml` - 2 patterns
4. `missing-access-control.yaml` - 1 pattern
5. `tx-origin-authentication.yaml` - 3 patterns
6. `unprotected-selfdestruct-v2.yaml` - 2 patterns
7. `logic-bugs-gas-optimization.yaml` - 10+ patterns

## Root Cause Analysis

### Hypothesis 1: Grammar Version Mismatch
- tree-sitter-solidity v1.2 is latest on crates.io
- v2.0 doesn't exist yet
- Grammar may have changed node names

### Hypothesis 2: Query Syntax Issues
- Queries work in unit tests with simple patterns
- Complex queries with nested nodes fail
- Possible syntax incompatibility

### Hypothesis 3: Language Binding Issues
- `tree_sitter_solidity::LANGUAGE.into()` conversion
- Possible ABI mismatch between tree-sitter versions

## Files Involved

### Core Implementation
- `crates/scpf-core/src/semantic.rs` - Semantic scanner
- `crates/scpf-core/src/scanner.rs` - Main scanner orchestration
- `Cargo.toml` - Dependency versions

### Templates (Failing)
- `templates/advanced_audit.yaml`
- `templates/front-running-v2.yaml`
- `templates/reentrancy-state-change-v4.yaml`
- `templates/missing-access-control.yaml`
- `templates/tx-origin-authentication.yaml`
- `templates/unprotected-selfdestruct-v2.yaml`
- `templates/logic-bugs-gas-optimization.yaml`

### Templates (Working)
- `templates/semantic_working.yaml` - Has working examples

## Questions for Advanced Intelligence

1. **Grammar Compatibility**: What are the correct node names for tree-sitter-solidity v1.2?
   - Current queries use: `for_statement`, `function_body`, `call_expression`
   - Are these correct for v1.2?

2. **Query Syntax**: What is the correct tree-sitter query syntax for:
   - Detecting loops with external calls
   - Finding missing access control modifiers
   - Identifying tx.origin usage
   - Detecting unbounded loops

3. **Debugging**: How to get detailed error messages from `Query::new()` failure?
   - Currently only returns generic "Failed to compile" error
   - Need specific syntax error details

4. **Grammar Inspection**: How to inspect tree-sitter-solidity grammar to see:
   - Available node types
   - Node structure
   - Field names

5. **Alternative Approach**: Should we:
   - Use different tree-sitter-solidity version?
   - Fork and fix tree-sitter-solidity?
   - Use alternative AST parser?
   - Generate queries programmatically?

## Impact

### Current Limitations
- **Detection Coverage**: ~60% (regex only)
- **False Negatives**: HIGH - Missing context-aware detection
- **False Positives**: MEDIUM - Regex less precise than AST
- **Production Ready**: NO - Critical patterns not working

### Required for Production
- ✅ Regex patterns (basic detection)
- ❌ Semantic patterns (advanced detection) - **CRITICAL**
- ✅ Report generation
- ✅ Multi-chain support

## Attempted Solutions

1. ❌ Upgrade to tree-sitter-solidity v2.0 - Doesn't exist
2. ❌ Suppress warnings - Doesn't fix underlying issue
3. ✅ Unit tests pass - Simple queries work
4. ❌ Complex queries fail - Production patterns broken

## Next Steps Needed

1. **Identify correct node names** for tree-sitter-solidity v1.2
2. **Fix query syntax** in all failing templates
3. **Add detailed error logging** to see exact query compilation errors
4. **Create grammar documentation** for template authors
5. **Validate all semantic patterns** before production use

## Code Locations

```
SmartContractPatternFinder/
├── crates/scpf-core/src/
│   ├── semantic.rs          # Semantic scanner implementation
│   └── scanner.rs           # Main scanner (calls semantic)
├── templates/
│   ├── advanced_audit.yaml  # 20+ failing patterns
│   ├── semantic_working.yaml # Working examples
│   └── [other templates]
└── Cargo.toml               # Dependencies
```

## Environment

- **OS**: Linux
- **Rust**: Latest stable
- **Build**: Release mode
- **Tree-sitter**: v0.26
- **Tree-sitter-solidity**: v1.2.13

## Priority

**CRITICAL** - Semantic patterns are essential for:
- Context-aware vulnerability detection
- Low false positive rate
- Production-grade security scanning
- Competitive advantage over regex-only tools

Without semantic patterns, SCPF is incomplete and not production-ready.
