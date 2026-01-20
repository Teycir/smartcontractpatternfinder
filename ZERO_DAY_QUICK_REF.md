# 0-Day Detection Quick Reference

## 🎯 Scan for Latest Exploits

```bash
# Scan with cutting-edge patterns
scpf scan 0xYourContract --templates ./templates/zero_day_emerging.yaml

# Full deep scan (all 81 patterns)
scpf scan 0xYourContract --templates ./templates

# Check for updates
./scripts/update_templates.sh check
```

## 🔥 Top 10 Recent Exploits Detected

1. **Read-only reentrancy** - Curve Finance ($60M+)
2. **ERC4626 inflation** - Multiple vaults ($5M+)
3. **Vyper lock bypass** - Curve pools ($70M+)
4. **Balancer composable** - Balancer V2 ($2M)
5. **Permit front-running** - Various protocols ($2M+)
6. **Cross-contract reentrancy** - Uniswap V3 style
7. **Arbitrum sequencer** - L2 oracle issues
8. **Uniswap V4 hooks** - Hook manipulation
9. **Eigenlayer restaking** - Slashing risks
10. **Blast yield** - Yield manipulation

## 📊 Template Coverage

| Template | Patterns | Focus | Updated |
|----------|----------|-------|---------|
| `zero_day_emerging.yaml` | 20 | Latest exploits | Weekly |
| `advanced_audit.yaml` | 21 | Core security | Quarterly |
| `defi_vulnerabilities.yaml` | 20 | DeFi protocols | Monthly |
| `logic_bugs_gas.yaml` | 20 | Code quality | Quarterly |
| **Total** | **81** | **Complete** | **Ongoing** |

## 🚨 Emergency Response

When a major hack happens:

```bash
# 1. Update immediately
./scripts/update_templates.sh force

# 2. Scan your contracts
scpf scan 0xYourContract --templates ./templates/zero_day_emerging.yaml

# 3. Check critical findings
scpf scan 0xYourContract --output json | jq '.[] | select(.severity == "critical")'
```

## 📡 Stay Updated

- **GitHub Releases**: https://github.com/Teycir/smartcontractpatternfinder/releases
- **Update Script**: `./scripts/update_templates.sh latest`
- **Automated**: GitHub Actions runs weekly
- **Manual**: `git pull origin main`

## 🎓 Learn More

- [Zero-Day Updates Guide](./docs/ZERO_DAY_UPDATES.md)
- [Deep Audit Guide](./docs/DEEP_AUDIT.md)
- [Template Changelog](./TEMPLATE_CHANGELOG.md)
