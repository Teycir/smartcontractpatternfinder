# Smart Contract Pattern Finder (SCPF)

A high-performance tool for detecting patterns in smart contracts across multiple blockchains.

## Features

- Multi-chain support (Ethereum, BSC, Polygon)
- YAML-based pattern templates
- Fast regex-based scanning
- Caching for performance
- CLI interface

## Installation

```bash
cargo build --release
```

## Usage

### Initialize a project

```bash
scpf init
```

### Scan contracts

```bash
scpf scan 0x123... --chain ethereum
```

## Project Structure

```
crates/
  scpf-types/    - Core types and data structures
  scpf-core/     - Core scanning and fetching logic
  scpf-cli/      - Command-line interface
```

## License

MIT
