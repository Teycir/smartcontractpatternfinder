# Vulnerability Report Generation Guide

## Overview

This guide explains how to generate, format, and use vulnerability reports for the SmartContractPatternFinder tool. Reports should follow the standardized format to ensure consistency and professionalism.

---

## Quick Reference

- **Standardization Rules:** [REPORT_STANDARDIZATION_RULES.md](../REPORT_STANDARDIZATION_RULES.md)
- **Reference Example:** [REPORT_EXAMPLE_TRUE_POSITIVES.md](REPORT_EXAMPLE_TRUE_POSITIVES.md)
- **Report Template:** See "Template Structure" section below

---

## Report Types

### 1. True Positive Reports
Focus on **confirmed vulnerabilities** with high confidence.

**Characteristics:**
- Manual verification completed
- False positives filtered out
- Risk scores validated
- Remediation steps specific

**Example:** [REPORT_EXAMPLE_TRUE_POSITIVES.md](REPORT_EXAMPLE_TRUE_POSITIVES.md)

### 2. Full Scan Reports
Include all findings (true positives + potential issues).

**Characteristics:**
- Comprehensive coverage
- Higher false positive rate
- Requires manual review
- Good for initial assessment

### 3. 0-Day Exploit Reports
Focus on **recent exploits** from DeFiHackLabs and similar sources.

**Characteristics:**
- Time-sensitive information
- Known attack patterns
- PoC availability
- Historical context

---

## Report Generation Process

### Step 1: Scan & Analysis
```bash
# Run scanner with desired parameters
cargo run -- scan \
  --chain ethereum \
  --limit 100 \
  --min-severity HIGH \
  --output report.json
```

### Step 2: Extract True Positives
```python
# Filter and validate findings
true_positives = []
for finding in all_findings:
    if validate_vulnerability(finding):
        true_positives.append(finding)
```

### Step 3: Gather Contract Metadata
```python
# Fetch contract names, websites, verification status
for contract in true_positives:
    contract['name'] = fetch_contract_name(contract['address'])
    contract['website'] = extract_website(contract['source'])
    contract['verified'] = check_verification(contract['address'])
```

### Step 4: Sort by Risk Score
```python
# CRITICAL: Always sort by risk score (descending)
true_positives.sort(key=lambda x: x['risk'], reverse=True)
```

### Step 5: Generate Markdown Report
```python
# Use standardized template
report = generate_report(
    contracts=true_positives,
    template='standardized',
    include_remediation=True
)
```

### Step 6: Validate Output
```python
# Run quality checks
validate_report(report)
- Check ordering
- Verify links
- Validate addresses
- Confirm statistics
```

---

## Template Structure

### Minimal Report Template

```markdown
# 🚨 TRUE POSITIVE VULNERABILITY REPORT
## Smart Contract Security Analysis - Report ID: {REPORT_ID}

**Generated:** {DATE}
**True Positives:** {COUNT} Contracts
**Severity:** {MAX_SEVERITY}

---

## 🔥 TOP VULNERABILITIES (Ordered by Risk Score)

### 1. {CONTRACT_NAME} {SEVERITY_BADGE}

| Field | Value |
|-------|-------|
| **Contract Name** | {NAME} |
| **Address** | `{ADDRESS}` |
| **Chain** | {CHAIN_ICON} {CHAIN} |
| **Risk Score** | **{RISK}** |
| **Severity** | {SEVERITY_BADGE} {SEVERITY_TEXT} |
| **Explorer** | [View on Explorer]({EXPLORER_URL}) |
| **Vulnerabilities** | {VULN_LIST} |

{IMPACT_DESCRIPTION}

---

[Repeat for all contracts...]

## 📋 VULNERABILITY MATRIX

| Rank | Contract | Chain | Risk | Init | Mint | Burn | Call | ... |
|------|----------|-------|------|------|------|------|------|-----|
| 1 | {NAME} | {CHAIN} | {RISK} | ✓ | ✓ | | ✓ | ... |

---

## 🛡️ REMEDIATION ROADMAP

### 🚨 Priority 0 - CRITICAL (Next 24 Hours)
{TOP_3_CONTRACTS}

### ⚡ Priority 1 - HIGH (Next 7 Days)
{HIGH_RISK_CONTRACTS}

### 📊 Priority 2 - MEDIUM (Next 30 Days)
{REMAINING_CONTRACTS}

---

## 📞 REPORT METADATA

- **Report ID:** {REPORT_ID}
- **Generated:** {TIMESTAMP}
- **Tool:** SmartContractPatternFinder
- **Source:** {SOURCE_PATH}
```

---

## Severity Classification

Use these exact criteria:

```python
def classify_severity(risk_score):
    if risk_score >= 150:
        return "🔴 CRITICAL", "Immediate Action Required"
    elif risk_score >= 100:
        return "🟠 HIGH", "Urgent Attention Needed"
    elif risk_score >= 50:
        return "🟡 MEDIUM", "Review & Patch Recommended"
    else:
        return "🟢 LOW", "Monitor & Plan Remediation"
```

---

## Vulnerability Icons Reference

Use consistent emoji for vulnerability types:

| Vulnerability | Icon | Usage |
|--------------|------|-------|
| Unprotected Initialize | 🔓 | `🔓 Unprotected Initialize` |
| Token Mint/Burn | 💰 | `💰 External Mint Without Modifier` |
| Arbitrary Calls | ⚡ | `⚡ Arbitrary Call Without Checks` |
| Unsafe Downcast | 📉 | `📉 Unsafe Type Downcast` |
| Flash Loan Vulnerable | 🌊 | `🌊 Deposit Without Block Check` |
| Critical Functions | 🔑 | `🔑 Critical Function Without Modifier` |

---

## Chain Icons Reference

| Chain | Icon | Markdown |
|-------|------|----------|
| Ethereum | ⟠ | `⟠ Ethereum` |
| Polygon | 🟣 | `🟣 Polygon` |
| Arbitrum | 🔵 | `🔵 Arbitrum` |
| BSC | 🟡 | `🟡 BSC` |
| Avalanche | 🔴 | `🔴 Avalanche` |

---

## Impact Descriptions

Use these standardized descriptions based on vulnerability type:

### Unprotected Initialize
```markdown
⚠️ **Critical Risk:** Unprotected initialization allows contract takeover
```

### Token Mint/Burn
```markdown
💰 **Economic Risk:** Unrestricted token supply manipulation possible
```

### Arbitrary Calls
```markdown
⚡ **Code Injection Risk:** {COUNT} arbitrary external call vector(s) detected
```

### Flash Loan
```markdown
🌊 **Flash Loan Risk:** Vulnerable to same-block price manipulation attacks
```

---

## Quality Checklist

Before publishing a report, verify:

- [ ] **Ordering**: Contracts sorted by risk score (descending)
- [ ] **Addresses**: All addresses validated (40 hex chars)
- [ ] **Links**: All explorer links functional
- [ ] **Severity**: Badges correctly assigned
- [ ] **Statistics**: Counts and percentages accurate
- [ ] **Format**: No broken markdown
- [ ] **Tables**: Properly aligned
- [ ] **Emoji**: Consistent use of icons
- [ ] **Spelling**: No typos in critical sections
- [ ] **Timestamps**: ISO 8601 format
- [ ] **Metadata**: Complete report information

---

## Automation Script

### Python Report Generator

```python
#!/usr/bin/env python3
import json
from pathlib import Path

def generate_report(contracts, output_path):
    """
    Generate standardized vulnerability report
    
    Args:
        contracts: List of contract dicts with risk scores
        output_path: Where to save the report
    """
    # Sort by risk score (DESCENDING)
    contracts.sort(key=lambda x: x['risk'], reverse=True)
    
    # Generate header
    report = f"""# 🚨 TRUE POSITIVE VULNERABILITY REPORT
## Smart Contract Security Analysis

**Generated:** {datetime.now().isoformat()}
**True Positives:** {len(contracts)} Contracts

---

"""
    
    # Add detailed findings
    for rank, contract in enumerate(contracts, 1):
        severity, desc = classify_severity(contract['risk'])
        
        report += f"""### {rank}. {contract['name']} {severity}

| Field | Value |
|-------|-------|
| **Contract Name** | {contract['name']} |
| **Address** | `{contract['address']}` |
| **Chain** | {get_chain_icon(contract['chain'])} {contract['chain']} |
| **Risk Score** | **{contract['risk']}** |
| **Severity** | {severity} {desc} |
| **Explorer** | [View on Explorer]({contract['explorer']}) |
| **Vulnerabilities** | {contract['vulnerabilities']} |

{get_impact_description(contract)}

---

"""
    
    # Save report
    Path(output_path).write_text(report)
    print(f"✅ Report generated: {output_path}")

# Usage
contracts = load_true_positives()
generate_report(contracts, 'TRUE_POSITIVE_REPORT.md')
```

---

## Best Practices

### DO ✅

1. **Always sort by risk score** (highest first)
2. **Validate all addresses** before including
3. **Test all explorer links** for functionality
4. **Use consistent formatting** throughout
5. **Include complete metadata** for traceability
6. **Add specific impact descriptions** per vulnerability
7. **Provide actionable remediation** steps
8. **Double-check statistics** and counts
9. **Use professional language** suitable for stakeholders
10. **Reference the example report** when uncertain

### DON'T ❌

1. **Don't sort alphabetically** - use risk score
2. **Don't skip validation** of contract data
3. **Don't use generic descriptions** - be specific
4. **Don't forget to update timestamps**
5. **Don't include false positives** without warning
6. **Don't mix formatting styles**
7. **Don't omit remediation steps**
8. **Don't forget to test markdown rendering**
9. **Don't use broken emoji** or inconsistent icons
10. **Don't publish without quality check**

---

## Examples

### Good Report Header
```markdown
# 🚨 TRUE POSITIVE VULNERABILITY REPORT
## Smart Contract Security Analysis - Report ID: 1769637304

**Generated:** January 28, 2026  
**Total Contracts Scanned:** 75  
**True Positives Identified:** 17 Contracts  
**False Positive Rate:** 66%  
**Severity Level:** CRITICAL  
```
✅ Clear, complete, properly formatted

### Bad Report Header
```markdown
# Vulnerability Report
Generated: 1/28/26
Found: 17 issues
```
❌ Missing details, inconsistent format, unclear

### Good Contract Entry
```markdown
### 1. ChannelImplementation 🔴 CRITICAL

| Field | Value |
|-------|-------|
| **Contract Name** | ChannelImplementation |
| **Address** | `0x7588248c3e40a21a183c0372674abff67593bad1` |
| **Chain** | 🟣 Polygon |
| **Risk Score** | **355** |
| **Severity** | 🔴 CRITICAL Immediate Action Required |
| **Explorer** | [View on Explorer](https://polygonscan.com/address/0x...) |
| **Vulnerabilities** | Init (2), Downcast (1) |

⚠️ **Critical Risk:** Unprotected initialization allows contract takeover
```
✅ Complete information, clear structure, actionable

### Bad Contract Entry
```markdown
Contract: 0x758...ad1 - Risk: 355
Problems: init, downcast
```
❌ Missing details, poor formatting, not actionable

---

## Troubleshooting

### Issue: Contracts not in risk order
**Solution:** Verify sort function uses `reverse=True`
```python
contracts.sort(key=lambda x: x['risk'], reverse=True)
```

### Issue: Broken explorer links
**Solution:** Use correct explorer for each chain
```python
explorers = {
    'ethereum': 'https://etherscan.io',
    'polygon': 'https://polygonscan.com',
    'arbitrum': 'https://arbiscan.io'
}
```

### Issue: Incorrect statistics
**Solution:** Calculate after filtering and sorting
```python
total = len(contracts)
eth_count = sum(1 for c in contracts if c['chain'] == 'ethereum')
```

### Issue: Markdown not rendering
**Solution:** Validate markdown syntax
```bash
# Use markdown linter
markdownlint report.md
```

---

## Publishing Reports

### Internal Use
- Save to project reports directory
- Version control with git
- Include in documentation

### External Sharing
- Anonymize sensitive information if needed
- Add appropriate disclaimers
- Ensure stakeholder review
- Consider PDF export for formal delivery

### Public Disclosure
- Follow responsible disclosure practices
- Coordinate with affected parties
- Redact if necessary
- Include timeline of events

---

## Version Control

All reports should be versioned:

```
reports/
├── report_1769637304/          # Unix timestamp ID
│   ├── TRUE_POSITIVE_REPORT.md
│   ├── vuln_summary.md
│   ├── 0day_summary.md
│   └── sources/
│       ├── ethereum/
│       └── polygon/
```

---

## Related Documentation

- [REPORT_STANDARDIZATION_RULES.md](../REPORT_STANDARDIZATION_RULES.md) - Complete formatting rules
- [REPORT_EXAMPLE_TRUE_POSITIVES.md](REPORT_EXAMPLE_TRUE_POSITIVES.md) - Reference example
- [RISK_SCORING.md](RISK_SCORING.md) - Risk score calculation methodology
- [EXPLOITABILITY_SCORING.md](EXPLOITABILITY_SCORING.md) - Exploitability metrics

---

**Last Updated:** January 28, 2026  
**Maintainer:** SmartContractPatternFinder Team
