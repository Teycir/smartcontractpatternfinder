# Ecosystem & Extensibility Features

## 🎯 Overview

SCPF now includes a foundation for ecosystem growth and extensibility:

1. **Template Registry** - Centralized template management
2. **Template Marketplace** - Install collections by category
3. **Plugin System** - Foundation for custom rules (Rust/WASM)

---

## 1. Template Registry 📦

### What It Does
Centralized registry of template collections organized by category (DeFi, ERC, Gas, etc.).

### Usage

```bash
# List available collections
scpf templates registry

# Output:
# 📦 Template Collections:
#   • core - Essential security patterns (6 templates)
#   • defi - DeFi protocol vulnerabilities (4 templates)
#   • erc - ERC standard compliance (3 templates)
#   • gas - Gas optimization (2 templates)
#   • audit - Advanced audit patterns (2 templates)
#   • zeroday - Emerging threats (1 template)
```

### Registry Structure

**File**: `registry.yaml`

```yaml
collections:
  core:
    description: "Essential security patterns"
    templates:
      - reentrancy.yaml
      - delegatecall_user_input.yaml
      - tx_origin_auth.yaml
  
  defi:
    description: "DeFi protocol vulnerabilities"
    templates:
      - defi_vulnerabilities.yaml
      - front_running_v2.yaml

aliases:
  all: ["core", "defi", "erc", "gas", "audit"]
  essential: ["core", "defi", "erc"]
  security: ["core", "audit", "zeroday"]
```

---

## 2. Template Marketplace 🛒

### Install Collections

```bash
# Install single collection
scpf templates install defi

# Output:
# 📦  Installing 1 collection(s)...
# →  defi (4 templates)
#   ✓  defi_vulnerabilities.yaml (already exists)
#   ✓  front_running_v2.yaml (already exists)

# Install using alias
scpf templates install essential
# Installs: core, defi, erc

# Install all templates
scpf templates install all
```

### Update Templates

```bash
# Check for updates
scpf templates update

# Output:
# 🔄  Checking for template updates...
# ✓  Found 21 local templates
```

### List Installed Templates

```bash
# List all templates
scpf templates list

# Show specific template
scpf templates show reentrancy-basic
```

---

## 3. Plugin System 🔌

### What It Does
Foundation for custom, proprietary scanning rules without forking SCPF.

### Configuration

**File**: `plugins.yaml`

```yaml
version: "1.0"

plugins:
  # Rust crate plugin
  - name: "mycompany-rules"
    type: "crate"
    source: "path/to/mycompany_scpf_rules"
    enabled: true
  
  # WASM plugin (future)
  - name: "custom-wasm"
    type: "wasm"
    source: "plugins/custom.wasm"
    enabled: true

settings:
  enabled: false
  search_paths:
    - "./plugins"
    - "~/.scpf/plugins"
  timeout: 30
```

### Plugin Interface (Rust)

```rust
use scpf_types::{Template, Match, Pattern};

#[no_mangle]
pub extern "C" fn scpf_plugin_init() -> *const PluginInfo {
    // Initialize plugin
    Box::into_raw(Box::new(PluginInfo {
        name: "MyCompany Rules",
        version: "1.0.0",
    }))
}

#[no_mangle]
pub extern "C" fn scpf_plugin_scan(source: &str) -> Vec<Match> {
    // Custom scanning logic
    let mut matches = Vec::new();
    
    // Example: Detect proprietary pattern
    if source.contains("MyCompanyToken") {
        matches.push(Match {
            template_id: "mycompany-token".to_string(),
            pattern_id: "token-usage".to_string(),
            message: "MyCompany token detected".to_string(),
            // ... other fields
        });
    }
    
    matches
}
```

### Plugin Use Cases

1. **Proprietary Patterns** - Company-specific security rules
2. **Custom Standards** - Internal coding standards
3. **Domain-Specific** - Industry-specific vulnerabilities
4. **Private Templates** - Keep sensitive patterns private

---

## 📊 Implementation Status

| Feature | Status | Complexity |
|---------|--------|------------|
| Registry Config | ✅ Complete | Low |
| Registry CLI | ✅ Complete | Low |
| Install Command | ✅ Complete | Low |
| Update Command | ✅ Complete | Low |
| Plugin Config | ✅ Complete | Low |
| Plugin Loading | 🚧 Foundation | Medium |
| Remote Download | 📋 Planned | Medium |
| WASM Support | 📋 Planned | High |

---

## 🚀 Quick Start

### 1. View Registry

```bash
scpf templates registry
```

### 2. Install Collection

```bash
# Install DeFi templates
scpf templates install defi

# Install essential templates
scpf templates install essential
```

### 3. List Templates

```bash
scpf templates list
```

### 4. Update Templates

```bash
scpf templates update
```

---

## 🔮 Future Enhancements

### Remote Registry (Planned)
```bash
# Download from GitHub
scpf templates install defi --remote

# Add custom registry
scpf registry add mycompany https://github.com/mycompany/scpf-templates

# Install from custom registry
scpf templates install mycompany/internal-rules
```

### WASM Plugins (Planned)
```javascript
// plugin.js
export function scan(source) {
    const matches = [];
    
    // Custom logic in JavaScript/TypeScript
    if (source.includes("dangerous_pattern")) {
        matches.push({
            message: "Dangerous pattern detected",
            severity: "high"
        });
    }
    
    return matches;
}
```

### LSP Integration (Planned)
```bash
# Start SCPF daemon
scpf daemon start

# VS Code extension connects to daemon
# Real-time scanning on file save
```

---

## 📚 Files

- `registry.yaml` - Template registry configuration
- `plugins.yaml` - Plugin system configuration
- `crates/scpf-cli/src/commands/templates.rs` - Registry commands

---

## 🎯 Benefits

### For Teams
- **Centralized Management** - One place for all templates
- **Easy Discovery** - Browse collections by category
- **Version Control** - Track template changes
- **Private Rules** - Keep proprietary patterns secure

### For Community
- **Sharing** - Contribute templates to registry
- **Reusability** - Install proven patterns
- **Collaboration** - Build on existing work
- **Standards** - Common vulnerability definitions

### For Developers
- **Extensibility** - Add custom logic via plugins
- **Flexibility** - Choose what to install
- **Automation** - Update all templates at once
- **Integration** - Plugin system for CI/CD

---

## 🔗 Related

- [Quick Wins](QUICK_WINS.md) - ERC compliance, L2 support, risk scoring
- [Template Changelog](TEMPLATE_CHANGELOG.md) - Template version history
- [GitHub Action](docs/GITHUB_ACTION.md) - CI/CD integration

---

**Status**: ✅ Foundation Complete  
**Next**: Remote registry, WASM plugins, LSP integration  
**Date**: 2026-01-20
