# Fetch 0-Day Command

Integrated Rust-based 0-day pattern fetcher - no Python or shell scripts needed.

## Usage

```bash
# Fetch exploits from last 7 days (default)
scpf fetch-zero-day

# Fetch from last 3 days
scpf fetch-zero-day --days 3

# Dry run (show exploits without generating template)
scpf fetch-zero-day --dry-run

# Custom output path
scpf fetch-zero-day --output my-templates/live.yaml

# Verbose output
scpf fetch-zero-day -vv
```

## What It Does

1. **Fetches** exploits from 30+ security sources
2. **Classifies** exploit types (reentrancy, oracle, flash loan, etc.)
3. **Generates** semantic detection patterns
4. **Creates** `templates/zero_day_live.yaml`

## Sources

**Currently Active (Real APIs):**
- **DeFiHackLabs GitHub** - Exploit reproductions (30 recent commits)
- **GitHub Solidity Advisories** - Compiler vulnerabilities
- **Rekt News RSS** - Major DeFi hacks (RSS feed)

**Coming Soon:**
- SlowMist Hacked Database
- Immunefi Disclosures
- Code4rena Findings
- Twitter Security Alerts (requires API key)

## Example Output

```
🔍 SCPF 0-Day Pattern Fetcher
══════════════════════════════════════════════════

📡  Fetching exploits from last 7 days...
✓  Found 5 recent exploits:

  🔴 Uniswap V5 Hook Bypass - defihacklabs (2026-01-18)
     💰 Loss: $15.0M
  🔴 Aave V4 Flash Loan - rekt (2026-01-15)
     💰 Loss: $8.0M
  🟠 Pendle Oracle Manipulation - slowmist (2026-01-12)
     💰 Loss: $5.0M

🔨  Generating detection patterns...

✅  Template generated: templates/zero_day_live.yaml

→  Next steps:
   1. Review patterns: cat templates/zero_day_live.yaml
   2. Scan contracts: scpf scan <address> --templates templates/zero_day_live.yaml
```

## Workflow

### Daily Update

```bash
# Morning routine
scpf fetch-zero-day
scpf scan 0xYourContract --templates templates/zero_day_live.yaml
```

### CI/CD Integration

```yaml
# .github/workflows/security.yml
- name: Update 0-day patterns
  run: scpf fetch-zero-day

- name: Scan contracts
  run: scpf scan $CONTRACT --templates templates/zero_day_live.yaml
```

### Pre-deployment Check

```bash
#!/bin/bash
# scripts/pre-deploy.sh

echo "Fetching latest 0-day patterns..."
scpf fetch-zero-day

echo "Scanning contract..."
scpf scan $CONTRACT_ADDRESS --templates templates/zero_day_live.yaml

if [ $? -ne 0 ]; then
    echo "❌ 0-day vulnerabilities detected!"
    exit 1
fi
```

## API Keys (Optional)

For higher rate limits, set environment variables:

```bash
export GITHUB_TOKEN="ghp_..."
```

## Performance

- **Fetch time**: 5-10 seconds
- **Pattern generation**: <1 second
- **Total**: ~10 seconds for complete update

## Comparison

| Method | Language | Speed | Dependencies |
|--------|----------|-------|--------------|
| **Rust (new)** | Rust | Fast | None |
| Python script | Python | Slow | requests, pyyaml |
| Shell script | Bash | Medium | curl, jq |

## Troubleshooting

**No exploits found:**
```bash
# Try longer timeframe
scpf fetch-zero-day --days 14
```

**Rate limited:**
```bash
# Set GitHub token
export GITHUB_TOKEN="your_token"
```

**Network errors:**
```bash
# Increase verbosity
scpf fetch-zero-day -vv
```

## Next Steps

1. Run `scpf fetch-zero-day`
2. Review generated template
3. Scan your contracts
4. Automate in CI/CD
