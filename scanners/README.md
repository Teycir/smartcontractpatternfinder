# Scanner Configurations

Pre-configured scan profiles for different use cases.

## Template Update Feature

All scanners support `--update-templates <N>` parameter:
- `--update-templates 0`: No update (use existing templates)
- `--update-templates 1`: Update with exploits from last 24 hours
- `--update-templates 3`: Update with exploits from last 3 days
- `--update-templates 7`: Update with exploits from last 7 days

## Available Scanners

### 🚨 critical-only.sh
**Critical vulnerabilities only - Last 7 days - All chains - No template update**
```bash
./scanners/critical-only.sh
```
Use for: Emergency response, zero-day detection

### ⚠️ high-critical.sh
**High to Critical - Last 30 days - All chains - No template update**
```bash
./scanners/high-critical.sh
```
Use for: Regular security monitoring (default)

### 📊 medium-plus.sh
**Medium and above - Last 60 days - All chains - No template update**
```bash
./scanners/medium-plus.sh
```
Use for: Comprehensive security review

### 🔍 full-audit.sh
**All severities - Last 90 days - All chains - Update templates (7 days)**
```bash
./scanners/full-audit.sh
```
Use for: Complete security audit with latest patterns (console output)

### 📋 full-audit-with-reports.sh
**All severities - Last 90 days - All chains - Update templates (7 days) - Saves reports**
```bash
./scanners/full-audit-with-reports.sh
```
Use for: Complete security audit with saved reports (console, JSON, SARIF)
Reports saved to:
- Linux/macOS: `~/smartcontractpatternfinderReports/`
- Windows: `%USERPROFILE%/smartcontractpatternfinderReports/`

### ⚡ quick-scan.sh
**Critical only - Last 24 hours - Ethereum only - Update templates (1 day)**
```bash
./scanners/quick-scan.sh
```
Use for: Fast daily checks with fresh patterns

### 💰 defi-focused.sh
**High+ - Last 14 days - Ethereum, BSC, Polygon - Update templates (3 days)**
```bash
./scanners/defi-focused.sh
```
Use for: DeFi protocol monitoring, outputs JSON files

### 🌉 l2-chains.sh
**High+ - Last 30 days - Arbitrum, Optimism, Base - No template update**
```bash
./scanners/l2-chains.sh
```
Use for: Layer 2 ecosystem monitoring

### 🤖 ci-cd.sh
**High+ - Last 7 days - SARIF output - Update templates (1 day)**
```bash
./scanners/ci-cd.sh
```
Use for: CI/CD pipeline integration with latest patterns

### 🔴 zeroday-watch.sh
**Critical only - Last 3 days - All chains - Update templates (3 days)**
```bash
./scanners/zeroday-watch.sh
```
Use for: Active zero-day monitoring with real-time pattern updates

## Quick Reference

| Scanner | Days | Chains | Severity | Template Update | Output |
|---------|------|--------|----------|-----------------|--------|
| critical-only | 7 | All | Critical | 0 (none) | Console |
| high-critical | 30 | All | High+ | 0 (none) | Console |
| medium-plus | 60 | All | Medium+ | 0 (none) | Console |
| full-audit | 90 | All | All | 7 days | Console |
| quick-scan | 1 | Ethereum | Critical | 1 day | Console |
| defi-focused | 14 | ETH/BSC/Polygon | High+ | 3 days | JSON |
| l2-chains | 30 | ARB/OP/Base | High+ | 0 (none) | Console |
| ci-cd | 7 | All | High+ | 1 day | SARIF |
| zeroday-watch | 3 | All | Critical | 3 days | Console |

## Custom Configuration

Create your own scanner:
```bash
#!/bin/bash
cargo run --release -- scan \
  --days <N> \
  --chain <CHAIN> \
  --min-severity <LEVEL> \
  --update-templates <DAYS> \
  --output <FORMAT>
```

Parameters:
- `--days`: 1-365
- `--chain`: ethereum, bsc, polygon, arbitrum, optimism, base
- `--all-chains`: Scan all chains
- `--min-severity`: info, low, medium, high, critical
- `--update-templates`: 0 (no update), 1-30 (days of exploits to fetch)
- `--output`: console, json, sarif
