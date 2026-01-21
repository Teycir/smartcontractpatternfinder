# CRITICAL: False Positive Crisis

## Real-World Test Results

### Uniswap V2 Factory (0x5c69...aa6f)
- **Status**: Production contract, heavily audited, billions in TVL
- **SCPF Findings**: **12,539 issues**
- **Reality**: Safe, battle-tested contract

### Pattern Breakdown
- `rebasing-token-balance`: Matches every `uint amount` and `uint balance`
- `token-supply-overflow`: Matches every function name
- Result: Noise, not signal

## Root Cause

**Patterns are matching syntax, not vulnerabilities.**

Example bad pattern:
```yaml
pattern: 'uint amount'  # Matches EVERYTHING
```

Should be:
```yaml
pattern: 'balances\[.*\]\s*=.*\.call'  # Specific vulnerability context
```

## Impact

- **Precision**: <1% (12,539 false positives / 12,539 total)
- **Recall**: Unknown
- **F1 Score**: ~0.01 (far below 0.80 target)
- **Production Ready**: ❌ Absolutely not

## Action Required

**STOP. Fix patterns before ANY other work.**

1. Disable overly broad patterns immediately
2. Rewrite patterns with context requirements
3. Test on 10 known-safe contracts
4. Only re-enable when precision >85%

## Patterns to Disable Immediately

1. `rebasing-token-balance` - Matches all uint variables
2. `token-supply-overflow` - Matches all function names
3. Any pattern matching >100 times per contract

## Next Steps

Day 3 Morning:
1. Disable bad patterns
2. Re-scan Uniswap V2
3. Target: <10 findings
4. Verify each finding is real

**No other work until this is fixed.**
