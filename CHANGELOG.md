# CHANGELOG - v0.1.0 "Foundation"

## 🚀 Initial Release: v0.1.0
This marks the completion of the core MVP development phase (Semana 1 to 12).

### ✨ Features
- **Phase 1 (Architecture):** `UnifiedNode` struct containing vectors, edges, and relational data. Custom `RwLock` in-memory engine and `bincode` WAL for crash recovery.
- **Phase 2 (Query Engine):** EBNF `nom` parser resolving hybrid syntax (`FROM`, `SIGUE`, `~`, `RANK BY`). Logical Planner and basic CBO integrated. RocksDB persistence layer utilizing zero-copy zero-alloc buffer pinning. CP-Index HNSW implementation.
- **Phase 3 (Integrations):** Added Resource Governor (OOM guard & Temperature execution). Scaffolded API handlers targeting Ollama LLM proxying and generic vector store clients.

### 📦 Installation
Available via crates.io or compiling from source: `cargo install --path .`
