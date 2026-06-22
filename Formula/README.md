# VantaDB Homebrew Tap

A [Homebrew](https://brew.sh) tap for installing VantaDB CLI tools.

## Usage

```bash
# Add the tap
brew tap ness-e/Vantadb

# Install the VantaDB CLI
brew install vantadb

# Verify installation
vanta-cli --version
```

## What's installed

| Binary | Description |
|--------|-------------|
| `vanta-cli` | VantaDB CLI — query, manage, and interact with embedded VantaDB databases |
| `vantadb-server` | Optional HTTP server for remote VantaDB access |
| `vantadb-mcp` | MCP (Model Context Protocol) server for AI tool integration |

## Requirements

- macOS (Intel or Apple Silicon) or Linux (x86_64)
- Homebrew >= 4.0

## Updating

```bash
brew update && brew upgrade vantadb
```

## Local development

```bash
brew install --head ness-e/Vantadb/vantadb
```

> **Note:** Running from HEAD builds from source via `cargo`. Ensure Rust toolchain is installed.

## Release workflow

On every tagged release (`v*.*.*`), GitHub Actions publishes precompiled binaries to the [releases page](https://github.com/ness-e/Vantadb/releases). This tap formula points to those artifacts.

Before the first install of a new version, verify and update the SHA256 checksums in `vantadb.rb`.

## Architecture support

| Platform | Status |
|----------|--------|
| macOS x86_64 | ✅ Available |
| macOS ARM64 | 🚧 Planned |
| Linux x86_64 | ✅ Available |
| Windows | 🚧 Planned (use scoop/choco instead) |
