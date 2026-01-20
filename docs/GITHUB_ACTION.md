# SCPF GitHub Action

Automatically scan your smart contracts for security vulnerabilities in CI/CD pipelines.

## Quick Start

Add this to `.github/workflows/security.yml`:

```yaml
name: Security Scan

on: [push, pull_request]

jobs:
  scpf-scan:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      security-events: write
      
    steps:
      - uses: actions/checkout@v4
      - uses: teycir/smartcontractpatternfinder@v1
```

## Features

✅ **Zero Configuration** - Works out of the box  
✅ **SARIF Integration** - Results appear in GitHub Security tab  
✅ **Fast** - Cached installation, parallel scanning  
✅ **Customizable** - Control severity thresholds and templates  

## Inputs

| Input | Description | Default |
|-------|-------------|---------|
| `severity` | Minimum severity to fail build | `high` |
| `templates` | Path to custom templates | `templates` |
| `output-format` | Output format (console, json, sarif) | `sarif` |
| `fail-on-findings` | Fail build on vulnerabilities | `true` |

## Examples

### Basic Usage

```yaml
- uses: teycir/smartcontractpatternfinder@v1
```

### Custom Severity

```yaml
- uses: teycir/smartcontractpatternfinder@v1
  with:
    severity: medium
    fail-on-findings: true
```

### Custom Templates

```yaml
- uses: teycir/smartcontractpatternfinder@v1
  with:
    templates: ./my-templates
```

### Scan Only Changed Files (PR)

```yaml
on:
  pull_request:
    paths:
      - '**.sol'

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: teycir/smartcontractpatternfinder@v1
```

### Upload Results as Artifacts

```yaml
- uses: teycir/smartcontractpatternfinder@v1
  id: scpf
  
- uses: actions/upload-artifact@v3
  if: always()
  with:
    name: security-results
    path: scpf-results.sarif
```

## Outputs

| Output | Description |
|--------|-------------|
| `findings-count` | Number of vulnerabilities found |
| `sarif-file` | Path to SARIF output file |

## Local Testing

Test the action locally before pushing:

```bash
# Install SCPF
cargo install --git https://github.com/Teycir/smartcontractpatternfinder scpf-cli

# Scan your project
scpf scan

# Scan only changed files
scpf scan --diff main..HEAD
```

## CI/CD Integration

### GitHub Actions ✅
See examples above

### GitLab CI

```yaml
scpf-scan:
  image: rust:latest
  script:
    - cargo install --git https://github.com/Teycir/smartcontractpatternfinder scpf-cli
    - scpf scan --output json > results.json
  artifacts:
    reports:
      sast: results.json
```

### CircleCI

```yaml
version: 2.1
jobs:
  security-scan:
    docker:
      - image: rust:latest
    steps:
      - checkout
      - run: cargo install --git https://github.com/Teycir/smartcontractpatternfinder scpf-cli
      - run: scpf scan
```

## Troubleshooting

### No templates found
Ensure `templates/` directory exists with `.yaml` files. Run `scpf init` to create default templates.

### Permission denied
Add `permissions: { security-events: write }` to your workflow.

### Scan takes too long
Use `--diff` flag to scan only changed files in PRs.

## Support

- 📖 [Documentation](https://github.com/Teycir/smartcontractpatternfinder)
- 🐛 [Report Issues](https://github.com/Teycir/smartcontractpatternfinder/issues)
- 💬 [Discussions](https://github.com/Teycir/smartcontractpatternfinder/discussions)

## License

MIT - See [LICENSE](LICENSE)
