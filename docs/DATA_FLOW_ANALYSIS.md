# Data Flow Analysis & State Mutation Tracking

## Overview

SCPF now includes advanced data flow analysis to detect reentrancy vulnerabilities by tracking:
- External call sequences
- State variable mutations
- Temporal relationships between calls and state changes

## Features

### 1. External Call Detection
Tracks all external calls with metadata:
- Method type (`call`, `delegatecall`, `send`, `transfer`)
- Line number
- Value transfer flag

### 2. State Mutation Tracking
Monitors state changes:
- Direct assignments (`x = 5`)
- Mapping writes (`balances[addr] = 0`)
- Array writes (`items[i] = value`)
- Augmented assignments (`balance -= amount`)

### 3. Reentrancy Risk Analysis
Correlates calls with state changes:
- **Critical**: Value transfer + balance update after call
- **High**: External call + state mutation after
- **Medium**: Potential issues requiring review

## Architecture

```
┌─────────────────────────────────────────┐
│         Scanner (scanner.rs)            │
│  ┌───────────────────────────────────┐  │
│  │   Data Flow Analysis Integration  │  │
│  └───────────────────────────────────┘  │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│    DataFlowAnalysis (dataflow.rs)       │
│  ┌───────────────────────────────────┐  │
│  │  1. Parse AST                     │  │
│  │  2. Extract external calls        │  │
│  │  3. Track state mutations         │  │
│  │  4. Correlate call→mutation       │  │
│  │  5. Generate risk reports         │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

## Implementation

### Core Types

```rust
pub struct ExternalCall {
    pub method: String,      // call, delegatecall, send, transfer
    pub line: usize,
    pub has_value: bool,     // true if {value: ...}
}

pub enum StateChange {
    Assignment { var: String, line: usize },
    MapWrite { map: String, line: usize },
    Increment { var: String, line: usize },
    Decrement { var: String, line: usize },
}

pub struct ReentrancyRisk {
    pub call_line: usize,
    pub call_method: String,
    pub state_change_line: usize,
    pub state_var: String,
    pub severity: RiskSeverity,
}
```

### Detection Algorithm

```rust
fn detect_reentrancy(calls: &[ExternalCall], changes: &[StateChange]) {
    for call in calls {
        for change in changes {
            // Check if state change occurs after call
            if change.line > call.line && (change.line - call.line) < 50 {
                // Determine severity
                let severity = if call.has_value && is_balance_update(change) {
                    Critical  // Classic reentrancy
                } else if is_dangerous_call(call) {
                    High      // Potential reentrancy
                } else {
                    Medium    // Review needed
                };
                
                report_risk(call, change, severity);
            }
        }
    }
}
```

## Usage

Data flow analysis runs automatically during scans:

```bash
# Scan with data flow analysis
scpf scan vulnerable.sol

# Output includes data flow findings
[CRITICAL] Line 12: Data flow analysis: call on line 12 followed by 
           state mutation of 'balances' on line 13. Potential reentrancy.
```

## Example Detection

**Vulnerable Code:**
```solidity
function withdraw() public {
    uint amount = balances[msg.sender];
    (bool success, ) = msg.sender.call{value: amount}("");  // Line 12
    balances[msg.sender] = 0;  // Line 13 - AFTER call!
}
```

**Data Flow Analysis Output:**
```
[CRITICAL] Line 12: Data flow analysis: call on line 12 followed by 
           state mutation of 'balances' on line 13. 
           Potential reentrancy vulnerability.
```

## Benefits

1. **Precise Detection**: Tracks actual data flow, not just patterns
2. **Low False Positives**: Understands temporal relationships
3. **Severity Classification**: Automatic risk assessment
4. **Complementary**: Works alongside regex/semantic patterns

## Limitations

- Cannot distinguish storage vs memory variables
- Does not detect reentrancy guards (nonReentrant modifier)
- Limited to 50-line window for performance
- Requires manual review for complex cases

## Future Enhancements

- [ ] Cross-function data flow analysis
- [ ] Modifier detection (nonReentrant, onlyOwner)
- [ ] Storage vs memory distinction
- [ ] Call graph construction
- [ ] Taint analysis for user input tracking
- [ ] Inter-contract call tracking

## Files

- `crates/scpf-core/src/dataflow.rs` - Core analysis engine
- `crates/scpf-core/src/scanner.rs` - Integration point
- `crates/scpf-core/src/semantic.rs` - AST parsing support
