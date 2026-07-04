---
title: "ADR 007: PyO3 Binding Architecture for Python SDK"
type: adr
status: active
tags: [vantadb, architecture, adr]
last_reviewed: 2026-07-03
aliases: []
---

# ADR 007: PyO3 Binding Architecture for Python SDK

## Status

Status: Approved

## Context

VantaDB's Python SDK (`vantadb-python`) must expose the full embedded engine API to Python consumers, including the data plane (vector insert, delete, query) and the control plane (configuration, checkpoint, backup). The binding layer must satisfy:

1. **Zero-copy numpy integration:** ANN queries return embedding vectors that are frequently consumed as numpy arrays for downstream processing (clustering, visualization, ML pipelines). Copies on every query are prohibitive at scale.
2. **Synchronous API surface:** The core engine is synchronous (see ADR 003). The Python SDK must not force users into asyncio patterns.
3. **Type-safe schema mapping:** Python data models (dataclasses, TypedDicts) must map ergonomically to VantaDB's column-family schema without verbose serialization code.
4. **Installation simplicity:** The package must install via `pip install vantadb` without requiring the user to have a Rust toolchain installed.

Three approaches were evaluated: PyO3 (native Rust bindings), CFFI (C ABI handshake), and `maturin`-managed C extension compilation.

## Decision

1. **PyO3 with Maturin Build System:** The Python SDK is implemented as a PyO3 (v0.22+) native extension, packaged and built via `maturin`. PyO3 was chosen over CFFI because:
   - Direct Rust type mapping eliminates the C ABI intermediary layer, reducing serialization overhead by approximately 40% in microbenchmarks.
   - Maturin provides `sdist` publishing with pre-built wheels for major platforms, satisfying the `pip install` requirement.
   - PyO3's Python GIL management (`Python::with_gil`) allows fine-grained control over lock duration during engine calls.

2. **Buffer Protocol for Zero-Copy Arrays:** Query methods return numpy arrays through Python's buffer protocol, not through `Vec -> PyList` conversion. The engine's internal `Vec<f32>` storage is exposed as a `PyBuffer` via `PyO3`'s `#[pyclass]` integration:
   ```rust
   impl PyBufferProtocol for VectorBuffer {
       fn bf_getbuffer(
           slf: &PyAny,
           view: *mut ffi::Py_buffer,
           flags: c_int,
       ) -> PyResult<()> {
           // Points view->buf directly at engine-owned memory
       }
   }
   ```
   This avoids a memcpy for every result set. The returned buffer wraps a reference-counted handle to the engine's internal `Arc<Vec<f32>>`, ensuring the data lives long enough for the Python consumer.

3. **Async Support Strategy:** The Python SDK is intentionally synchronous at the call-site (see ADR 003). For applications requiring concurrent query execution, users spawn Python `threading.Thread` or `concurrent.futures.ThreadPool` workers. Each thread holds a handle to a separate engine instance or uses the engine's internal `RwLock` for concurrent reads. A future `vantadb-python-async` shim may be offered as a separate package for asyncio-native applications, but it will always delegate to the synchronous core via `loop.run_in_executor`.

## Consequences

### Benefits

- **Zero-Copy Throughput:** Large batch queries (10K+ vectors) incur zero Python-side allocation for vector data, matching C-native performance in benchmarks.
- **Single-Command Install:** Pre-built wheels for `manylinux`, `macos`, and `windows` are published to PyPI. Users never need Rust or a C++ compiler.
- **Ergonomic Typing:** PyO3's `#[pyclass]` and `#[pymethods]` produce idiomatic Python classes with property accessors and docstrings visible via `help(vantadb)`.
- **Thread Safety at Python Level:** Each Python thread acquires the GIL independently and releases it during blocking engine calls via `Python::allow_threads`, allowing other Python threads to make progress.

### Technical Debt / Costs

- **Precompiled Wheel Matrix:** Maintaining pre-built wheels for `cpython` 3.9–3.13 across 6 architectures and 3 OS families requires CI investment and nightly build pipelines.
- **GIL Release Granularity:** Long-running queries (> 100ms) must explicitly release the GIL using `Python::allow_threads` to avoid blocking the entire Python process. This adds a small ergonomic burden on the binding author.
- **ABI Stability:** PyO3 internal ABI changes between minor CPython versions require wheel rebuilds. The maturin `abi3` feature flag partially mitigates this but limits the use of nightly PyO3 features.
