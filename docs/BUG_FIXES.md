# Bug Fixes Applied

## Critical Errors Fixed

### 1. ✅ Severity Filter Logic Error (Line 202)
**Problem**: Hardcoded `Severity::High` check overrode user's `min_severity` setting.
```rust
// BEFORE (WRONG)
.filter(|m| m.severity >= min_severity && m.severity >= Severity::High)

// AFTER (FIXED)
.filter(|m| m.severity >= min_severity)
```

### 2. ✅ Runtime Panic in spawn_blocking (Lines 174-180)
**Problem**: `tokio::runtime::Handle::current()` panics in blocking context.
```rust
// BEFORE (WRONG)
tokio::task::spawn_blocking(move || {
    let rt = tokio::runtime::Handle::current();  // PANIC!
    rt.block_on(async { ... })
})

// AFTER (FIXED)
async {
    scanner_clone.lock().await.scan(...)
}
```

### 3. ✅ Panic Instead of Error Handling (Line 322)
**Problem**: `panic!` crashes entire program on invalid severity.
```rust
// BEFORE (WRONG)
_ => panic!("Invalid severity: {}", s),

// AFTER (FIXED)
_ => Err(anyhow::anyhow!("Invalid severity: '{}'...", s)),
```
**Bonus**: Now supports all severity levels (info, low, medium, high, critical).

### 4. ✅ Hardcoded User Path (Lines 64, 432)
**Problem**: `/home/teycir/...` not portable across systems.
```rust
// BEFORE (WRONG)
format!("/home/teycir/smartcontractpatternfinderReports/report_{}", timestamp)

// AFTER (FIXED)
dirs::home_dir()
    .map(|h| h.join("smartcontractpatternfinderReports")...)
    .unwrap_or_else(|| format!("./reports/report_{}", timestamp))
```

### 5. ✅ Index Out of Bounds (Multiple locations)
**Problem**: `&address[..12]` panics if address < 12 chars.
```rust
// BEFORE (WRONG)
&r.address[..12]

// AFTER (FIXED)
&r.address[..r.address.len().min(12)]
```
**Fixed in**: 5 locations (rank_and_score, 2x extraction loops, sources loop).

### 6. ✅ Duplicate Timestamp Calculation
**Problem**: Two different timestamps caused path mismatch.
```rust
// BEFORE (WRONG)
// Line 59: let timestamp = now()...
// Line 382: let timestamp = now()...  // Different value!

// AFTER (FIXED)
// Calculate once in scan_vulnerabilities, pass to scan_contracts
```

### 7. ✅ Panic on NaN Comparison (Multiple locations)
**Problem**: `partial_cmp().unwrap()` panics on NaN values.
```rust
// BEFORE (WRONG)
.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap())

// AFTER (FIXED)
.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal))
```
**Fixed in**: 4 locations (rank_and_score, with_poc_scores, 2x sorted_exploitable).

### 8. ✅ Inefficient Count Operations
**Problem**: Unnecessary iteration when count is known.
```rust
// BEFORE (WRONG)
sorted_exploitable.iter().take(top_n).count()

// AFTER (FIXED)
sorted_exploitable.len().min(top_n)
```
**Fixed in**: 2 locations.

### 9. ✅ Silent Error Suppression
**Problem**: Write errors ignored with `let _`.
```rust
// BEFORE (WRONG)
let _ = writeln!(file, "{}", result_json);
let _ = file.flush();

// AFTER (FIXED)
if let Err(e) = writeln!(file, "{}", result_json) {
    tracing::warn!("Failed to write incremental result: {}", e);
}
```

## Summary

| Error Type | Severity | Count | Status |
|------------|----------|-------|--------|
| Logic Errors | Critical | 2 | ✅ Fixed |
| Runtime Panics | Critical | 6 | ✅ Fixed |
| Portability Issues | High | 1 | ✅ Fixed |
| Performance Issues | Low | 2 | ✅ Fixed |
| Error Handling | Medium | 1 | ✅ Fixed |

**Total Bugs Fixed**: 12

## Testing Checklist

- [ ] Test with severity filter: `--min-severity critical`
- [ ] Test with severity filter: `--min-severity low`
- [ ] Test with short addresses (< 12 chars)
- [ ] Test on non-teycir user account
- [ ] Test with NaN risk scores
- [ ] Test incremental file write errors
- [ ] Test scan timeout on large files
- [ ] Test with invalid severity input

## Files Modified

- `crates/scpf-cli/src/commands/scan.rs` - All fixes applied

## Breaking Changes

None. All changes are backward compatible.

## New Features

- Support for all severity levels (info, low, medium, high, critical)
- Better error messages for invalid severity
- Portable home directory detection
- Error logging for incremental writes
