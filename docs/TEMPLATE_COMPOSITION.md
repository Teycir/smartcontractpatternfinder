# Template Composition

## Overview

Template composition allows combining multiple detection templates to create comprehensive vulnerability detection with reduced false positives.

## Concept

```
┌─────────────────────────────────────────────────────────────┐
│                  Template Composition                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │  Template A  │  │  Template B  │  │  Template C  │       │
│  │  (Pattern)   │  │  (Dataflow)  │  │  (Mutation)  │       │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘       │
│         │                 │                 │                │
│         └─────────────────┼─────────────────┘                │
│                           ▼                                  │
│                  ┌─────────────────┐                         │
│                  │ Composition     │                         │
│                  │ Rules Engine    │                         │
│                  └────────┬────────┘                         │
│                           │                                  │
│                           ▼                                  │
│                  ┌─────────────────┐                         │
│                  │ Combined Result │                         │
│                  │ (High Confidence)│                        │
│                  └─────────────────┘                         │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Composition Rules

### 1. AllOf - All templates must match
```yaml
rules:
  - type: all_of
    templates:
      - external-call
      - state-mutation
      - no-guard
    severity: critical
```

**Use Case**: High-confidence detection requiring multiple indicators.

### 2. AnyOf - Any template must match
```yaml
rules:
  - type: any_of
    templates:
      - pattern-a
      - pattern-b
      - pattern-c
    severity: medium
```

**Use Case**: Broad detection with multiple possible indicators.

### 3. Sequential - Patterns in order
```yaml
rules:
  - type: sequential
    templates:
      - external-call
      - state-change
    max_distance: 50
    severity: high
```

**Use Case**: Temporal relationships (e.g., CEI violations).

### 4. AndNot - Exclude false positives
```yaml
rules:
  - type: and_not
    required: critical-function
    excluded: access-modifier
    severity: high
```

**Use Case**: Filter out protected code.

### 5. ExactlyN - Specific count
```yaml
rules:
  - type: exactly_n
    count: 2
    templates:
      - indicator-1
      - indicator-2
      - indicator-3
    severity: medium
```

**Use Case**: Require specific number of indicators.

## Example: Comprehensive Reentrancy

```yaml
id: reentrancy-comprehensive
name: Comprehensive Reentrancy Detection
severity: critical

composition:
  base_templates:
    - reentrancy-state-change-v4
    - unchecked-return-value-v4
    - strict-balance-equality-v4
  
  rules:
    # High confidence: All indicators
    - type: all_of
      templates:
        - external-call-with-value
        - balance-mapping-write
        - no-reentrancy-guard
      severity: critical
    
    # Medium confidence: Sequential pattern
    - type: sequential
      templates:
        - external-call
        - state-mutation
      max_distance: 50
      severity: high
    
    # Filter false positives
    - type: and_not
      required: external-call
      excluded: nonreentrant-modifier
      severity: high

scoring:
  pattern_scores:
    external-call-with-value: 40
    balance-mapping-write: 30
    no-reentrancy-guard: 20
  thresholds:
    critical: 80
    high: 50
    medium: 30
```

## Benefits

### 1. Reduced False Positives
- Requires multiple indicators
- Filters protected code
- Context-aware detection

### 2. Higher Confidence
- Combines multiple detection methods
- Scoring system for severity
- Explicit false positive filters

### 3. Comprehensive Coverage
- Pattern matching
- Data flow analysis
- State mutation tracking
- All in one template

### 4. Maintainability
- Reuse existing templates
- Centralized configuration
- Easy to update rules

## Usage

### Create Composed Template

```rust
use scpf_core::composition::{TemplateComposer, ComposedTemplate, CompositionRule};

let mut composer = TemplateComposer::new();

// Add base templates
composer.add_template(reentrancy_template);
composer.add_template(balance_template);

// Create composition
let composition = ComposedTemplate {
    id: "reentrancy-comprehensive".to_string(),
    name: "Comprehensive Reentrancy".to_string(),
    severity: Severity::Critical,
    base_templates: vec![
        "reentrancy-state-change-v4".to_string(),
        "strict-balance-equality-v4".to_string(),
    ],
    composition_rules: vec![
        CompositionRule::Sequential {
            templates: vec!["call".to_string(), "mutation".to_string()],
            max_distance: 50,
        },
    ],
};

composer.add_composition(composition);
```

### Evaluate Composition

```rust
// Collect matches from base templates
let mut matches = HashMap::new();
matches.insert("external-call".to_string(), vec![10, 20]);
matches.insert("state-mutation".to_string(), vec![15, 25]);

// Evaluate composition rules
let result = composer.evaluate_composition(&composition, &matches);
```

## Predefined Compositions

### 1. Reentrancy Comprehensive
```rust
let comp = create_reentrancy_composition();
```
Combines: Pattern matching + State tracking + Balance checks

### 2. Access Control Comprehensive
```rust
let comp = create_access_control_composition();
```
Combines: Selfdestruct + tx.origin + Missing modifiers

### 3. DeFi Comprehensive
```rust
let comp = create_defi_composition();
```
Combines: Front-running + Price manipulation + Flash loans

## Scoring System

### Pattern Scores
```yaml
pattern_scores:
  external-call-with-value: 40    # High risk
  balance-mapping-write: 30       # Medium-high risk
  no-reentrancy-guard: 20         # Medium risk
  unchecked-return: 10            # Low risk
```

### Severity Thresholds
```yaml
thresholds:
  critical: 80   # Score >= 80
  high: 50       # Score >= 50
  medium: 30     # Score >= 30
  low: 10        # Score >= 10
```

### Example Calculation
```
Matched patterns:
- external-call-with-value: 40
- balance-mapping-write: 30
- no-reentrancy-guard: 20
Total: 90 → CRITICAL
```

## False Positive Filters

### Exclude Patterns
```yaml
filters:
  exclude_if_present:
    - nonreentrant-modifier
    - reentrancy-guard-pattern
    - mutex-lock-pattern
```

### Exclude Contexts
```yaml
filters:
  exclude_contexts:
    - in_view_function
    - in_pure_function
    - after_require_check
```

## Output Format

```
════════════════════════════════════════════════════════════
!  contract.sol (150ms)

   [CRITICAL] Line 23: Comprehensive Reentrancy Detected
   Composition: reentrancy-comprehensive
   Matched Templates:
     ✓ external-call-with-value (score: 40)
     ✓ balance-mapping-write (score: 30)
     ✓ no-reentrancy-guard (score: 20)
   Total Score: 90/100
   Confidence: 95%
   
   Rule Evaluation:
     ✓ Sequential: call(10) → mutation(15) [distance: 5]
     ✓ AllOf: All 3 indicators present
     ✓ AndNot: No reentrancy guard detected
   
   Recommendation: Implement CEI pattern or add nonReentrant modifier
════════════════════════════════════════════════════════════
```

## Performance

- **Overhead**: ~2-5ms per composition
- **Caching**: Reuses base template results
- **Scalable**: Handles 10+ base templates
- **Efficient**: Rule evaluation is O(n)

## Best Practices

1. **Start Simple**: Begin with 2-3 base templates
2. **Use Sequential**: For temporal relationships
3. **Filter Aggressively**: Reduce false positives
4. **Score Appropriately**: Weight critical indicators higher
5. **Test Thoroughly**: Validate on known vulnerabilities

## Future Enhancements

- [ ] Machine learning for scoring
- [ ] Dynamic rule generation
- [ ] Template dependency graphs
- [ ] Automatic composition suggestions
- [ ] Performance optimization

## Files

- `crates/scpf-core/src/composition.rs` - Core engine
- `templates/composed/` - Composed templates
- `docs/TEMPLATE_COMPOSITION.md` - This document
