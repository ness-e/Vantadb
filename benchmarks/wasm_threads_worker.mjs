// STRAT-04 — worker side of the WASM threads benchmark.
//
// Instantiates the lock-free kernel with the host-created shared memory,
// points this instance's stack at its own disjoint region, and runs the
// kernel when the main thread says "go". See benchmarks/wasm_threads_bench.mjs
// and docs/dev/architecture/WASM_THREADS.md.
import { parentPort, workerData } from 'node:worker_threads';
import { readFile } from 'node:fs/promises';

const { wasmPath, memory, stackTop, layout, chunk } = workerData;

const bytes = await readFile(wasmPath);
const { instance } = await WebAssembly.instantiate(bytes, { env: { memory } });
const { exports } = instance;

// Wasm threads model: every instance shares ONE linear memory, so each
// instance must run on a DISJOINT stack region or concurrent frames collide.
// The module exports `__stack_pointer` (mutable global) for exactly this.
exports.__stack_pointer.value = stackTop;

parentPort.postMessage({ type: 'ready' });

parentPort.on('message', (msg) => {
  if (msg?.type !== 'go') return;
  const t0 = performance.now();
  const status = exports.score_batch_chunked(
    layout.vectorsPtr,
    layout.count,
    layout.dims,
    layout.queryPtr,
    layout.scoresPtr,
    layout.cursorPtr,
    chunk,
  );
  const kernelMs = performance.now() - t0;
  parentPort.postMessage({ type: 'done', status, kernelMs });
});
