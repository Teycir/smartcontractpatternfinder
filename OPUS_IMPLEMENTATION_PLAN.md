# Implementation Plan: Template Quality Fixes

**Based on**: Claude Opus 4.5 Response  
**Date**: 2024-01-21  
**Priority**: CRITICAL - Fix templates before any other work

---

## 🎯 Immediate Actions (This Week)

### 1. Disable Broken Templates (Day 1)

Create `templates/.disabled/` directory and move:
- `cryptography-signatures.yaml` (weak-randomness-prevrandao broken)
- `tx_origin_auth.yaml` (matches everything)
- Any template with >80% FP rate

**Impact**: Immediate precision improvement from 0% to ~30%

### 2. Deploy Fixed Templates (Days 2-3)

Opus provided fixed versions for top 3 worst:

**A. weak-randomness-prevrandao (FIXED)**
- Uses hybrid approach: regex + validation + exclusions
- Excludes comments, strings, events, logging
- Only matches when used for randomness/logic
- Expected: 0 matches on safe contracts

**B. tx-origin-auth (FIXED)**
- Matches only authentication patterns (require, if, assert)
- Excludes comments, strings, events
- Reduces severity for `tx.origin == msg.sender` (contract detection)
- Expected: <5 matches on safe contracts

**C. signature-return-unchecked (FIXED)**
- Matches ecrecover/ECDSA.recover calls
- Validates return value is unchecked
- Tracks variable usage for 10 lines
- Excludes require/if checks
- Expected: <10 matches on safe contracts

### 3. Add Global Comment Exclusion (Day 1)

Add to all templates:
```yaml
exclusions:
  - type: "line_is_comment"
    pattern: '^\s*(//|/\*|\*)'
  - type: "in_string_literal"
    pattern: '["'].*{pattern}.*["']'
```

---

## 📊 Short-term Actions (Weeks 2-4)

### 4. Build Test Corpus (Week 2)

**Structure**:
```
test-contracts/
├── vulnerable/
│   ├── reentrancy/
│   ├── tx-origin/
│   ├── weak-randomness/
│   └── signature/
└── safe/
    ├── production/ (USDC, DAI, Uniswap)
    └── protected/ (with guards)
```

**Metadata format**:
```json
{
  "contract": "basic-reentrancy.sol",
  "expected_findings": [
    {
      "template_id": "reentrancy-state-change",
      "line": 15,
      "severity": "high"
    }
  ]
}
```

### 5. Implement Template Validator (Week 3)

Python script to:
- Run each template against test corpus
- Calculate precision, recall, F1
- Assign tier (1=enabled, 2=warning, 3=disabled)
- Generate quality report

**Tiers**:
- Tier 1: Precision ≥85%, Recall ≥75% → Enabled
- Tier 2: Precision ≥70%, Recall ≥60% → Warning
- Tier 3: Below thresholds → Disabled

### 6. Pattern-Based Protection Detection (Week 4)

Implement detectors for:

**A. Custom Reentrancy Guards**
```python
def detect_custom_lock(code):
    has_lock_var = re.search(r'bool.*_?locked', code)
    has_require = re.search(r'require\(!_?locked\)', code)
    has_set = re.search(r'_?locked = true', code)
    return all([has_lock_var, has_require, has_set])
```

**B. Custom Access Control**
```python
def detect_inline_auth(func_body):
    # Check first statement is msg.sender check
    first_stmt = func_body.split(';')[0]
    return bool(re.search(r'require\(msg\.sender\s*==', first_stmt))
```

**C. CEI Pattern**
```python
def is_cei_compliant(func_body):
    call_pos = find_external_call_position(func_body)
    state_changes = find_state_changes(func_body)
    return all(pos < call_pos for pos in state_changes)
```

---

## 🔄 Long-term Actions (Months 2-3)

### 7. Hybrid Detection Pipeline (Month 2)

**Architecture**:
1. **Phase 1 (Semantic)**: Extract structure with tree-sitter
2. **Phase 2 (Regex)**: Validate content with patterns
3. **Phase 3 (Context)**: Check protections

**Example** (from Opus):
```python
class HybridReentrancyDetector:
    def detect(self, source):
        # Phase 1: Find external calls (semantic)
        candidates = self._semantic_extraction(source)
        
        # Phase 2: Validate patterns (regex)
        validated = self._regex_validation(candidates)
        
        # Phase 3: Check protections (context)
        findings = self._contextual_analysis(validated)
        
        return findings
```

### 8. Template Quality Dashboard (Month 3)

Web dashboard showing:
- Precision/recall per template
- Tier assignments
- Historical trends
- False positive examples

---

## 📈 Success Metrics

| Metric | Current | Target (30d) | Target (90d) |
|--------|---------|--------------|--------------|
| Precision | 0% | ≥70% | ≥85% |
| Recall | Unknown | ≥75% | ≥80% |
| Findings/safe contract | 5,000+ | <100 | <20 |
| Tier 1 templates | 0 | 10 | 25 |
| Disabled templates | 0 | 15 | 5 |

---

## 🚀 Implementation Order

### Week 1: Quick Wins
1. ✅ Disable broken templates → 30% precision
2. ✅ Deploy 3 fixed templates → 40% precision
3. ✅ Add comment exclusions → 50% precision

### Week 2: Foundation
4. ✅ Build test corpus (50 vuln, 100 safe)
5. ✅ Validate all templates
6. ✅ Disable Tier 3 templates

### Week 3: Protection Detection
7. ✅ Custom reentrancy guard detection
8. ✅ Custom access control detection
9. ✅ CEI pattern detection

### Week 4: Validation
10. ✅ Test on production contracts
11. ✅ Measure actual precision/recall
12. ✅ Document results

---

## 💡 Key Insights from Opus

### Root Cause
Templates perform **syntactic matching** without **semantic validation**.

### Solution Architecture
Three-layer approach:
1. **Template Layer**: Precise patterns
2. **Validation Layer**: Pre/post-match checks
3. **Quality Layer**: Automated testing

### Hybrid Approach
- Semantic: Extract structure (functions, modifiers)
- Regex: Validate content (specific patterns)
- Context: Check protections (guards, CEI)

### Quality Framework
- Measure precision/recall per template
- Tier-based enablement
- Automated validation in CI

---

## 📋 Next Steps

1. **Immediate**: Create fixed template files from Opus response
2. **Day 1**: Disable broken templates, add comment exclusions
3. **Day 2-3**: Deploy fixed templates, test on production
4. **Week 2**: Build test corpus, implement validator
5. **Week 3-4**: Pattern-based protection detection

---

**Status**: Ready to implement  
**Expected Impact**: 0% → 70% precision in 30 days  
**Confidence**: High (detailed solution from Opus)
