# SCPF v2 Implementation Plan - Semantic Context Layer

**Based on**: Claude Opus 4.5 Architecture Solution  
**Date**: 2024-01-21  
**Timeline**: 2 weeks (MVP in Week 1, Enhanced in Week 2)  
**Expected Results**: 60% precision (Week 1) → 85% precision (Week 2)

---

## Solution Summary

**Key Insight**: Add a **Semantic Context Layer** that understands protection mechanisms, not just syntax.

**Approach**: Multi-pass analysis pipeline
1. **Pass 1**: Symbol Collection (functions, modifiers, state variables)
2. **Pass 2**: Modifier Classification (reentrancy guards, access control)
3. **Pass 3**: CFG Construction + State Tracking
4. **Pass 4**: Contextual Pattern Matching (with protection filtering)
5. **Pass 5**: Finding Validation (confidence scoring)

**Result**: Patterns evaluated with context → Filter out protected code → Reduce false positives by 99%

---

## Week 1: MVP Implementation (Target: 60% Precision)

### Day 1-2: Core Data Structures

**Goal**: Foundation for semantic analysis

**Tasks**:
- [ ] Create `scpf-types/src/semantic.rs`
  - [ ] `ContractContext` struct
  - [ ] `FunctionContext` struct
  - [ ] `ModifierContext` struct
  - [ ] `ExternalCall` struct
  - [ ] `StateChange` struct
  - [ ] `ProtectionSet` struct
  - [ ] Enums: `ModifierType`, `ExternalCallKind`, `ReturnCheckStatus`

- [ ] Create `scpf-types/src/cfg.rs`
  - [ ] `ControlFlowGraph` struct
  - [ ] `CfgNode` and `CfgNodeKind` enums
  - [ ] `CfgEdge` enum
  - [ ] Basic CFG methods (`can_reach`, `nodes_between`)

- [ ] Update `scpf-types/src/finding.rs`
  - [ ] Add `confidence: f32` field
  - [ ] Add `evidence: Vec<Evidence>` field
  - [ ] Create `Evidence` struct
  - [ ] Create `Mitigation` struct

**Dependencies to add**:
```toml
petgraph = "0.6"
rustc-hash = "1.1"
smallvec = "1.11"
```

**Validation**: Compile successfully, unit tests for data structures

---

### Day 3-4: Symbol Collection & Modifier Classification

**Goal**: Extract all symbols and classify modifiers

**Tasks**:
- [ ] Create `scpf-core/src/analysis/mod.rs`
- [ ] Create `scpf-core/src/analysis/symbol_collector.rs`
  - [ ] `SymbolCollector` struct
  - [ ] `collect()` method - main entry point
  - [ ] `collect_functions()` - extract all functions
  - [ ] `collect_modifiers()` - extract all modifiers
  - [ ] `collect_state_variables()` - extract state vars
  - [ ] `get_function_modifiers()` - extract modifier list from function

- [ ] Create `scpf-core/src/analysis/modifier_classifier.rs`
  - [ ] `ModifierClassifier` struct
  - [ ] Known modifier name lists (REENTRANCY_GUARD_NAMES, ACCESS_CONTROL_NAMES)
  - [ ] `classify_modifier()` - classify by name and pattern
  - [ ] `has_reentrancy_pattern()` - detect guard pattern
  - [ ] `has_access_control_pattern()` - detect access control
  - [ ] `compute_confidence()` - confidence scoring

**Test Cases**:
```solidity
// Test 1: Known modifier names
modifier nonReentrant() { ... }
modifier onlyOwner() { ... }

// Test 2: Pattern detection
modifier lock() {
    require(!locked);
    locked = true;
    _;
    locked = false;
}
```

**Validation**: 
- Correctly identify `nonReentrant` as ReentrancyGuard (confidence: 0.95)
- Correctly identify `onlyOwner` as AccessControl (confidence: 0.85)
- Detect custom guard patterns (confidence: 0.75)

---

### Day 5-6: Simple Protection Detection

**Goal**: Filter findings based on detected protections

**Tasks**:
- [ ] Update `scpf-core/src/scanner.rs`
  - [ ] Add `contextual_enabled: bool` to config
  - [ ] Add `build_context()` method
  - [ ] Add `compute_protections()` method
  - [ ] Integrate symbol collection
  - [ ] Integrate modifier classification

- [ ] Create simple protection filter
  - [ ] Check if function has reentrancy guard modifier
  - [ ] Check if function has access control modifier
  - [ ] Filter out protected functions from findings

**Test Cases**:
```solidity
// Should NOT flag (has guard)
function withdraw() external nonReentrant {
    msg.sender.call{value: balance}("");
}

// SHOULD flag (no guard)
function withdraw() external {
    msg.sender.call{value: balance}("");
}
```

**Validation**:
- Scan Uniswap V2: Expect ~2,000 findings (down from 6,378)
- Scan USDC: Expect ~400 findings (down from 1,147)
- **Target**: 60-70% reduction in false positives

---

### Day 7: Integration & Testing

**Goal**: End-to-end MVP working

**Tasks**:
- [ ] Update template format to support `contextual` section
- [ ] Create example contextual templates
  - [ ] `reentrancy-contextual.yaml`
  - [ ] `access-control-contextual.yaml`
  - [ ] `unchecked-return-contextual.yaml`

- [ ] Integration testing
  - [ ] Test on 6 safe contracts
  - [ ] Measure precision improvement
  - [ ] Document results

- [ ] Update documentation
  - [ ] Template format guide
  - [ ] Migration guide for existing templates

**Expected Results**:
| Contract | Before | After MVP | Reduction |
|----------|--------|-----------|-----------|
| USDC | 1,147 | ~400 | 65% |
| Uniswap V2 | 6,378 | ~2,000 | 69% |
| **Precision** | 0% | ~60% | +60 pts |

---

## Week 2: Enhanced Implementation (Target: 85% Precision)

### Day 8-9: CFG Construction

**Goal**: Build control flow graphs for order analysis

**Tasks**:
- [ ] Create `scpf-core/src/analysis/cfg_builder.rs`
  - [ ] `CfgBuilder` struct
  - [ ] `build_function_cfg()` - main entry point
  - [ ] `build_block()` - process statement blocks
  - [ ] `build_if_statement()` - handle conditionals
  - [ ] `build_loop()` - handle loops
  - [ ] `analyze_call()` - detect external calls
  - [ ] `analyze_assignment()` - detect state changes
  - [ ] `check_return_value_handling()` - verify return checks

- [ ] Integrate CFG into scanner
  - [ ] Build CFG for each function
  - [ ] Extract external calls from CFG
  - [ ] Extract state changes from CFG

**Test Cases**:
```solidity
function test() external {
    // Node 1: Entry
    require(condition);  // Node 2: Require
    balance = 0;         // Node 3: State change
    call();              // Node 4: External call
    // Node 5: Exit
}
```

**Validation**:
- CFG correctly represents control flow
- Can detect node ordering (state before/after call)
- Performance: <100ms per function

---

### Day 10-11: Order Analysis & CEI Detection

**Goal**: Detect Checks-Effects-Interactions pattern violations

**Tasks**:
- [ ] Implement CFG traversal algorithms
  - [ ] `can_reach(from, to)` - reachability check
  - [ ] `nodes_between(from, to)` - path finding
  - [ ] `call_before_state_change()` - order check

- [ ] Create `scpf-core/src/analysis/cei_detector.rs`
  - [ ] `uses_checks_effects_interactions()` - CEI pattern detection
  - [ ] `has_state_change_after_call()` - violation detection
  - [ ] `get_call_state_pairs()` - find problematic pairs

- [ ] Update protection detection
  - [ ] Add CEI pattern to `ProtectionSet`
  - [ ] Filter findings if CEI pattern used

**Test Cases**:
```solidity
// Good: CEI pattern
function withdraw() external {
    balance = 0;  // Effect
    call();       // Interaction
}

// Bad: State after call
function withdraw() external {
    call();       // Interaction
    balance = 0;  // Effect (AFTER!)
}
```

**Validation**:
- Correctly identify CEI pattern usage
- Correctly identify violations
- Filter out CEI-protected functions

---

### Day 12-13: Contextual Matcher

**Goal**: Full contextual pattern matching with evidence

**Tasks**:
- [ ] Create `scpf-core/src/analysis/contextual_matcher.rs`
  - [ ] `ContextualMatcher` struct
  - [ ] `match_pattern()` - main matching logic
  - [ ] `check_condition()` - verify match conditions
  - [ ] `check_requirement()` - verify requirements
  - [ ] `check_exclusion()` - verify protections
  - [ ] `generate_finding()` - create finding with evidence

- [ ] Implement condition checks
  - [ ] `HasExternalCall`
  - [ ] `HasStateChange`
  - [ ] `UncheckedReturnValue`
  - [ ] `SendsValue`
  - [ ] `IsPublicOrExternal`

- [ ] Implement exclusion checks
  - [ ] `HasModifier`
  - [ ] `UsesChecksEffectsInteractions`
  - [ ] `HasAccessControl`
  - [ ] `IsView`

- [ ] Add confidence scoring
  - [ ] Base confidence from template
  - [ ] Evidence modifiers (+/- confidence)
  - [ ] Final confidence calculation

**Template Example**:
```yaml
contextual:
  matches:
    - has_external_call: { kind: [low_level_call] }
    - has_state_change: { after_call: true }
  excludes:
    - has_modifier: { type: reentrancy_guard }
    - uses_checks_effects_interactions: true
  evidence:
    - sends_value: +0.2
    - unchecked_return: +0.1
```

**Validation**:
- Correctly match contextual patterns
- Correctly filter by exclusions
- Generate evidence chains

---

### Day 14: Polish & Validation

**Goal**: Fine-tune and validate against targets

**Tasks**:
- [ ] Fine-tune confidence thresholds
  - [ ] Test different thresholds (0.70, 0.75, 0.80)
  - [ ] Optimize for precision/recall balance

- [ ] Add more modifier patterns
  - [ ] Pausable patterns
  - [ ] Input validation patterns
  - [ ] Custom guard patterns

- [ ] Comprehensive testing
  - [ ] Test on all 6 safe contracts
  - [ ] Test on vulnerable contracts (measure recall)
  - [ ] Calculate final metrics

- [ ] Documentation
  - [ ] Architecture documentation
  - [ ] Template migration guide
  - [ ] Performance benchmarks

**Expected Results**:
| Metric | Before | After Enhanced | Target | Status |
|--------|--------|----------------|--------|--------|
| Precision | 0% | ~85% | ≥85% | ✅ |
| Findings/contract | 5,000 | ~20 | <10 | 🟡 |
| False Positives | 30,321 | ~60 | 0 | 🟡 |

---

## Implementation Priorities

### Must Have (Week 1 MVP)
1. ✅ Symbol collection
2. ✅ Modifier classification by name
3. ✅ Simple protection filtering
4. ✅ Basic contextual templates

### Should Have (Week 2 Enhanced)
1. ✅ CFG construction
2. ✅ Order analysis (CEI detection)
3. ✅ Full contextual matcher
4. ✅ Confidence scoring

### Nice to Have (Future)
1. ⏳ Inherited modifier resolution
2. ⏳ Cross-function analysis
3. ⏳ Data flow analysis
4. ⏳ Symbolic execution (lightweight)

---

## Risk Mitigation

### Risk 1: CFG Construction Complexity
**Mitigation**: Start with simple CFG (linear blocks), add conditionals/loops incrementally

### Risk 2: Performance Degradation
**Mitigation**: Profile early, optimize hot paths, use efficient data structures (petgraph, rustc-hash)

### Risk 3: False Negative Increase
**Mitigation**: Conservative filtering - only exclude if high confidence in protection

### Risk 4: Template Migration
**Mitigation**: Support both old and new formats, provide migration tool

---

## Success Criteria

### Week 1 MVP Success
- [ ] Precision ≥60% (from 0%)
- [ ] Findings reduced by 60-70%
- [ ] Performance <5 seconds per contract
- [ ] No regressions in infrastructure

### Week 2 Enhanced Success
- [ ] Precision ≥85%
- [ ] Recall ≥75% (measure on vulnerable contracts)
- [ ] F1 Score ≥0.80
- [ ] Findings <50 per safe contract

### Production Ready
- [ ] All tests passing
- [ ] Documentation complete
- [ ] Performance benchmarks published
- [ ] Migration guide available

---

## Next Steps

1. **Immediate**: Review and approve this plan
2. **Day 1**: Start implementing core data structures
3. **Day 7**: MVP demo and metrics
4. **Day 14**: Final validation and release

---

**Status**: Ready to implement  
**Confidence**: High (clear architecture, proven approach)  
**Timeline**: 2 weeks (realistic with single developer)
