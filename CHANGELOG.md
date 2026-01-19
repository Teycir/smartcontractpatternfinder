# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- **Critical**: Line number calculation bug in scanner (now uses newline counting)
- Error masking in fetcher - now preserves full error context
- Non-atomic cache writes - implemented atomic write-then-rename pattern
- Template loading blocking threads - converted to async I/O
- Pattern deduplication - prevents duplicate matches from overlapping patterns

### Security
- **Regex DoS Protection** - Validates patterns for catastrophic backtracking
  - Detects nested quantifiers: `(a+)+`, `(a*)*`
  - Enforces pattern complexity limits
  - Prevents exponential backtracking attacks

### Changed
- **Type Safety**: Chain identifiers now use strongly-typed `Chain` enum instead of strings
- **Testability**: API key management refactored to `ApiKeyConfig` struct (removes env var coupling)
- Replaced MD5 with xxhash for cache hashing (performance improvement)
- Cache directory now uses system cache path via `dirs` crate
- Output format argument now uses strongly-typed enum instead of strings
- Multi-chain API key support (ETHERSCAN, BSCSCAN, POLYGONSCAN)
- Removed brittle sleep-based rate limiting (semaphore is sufficient)
- Removed unused utils modules (hash, retry) - following YAGNI principle

### Performance
- **Scanner hot path optimization** - 5-10x faster for typical contracts
  - Precomputed line index: O(N×M) → O(N + M×log L)
  - Numeric pattern indices for faster deduplication hashing
  - Binary search for line number lookup instead of repeated string scanning
  - Pre-allocated vectors to avoid repeated reallocations

### Added
- JSON output format support
- SARIF 2.1.0 output format for CI/CD integration
- Enhanced context capture for matches (handles large matches with padding)
- Regex DoS protection with pattern validation
- Integration test suite for CLI commands
- Benchmark suite with criterion (5 benchmarks)
- Comprehensive test suite (29 tests, 100% passing)
  - Cache atomic write tests
  - Scanner line number accuracy tests
  - Invalid regex handling tests
  - Template deserialization tests
  - Fetcher validation tests
  - Pattern deduplication tests
  - Large match context tests
  - Regex DoS protection tests (5 tests)
  - CLI integration tests (6 tests)
- Scan timing metrics in results
- `Serialize` derives for `Match` and `ScanResult` types
- `OutputFormat` enum with `ValueEnum` for type-safe CLI args
- `Chain` enum for type-safe chain identifiers
- `ApiKeyConfig` struct for testable API key management

### Dependencies
- Added `xxhash-rust` for fast cache hashing
- Added `dirs` for system directory paths
- Added `serde_json` to CLI for JSON output
- Added `serde_yaml` to types dev-dependencies for tests
- Added `criterion` for benchmarking

### Documentation
- Added CODE_IMPROVEMENTS.md with detailed fix documentation
- Added IMPROVEMENT_VALIDATION.md assessing future enhancement suggestions
- Added ROADMAP.md with prioritized improvement plan
- Updated template example (fixed pattern ID naming)

## [0.1.0] - 2025-01-19

### Added
- Initial release of Smart Contract Pattern Finder (SCPF)
- Multi-chain support (Ethereum, BSC, Polygon)
- YAML-based template system for pattern definitions
- Regex-based pattern matching engine
- Smart caching system to avoid redundant API calls
- CLI with `scan` and `init` commands
- Contract fetcher with Etherscan API integration
- Modular architecture with 3 crates (scpf-types, scpf-core, scpf-cli)
- Example reentrancy detection template
- Amazon Q code quality rules
- Comprehensive documentation

### Features
- 🌐 Multi-chain support (Ethereum, BSC, Polygon)
- 📝 YAML templates for easy pattern definitions
- ⚡ Fast regex-based scanning
- 💾 Smart caching system
- 🎯 Modular architecture
- 🔒 Security-focused vulnerability detection
- 🚀 High performance with Rust
- 🔧 Extensible template system

### Documentation
- Comprehensive README with table of contents
- Use cases for security auditing, DeFi research, bug bounty hunting
- CLI command examples
- Template creation guide
- Architecture overview
- Attribution to Teycir Ben Soltane

[Unreleased]: https://github.com/Teycir/smartcontractpatternfinder/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Teycir/smartcontractpatternfinder/releases/tag/v0.1.0
