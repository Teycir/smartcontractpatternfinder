# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

No unreleased changes.

## [v0.2.0] - 2026-04-22

### Added
- **Desktop Application**
  - Full Tauri-based desktop app with native packaging (.deb, .rpm, .AppImage)
  - Dedicated local backend server on port 32145
  - Desktop launcher and icon integration
  - Automatic .env.example seeding on first launch
- **scpf-config Crate**
  - Centralized configuration management
  - Runtime environment detection and .env file discovery
  - API key loading with cascade fallback support
  - Server configuration for both web and desktop modes
- **Enhanced Frontend**
  - Real-time SSE log streaming
  - Progress tracking with ETA and rate display
  - Template selector with multi-select support
  - Export functionality for reports and logs
  - Connection state management with auto-reconnect
  - Responsive UI with improved styling

### Changed
- **Modular Architecture**
  - Extracted configuration logic into scpf-config crate
  - Moved API key management from CLI to shared config
  - Unified .env file loading across CLI, server, and desktop
- **Server Improvements**
  - Fixed report path parser for 0-day-only flow
  - Enhanced progress tracking with atomic operations
  - Better error handling and logging
  - Support for both web (8080) and desktop (32145) modes

### Fixed
- **Critical**: Export endpoint returning "No report available" for 0-day reports
  - Parser now correctly extracts full absolute path from log lines
  - Handles both "Vulnerability summary:" and "0-Day summary:" patterns
- **Critical**: .env file loading mechanism
  - Searches multiple locations (project root, config dirs, data dirs)
  - Supports explicit path via SCPF_ENV_FILE
  - Desktop app uses ~/.local/share/com.teycir.scpf.desktop/.env
- Report path extraction from SSE logs
- Progress tracking for contract scanning and extraction
- Desktop app server port conflicts

### Performance
- Atomic operations for progress tracking (reduced lock contention)
- Efficient SSE streaming with keep-alive
- Optimized frontend rendering with React hooks

### Status
- **Desktop Ready**: Full desktop application with native packaging
- **Production Ready**: Stable server and frontend with comprehensive error handling
- **Modular**: Clean separation of concerns with scpf-config crate

## [v1.0.0] - 2026-03-05

### Added
- Initial GitHub Marketplace release draft in `docs/ACTION_MARKETPLACE_RELEASE_DRAFT.md`
- Marketplace-ready release metadata for the composite GitHub Action in `action.yml`
- Stable action version tags `v1.0.0` and `v1` for GitHub Actions consumers

### Changed
- Standardized the repository on the MIT license across the root license file, README, Cargo workspace metadata, and frontend package metadata
- Updated README badges, tags, and license text to match Marketplace publication requirements
- Upgraded frontend `axios` from `1.13.3` to `1.13.6`

### Fixed
- Resolved workspace Rust formatting issues so `cargo fmt --all -- --check` passes
- Resolved Clippy failures across crates, benches, tests, and workflow targets
- Verified local CI-equivalent commands for `check`, `test`, `clippy`, release builds, and frontend production builds

## [v1.4] - 2025-01-25

### Added
- **Filtered Findings System**
  - Added `filtered` field to Match struct
  - Filtered findings excluded from risk scoring (return 0)
  - Findings marked as filtered instead of removed
  - Preserves all findings for transparency
- **Aggressive False Positive Filtering**
  - Timestamp pattern filter (block.timestamp, block.number)
  - OpenZeppelin Address library filter (sendValue, functionCall)
  - Standard proxy pattern filter (Proxy, ERC1967, BeaconProxy)
  - Safe NFT pattern filter (ERC721/ERC1155 standard functions)
  - 4 new regex filters compiled at initialization
- **API Key Fallback Fix**
  - Fixed broken fallback mechanism (was retrying same key 3x)
  - Now immediately tries next key on any error
  - Removed retry wrapper blocking fallback
  - Simplified error handling: any error → try next key
  - Keys shuffled on load to distribute load
- **Template Severity Overhaul**
  - Reduced 8 templates from overly broad to critical patterns only
  - unchecked_return_value: requires NO checking (bool/require/if)
  - reentrancy: removed delegatecall (has own template)
  - precision_loss: 7→2 patterns (only real exploit patterns)
  - missing_access_control: CRITICAL + negative lookahead
  - denial_of_service: 10→2 patterns
  - front_running: 4→1 pattern
  - signature_unchecked: simplified
  - integer_overflow: 2→1 pattern

### Changed
- **Risk Scoring Formula**: CRITICAL×30 + HIGH×1 (was 10×1)
  - CRITICAL findings now properly dominate ranking
  - Balanced weighting prevents HIGH dilution
  - Real vulnerabilities rank higher
- **Scanner Performance**: 6-10× speedup
  - Moved regex compilation from scan() to new()
  - 4 protection regexes compiled once at init
  - Eliminated 400+ regex compilations per 100 contracts
- **False Positive Reduction**: 32% fewer findings
  - 294→200 findings with new filters
  - 24→20 flagged contracts
  - 94 obvious patterns filtered out

### Fixed
- **Critical**: API key fallback not rotating through keys
- **Critical**: Regex patterns compiled on every scan (1000× slowdown)
- **Critical**: Templates too broad causing 90% false positives
- **Critical**: Scoring didn't account for filtered findings
- Templates marking low-severity patterns as HIGH/CRITICAL
- Small contracts with many findings ranking too high
- Filtered findings still counted in risk score

### Performance
- **Scanner**: 0.33s per contract (was 3-5s) - 10× faster
- **API Fallback**: Immediate rotation (was 3× retry delay)
- **False Positives**: 32% reduction (294→200 findings)
- **Template Precision**: Significantly improved with stricter patterns

### Status
- **Production Ready**: Major improvements to accuracy and performance
- Real vulnerabilities now rank in top 3 consistently
- API key fallback working correctly with 6-key rotation
- Templates significantly more severe and accurate
- Filtered findings system provides transparency
- 10× scanner performance improvement

## [v1.3] - Previous Release
(See previous changelog entries above)
