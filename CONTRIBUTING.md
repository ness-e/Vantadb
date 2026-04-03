# Contributing to ConnectomeDB

First off, thank you for considering contributing to ConnectomeDB! It's people like you that make ConnectomeDB a great tool for the local AI ecosystem.

## 🧠 Core Philosophy
ConnectomeDB is designed to be a **local-first, multi-model engine** focusing on absolute efficiency over edge-case complexity. 
Before submitting major architectural changes, please open an Issue to discuss it. We value:
- **Zero-Copy Serialization:** Anything that avoids allocations during read operations is prioritized.
- **Dependency Minimalism:** We try to keep our dependency tree tiny to ensure sub-second compilation times and small `<15MB` binary sizes.
- **Agent Context:** Features should be evaluated with "How does this help an autonomous AI agent reason better?"

## 🚀 Getting Started

### 1. Prerequisites
- Rust `1.75` or higher.
- `cargo` and `rustfmt`.

### 2. Local Setup
Fork the repository and clone it to your local machine:
```bash
git clone https://github.com/YOUR-USERNAME/ConnectomeDB.git
cd ConnectomeDB
```

Verify that the core logic compiles and tests pass:
```bash
cargo check
cargo test --all-features
```

### 3. Making Changes
1. Create a new branch: `git checkout -b feature/your-feature-name`
2. Make your modifications.
3. Keep your commits atomic, and write clear, imperative commit messages (`feat: add LRU cache for graph traversals`).
4. **Formatting:** Before committing, ensure the code is formatted:
```bash
cargo fmt --all
cargo clippy -- -D warnings
```

### 4. Submitting a Pull Request
1. Push your branch to your fork.
2. Open a Pull Request against the `main` branch of this repository.
3. Provide a clear description referencing any issues your PR resolves (e.g., `Closes #42`).
4. Wait for CI checks to pass and for a maintainer to review your code.

## 🧪 Testing Guidelines
- **Unit Tests:** Any new algorithmic logic in `core/` must include unit tests. 
- **Benchmarks:** If you are modifying the `UnifiedNode` serialization or the `HNSW` layers, please run `cargo bench` to prove no performance regressions.

Thank you for contributing to the future of AI databases!
