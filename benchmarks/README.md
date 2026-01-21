# SCPF Benchmark Suite

## Purpose

This directory contains the **ground truth corpus** for measuring SCPF's accuracy, precision, and recall.

## Structure

```
benchmarks/
├── accuracy.rs              # Accuracy evaluation framework
├── ground-truth.json        # Labeled vulnerabilities
├── swc-registry/            # SWC test cases
├── known-exploits/          # Historical vulnerabilities
├── defi-protocols/          # Real-world DeFi contracts
└── false-positives/         # Known clean code
```

## Ground Truth Format

```json
{
  "contracts": [
    {
      "file_path": "benchmarks/known-exploits/dao-reentrancy.sol",
      "description": "The DAO hack - classic reentrancy",
      "vulnerabilities": [
        {
          "pattern_id": "reentrancy-unprotected",
          "line_number": 15,
          "severity": "critical",
          "description": "Reentrancy in splitDAO function",
          "exploitable": true
        }
      ],
      "safe_patterns": []
    }
  ]
}
```

## Running Accuracy Tests

```bash
# Run full benchmark suite
cargo test --release accuracy_tests

# Run specific category
cargo test --release accuracy_tests::reentrancy

# Generate accuracy report
cargo run --bin accuracy_report
```

## Metrics

### Overall Metrics
- **Precision**: TP / (TP + FP) - How many findings are correct?
- **Recall**: TP / (TP + FN) - How many vulnerabilities did we find?
- **F1 Score**: 2 * (P * R) / (P + R) - Harmonic mean

### Per-Category Metrics
- Reentrancy
- Access Control
- Integer Overflow
- Delegatecall
- Unchecked Returns
- etc.

## Quality Grades

| F1 Score | Grade | Status |
|----------|-------|--------|
| ≥ 0.90 | A | Excellent |
| ≥ 0.80 | B | Good |
| ≥ 0.70 | C | Acceptable |
| ≥ 0.60 | D | Needs Improvement |
| < 0.60 | F | Unacceptable |

## Current Status

**Last Run**: Not yet executed
**F1 Score**: TBD
**Grade**: TBD

## Adding Test Cases

### 1. Add Contract File
```bash
# Add vulnerable contract
cp your-contract.sol benchmarks/known-exploits/

# Or add safe contract
cp safe-contract.sol benchmarks/false-positives/
```

### 2. Label in ground-truth.json
```json
{
  "file_path": "benchmarks/known-exploits/your-contract.sol",
  "description": "Description of vulnerability",
  "vulnerabilities": [
    {
      "pattern_id": "pattern-name",
      "line_number": 42,
      "severity": "high",
      "description": "What's wrong",
      "exploitable": true
    }
  ]
}
```

### 3. Run Tests
```bash
cargo test accuracy_tests
```

## Sources

### SWC Registry
- https://swcregistry.io/
- Standardized vulnerability test cases

### Known Exploits
- The DAO (2016)
- Parity Wallet (2017)
- Bancor (2018)
- etc.

### DeFi Protocols
- Uniswap V2/V3
- Aave V2/V3
- Compound V2/V3
- Curve
- Balancer

### False Positives
- OpenZeppelin contracts
- Well-audited protocols
- Known safe patterns

## CI Integration

```yaml
# .github/workflows/accuracy.yml
name: Accuracy Tests

on: [push, pull_request]

jobs:
  accuracy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
      - name: Run Accuracy Tests
        run: cargo test --release accuracy_tests
      - name: Check F1 Score
        run: |
          F1=$(cargo run --bin accuracy_report | grep "F1 Score" | awk '{print $3}')
          if (( $(echo "$F1 < 0.80" | bc -l) )); then
            echo "F1 score $F1 below threshold 0.80"
            exit 1
          fi
```

## Contribution Guidelines

### Adding Vulnerabilities
1. Must be from real exploits or SWC registry
2. Must include exact line numbers
3. Must mark exploitability
4. Must include description

### Adding Safe Patterns
1. Must be from audited contracts
2. Must document why it's safe
3. Must list protection mechanisms

### Quality Standards
- All contracts must compile
- All labels must be accurate
- All line numbers must be exact (±2 lines tolerance)

## Roadmap

### Phase 1: Foundation (Current)
- [ ] 20+ SWC test cases
- [ ] 10+ known exploits
- [ ] 5+ false positive cases
- [ ] Baseline accuracy measurement

### Phase 2: Expansion
- [ ] 50+ SWC test cases
- [ ] 30+ known exploits
- [ ] 20+ DeFi protocol contracts
- [ ] 20+ false positive cases

### Phase 3: Comprehensive
- [ ] 100+ total test cases
- [ ] All major vulnerability classes
- [ ] Multiple DeFi protocols
- [ ] Continuous accuracy monitoring

## References

- [SWC Registry](https://swcregistry.io/)
- [Rekt News](https://rekt.news/)
- [DeFi Hack Labs](https://github.com/SunWeb3Sec/DeFiHackLabs)
- [Immunefi](https://immunefi.com/explore/)
- [OpenZeppelin](https://github.com/OpenZeppelin/openzeppelin-contracts)
