# Smart Contract Vulnerability Report Standardization Rules

## Purpose
This document defines the standard format and rules for generating vulnerability reports to ensure consistency, clarity, and actionable insights.

## Reference Example
📄 **Complete example report:** [docs/REPORT_EXAMPLE_TRUE_POSITIVES.md](docs/REPORT_EXAMPLE_TRUE_POSITIVES.md)

This example demonstrates:
- Proper ordering by risk score (descending)
- Standardized formatting and structure
- Complete vulnerability documentation
- Correct use of severity badges and icons
- Professional presentation suitable for stakeholders

---

## Core Principles

1. **Most Vulnerable First**: Always order findings by risk score (descending)
2. **Actionable**: Include specific remediation steps
3. **Comprehensive**: Cover all critical information
4. **Consistent**: Use standardized formatting and terminology
5. **Verifiable**: Include links to blockchain explorers

---

## Report Structure

### 1. Header Section
```markdown
# 🚨 TRUE POSITIVE VULNERABILITY REPORT
## Smart Contract Security Analysis - Report ID: {REPORT_ID}

**Generated:** {DATE}  
**Analysis Period:** {PERIOD}  
**Total Contracts Scanned:** {TOTAL_SCANNED}  
**True Positives Identified:** {TRUE_POSITIVES} Contracts  
**False Positive Rate:** {FP_RATE}%  
**Severity Level:** {MAX_SEVERITY}
```

### 2. Executive Summary
- Brief overview of findings
- Key vulnerability types identified
- Immediate action items
- Impact assessment

### 3. Vulnerability Distribution
- Table showing vulnerability type counts
- Sorted by frequency (most common first)

### 4. Detailed Findings

**CRITICAL RULE: Findings MUST be ordered by risk score (highest to lowest)**

For each contract, include:
- **Rank** (based on risk score)
- **Contract Name** (extracted from source code)
- **Address** (full contract address)
- **Chain** (Ethereum, Polygon, Arbitrum, etc.)
- **Risk Score** (numerical value)
- **Severity Badge** (🔴 CRITICAL, 🟠 HIGH, 🟡 MEDIUM, 🟢 LOW)
- **Website** (if available)
- **Explorer Link** (clickable URL)
- **Vulnerabilities** (detailed list)
- **Impact Description** (specific to vulnerability type)

### 5. Vulnerability Matrix
- Consolidated table showing all contracts and their vulnerability types
- Use checkmarks (✓) for presence of vulnerability
- Enables quick pattern recognition

### 6. Chain Distribution
- Breakdown by blockchain network
- Percentage calculations
- Visual representation preferred

### 7. Vulnerability Definitions
Standard definitions for each vulnerability type:
- 🔓 Unprotected Initialize
- 💰 External Mint/Burn Without Modifier
- ⚡ Arbitrary Call Without Checks
- 📉 Unsafe Type Downcast
- 🌊 Deposit Without Block Check
- 🔑 Critical Function Without Modifier

Each definition must include:
- **Risk**: Primary risk category
- **Description**: What the vulnerability is
- **Impact**: Potential consequences
- **Remediation**: How to fix it

### 8. Remediation Roadmap
Prioritized action items:
- **Priority 0 (CRITICAL)**: 0-24 hours
- **Priority 1 (HIGH)**: 1-7 days
- **Priority 2 (MEDIUM)**: 1-30 days

### 9. Metadata & Disclaimer
- Report ID
- Generation timestamp
- Tool name and version
- Analysis methodology
- Disclaimer about limitations
- Security resources

---

## Severity Classification Rules

```
Risk Score >= 150: 🔴 CRITICAL - Immediate Action Required
Risk Score >= 100: 🟠 HIGH - Urgent Attention Needed
Risk Score >= 50:  🟡 MEDIUM - Review & Patch Recommended
Risk Score < 50:   🟢 LOW - Monitor & Plan Remediation
```

---

## Vulnerability Impact Descriptions

### When Init vulnerability present:
```
⚠️ **Critical Risk:** Unprotected initialization allows contract takeover
```

### When Mint/Burn vulnerability present:
```
💰 **Economic Risk:** Unrestricted token supply manipulation possible
```

### When Call vulnerability present:
```
⚡ **Code Injection Risk:** {COUNT} arbitrary external call vector(s) detected
```

### When Flash Loan vulnerability present:
```
🌊 **Flash Loan Risk:** Vulnerable to same-block price manipulation attacks
```

---

## Formatting Standards

### Contract Names
- Use actual contract name from source code
- If multiple contracts in file, use the main implementation contract
- Fallback to "Unknown" if name cannot be determined

### Addresses
- Always display full address in monospace: `0x...`
- Include shortened version in summary tables: `0x1234...5678`

### Chain Names
- Ethereum: ⟠ Ethereum
- Polygon: 🟣 Polygon
- Arbitrum: 🔵 Arbitrum
- BSC: 🟡 BSC
- Avalanche: 🔴 Avalanche

### Links
- Explorer links must be clickable markdown: `[View on Explorer](URL)`
- Website links must be verified and clickable
- Use appropriate explorer for each chain:
  - Ethereum: https://etherscan.io
  - Polygon: https://polygonscan.com
  - Arbitrum: https://arbiscan.io

### Tables
- Use markdown tables with proper alignment
- Header row must be bolded
- Use separator row with appropriate dashes

---

## Ordering Rules

### Primary Ordering: Risk Score (Descending)
```python
contracts.sort(key=lambda x: x['risk'], reverse=True)
```

### Secondary Ordering (when risk scores equal): Contract Address (Ascending)
```python
contracts.sort(key=lambda x: (x['risk'], x['address']), reverse=(True, False))
```

### Vulnerability Type Ordering (in tables): Frequency (Descending)
```python
vulnerabilities.sort(key=lambda x: x['count'], reverse=True)
```

---

## Data Validation Rules

1. **Risk Score**: Must be positive integer
2. **Contract Address**: Must match regex `^0x[a-fA-F0-9]{40}$`
3. **Chain**: Must be in allowed list: [ethereum, polygon, arbitrum, bsc, avalanche]
4. **Website**: Must be valid URL or null
5. **Vulnerability Count**: Must be >= 1 for true positives

---

## Quality Checks

Before generating report, verify:
- [ ] All contracts ordered by risk score (descending)
- [ ] All addresses are valid and checksummed
- [ ] All explorer links are functional
- [ ] All severity badges are correctly assigned
- [ ] All vulnerability counts are accurate
- [ ] Report ID matches scan timestamp
- [ ] Statistics are calculated correctly
- [ ] No broken markdown formatting
- [ ] All tables are properly aligned

---

## Example Template

```markdown
### 1. ChannelImplementation 🔴 CRITICAL

| Field | Value |
|-------|-------|
| **Contract Name** | ChannelImplementation |
| **Address** | `0x7588248c3e40a21a183c0372674abff67593bad1` |
| **Chain** | 🟣 Polygon |
| **Risk Score** | **355** |
| **Severity** | 🔴 CRITICAL Immediate Action Required |
| **Website** | https://example.com |
| **Explorer** | [View on Explorer](https://polygonscan.com/address/0x...) |
| **Vulnerabilities** | Init (2), Downcast (1) |

⚠️ **Critical Risk:** Unprotected initialization allows contract takeover

---
```

---

## Automated Generation Guidelines

When using AI/scripts to generate reports:

1. **Always fetch latest contract data** from blockchain explorers
2. **Extract contract names** from verified source code
3. **Calculate risk scores** using weighted vulnerability algorithm
4. **Sort before rendering** to ensure correct ordering
5. **Validate all data** before writing to file
6. **Use consistent emoji** for visual categorization
7. **Include timestamps** in ISO 8601 format
8. **Version control** report templates

---

## Version History

- **v1.0** (2026-01-28): Initial standardization rules
  - Defined core structure
  - Established ordering requirements
  - Created severity classification system

---

## Future Enhancements

- [ ] Add CVE-style vulnerability IDs
- [ ] Include CVSS scoring
- [ ] Add automated remediation suggestions
- [ ] Include code snippets of vulnerable sections
- [ ] Generate executive summary in multiple languages
- [ ] Add visual charts and graphs
- [ ] Include historical trend analysis
