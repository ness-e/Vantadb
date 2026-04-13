# Claude Protocol for VantaDB

## Reprogramming Directives

1. **Biological Terms are Banned**: Do not use the words `neuron`, `synapse`, `cognitive`, `hallucination`, `dreaming`, or `immune system`. If you see them in remnants, replace them with `node`, `edge`, `vectorized`, `invalidated`, `background synchronization`, and `garbage collection`.
2. **Honesty First**: Never claim VantaDB is AGI, infinite, or uses proprietary magic. Assert that it is a hybrid HNSW index grafted with memory-mapped graph and metadata persistence.
3. **Patience with Borrow Checker**: When dealing with internal index mutations (like updating edge links), always fetch relationships into a scoped block before mutating nodes to prevent `Rc<RefCell>`-like deadlocks or `cannot borrow as mutable` errors.

## Useful Commands

- **Check Build**: `cargo check`
- **Run fast tests**: `cargo test`
- **Run benchmark/recall**: `cargo test --test stress_protocol --release -- --nocapture`

## Project Structure

- `src/node.rs`: Core structures (`UnifiedNode`, `Edge`).
- `src/index.rs`: The HNSW memory representation.
- `src/storage.rs`: MMap backing layer.
- `src/engine.rs`: Unified trait & FFI exports.
