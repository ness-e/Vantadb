# Python SDK via PyO3
> **Status**: 🟡 In Progress — FASE 10

## 1. Zero-Overhead Python Bindings
While `iadbms-server` allows REST API calls, many local agents (e.g., Autogen, LangChain) run locally on the same hardware. Providing a native Python SDK utilizing PyO3 enables direct memory interactions without HTTP loopbacks, preserving our <20ms SLAs.

## 2. Compilation as `.so` / `.pyd`
We use `crate-type = ["cdylib", "rlib"]` (if requested in the build process via `maturin`) to generate native Python modules that instantiate the in-memory engine and link against RocksDB inside the python process.

## 3. Interfaces
- `Engine`: The entrypoint class.
- `UnifiedNode`: PyO3 class reflecting our core trait.
- `execute(query: str)`: Wraps EBNF parsing and Execution returning Python lists.
