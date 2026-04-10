# VantaDB Public Roadmap

### Current Focus: Engine Stabilization

**Q2 2026: Core Audit & Benchmarks**
* [x] Formally rename from ConnectomeDB to VantaDB.
* [x] Stabilize HNSW Vector index with testable recall.
* [ ] Formalize Multi-threaded Concurrent Insertions.
* [ ] Enhance PyO3 serialization mapping structures.
* [ ] Exhaustive test matrix for Node Invalidations & Garbage Collection.

### Next Horizons: Usability & Integrations

**Q3 2026: The Pipeline Edge**
* Optimize zero-copy queries natively executing pandas DataFrames or numpy arrays.
* Expand the `vantadb-python` SDK to integrate naturally with LangChain or LlamaIndex retrievers.
* Publish continuous benchmarks against established persistent storage backends (SQLite+vec, PGVector).

**Q4 2026: Cloud Sync & Advanced Embeddings**
* Asynchronous replication of the memory-mapped backing store to S3/GCP.
* Support for Sub-Byte Quantization implementations internally within the Vector representations.
