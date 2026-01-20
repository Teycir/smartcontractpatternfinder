# Refactoring Assessment: Scanner & RiskScorer

## ✅ APPROVED - Excellent Refactoring Plan

This refactoring makes perfect sense and follows all Amazon Q rules.

---

## Analysis

### 1. Scanner::new Refactoring ✅

**Current Issues:**
- Method is 110+ lines long (exceeds 50-line rule)
- Pattern compilation logic mixed with initialization
- Difficult to test individual compilation steps

**Proposed Solution:**
```rust
// Extract to helper functions:
fn compile_pattern(pattern: &Pattern, template_id: &str, index: u32) -> Result<CompiledPattern>
fn compile_template(template: Template, pattern_index: &mut u32) -> Result<CompiledTemplate>
```

**Benefits:**
- ✅ Follows Single Responsibility Principle
- ✅ Each function < 50 lines
- ✅ Easier to test
- ✅ Better error handling isolation

---

### 2. Scanner::scan Context Extraction ✅

**Current Issues:**
- Context generation logic duplicated (lines 220-240)
- Complex inline calculations for line boundaries
- Hard to maintain and test

**Proposed Solution:**
```rust
fn get_match_context(
    source: &str,
    newlines: &[usize],
    match_start: usize,
    match_end: usize,
    line_number: usize
) -> String
```

**Benefits:**
- ✅ DRY principle (eliminates duplication)
- ✅ Reusable across regex and semantic matches
- ✅ Testable in isolation
- ✅ Clearer intent

---

### 3. RiskScorer Constants ✅

**Current Issues:**
```rust
// Magic strings prone to typos:
self.has_pattern(matches, "external-call")      // Line 157
self.has_pattern(matches, "state-mutation")     // Line 157
self.has_pattern(matches, "critical-function")  // Line 162
self.has_pattern(matches, "access-modifier")    // Line 163
self.has_pattern_in_map(patterns, "reentrancy") // Line 207
self.has_pattern_in_map(patterns, "access-control") // Line 211
```

**Proposed Solution:**
```rust
// At module level:
mod pattern_ids {
    pub const EXTERNAL_CALL: &str = "external-call";
    pub const STATE_MUTATION: &str = "state-mutation";
    pub const CRITICAL_FUNCTION: &str = "critical-function";
    pub const ACCESS_MODIFIER: &str = "access-modifier";
    pub const REENTRANCY: &str = "reentrancy";
    pub const ACCESS_CONTROL: &str = "access-control";
}

// Usage:
self.has_pattern(matches, pattern_ids::EXTERNAL_CALL)
```

**Benefits:**
- ✅ Compile-time typo detection
- ✅ IDE autocomplete support
- ✅ Single source of truth
- ✅ Easier refactoring (rename all references)
- ✅ Self-documenting code

---

## Compliance Check

### Amazon Q Rules Compliance

| Rule | Status | Notes |
|------|--------|-------|
| **Refactoring Rules** | ✅ | Function > 50 lines → Split |
| **Modular Architecture** | ✅ | Pure functions, single responsibility |
| **Bug Fixing** | ✅ | No features removed, only restructured |
| **Error Handling** | ✅ | Maintains explicit error handling |

---

## Implementation Priority

### Phase 1: High Impact (Do First)
1. **RiskScorer Constants** - Quick win, prevents bugs
2. **Scanner Context Extraction** - Eliminates duplication

### Phase 2: Structural (Do Second)
3. **Scanner::new Refactoring** - Larger change, needs careful testing

---

## Recommended Implementation

### 1. RiskScorer Constants (Minimal Code)

```rust
// Add at top of risk_scoring.rs
mod pattern_ids {
    pub const EXTERNAL_CALL: &str = "external-call";
    pub const STATE_MUTATION: &str = "state-mutation";
    pub const CRITICAL_FUNCTION: &str = "critical-function";
    pub const ACCESS_MODIFIER: &str = "access-modifier";
    pub const REENTRANCY: &str = "reentrancy";
    pub const ACCESS_CONTROL: &str = "access-control";
}

// Replace all string literals with constants
```

### 2. Scanner Context Helper (Minimal Code)

```rust
fn get_match_context(
    source: &str,
    newlines: &[usize],
    match_start: usize,
    match_end: usize,
    line_number: usize,
) -> String {
    const MAX_CONTEXT_CHARS: usize = 200;
    const CONTEXT_PADDING: usize = 50;

    let match_len = match_end - match_start;
    
    if match_len > MAX_CONTEXT_CHARS {
        let start = match_start.saturating_sub(CONTEXT_PADDING);
        let end = (match_end + CONTEXT_PADDING).min(source.len());
        source[start..end].to_string()
    } else {
        let context_start = if line_number > 1 {
            newlines[line_number - 2] + 1
        } else {
            0
        };
        let context_end = newlines
            .get(line_number - 1)
            .copied()
            .unwrap_or(source.len());
        source[context_start..context_end].to_string()
    }
}
```

### 3. Scanner Compilation Helpers (Minimal Code)

```rust
fn compile_pattern(
    pattern: &Pattern,
    template_id: &str,
    index: u32,
) -> Result<CompiledPattern> {
    if pattern.kind == PatternKind::Semantic {
        return Ok(CompiledPattern {
            regex: RegexBuilder::new(".*").build().unwrap(),
            pattern: pattern.clone(),
            index,
        });
    }

    RegexValidator::validate_pattern(&pattern.pattern)?;

    let regex = RegexBuilder::new(&pattern.pattern)
        .multi_line(true)
        .dot_matches_new_line(true)
        .build()
        .map_err(|e| anyhow::anyhow!("Invalid regex in template '{}', pattern '{}': {}", template_id, pattern.id, e))?;

    Ok(CompiledPattern {
        regex,
        pattern: pattern.clone(),
        index,
    })
}

fn compile_template(
    template: Template,
    pattern_index: &mut u32,
) -> Result<Option<CompiledTemplate>> {
    let mut compiled_patterns = Vec::with_capacity(template.patterns.len());

    for pattern in &template.patterns {
        match compile_pattern(pattern, &template.id, *pattern_index) {
            Ok(compiled) => {
                compiled_patterns.push(compiled);
                *pattern_index += 1;
            }
            Err(e) => {
                warn!("Failed to compile pattern '{}' in template '{}': {}", pattern.id, template.id, e);
                return Err(e);
            }
        }
    }

    if compiled_patterns.is_empty() {
        return Ok(None);
    }

    Ok(Some(CompiledTemplate {
        template,
        patterns: compiled_patterns,
    }))
}
```

---

## Testing Strategy

### Unit Tests to Add

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_pattern_regex() {
        let pattern = Pattern {
            id: "test".to_string(),
            pattern: r"\d+".to_string(),
            message: "test".to_string(),
            kind: PatternKind::Regex,
        };
        let result = compile_pattern(&pattern, "test-template", 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_pattern_semantic() {
        let pattern = Pattern {
            id: "test".to_string(),
            pattern: "(identifier) @id".to_string(),
            message: "test".to_string(),
            kind: PatternKind::Semantic,
        };
        let result = compile_pattern(&pattern, "test-template", 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_match_context_short() {
        let source = "line1\nline2\nline3\n";
        let newlines = vec![5, 11, 17];
        let context = get_match_context(source, &newlines, 6, 11, 2);
        assert_eq!(context, "line2");
    }

    #[test]
    fn test_pattern_constants() {
        use pattern_ids::*;
        assert_eq!(EXTERNAL_CALL, "external-call");
        assert_eq!(STATE_MUTATION, "state-mutation");
    }
}
```

---

## Verification Checklist

- [ ] All tests pass: `cargo test`
- [ ] No compilation errors: `cargo check --all`
- [ ] No clippy warnings: `cargo clippy`
- [ ] Functions < 50 lines
- [ ] No duplicate code
- [ ] Constants used instead of magic strings
- [ ] Error handling preserved
- [ ] Performance unchanged

---

## Conclusion

**APPROVED** ✅

This refactoring:
- Follows all Amazon Q rules
- Improves code quality significantly
- Reduces technical debt
- Prevents future bugs (typos in pattern IDs)
- Makes code more testable and maintainable

**Recommendation:** Implement in the order suggested (constants → context → compilation) for incremental, safe refactoring.
