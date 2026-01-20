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
use scpf_types::{Template, Pattern, Severity, ScanResult};
use std::path::PathBuf;

// Create a scanner with templates
let templates = vec![
    Template {
        id: "reentrancy-v4".to_string(),
        name: "Reentrancy Detection".to_string(),
        description: "Detects reentrancy vulnerabilities".to_string(),
        tags: vec!["reentrancy".to_string()],
        patterns: vec![
            Pattern {
                id: "external-call".to_string(),
                pattern: r"call\s*\{".to_string(),
                message: "External call detected".to_string(),
            }
        ],
        severity: Severity::High,
    }
];

let scanner = Scanner::new(templates)?;
let contract = "pragma solidity ^0.8.0;\ncontract Test { function test() { call{}(address(0)); } }";

let result: ScanResult = scanner.scan(contract, PathBuf::from("test.sol"))?;
let scorer = RiskScorer::with_defaults();
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
use scpf_types::{Severity, ComposedTemplate, CompositionRule};
use std::collections::HashMap;

// Create a reentrancy composition
let composition = ComposedTemplate {
    id: "reentrancy-comprehensive".to_string(),
    name: "Comprehensive Reentrancy Detection".to_string(),
    description: "Combines external calls with state mutations".to_string(),
    severity: Severity::Critical,
    base_templates: vec![],
    composition_rules: vec![CompositionRule::Sequential {
        templates: vec!["external-call".to_string(), "state-mutation".to_string()],
        max_distance: 50,
    }],
};

let composer = TemplateComposer::new();
let scorer = RiskScorer::with_defaults();

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

The following example shows a complete risk assessment with composition bonuses applied.
The `total_score` is calculated as the sum of `severity_breakdown` values (45) plus the `composition_score` (10):

```json
{
  "total_score": 55,
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
