# Export Messages Feature

## Overview

Added clear console messages when data is automatically exported at the end of scans.

## Messages Added

### 1. Vulnerability Summary Export
```
📤 Exported vulnerability summary to: /path/to/vuln_summary.md
   5 exploitable contracts, 23 total findings
```

### 2. Exploitable Contracts Export
```
📄 Extracting top 10 exploitable contracts...
   ✅ [1] 0x123... - Weighted Risk: 85.3 (45 KB, 1234 lines)
   ✅ [2] 0x456... - Weighted Risk: 72.1 (32 KB, 890 lines)
   ...
   📤 Exported 10 exploitable contracts
```

### 3. Risk-Based Contracts Export
```
📄 Extracting top 10 contracts by risk score...
   ✅ [1] 0x789... - Weighted Risk: 65.2 (28 KB, 756 lines)
   ...
   📤 Exported 10 contracts by risk score
```

### 4. Contract Sources Export
```
📁 Extracting top 20 riskiest contract sources...
   [1/20] 0x123... (ethereum) - Risk: 85.3
   [2/20] 0x456... (polygon) - Risk: 72.1
   ...
   ✅ Extracted 20 contract sources (2.3 MB total) to /path/to/sources
   📤 Contract sources exported successfully
```

### 5. Final Summary
```
✅ Scan complete! All data exported to: /path/to/report_1234567890
```

## Example Output

```bash
$ scpf scan --days 7 --extract-sources 20

🔍 Scanning contracts updated in last 7 days...
   Severity filter: high and above

📡 Fetching from ethereum...
   ✓ Found 150 contracts

⏳ Loading templates...
⏳ Scanning 150 contracts...

   [1/150] (0.7%) Scanning 0x123... (ethereum)...
   ...
   [150/150] (100.0%) Scanning 0xabc... (ethereum)...

✅ Scanning complete

📄 Incremental results saved to: /path/to/incremental_results.jsonl

🌳 Exploitable Contracts:

📈 Summary:
   🚨 Exploitable: 5 contracts with 23 findings
   ❌ False Positives: 12 findings
   ⚠️  Needs Review: 8 findings
   📊 Total: 43 findings across 15 contracts

📤 Exported vulnerability summary to: /path/to/vuln_summary.md
   5 exploitable contracts, 43 total findings

📄 Extracting top 10 exploitable contracts...
   ✅ [1] 0x123... - Weighted Risk: 85.3 (45 KB, 1234 lines)
   ✅ [2] 0x456... - Weighted Risk: 72.1 (32 KB, 890 lines)
   ...
   📤 Exported 10 exploitable contracts

📁 Extracting top 20 riskiest contract sources...
   [1/20] 0x123... (ethereum) - Risk: 85.3
   [2/20] 0x456... (polygon) - Risk: 72.1
   ...
   ✅ Extracted 20 contract sources (2.3 MB total) to /path/to/sources
   📤 Contract sources exported successfully

✅ Scan complete! All data exported to: /path/to/report_1234567890
```

## Benefits

1. **Clear visibility**: Users know exactly what was exported
2. **Confirmation**: No ambiguity about whether export succeeded
3. **Location tracking**: Easy to find exported files
4. **Progress feedback**: Shows what's happening during export
5. **Summary stats**: Quick overview of findings

## Implementation

### Changes Made

**File**: `crates/scpf-cli/src/commands/scan.rs`

```rust
// After writing vulnerability summary
eprintln!("\n📤 Exported vulnerability summary to: {}", vuln_summary.display());
eprintln!("   {} exploitable contracts, {} total findings", ...);

// After extracting exploitable contracts
eprintln!("   📤 Exported {} exploitable contracts", count);

// After extracting by risk score
eprintln!("   📤 Exported {} contracts by risk score", count);

// After extracting sources
eprintln!("   📤 Contract sources exported successfully");

// Final message
eprintln!("\n✅ Scan complete! All data exported to: {}", root_dir.display());
```

## Message Format

All export messages follow this pattern:
- 📤 emoji for export actions
- ✅ emoji for successful completion
- Clear action verb (Exported, Extracted)
- Count of items exported
- File path or location

## Testing

```bash
# Run a scan and observe export messages
cargo run --release -p scpf-cli -- scan --days 1 --extract-sources 5

# Expected output includes:
# - "📤 Exported vulnerability summary to: ..."
# - "📤 Exported N exploitable contracts"
# - "📤 Contract sources exported successfully"
# - "✅ Scan complete! All data exported to: ..."
```

## Related Files

- `crates/scpf-cli/src/commands/scan.rs` - Export message implementation
- `docs/INCREMENTAL_RESULTS.md` - Related incremental results feature
- `docs/IMPROVEMENTS_SUMMARY.md` - Overall improvements summary
