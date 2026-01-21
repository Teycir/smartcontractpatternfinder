# Template Fixes: Day 1 Implementation

**Date**: 2024-01-21  
**Status**: Partial Implementation Complete

---

## ✅ What Was Implemented

### 1. Disabled Broken Templates
Moved to `templates/.disabled/`:
- `cryptography-signatures.yaml` (weak-randomness-prevrandao broken)
- `tx_origin_auth.yaml` (matches everything)

### 2. Created Fixed Templates (v2)
Based on Claude Opus 4.5 recommendations:

**A. weak_randomness_prevrandao_v2.yaml**
- Uses regex with negative lookahead for comments
- Excludes event emissions and logging
- Severity: medium (was critical)

**B. tx_origin_auth_v2.yaml**
- Matches only authentication patterns (require, if, assert)
- Separate pattern for contract detection (info severity)
- Excludes comments and safe uses

**C. signature_return_unchecked_v2.yaml**
- Matches unassigned ecrecover/ECDSA.recover
- Excludes already-checked patterns (require, if, return)
- Separate info patterns for safe usage

### 3. Test Results

**Before** (with broken templates):
- 1,642 findings on 70-line test contract

**After** (disabled 2, added 3 fixed):
- 1,638 findings on 70-line test contract
- **Reduction**: 0.2% (4 findings)

---

## ❌ Why Minimal Impact

### Root Cause
Other templates are still broken:
- 33 templates remaining (36 total - 3 new)
- Most still match syntax without context
- Example: "Initialization function" - 374 CRITICAL findings

### The Real Problem
We fixed 3 templates but 30+ others are still broken:
- `advanced_audit.yaml` - overly broad patterns
- `semantic_vulnerabilities.yaml` - matches everything
- `erc20_compliance.yaml` - false positives on safe contracts
- Many more...

---

## 📊 Current State

### Templates
- **Total**: 36 templates
- **Disabled**: 2 (broken)
- **Fixed (v2)**: 3 (new versions)
- **Still Broken**: ~30

### Findings on Test Contract
- **Total**: 1,638
- **CRITICAL**: 374 (initialization functions)
- **HIGH**: 1,160 (various)
- **MEDIUM**: 104

### Precision
- **Target**: 70% (2 findings on test contract)
- **Actual**: Still ~0% (1,638 findings)
- **Gap**: Need to fix 30+ more templates

---

## 🚀 Next Steps (Priority Order)

### Immediate (This Week)

**1. Disable All Semantic Templates**
Most semantic templates are broken. Disable:
- `advanced_audit.yaml`
- `advanced_audit_fixed.yaml`
- `semantic_vulnerabilities.yaml`
- `semantic_working.yaml`
- `missing_access_control.yaml`
- All templates using `kind: semantic`

**Expected Impact**: 1,638 → ~200 findings

**2. Keep Only High-Quality Regex Templates**
Enable only templates with specific patterns:
- `delegatecall_user_input.yaml` (good)
- `reentrancy.yaml` (needs CEI detection)
- Simple, specific patterns only

**Expected Impact**: ~200 → ~50 findings

**3. Add Global Comment Exclusion**
Modify scanner to skip lines starting with `//` or `/*`

**Expected Impact**: ~50 → ~20 findings

### Short-term (Next Week)

**4. Build Test Corpus**
- 10 vulnerable contracts (known exploits)
- 10 safe contracts (USDC, DAI, etc.)
- Automated validation script

**5. Implement Template Validator**
Python script to:
- Test each template against corpus
- Calculate precision/recall
- Assign tiers
- Disable Tier 3 templates

**6. Fix Top 10 Templates**
Based on validator results, fix worst offenders

---

## 💡 Key Learnings

### What Worked
- Fixed templates have better patterns
- Hybrid approach (regex + exclusions) is correct
- Disabling broken templates is necessary

### What Didn't Work
- Fixing 3 templates isn't enough
- Need to disable/fix 30+ templates
- Can't fix templates one-by-one (too slow)

### Better Approach
1. **Disable all broken templates** (immediate precision gain)
2. **Keep only 5-10 good templates** (high precision, lower recall)
3. **Gradually re-enable fixed templates** (increase recall while maintaining precision)

---

## 📈 Realistic Timeline

### Week 1 (This Week)
- Day 1: ✅ Disable 2 worst templates, create 3 fixed versions
- Day 2: Disable all semantic templates → expect 80% reduction
- Day 3: Keep only 10 best regex templates → expect 95% reduction
- Day 4: Add comment exclusion → expect 98% reduction
- Day 5: Test on production contracts, measure actual precision

**Target**: <100 findings per safe contract (from 1,000-12,000)

### Week 2
- Build test corpus (10 vuln, 10 safe)
- Implement validator script
- Measure precision/recall per template
- Disable Tier 3 templates

**Target**: 50% precision, 60% recall

### Week 3-4
- Fix top 10 templates based on validator
- Re-enable fixed templates gradually
- Test on production after each fix

**Target**: 70% precision, 75% recall

---

## 🎯 Success Criteria

### Immediate (End of Week 1)
- [ ] <100 findings per safe contract
- [ ] >30% precision
- [ ] No broken templates enabled

### Short-term (End of Week 2)
- [ ] Test corpus built
- [ ] Validator working
- [ ] All templates tiered

### Medium-term (End of Week 4)
- [ ] >70% precision
- [ ] >75% recall
- [ ] 10+ Tier 1 templates

---

## 📝 Files Created

1. `templates/.disabled/` - Directory for broken templates
2. `templates/weak_randomness_prevrandao_v2.yaml` - Fixed version
3. `templates/tx_origin_auth_v2.yaml` - Fixed version
4. `templates/signature_return_unchecked_v2.yaml` - Fixed version
5. `TEMPLATE_FIXES_DAY1.md` - This document

---

**Status**: Day 1 complete, minimal impact  
**Next**: Disable all semantic templates (Day 2)  
**Expected**: 80% reduction in findings
