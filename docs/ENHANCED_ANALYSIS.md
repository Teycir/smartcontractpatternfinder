# Enhanced Data Flow Analysis & State Mutation Tracking

## Overview

SCPF now implements **Level 3 & 4 analysis** - true semantic understanding beyond pattern matching:

```
┌─────────────────────────────────────────────────────────────────────┐
│                      Analysis Hierarchy                              │
├─────────────────────────────────────────────────────────────────────┤
│  Level 1: Regex          → Text patterns (fallback)          ✅     │
│  Level 2: AST Patterns   → Tree-sitter queries               ✅     │
│  Level 3: Data Flow      → Track value propagation           ✅ NEW │
│  Level 4: State Mutation → Track storage changes             ✅ NEW │
│  Level 5: Symbolic Exec  → Path constraints (future)         🔜     │
└─────────────────────────────────────────────────────────────────────┘
```

## Part 1: Taint Analysis (Data Flow)

### Concept

Track how untrusted data flows from **sources** to **sinks**:

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   SOURCES    │ ──▶ │  TRANSFORMS  │ ──▶ │    SINKS     │
├──────────────┤     ├──────────────┤     ├──────────────┤
│ msg.sender   │     │ assignments  │     │ selfdestruct │
│ msg.value    │     │ arithmetic   │     │ delegatecall │
│ msg.data     │     │ storage r/w  │     │ transfer     │
│ tx.origin    │     │ function ret │     │ call{value}  │
│ block.*      │     │ array access │     │ sstore       │
│ calldata     │     │              │     │ external call│
└──────────────┘     └──────────────┘     └──────────────┘
```

### Implementation

```rust
pub struct DataFlowAnalyzer {
    taint_map: HashMap<String, TaintInfo>,
    pub findings: Vec<DataFlowFinding>,
}

pub enum TaintSource {
    MsgSender,
    MsgValue,
    MsgData,
    TxOrigin,
    CallData,
    FunctionParam(String),
    ExternalCallReturn,
    BlockProperty(String),
}

pub enum TaintSink {
    Selfdestruct,
    DelegateCall,
    Call,
    Transfer,
    StorageWrite(String),
    ExternalCallArg,
}
```

### Detection Example

**Vulnerable Code:**
```solidity
function destroy(address target) public {
    address impl = target;  // Tainted from param
    impl.delegatecall(...);  // CRITICAL: User input to delegatecall
}
```

**Analysis Output:**
```
[CRITICAL] Line 3: Data flow violation
  Source: FunctionParam(target)
  Sink: DelegateCall
  Path: target → impl → delegatecall
  Severity: Critical
```

## Part 2: State Mutation Tracking

### Concept

Monitor all storage modifications and correlate with external calls:

```
┌─────────────────────────────────────────────────────────────────┐
│                   State Mutation Lifecycle                       │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────┐    ┌─────────────┐    ┌─────────────┐              │
│  │ DECLARE │───▶│    READ     │───▶│   MODIFY    │              │
│  │ Storage │    │ (SLOAD)     │    │ (SSTORE)    │              │
│  └─────────┘    └─────────────┘    └─────────────┘              │
│       │                                   │                      │
│       ▼                                   ▼                      │
│  ┌─────────────────────────────────────────────────────┐        │
│  │              Mutation Events                         │        │
│  ├─────────────────────────────────────────────────────┤        │
│  │ • Variable assignment (x = y)                        │        │
│  │ • Mapping write (map[k] = v)                         │        │
│  │ • Array push/pop (arr.push(x))                       │        │
│  │ • Struct field write (s.field = x)                   │        │
│  │ • Delete (delete x)                                  │        │
│  │ • Increment/Decrement (x++, --y)                     │        │
│  └─────────────────────────────────────────────────────┘        │
└─────────────────────────────────────────────────────────────────┘
```

### Implementation

```rust
pub struct StateMutationTracker {
    pub state_variables: HashMap<String, StateVariable>,
    pub mutations: Vec<MutationEvent>,
    pub external_calls: HashMap<String, Vec<usize>>,
}

pub enum MutationType {
    DirectAssignment,
    MappingWrite,
    ArrayWrite,
    ArrayPush,
    ArrayPop,
    Increment,
    Decrement,
    CompoundAssignment,
    Delete,
}

pub struct MutationEvent {
    pub variable: String,
    pub mutation_type: MutationType,
    pub function_name: String,
    pub line: usize,
    pub after_external_call: bool,
}
```

### Detection Example

**Vulnerable Code:**
```solidity
function withdraw() public {
    uint amount = balances[msg.sender];
    msg.sender.call{value: amount}("");  // Line 12: External call
    balances[msg.sender] = 0;            // Line 13: State change AFTER
}
```

**Analysis Output:**
```
[CRITICAL] CEI Violation Detected
  Variable: balances
  Function: withdraw
  External Call: Line 12
  State Mutation: Line 13
  Type: MappingWrite
  Severity: Critical (balance update after value transfer)
```

## Part 3: Combined Analysis

### Multi-Layer Detection

```
┌─────────────────────────────────────────────────────────────────┐
│                    Analysis Pipeline                             │
├─────────────────────────────────────────────────────────────────┤
│   Source Code                                                    │
│       │                                                          │
│       ▼                                                          │
│   ┌───────────────┐                                              │
│   │  Tree-Sitter  │                                              │
│   │    Parser     │                                              │
│   └───────┬───────┘                                              │
│           │                                                      │
│           ▼                                                      │
│   ┌───────────────────────────────────────────────────────────┐ │
│   │                      AST                                   │ │
│   └───────────────────────────────────────────────────────────┘ │
│           │                   │                   │              │
│           ▼                   ▼                   ▼              │
│   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐       │
│   │   Pattern   │     │  Data Flow  │     │  Mutation   │       │
│   │   Matching  │     │  Analysis   │     │  Tracking   │       │
│   └──────┬──────┘     └──────┬──────┘     └──────┬──────┘       │
│          │                   │                   │               │
│          └───────────────────┼───────────────────┘               │
│                              ▼                                   │
│                    ┌─────────────────┐                           │
│                    │    Findings     │                           │
│                    │   Correlation   │                           │
│                    └────────┬────────┘                           │
│                             ▼                                    │
│                    ┌─────────────────┐                           │
│                    │     Report      │                           │
│                    └─────────────────┘                           │
└─────────────────────────────────────────────────────────────────┘
```

## Usage

### Automatic Integration

Data flow analysis runs automatically:

```bash
# Standard scan includes all analysis layers
scpf scan vulnerable.sol

# Output includes:
# - Pattern matches (Level 1-2)
# - Data flow findings (Level 3)
# - State mutation violations (Level 4)
```

### Example Output

```
════════════════════════════════════════════════════════════
!  vulnerable.sol (125ms)

   [CRITICAL] Line 15: Data Flow Violation
   Source: FunctionParam(target)
   Sink: DelegateCall
   Path: target → impl → delegatecall
   
   [CRITICAL] Line 23: CEI Violation
   Variable: balances
   External Call: Line 21
   State Mutation: Line 23 (MappingWrite)
   
   [HIGH] Line 12: Pattern Match
   External call with value transfer detected
   
────────────────────────────────────────────────────────────
📊  Summary:
   Scanned: 1 | Failed: 0
   Severity: CRITICAL: 2 | HIGH: 1
   
   Analysis Breakdown:
   • Pattern Matches: 1
   • Data Flow Findings: 1
   • State Mutations: 1
════════════════════════════════════════════════════════════
```

## Severity Classification

### Data Flow

| Source | Sink | Severity |
|--------|------|----------|
| tx.origin | selfdestruct | **Critical** |
| msg.sender | delegatecall | **Critical** |
| calldata | delegatecall | **Critical** |
| function_param | delegatecall | **High** |
| * | selfdestruct | **High** |
| * | call | **Medium** |

### State Mutation

| Mutation | After Call | Severity |
|----------|-----------|----------|
| Balance mapping | Value transfer | **Critical** |
| Any state var | delegatecall | **High** |
| Any state var | call | **High** |
| Any state var | send/transfer | **Medium** |

## Architecture

### Module Structure

```
crates/scpf-core/src/
├── dataflow.rs          ← Data flow analyzer
│   ├── DataFlowAnalyzer
│   ├── TaintSource/TaintSink
│   ├── StateMutationTracker
│   └── MutationEvent
├── scanner.rs           ← Integration point
└── semantic.rs          ← AST parsing
```

### Key Components

1. **DataFlowAnalyzer**: Taint tracking engine
2. **StateMutationTracker**: Storage change monitor
3. **Scanner Integration**: Automatic analysis
4. **Finding Correlation**: Cross-layer detection

## Benefits

✅ **Precise Detection**: Understands actual data flow  
✅ **Low False Positives**: Semantic understanding  
✅ **Automatic Severity**: Risk-based classification  
✅ **Complementary**: Works with existing patterns  
✅ **Zero Configuration**: Enabled by default  

## Limitations

- Cannot track cross-contract calls
- No modifier detection (nonReentrant)
- Limited to 50-line analysis window
- Requires manual review for complex cases

## Future Enhancements

- [ ] Cross-function data flow
- [ ] Modifier awareness
- [ ] Storage vs memory distinction
- [ ] Call graph construction
- [ ] Symbolic execution (Level 5)
- [ ] Inter-contract analysis

## Files

- `crates/scpf-core/src/dataflow.rs` - Core analysis engine
- `crates/scpf-core/src/scanner.rs` - Integration
- `docs/DATA_FLOW_ANALYSIS.md` - This document
