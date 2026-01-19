# Smart Contract Pattern Finder (SCPF)

🔍 High-performance tool for detecting security vulnerabilities and patterns in smart contracts across multiple blockchains.

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)

**Tags:** `rust` `smart-contracts` `security` `scanner` `ethereum` `blockchain` `vulnerability-detection` `pattern-matching` `defi` `web3` `solidity` `static-analysis`

---

## 📑 Table of Contents

- [Features](#-features)
- [Use Cases](#-use-cases)
- [Quick Start](#-quick-start)
- [Template Example](#-template-example)
- [Architecture](#️-architecture)
- [CLI Commands](#️-cli-commands)
- [Supported Chains](#-supported-chains)
- [Configuration](#-configuration)
- [Output Formats](#-output-formats)
- [Development](#-development)
- [Creating Templates](#-creating-templates)
- [Contributing](#-contributing)
- [License](#-license)
- [Author](#-author)
- [Links](#-links)

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

Templates are YAML files defining patterns to detect:

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
