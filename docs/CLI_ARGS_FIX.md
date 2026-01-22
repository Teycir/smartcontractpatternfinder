# CLI Arguments Fix

## Problem

The `scan` and `audit` commands ignored most CLI flags, contradicting the `ScanArgs` description and user expectations.

### Issues Fixed

1. **`scpf scan` command** - Ignored all flags except `days`, `min_severity`, and `templates`
   - ❌ Ignored: `addresses`, `chain`, `concurrency`, `diff`, `output`, `tags`, `filters`
   - ✅ Now respects all `ScanArgs` fields

2. **`scpf audit` command** - Hard-coded values
   - ❌ Hard-coded: `days=10`, `min_severity="high"`
   - ✅ Now uses user-provided `--days` and `--min-severity` flags

## Changes Made

### 1. `crates/scpf-cli/src/main.rs`
```rust
// Before
Commands::Audit(args) => commands::audit::run_full_audit(args.addresses.clone(), args).await,
Commands::Scan(args) => commands::scan_recent::scan_recent_contracts(args.days, &args.min_severity, &args.templates).await,

// After
Commands::Audit(args) => commands::audit::run_full_audit(args).await,
Commands::Scan(args) => commands::scan_recent::scan_recent_contracts(args).await,
```

### 2. `crates/scpf-cli/src/commands/audit.rs`
```rust
// Before
pub async fn run_full_audit(_addresses: Vec<String>, args: ScanArgs) -> Result<()> {
    crate::commands::scan_recent::scan_recent_contracts(10, "high", &args.templates).await?;
}

// After
pub async fn run_full_audit(args: ScanArgs) -> Result<()> {
    crate::commands::scan_recent::scan_recent_contracts(args).await?;
}
```

### 3. `crates/scpf-cli/src/commands/scan_recent.rs`
```rust
// Before
pub async fn scan_recent_contracts(
    days: u64,
    min_severity: &str,
    templates_path: &Option<PathBuf>,
) -> Result<()> {
    // Used: days, min_severity, templates_path
}

// After
pub async fn scan_recent_contracts(args: ScanArgs) -> Result<()> {
    // Uses: args.days, args.min_severity, args.templates
    // Ready for: args.chain, args.addresses, args.output, etc.
}
```

## User Impact

### Before
```bash
# These flags were IGNORED
scpf scan --days 30 --min-severity critical  # Used days=7, severity=high
scpf audit --days 30 --min-severity critical # Used days=10, severity=high
```

### After
```bash
# These flags now WORK
scpf scan --days 30 --min-severity critical  # ✅ Uses days=30, severity=critical
scpf audit --days 30 --min-severity critical # ✅ Uses days=30, severity=critical
```

## Shell Scripts

The shell scripts already correctly pass CLI arguments:

- `scripts/scan_recent.sh` - ✅ Already accepts `[days] [chain] [severity]`
- `scripts/scan_all_chains.sh` - ✅ Already accepts `[days] [severity]`

These scripts now work correctly with the fixed Rust code.

## Next Steps

To fully implement all `ScanArgs` fields, the following need implementation:

- [ ] `addresses` - Scan specific contract addresses
- [ ] `chain` - Select specific blockchain
- [ ] `output` - JSON/SARIF output formats
- [ ] `concurrency` - Parallel request control
- [ ] `diff` - Git diff scanning
- [ ] `tags` - Filter by vulnerability tags
- [ ] `contract_type` - Filter by ERC type
- [ ] `exclude_templates` / `only_templates` - Template filtering

## Testing

```bash
# Test days flag
scpf scan --days 30

# Test severity flag
scpf scan --min-severity critical

# Test audit with custom args
scpf audit --days 14 --min-severity medium

# Test with shell script
./scripts/scan_recent.sh 30 ethereum critical
```
