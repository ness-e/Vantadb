# VantaDB CI & Certification Policy

To maintain a rapid development iteration cycle and guarantee mathematical precision in our HNSW
engine, VantaDB enforces a split Continuous Integration architecture.

## The Two-Tiered CI Architecture

### 1. The Fast Gate (`rust_ci.yml`)

The fast gate is triggered automatically on every pull request and push to the `main` branch.
**Goal:** Deliver PR feedback in under 5 minutes.

The fast gate validates the production-facing MVP boundary only: embedded core behavior, stable
SDK/CLI flows, durability, namespace and metadata indexes, vector retrieval, BM25, Hybrid Retrieval
v1, rebuild/audit, and local deterministic integration tests. Historical or experimental surfaces
such as IQL/LISP/DQL, MCP, LLM/Ollama integration, graph traversal beyond stored local edges, and
governance semantics are excluded from the default fast lane.

**What it runs:**

- Static analysis: `cargo fmt` and `cargo clippy`.
- Unit tests and fast integration tests (`cargo test --test <name>`).
- API contract verifications that do not depend on external systems.
- Embedded CLI diagnostics such as `audit-index`, when covered through local integration tests and
  temporary database directories.

**Strict Rules for the Fast Gate:**

- **Deterministic:** Tests must not rely on random timing or external networking.
- **Local:** No external dependencies are allowed (e.g., no external LLM services, no Ollama
  required).
- **Fast:** Any test exceeding a few seconds must be moved to heavy certification or heavily
  optimized.

### Experimental Suite

Experimental tests are retained for local/manual diagnostics but do not define the v0.1.x MVP. Run
them explicitly with:

```bash
cargo nextest run --profile experimental --workspace --features experimental
```

Failures in this suite should be triaged, but they do not block the Fast Gate unless the failure is
caused by a change to production-facing MVP behavior.

### 2. Heavy Certification (`heavy_certification.yml`)

The heavy certification suite validates the engine's capability to run under production stress,
ensuring recall guarantees and scaling limits. **Goal:** Validate engine stability, recall, and
scale capabilities without bottlenecking daily development.

**What it runs:**

- `stress_protocol`: Validates dynamic scaling (10K, 50K, 100K vectors), persistence, latency, and
  0.95+ Recall@10.
- `hnsw_validation` & `hnsw_recall_certification`.
- `sift_validation` (optional): Tests the engine against standard public datasets.
- `competitive_bench` (optional): Validates against FAISS/HNSWlib.

**Why are these tests separated?** Running `stress_protocol` can take close to 2 hours on hosted
runners and requires significant system resources (AVX2 plus heavy swap). It runs in its own
scheduled/manual job with a 150 minute step timeout so it can complete without blocking the other
certification checks. Running this on every PR would paralyze development velocity.

### 3. Python Wheel Certification (`python_wheels.yml`)

The wheel workflow builds the Python SDK on Linux, macOS, and Windows with `maturin`, installs the
generated wheel by resolved path, and runs the Python SDK smoke suite. Manual TestPyPI upload is
available only through an explicit workflow input and the `TEST_PYPI_API_TOKEN` secret. Production
PyPI publication and signing remain deferred.

## External Dependencies (Ollama/LLMs)

VantaDB integrates with external LLMs for embeddings and semantic queries. However, **integration
tests requiring network access to LLMs (like Ollama) are strictly excluded from the Fast Gate.**
They are either marked with `#[ignore]` or gated behind environment variables (e.g.,
`VANTADB_RUN_LLM_TESTS=1`). This ensures the core engine can be built and tested completely offline.

## Running Heavy Certification Manually

The `heavy_certification.yml` workflow runs automatically via a CRON schedule (e.g., weekly on
Sundays). The scheduled lane runs the local deterministic core certification jobs. SIFT-1M
validation and competitive benchmarks are manual opt-ins because they require external datasets. You
can also trigger it manually from the GitHub Actions UI:

1. Navigate to the **Actions** tab in the repository.
2. Select **VantaDB Heavy Certification** from the left sidebar.
3. Click **Run workflow**.
4. You can optionally check the boxes to include `SIFT-1M validation` or `Competitive benchmarks`.
