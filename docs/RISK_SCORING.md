# Risk Scoring System

The risk scoring system provides weighted vulnerability assessment for smart contracts.

## Features

- **Severity-based Weights** - Critical (30), High (15), Medium (7), Low (3), Info (1)
- **Pattern Multipliers** - Custom weights for specific vulnerability patterns
- **Composition Bonuses** - Extra score for combined vulnerabilities
- **Risk Levels** - None, Low, Medium, High, Critical
- **Recommendations** - Actionable security advice

## Usage

### Basic Risk Assessment

```rust
use scpf_core::{RiskScorer, Scanner};
use scpf_types::ScanResult;

let scorer = RiskScorer::with_defaults();
let result: ScanResult = scanner.scan(contract)?;
let assessment = scorer.assess(&result);

println!("Risk Score: {}", assessment.total_score);
println!("Risk Level: {}", assessment.risk_level.as_str());
```

### Custom Risk Configuration

```rust
use scpf_core::{RiskScorer, RiskConfig};
use scpf_types::Severity;
use std::collections::HashMap;

let mut config = RiskConfig::default();

// Custom severity weights
config.severity_weights.insert(Severity::Critical, 50);

// Pattern multipliers
config.pattern_multipliers.insert("reentrancy".to_string(), 2.0);
config.pattern_multipliers.insert("delegatecall".to_string(), 1.5);

// Custom thresholds
config.thresholds.critical = 100;

let scorer = RiskScorer::new(config);
```

### Composition Risk Scoring

```rust
use scpf_core::{TemplateComposer, RiskScorer};
use std::collections::HashMap;

let composer = TemplateComposer::new();
let scorer = RiskScorer::with_defaults();
let composition = create_reentrancy_composition();

let mut matches = HashMap::new();
matches.insert("external-call".to_string(), vec![10, 20]);
matches.insert("state-mutation".to_string(), vec![15, 25]);

let (matched, risk_score) = composer.evaluate_with_risk(
    &composition,
    &matches,
    &scorer
);

if matched {
    println!("Composition matched with risk score: {}", risk_score);
}
```

## Risk Levels

| Level | Score Range | Description |
|-------|-------------|-------------|
| **None** | 0 | No vulnerabilities detected |
| **Low** | 1-4 | Minor issues, low impact |
| **Medium** | 5-14 | Moderate issues, requires attention |
| **High** | 15-29 | Serious issues, immediate action needed |
| **Critical** | 30+ | Severe vulnerabilities, do not deploy |

## Severity Weights

| Severity | Base Score | Example Patterns |
|----------|-----------|------------------|
| **Critical** | 30 | Reentrancy, Unprotected selfdestruct |
| **High** | 15 | Access control, Delegatecall |
| **Medium** | 7 | Front-running, Balance equality |
| **Low** | 3 | Code quality, Gas optimization |
| **Info** | 1 | Best practices, Style issues |

## Composition Bonuses

Extra risk score for combined vulnerabilities:

- **Reentrancy Pattern** (+10): External call + State mutation
- **Access Control** (+10): Critical function without modifier
- **DeFi Attack** (+10): Price manipulation + Flash loan

## Pattern Multipliers

Adjust risk scores for specific patterns:

```rust
config.pattern_multipliers.insert("reentrancy".to_string(), 2.0);  // 2x weight
config.pattern_multipliers.insert("delegatecall".to_string(), 1.5); // 1.5x weight
```

## Recommendations

The system generates actionable recommendations:

- Critical vulnerabilities → "URGENT: Do not deploy"
- High severity → "Immediate attention required"
- Reentrancy detected → "Implement checks-effects-interactions"
- Access control issues → "Add proper modifiers"

## CLI Integration

```bash
# Scan with risk assessment
scpf scan 0x123... --chain ethereum --risk-threshold high

# Export risk report
scpf scan --output json | jq '.risk_assessment'
```

## Example Output

```json
{
  "total_score": 45,
  "risk_level": "Critical",
  "severity_breakdown": {
    "Critical": 30,
    "High": 15
  },
  "pattern_breakdown": {
    "reentrancy": 30,
    "access-control": 15
  },
  "composition_score": 10,
  "recommendations": [
    "URGENT: Critical vulnerabilities detected. Do not deploy.",
    "Implement checks-effects-interactions pattern.",
    "Add proper access control modifiers."
  ]
}
```
