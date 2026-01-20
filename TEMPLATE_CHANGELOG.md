# Template Changelog

Track updates to vulnerability detection templates.

## [2026-01-20] Real-Time 0-Day Detection

### Added - Live Pattern Fetching
- `scripts/fetch_0day_patterns.py` - Auto-fetch from security feeds
- `templates/zero_day_live.yaml` - Auto-generated patterns
- GitHub Actions runs 3x daily (00:00, 08:00, 16:00 UTC)
- Average detection: 24-48 hours from disclosure

### Security Feed Integration
- Rekt News API
- BlockSec Alerts
- PeckShield Alerts
- Immunefi Disclosures

### Pattern Auto-Generation
- Reentrancy variants
- Oracle manipulation
- Access control bypasses
- Flash loan exploits

## [2026-01-20] Zero-Day Template Release

### Added - `zero_day_emerging.yaml`
20 cutting-edge patterns from recent exploits:

**Critical Vulnerabilities:**
- Read-only reentrancy (Curve Finance, Jul 2023)
- ERC4626 inflation attack (2024)
- Permit front-running (2023-2024)
- Cross-contract reentrancy (Uniswap V3 style)
- ERC777 reentrancy hooks
- Vyper reentrancy lock bypass (Jul 2023)

**L2 & Emerging Chains:**
- Arbitrum sequencer downtime check
- Blast yield manipulation (2024)
- EIP-4844 blob data validation

**Protocol-Specific:**
- Uniswap V4 hook manipulation
- Eigenlayer restaking risks
- Pendle PT/YT manipulation
- Balancer composable pool reentrancy
- Aave V3 e-mode manipulation
- Compound V3 absorb exploit
- GMX V2 price impact manipulation
- Morpho optimizer rate manipulation
- Frax V3 AMO exploit

**Account Abstraction:**
- ERC-4337 validation bypass
- Multicall reentrancy (2024)

### Infrastructure
- Added `scripts/update_templates.sh` for automated updates
- Added `.github/workflows/update_templates.yml` for CI/CD
- Created `docs/ZERO_DAY_UPDATES.md` documentation

## [2025-01-20] Deep Audit Templates

### Added - `advanced_audit.yaml`
21 patterns for sophisticated vulnerabilities

### Added - `defi_vulnerabilities.yaml`
20 patterns for DeFi protocols

### Added - `logic_bugs_gas.yaml`
20 patterns for code quality

## Update Schedule

- **Zero-day**: Weekly (Mondays)
- **DeFi**: Monthly (1st of month)
- **Advanced/Logic**: Quarterly

## Sources

- Rekt News
- BlockSec Alerts
- PeckShield Alerts
- Certora Research
- Immunefi Disclosures
- Code4rena Findings
- Trail of Bits Research
