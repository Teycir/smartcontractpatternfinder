# Smart Contract Pattern Finder (SCPF)

<!-- markdownlint-disable MD033 -->
<div align="center">
  <img src="assets/scpf_banner.gif" width="60%" alt="Smart Contract Pattern Finder Banner" style="border-radius: 10px; box-shadow: 0 0 20px rgba(0, 255, 255, 0.2);" />
</div>
<!-- markdownlint-enable MD033 -->

🔍 High-performance tool for detecting security vulnerabilities and patterns in smart contracts across multiple blockchains.

**How it works:** Define patterns in YAML templates → SCPF scans smart contracts → Finds matching patterns → Reports vulnerabilities

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/scpf-cli.svg?style=for-the-badge)](https://crates.io/crates/scpf-cli)
[![Docs.rs](https://img.shields.io/badge/docs.rs-scpf--core-blue?style=for-the-badge&logo=docs.rs)](https://docs.rs/scpf-core)
[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/Teycir/smartcontractpatternfinder/rust.yml?style=for-the-badge&logo=github)](https://github.com/Teycir/smartcontractpatternfinder/actions)

**Tags:** `rust` `smart-contracts` `security` `scanner` `ethereum` `blockchain` `vulnerability-detection` `pattern-matching` `defi` `web3` `solidity` `static-analysis`

---

## 📑 Table of Contents

- [Smart Contract Pattern Finder (SCPF)](#smart-contract-pattern-finder-scpf)
  - [📑 Table of Contents](#-table-of-contents)
  - [✨ Features](#-features)
  - [💡 Use Cases](#-use-cases)
    - [Security Auditing](#security-auditing)
    - [DeFi Research](#defi-research)
    - [Bug Bounty Hunting](#bug-bounty-hunting)
    - [Development \& CI/CD](#development--cicd)
    - [Education \& Training](#education--training)
  - [🚀 Quick Start](#-quick-start)
    - [How It Works](#how-it-works)
    - [Installation](#installation)
    - [Initialize Project](#initialize-project)
    - [Scan a Contract](#scan-a-contract)
  - [📋 Template Example](#-template-example)
  - [🏗️ Architecture](#️-architecture)
    - [Module Overview](#module-overview)
  - [🛠️ CLI Commands](#️-cli-commands)
    - [`scpf scan`](#scpf-scan)
    - [`scpf init`](#scpf-init)
  - [🎯 Supported Chains](#-supported-chains)
  - [🔧 Configuration](#-configuration)
    - [Getting API Keys](#getting-api-keys)
  - [📊 Output Formats](#-output-formats)
  - [🧪 Development](#-development)
  - [📝 Creating Templates](#-creating-templates)
    - [How Templates Work](#how-templates-work)
    - [Creating a Template](#creating-a-template)
  - [🤝 Contributing](#-contributing)
    - [Contribution Guidelines](#contribution-guidelines)
  - [📄 License](#-license)
  - [👤 Author](#-author)
  - [🔗 Links](#-links)

---

## ✨ Features

- 🌐 **Multi-chain Support** - Ethereum, BSC, Polygon
- 📝 **YAML Templates** - Easy-to-write pattern definitions
- ⚡ **Fast Scanning** - Regex-based pattern matching
- 💾 **Smart Caching** - Avoid redundant API calls
- 🎯 **Modular Architecture** - Clean, testable code
- 🔒 **Security Focused** - Detect reentrancy, delegatecall, and more
- 🚀 **High Performance** - Built with Rust for speed
- 🔧 **Extensible** - Easy to add custom patterns

---

## 💡 Use Cases

### Security Auditing
- **Automated Vulnerability Detection** - Scan contracts for common vulnerabilities (reentrancy, delegatecall, unchecked calls)
- **Pre-deployment Checks** - Validate contracts before mainnet deployment
- **Continuous Monitoring** - Watch for newly deployed vulnerable contracts

### DeFi Research
- **Pattern Analysis** - Identify common patterns in DeFi protocols
- **Protocol Comparison** - Compare security implementations across projects
- **Risk Assessment** - Evaluate smart contract risk profiles

### Bug Bounty Hunting
- **Automated Reconnaissance** - Quickly scan multiple contracts for vulnerabilities
- **Pattern Discovery** - Find recurring vulnerability patterns
- **Batch Analysis** - Scan entire protocols at once

### Development & CI/CD
- **Pre-commit Hooks** - Validate contracts before commits
- **CI/CD Integration** - Automated security checks in pipelines
- **Code Review** - Assist manual code reviews with automated findings

### Education & Training
- **Learning Tool** - Understand common smart contract vulnerabilities
- **Template Library** - Study real-world vulnerability patterns
- **Security Training** - Train developers on secure coding practices

---

## 🚀 Quick Start

### How It Works

1. **Create Templates** - Define vulnerability patterns in YAML files
2. **Load Templates** - SCPF reads your pattern definitions
3. **Fetch Contracts** - Retrieves smart contract source code from blockchain explorers
4. **Scan & Match** - Applies regex patterns to find vulnerabilities
5. **Report Results** - Displays findings with severity levels and context

### Installation

```bash
git clone https://github.com/Teycir/smartcontractpatternfinder.git
cd smartcontractpatternfinder
cargo build --release
```

### Initialize Project

```bash
scpf init
```

### Scan a Contract

```bash
# Scan single contract
scpf scan 0x1234567890abcdef --chain ethereum

# Scan with custom templates
scpf scan 0x1234567890abcdef --templates ./my-templates

# Scan multiple contracts
scpf scan 0xabc... 0xdef... 0x123... --chain bsc

# Increase verbosity
scpf scan 0x1234567890abcdef -vv
```

---

## 📋 Template Example

Templates define patterns to search for in smart contracts. SCPF loads these templates and matches them against contract source code.

```yaml
id: reentrancy-basic
name: Basic Reentrancy Pattern
description: Detects potential reentrancy vulnerabilities
severity: high
tags:
  - security
  - reentrancy
patterns:
  - id: external-call-before-state
    pattern: '\.call\{value:'
    message: External call with value transfer detected
  - id: delegatecall-usage
    pattern: '\.delegatecall\('
    message: Delegatecall usage detected
```

**What happens:**
1. SCPF loads this template from `templates/reentrancy.yaml`
2. Fetches contract source code from blockchain explorer
3. Searches for `.call{value:` and `.delegatecall(` patterns
4. Reports any matches with line numbers and context

---

## 🏗️ Architecture

```
smartcontractpatternfinder/
├── crates/
│   ├── scpf-types/     # Core types and data structures
│   ├── scpf-core/      # Scanning, fetching, caching logic
│   └── scpf-cli/       # Command-line interface
├── templates/          # Pattern detection templates
└── .amazonq/rules/     # Code quality rules
```

### Module Overview

- **scpf-types**: Core data structures (Template, Pattern, Match, ScanResult)
- **scpf-core**: Business logic (Scanner, TemplateLoader, ContractFetcher, Cache)
- **scpf-cli**: User interface (CLI commands, output formatting)

---

## 🛠️ CLI Commands

### `scpf scan`

Scan smart contracts for patterns.

```bash
scpf scan [OPTIONS] [ADDRESSES]...

Options:
  -n, --chain <CHAIN>              Chain to scan [default: ethereum]
  -t, --templates <TEMPLATES>      Templates directory
  -o, --output <OUTPUT>            Output format [default: console]
      --concurrency <CONCURRENCY>  Concurrent requests [default: 10]
  -v, --verbose                    Increase verbosity (-v, -vv, -vvv)
  -h, --help                       Print help
```

**Examples:**

```bash
# Basic scan
scpf scan 0x1234567890abcdef

# Scan on BSC
scpf scan 0x1234567890abcdef --chain bsc

# Scan with custom templates
scpf scan 0x1234567890abcdef --templates ./custom-templates

# Scan multiple contracts
scpf scan 0xabc... 0xdef... 0x123...

# High concurrency
scpf scan 0x1234567890abcdef --concurrency 20
```

### `scpf init`

Initialize a new SCPF project.

```bash
scpf init [PATH]

Options:
  -y, --yes  Skip interactive prompts
  -h, --help Print help
```

**Examples:**

```bash
# Initialize in current directory
scpf init

# Initialize in specific directory
scpf init ./my-project

# Skip prompts
scpf init --yes
```

---

## 🎯 Supported Chains

| Chain | Network | API Provider |
|-------|---------|--------------|
| **Ethereum** | Mainnet | Etherscan API |
| **BSC** | Binance Smart Chain | BscScan API |
| **Polygon** | Polygon PoS | PolygonScan API |

---

## 🔧 Configuration

Set API keys via environment variables:

```bash
export ETHERSCAN_API_KEY="your-key"
export BSCSCAN_API_KEY="your-key"
export POLYGONSCAN_API_KEY="your-key"
```

### Getting API Keys

- **Etherscan**: https://etherscan.io/apis
- **BscScan**: https://bscscan.com/apis
- **PolygonScan**: https://polygonscan.com/apis

---

## 📊 Output Formats

- **console** - Human-readable terminal output (default)
- **json** - Machine-readable JSON
- **sarif** - SARIF format for CI/CD integration

---

## 🧪 Development

```bash
# Run tests
cargo test --all

# Check code
cargo check

# Format code
cargo fmt

# Lint
cargo clippy

# Build release
cargo build --release
```

---

## 📝 Creating Templates

Templates are the core of SCPF - they define what patterns to find in smart contracts.

### How Templates Work

1. **Define Patterns** - Write regex patterns that match vulnerable code
2. **Set Severity** - Classify findings (info, low, medium, high, critical)
3. **Add Context** - Provide descriptions and messages for findings
4. **Save as YAML** - Store in `templates/` directory
5. **SCPF Loads & Scans** - Tool automatically uses your templates

### Creating a Template

1. Create a `.yaml` file in `templates/`
2. Define patterns with regex
3. Set severity level (info, low, medium, high, critical)
4. Add descriptive tags

**Template Structure:**

```yaml
id: unique-template-id
name: Human Readable Name
description: Detailed description of what this detects
severity: high  # info | low | medium | high | critical
tags:
  - category
  - subcategory
patterns:
  - id: pattern-id
    pattern: 'regex-pattern'
    message: Description of what was found
```

---

## 🤝 Contributing

Contributions welcome! Please follow:

- Amazon Q rules in `.amazonq/rules/`
- Modular architecture principles
- Test-driven development
- Clean, documented code

### Contribution Guidelines

1. Fork the repository
2. Create a feature branch
3. Follow coding standards
4. Add tests for new features
5. Submit a pull request

---

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details

---

## 👤 Author

**Teycir Ben Soltane**

- Website: [teycirbensoltane.tn](https://teycirbensoltane.tn)
- GitHub: [@Teycir](https://github.com/Teycir)

---

## 🔗 Links

- [GitHub Repository](https://github.com/Teycir/smartcontractpatternfinder)
- [Issue Tracker](https://github.com/Teycir/smartcontractpatternfinder/issues)
- [Author Website](https://teycirbensoltane.tn)

---

**Built with ❤️ using Rust by [Teycir Ben Soltane](https://teycirbensoltane.tn)**
