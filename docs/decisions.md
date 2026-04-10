# VantaDB Architectural Decisions Record (ADR)

## Decision 1: Purging Biological Metaphors
**Background:** The project was initially conceived as "ConnectomeDB" with neural aliases like "Neurons" and "Synapses".
**Decision:** We transitioned fully to standard terminology ("VantaDB", `UnifiedNode`, `Edge`).
**Rationale:** Marketing hype obfuscates standard mathematical debugging. By using graph theory and database engineering terminology, external contributors can understand the data structures and index algorithms without deciphering a proprietary lexicon.

## Decision 2: Replacing the Prototype Index with HNSW
**Background:** The index module (`src/index.rs`) was initially a placeholder utilizing brute-force arrays.
**Decision:** Implemented a formalized HNSW (Hierarchical Navigable Small World) index structure.
**Rationale:** Standardizing the vector search index ensures bounded scaling. A benchmark suite `tests/hnsw_recall.rs` enforces testing guarantees where recall is verified strictly against an exact brute-force validation query. Current configurations guarantee >95% Recall@10.

## Decision 3: In-Process Execution with PyO3 
**Background:** Modern vector databases utilize gRPC and API containers.
**Decision:** We enforce an embedded design using PyO3 mappings directly into Python.
**Rationale:** Avoids TCP serialization overhead and Docker Compose lifecycle management for simpler integration within fast AI agent loops.
