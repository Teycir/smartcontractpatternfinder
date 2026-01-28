# Pattern Limitations: Execute Timelock Pattern

## Context
**Objective:** Detect `execute()` functions in governance contracts that lack timelock validation.
**Constraint:** The Regex Validator strictly prohibits nested quantifiers and specific lookahead combinations to prevent ReDoS (Regular Expression Denial of Service).

## Implementation Details

### Current Pattern
```yaml
- id: execute-function-found
  pattern: '^\s*function\s+execute\s*\('
  message: "Governance execute function found - manual review required to verify timelock."
  severity: critical
```

### The Trade-off
| Metric | Status | Reason |
| :--- | :--- | :--- |
| **Recall** | ✅ 100% | Pattern matches every `execute` function definition. |
| **Precision** | ⚠️ Low | Cannot differentiate between safe (timelocked) and unsafe functions. |
| **Safety** | ✅ High | Complies with strict O(n) regex performance requirements. |

**Why "Auto-Detection" Failed:**
Attempting to verify the *absence* of a string (e.g., "no timelock found") usually requires:
1.  **Negative Lookaheads (`(?!...)`)**: Combined with quantifiers (to scan the function body), these trigger ReDoS warnings.
2.  **Multiline Matching**: Scanning across newlines with `.*` is strictly prohibited by the validator for performance stability.

## Validator Technical Constraints
The regex engine enforces Atomic-like safety by rejecting:
1.  **Nested Quantifiers:** `(X+)+` or `\s+(?:\s+)*` (Causes exponential backtracking).
2.  **Bounded Quantifiers + Lookaheads:** `[^}]{0,500}(?!timelock)` (Performance risk on long inputs).
3.  **Ambiguous Groups:** `\s+(?:...)` (Creates potential overlaps if the group matches whitespace).

## Reviewer Guide
Since the tool acts as a "candidate finder" rather than a "vulnerability confirmer," the reviewer must verify:

### Manual Review Checklist:
1.  **Modifier Check:** Does the function have a `onlyTimelock`, `onlyGov`, or similar modifier in the definition?
2.  **Logic Check:** If no modifier, is there a `require(msg.sender == timelock)` or `_checkTimelock()` call in the first 3 lines?
3.  **Delay Validation:** Ensure the timelock enforces a delay (typically > 24 hours).
4.  **Bypass Check:** Ensure the address stored in the `timelock` variable cannot be overwritten by an unauthorized user.

## Technical Notes

**Regular Expressions are not parsers.** Trying to validate logic (like the presence of a modifier or a specific check inside a function body) using Regex leads to fragile patterns or ReDoS vulnerabilities.

**Security Context:** In vulnerability scanning, **Recall > Precision**:
- **High Recall:** Catch 100% of potentially vulnerable functions
- **Low Precision:** Flag safe functions (acceptable - wasted time)
- **False Negative:** Miss a vulnerability (unacceptable - security breach)

---
**Status:** Active  
**Last Updated:** 2026-01-28
