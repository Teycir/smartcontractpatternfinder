# AST-Based Pattern Matching - Design Document

## 🎯 Objective
Implement a second-pass AST-based analyzer to eliminate false positives that regex cannot detect.

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     SCPF Scanning Pipeline                   │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  1. Regex Pass (Fast)          2. AST Pass (Accurate)       │
│     ├─ Pattern matching            ├─ Parse Solidity AST    │
│     ├─ Quick filtering             ├─ Semantic analysis     │
│     └─ Candidate findings          ├─ Context validation    │
│            │                        └─ Filter false positives│
│            │                                 │                │
│            └────────────────────────────────┘                │
│                          │                                    │
│                    Final Results                             │
└─────────────────────────────────────────────────────────────┘
```

## 📦 Implementation Plan

### Phase 1: Add Solidity Parser Dependency

**Crate**: `solang-parser` (official Solidity parser in Rust)

```toml
# Cargo.toml
[dependencies]
solang-parser = "0.3"
```

### Phase 2: Create AST Analyzer Module

```
crates/scpf-core/src/
├── ast/
│   ├── mod.rs              # Public API
│   ├── parser.rs           # Parse Solidity to AST
│   ├── analyzer.rs         # Semantic analysis
│   └── validators/
│       ├── mod.rs
│       ├── initialize.rs   # Initialize function validator
│       ├── access_control.rs # Access control validator
│       └── delegatecall.rs # Delegatecall validator
```

### Phase 3: Implement AST Validators

#### Example: Initialize Function Validator

```rust
// crates/scpf-core/src/ast/validators/initialize.rs

use solang_parser::pt::*;

pub struct InitializeValidator;

impl InitializeValidator {
    pub fn validate(func: &FunctionDefinition) -> ValidationResult {
        // 1. Check function name
        if !func.name.as_ref().map_or(false, |n| n.name.contains("initialize")) {
            return ValidationResult::NotApplicable;
        }

        // 2. Check for initializer modifier
        if self.has_initializer_modifier(func) {
            return ValidationResult::Protected("Has initializer modifier");
        }

        // 3. Check for custom initialization guards
        if self.has_initialization_guard(func) {
            return ValidationResult::Protected("Has custom guard");
        }

        // 4. Check if used in proxy pattern
        if !self.is_proxy_pattern(func) {
            return ValidationResult::NotApplicable;
        }

        ValidationResult::Vulnerable
    }

    fn has_initializer_modifier(&self, func: &FunctionDefinition) -> bool {
        func.attributes.iter().any(|attr| {
            matches!(attr, FunctionAttribute::BaseOrModifier(_, base) 
                if base.name.identifiers.iter()
                    .any(|id| id.name == "initializer" || id.name == "reinitializer"))
        })
    }

    fn has_initialization_guard(&self, func: &FunctionDefinition) -> bool {
        // Check for: require(!initialized, "Already initialized");
        // Check for: if (initialized) revert AlreadyInitialized();
        // Check for: initialized = true;
        
        if let Some(Statement::Block { statements, .. }) = &func.body {
            for stmt in statements {
                if self.is_initialization_check(stmt) {
                    return true;
                }
            }
        }
        false
    }

    fn is_initialization_check(&self, stmt: &Statement) -> bool {
        match stmt {
            Statement::If(_, cond, _, _) => {
                // Check if condition references "initialized" variable
                self.references_initialized_flag(cond)
            }
            Statement::Expression(_, expr) => {
                // Check for require(!initialized, ...)
                self.is_require_initialized(expr)
            }
            _ => false
        }
    }

    fn is_proxy_pattern(&self, func: &FunctionDefinition) -> bool {
        // Check if contract inherits from proxy-related contracts
        // Check for storage slot patterns
        // Check for delegatecall usage
        true // Simplified for now
    }
}

pub enum ValidationResult {
    Vulnerable,
    Protected(&'static str),
    NotApplicable,
}
```

#### Example: Access Control Validator

```rust
// crates/scpf-core/src/ast/validators/access_control.rs

pub struct AccessControlValidator;

impl AccessControlValidator {
    pub fn validate_withdraw(&self, func: &FunctionDefinition) -> ValidationResult {
        // 1. Check for access control modifiers
        if self.has_access_modifier(func) {
            return ValidationResult::Protected("Has access control modifier");
        }

        // 2. Check for inline access control
        if self.has_inline_access_check(func) {
            return ValidationResult::Protected("Has inline access check");
        }

        // 3. Check if only withdraws caller's funds
        if self.only_withdraws_own_funds(func) {
            return ValidationResult::Protected("Only withdraws caller's funds");
        }

        ValidationResult::Vulnerable
    }

    fn has_access_modifier(&self, func: &FunctionDefinition) -> bool {
        func.attributes.iter().any(|attr| {
            matches!(attr, FunctionAttribute::BaseOrModifier(_, base) 
                if ["onlyOwner", "onlyAdmin", "onlyRole", "onlyGovernance"]
                    .iter()
                    .any(|&m| base.name.identifiers.iter().any(|id| id.name == m)))
        })
    }

    fn has_inline_access_check(&self, func: &FunctionDefinition) -> bool {
        if let Some(Statement::Block { statements, .. }) = &func.body {
            for stmt in statements.iter().take(3) { // Check first 3 statements
                if self.is_access_check(stmt) {
                    return true;
                }
            }
        }
        false
    }

    fn is_access_check(&self, stmt: &Statement) -> bool {
        match stmt {
            Statement::If(_, cond, then_branch, _) => {
                // Check for: if (msg.sender != owner) revert;
                self.is_sender_check(cond) && self.is_revert(then_branch)
            }
            Statement::Expression(_, expr) => {
                // Check for: require(msg.sender == owner);
                self.is_require_sender_check(expr)
            }
            _ => false
        }
    }

    fn only_withdraws_own_funds(&self, func: &FunctionDefinition) -> bool {
        // Analyze function body to check if:
        // - Transfer destination is msg.sender
        // - Amount is from msg.sender's balance
        // This requires data flow analysis
        false // Simplified - would need full implementation
    }
}
```

### Phase 4: Integrate with Scanner

```rust
// crates/scpf-core/src/scanner.rs

use crate::ast::{AstAnalyzer, ValidationResult};

pub struct Scanner {
    ast_analyzer: Option<AstAnalyzer>,
}

impl Scanner {
    pub fn scan_with_ast(&self, source: &str, template: &Template) -> Vec<Match> {
        // Pass 1: Regex matching (fast)
        let regex_matches = self.scan_regex(source, template);

        // Pass 2: AST validation (accurate)
        if let Some(analyzer) = &self.ast_analyzer {
            regex_matches
                .into_iter()
                .filter(|m| analyzer.validate(source, m).is_vulnerable())
                .collect()
        } else {
            regex_matches
        }
    }
}
```

### Phase 5: Add Configuration

```toml
# .scpf.toml

[scanning]
enable_ast_validation = true  # Enable second-pass AST analysis
ast_timeout_ms = 5000         # Timeout per file
ast_max_file_size = 500000    # Skip AST for files > 500KB

[ast]
# Validators to enable
validators = [
    "initialize",
    "access_control", 
    "delegatecall",
]
```

## 📊 Expected Impact

### False Positive Reduction

| Pattern | Current FP Rate | Expected FP Rate | Improvement |
|---------|----------------|------------------|-------------|
| unprotected-initialize | 100% | 5-10% | 90-95% |
| public-withdraw-no-auth | 100% | 10-20% | 80-90% |
| external-mint-no-modifier | ~80% | 10-15% | 65-70% |
| delegatecall-no-whitelist | ~50% | 15-25% | 25-35% |

### Performance Impact

- **Regex-only**: 0.51s per contract
- **Regex + AST**: 0.8-1.2s per contract (estimated)
- **Trade-off**: 50-100% slower, but 80-90% fewer false positives

## 🚀 Implementation Timeline

### Week 1: Foundation
- [ ] Add `solang-parser` dependency
- [ ] Create AST module structure
- [ ] Implement basic parser wrapper

### Week 2: Validators
- [ ] Implement InitializeValidator
- [ ] Implement AccessControlValidator
- [ ] Add unit tests

### Week 3: Integration
- [ ] Integrate with Scanner
- [ ] Add configuration support
- [ ] Performance optimization

### Week 4: Testing & Refinement
- [ ] Run on validation dataset
- [ ] Measure FP reduction
- [ ] Fine-tune validators
- [ ] Documentation

## 🧪 Testing Strategy

### Unit Tests
```rust
#[test]
fn test_initialize_with_modifier() {
    let code = r#"
        function initialize() public initializer {
            owner = msg.sender;
        }
    "#;
    
    let result = InitializeValidator::validate(parse(code));
    assert_eq!(result, ValidationResult::Protected);
}

#[test]
fn test_initialize_without_modifier() {
    let code = r#"
        function initialize() public {
            owner = msg.sender;
        }
    "#;
    
    let result = InitializeValidator::validate(parse(code));
    assert_eq!(result, ValidationResult::Vulnerable);
}
```

### Integration Tests
- Run on 31 validated findings
- Verify all 31 false positives are filtered
- Ensure no true positives are lost

## 📝 Alternative: Hybrid Approach

Instead of full AST parsing, use targeted AST analysis:

```rust
pub fn quick_ast_check(source: &str, match_pos: usize) -> bool {
    // Only parse the matched function, not entire file
    let func_source = extract_function(source, match_pos);
    let ast = parse_function(func_source)?;
    
    // Quick checks only
    has_access_modifier(&ast) || has_inline_check(&ast)
}
```

**Benefits**:
- Faster (only parse matched functions)
- Lower memory usage
- Simpler implementation

## 🎯 Success Criteria

1. **FP Reduction**: Reduce false positives by >80%
2. **Performance**: Keep scan time under 2s per contract
3. **Accuracy**: Maintain 100% true positive detection
4. **Maintainability**: Clean, testable code

---

**Status**: Design Complete - Ready for Implementation  
**Priority**: HIGH - Critical for production use  
**Estimated Effort**: 2-3 weeks
