# GitHub Marketplace Release Draft

## Suggested release

- **Tag:** `v1.0.0`
- **Release title:** `SCPF Security Scanner v1.0.0`
- **Marketplace major tag to move after publish:** `v1`

## Release body

```md
## SCPF Security Scanner v1.0.0

Initial GitHub Marketplace release of the SCPF Security Scanner Action.

SCPF scans Solidity projects for smart contract security patterns and publishes SARIF results to GitHub code scanning, so findings appear directly in the repository Security tab.

### What this release includes

- Composite GitHub Action with Marketplace-compatible metadata and branding
- SARIF upload support via `github/codeql-action/upload-sarif`
- Configurable inputs for severity threshold, templates path, output format, and fail-on-findings behavior
- Cached installation flow for the `scpf` CLI in GitHub Actions
- MIT licensing across the repository and Action metadata
- CI cleanup for `check`, `test`, `fmt`, `clippy`, and release builds

### Usage

```yaml
name: Smart Contract Security Scan

on:
  pull_request:
    paths:
      - '**.sol'

jobs:
  security-scan:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      security-events: write

    steps:
      - uses: actions/checkout@v4
      - uses: Teycir/smartcontractpatternfinder@v1
        with:
          severity: high
          output-format: sarif
          fail-on-findings: true
```

### Inputs

- `severity`: Minimum severity to fail the workflow. Default: `high`
- `templates`: Path to custom template directory. Default: `templates`
- `output-format`: One of `console`, `json`, or `sarif`. Default: `sarif`
- `fail-on-findings`: Whether findings should fail the job. Default: `true`

### Outputs

- `findings-count`: Number of findings detected
- `sarif-file`: Path to the generated SARIF file

### Validation

The repository was validated with:

- `cargo check --all-features`
- `cargo test --all-features`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo build --release --all-features`
- `npm run build` in `frontend/`

### Notes

- This release establishes the stable major tag `v1` for GitHub Actions consumers.
- If you publish from this release, move or create the `v1` tag to point at the same commit as `v1.0.0`.
- The separate accuracy benchmark workflow currently runs successfully but reports a low F1 score; that is a quality issue, not a release blocker for the Action.
```

## Publish checklist

- Create annotated tag `v1.0.0`
- Create or move major tag `v1` to the same commit
- Publish a GitHub Release using the body above
- Verify the repository is marked as a public action repository
- Verify the Marketplace listing picks up `action.yml` metadata
- Test `uses: Teycir/smartcontractpatternfinder@v1` from a separate repository
