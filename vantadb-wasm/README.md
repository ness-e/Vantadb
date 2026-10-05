# vantadb-wasm — browser AI agent memory

> **Status: active** (core-promise until 1.0 — one of the three active language connectors; published as `vantadb-wasm` on npm).
>
> **Browser AI agent memory.** VantaDB WASM is an embedded vector + graph memory
> engine that runs entirely in the browser — persistent agent memory (OPFS),
> HNSW vector search, BM25 + RRF hybrid retrieval — no server required. Try the
> **[browser AI agent demo](https://github.com/ness-e/Vantadb/tree/main/vantadb-wasm/demo)**
> (Transformers.js embeddings on-device + OPFS memory), or start from the
> higher-level [`vantadb`](https://www.npmjs.com/package/vantadb) SDK.
>
> **Why this file exists:** The WASM bundle is `~1.8 MB` raw (`1.77 MB` measured
> 2026-10-04 from `pkg/vantadb_wasm_bg.wasm` — see §1 for the
> repro command), which is **~31× the size of Orama**
> (23.8 KB gzipped per [bundlephobia](https://bundlephobia.com/package/@orama/orama)).
> That gap is real and intentional: VantaDB ships persistence (OPFS/WAL/fjall),
> HNSW vector index, BM25 full-text, RRF hybrid fusion, capability graphs, and
> WASM-bindgen FFI glue — features Orama does not include. This document is the
> honest engineering brief for adoption: what gets shipped, why it is that size,
> how lazy loading works in each runtime, and where the feature gap lives.

---

## 1. Bundle sizes (measured 2026-10-04)

Reproducible from `vantadb-wasm/pkg/` (generated, gitignored — see
`vantadb-wasm/pkg/.gitignore`; rebuild with `wasm-pack build --release`
from `vantadb-wasm/`, then sync the hand-written types with
`node dev-tools/build-wasm-types.mjs` — the same sequence the publish job
runs in `.github/workflows/release-npm-61.yml`):

| File | Raw bytes | Raw (KB / MB) | Gzipped | Gzipped (KB) | What it is |
|------|----------:|---------------|--------:|--------------|------------|
| `vantadb_wasm_bg.wasm`  |  1,856,638 | **1.77 MB** | 745,152 | **728 KB** | Engine binary (Rust → wasm32) |
| `vantadb_wasm_bg.js`    |     58,519 |   57.1 KB    |  11,265 |  11.0 KB     | wasm-bindgen glue (bundler ESM) |
| `vantadb_wasm.js`       |        245 |    245 B     |     156 |   156 B      | bundler re-export stub (`export { Client }`) |
| **TOTAL transfer (gzip)** | **1,915,402** | **1.83 MB** | **756,573** | **~739 KB** | What the browser actually fetches |

> **Build provenance (2026-10-04):** measured from a fresh
> `wasm-pack build --release` (wasm-pack 0.15.0, `wasm-opt -Oz`, Windows x64).
> Numbers shift between builds (toolchain + code) — re-run the commands below
> after rebuilding; never copy numbers between builds.

### How to reproduce

```powershell
# Raw sizes
Get-Item "vantadb-wasm/pkg/vantadb_wasm_bg.wasm","vantadb-wasm/pkg/vantadb_wasm_bg.js","vantadb-wasm/pkg/vantadb_wasm.js" |
    Select-Object Name, Length

# Gzipped sizes (PowerShell + .NET GzipStream, default level)
Add-Type -AssemblyName System.IO.Compression.FileSystem
function Get-GzipSize($Path) {
    $bytes = [System.IO.File]::ReadAllBytes($Path)
    $ms = New-Object System.IO.MemoryStream
    $gz = New-Object System.IO.Compression.GzipStream($ms, [System.IO.Compression.CompressionMode]::Compress)
    $gz.Write($bytes, 0, $bytes.Length); $gz.Close()
    return $ms.ToArray().Length
}
Get-GzipSize "vantadb-wasm/pkg/vantadb_wasm_bg.wasm"   # → 745,152 (measured 2026-10-04)
```

### Types sync (`.d.ts` src vs `pkg/`)

`vantadb-wasm/pkg/` also contains a generated `vantadb_wasm.d.ts` full of
`any` stubs. The source of truth is the hand-written
`vantadb-wasm/src/vantadb_wasm.d.ts`; sync it into `pkg/` after
`wasm-pack build` (invoked automatically by
`.github/workflows/release-npm-61.yml`):

```powershell
node dev-tools/build-wasm-types.mjs --check   # exit 0 = in sync; exit 3 = stale
node dev-tools/build-wasm-types.mjs           # apply: overwrite pkg/vantadb_wasm.d.ts
```

Source of truth: `wasm-pack build --release` (run from `vantadb-wasm/`)
produces `vantadb-wasm/pkg/`; CI runs the same command in the `publish-wasm`
job of `.github/workflows/release-npm-61.yml`. The `Cargo.toml` already opts
into `-Oz`:

```toml
[package.metadata.wasm-pack.profile.release]
# Explicit -Oz over default -Os: more aggressive binaryen size pass.
# wasm-opt defaults to true (-Os) since binaryen v121+ supports bulk-memory-opt;
# -Oz adds another size-reduction pass on top.
wasm-opt = ["-Oz"]
```

---

## 2. Lazy loading patterns by runtime

The `vantadb_wasm_bg.wasm` is **never loaded eagerly** — it is wired behind a
dynamic `import()` / native loader. The bundler, Node, and the browser each
load it on first method call, not on import.

### 2.1 Bundlers (Vite / Webpack / esbuild) — **requires** `vite-plugin-wasm`

The wasm-bindgen `bundler` target emits `import * as wasm from "./vantadb_wasm_bg.wasm"`
which is a binary imported as an ES module. Standard bundlers do not handle that
natively. Install the plugin that matches your bundler:

```bash
# Vite
npm install -D vite-plugin-wasm

# Webpack
npm install -D @wasm-tool/wasm-pack-plugin
```

Without the plugin the build fails at bundle time. The plugin fetches the `.wasm`
**on demand** when the first call lands, not on initial page load.

### 2.2 Node.js — file-based lazy loader (no plugin)

The wasm-bindgen Node target reads the `.wasm` from disk at first use:

```js
import { VantaDB } from "vantadb";
// ↑ first call lazy-loads pkg/vantadb_wasm_bg.wasm via fs.readFile()
const db = VantaDB.create();
```

No plugin required. Loader hits disk on first method call.

### 2.3 Vanta Studio desktop — code-split out (WASM-02 / WASM-03)

The Vanta Studio build externalizes the WASM glue so the lazy `import()` is
**never executed in Tauri or HTTP modes**:

| Build mode | WASM loaded? | Source |
|------------|-------------|--------|
| Tauri desktop | ❌ never (uses native backend) | `desktop/vite.config.ts` |
| HTTP mode | ❌ never (uses HTTP backend) | `desktop/vite.config.ts` |
| `vite build --mode wasm` | ✅ only on first WASM call | `desktop/vite.config.ts`, WASM-03 |

The lazy `import()` only fires when a user opts into the WASM backend explicitly.

### 2.4 Browser — `esm.sh` CDN (verified working) vs `jsDelivr` (fails)

See `vantadb-ts/README.md` §"Zero-install CDN usage (verified 2026-08-26)"
for the full table. Short version:

| CDN | Result | Why |
|-----|--------|-----|
| `https://cdn.jsdelivr.net/npm/vantadb@latest/+esm` | ❌ fails | jsDelivr's Rollup pipeline cannot resolve the wasm-bindgen `bundler`-target `import` of the `.wasm` file as an ES module |
| `https://esm.sh/vantadb@latest` | ✅ works | esm.sh inlines the `.wasm` as a base64 byte array into the served `.mjs`, no sidecar fetch |
| **Self-host** with `wasm-pack build --target web` | ✅ works | the `web` target uses `fetch()` + `WebAssembly.instantiateStreaming` |

### 2.5 SSR / React hooks — must lazy

```ts
// ❌ BAD: top-level eager instantiation breaks SSR
const db = VantaDB.create(); // throws if window is undefined

// ✅ GOOD: instantiate inside useEffect / useMemo / client boundary
useEffect(() => {
  const db = VantaDB.create();
  return () => db.close();
}, []);
```

---

## 3. Build feature flags (Cargo features)

Defined in `vantadb-wasm/Cargo.toml`:

| Feature | Default? | Purpose | Effect on bundle |
|---------|----------|---------|------------------|
| `tracing-wasm` | ✅ on | `console.log`-based tracing in browser | +~3 KB (gzip) |
| `opfs` | ❌ off | Worker-backed OPFS persistence (`connect_worker`, `worker_read/write/delete`) | +~12 KB (gzip) — measured empirically |
| `wasm` (in `vantadb` core) | ✅ on for WASM build | Tells the core to drop non-WASM backends (fjall/rocksdb) | **−700 KB vs full core** — biggest single win |

**`wasm` feature flag is the dominant size win.** Without it, the engine would
link `rocksdb` + `fjall` + `arrow` IPC paths. With it, the build tree-shadows
to `wasm-bindgen` + `getrandom` + `serde-wasm-bindgen` only.

For browser deployments you typically want the **default feature set**
(`tracing-wasm` only). The OPFS worker feature (`--features opfs`) is opt-in
because it pulls in the worker shim and OPFS bridge glue.

### Build commands

```bash
# Default (recommended for browser/Node):
wasm-pack build --release --target bundler      # for Vite/Webpack/esbuild
wasm-pack build --release --target web          # for plain HTML/JS browsers (self-host)
wasm-pack build --release --target nodejs       # for Node

# With OPFS worker:
wasm-pack build --release --target bundler --features opfs

# Without tracing-wasm (smallest possible):
wasm-pack build --release --target bundler --no-default-features
```

### Console logging

The default build installs a `console.log`-based tracing subscriber
(feature `tracing-wasm`) at level **`WARN`**: the core emits a `DEBUG` trace
for every env-var read while building its config, which would otherwise flood
the console on every `Client.create()`.

To opt into more detail, set the global **before the first client is
created** (the WASM analog of the core's `RUST_LOG`, see `src/console.rs`):

```js
globalThis.VANTADB_LOG = "debug"; // "trace" | "debug" | "info" | "warn" | "error"
const db = new Client();
```

The value is read once per process (the tracing subscriber is global and can
only be installed once); an absent or invalid value falls back to `WARN`.

### Multi-tab safety (OPFS)

OPFS writes (`save()`, `append_file`, `delete_file`) acquire a per-file
[Web Lock](https://developer.mozilla.org/en-US/docs/Web/API/Web_Locks_API)
(`vantadb-opfs-write:<directory>:<path>`): multiple tabs or workers on the same
origin serialize writes to the same file — no interleaved writes, no lost
appends, no clobbered temp file. (Whole-file `save()` remains last-writer-wins
across tabs, now without corruption.) Writes to different files do not contend,
and reads are lock-free (writes publish atomically via temp file + rename).

When the Web Locks API is unavailable (Safari 15.2–15.3), OPFS **writes fail
with a descriptive error** rather than risk multi-tab corruption — reads keep
working. On those browsers use `connect_idb` (IndexedDB) if you need writes.

---

## 4. Honest comparison vs JavaScript-only search engines

**Regla 11 note:** every number below links to a reproducible source. Re-run
`bundlephobia.com/package/<name>` for current numbers (the JS ecosystem
re-ships often; the VantaDB row was measured 2026-10-04, competitor rows as
sourced below).

| Library | Version | Min | **Gzipped** | Vector? | Hybrid? | Persistence? | Feature parity with VantaDB |
|---------|---------|----:|------------:|---------|---------|--------------|-----------------------------|
| **@orama/orama** | 3.1.18 | 75.2 KB | **23.8 KB** | ✅ (`mode:'vector'`) | ✅ (`mode:'hybrid'`, RRF) | ❌ in-memory only (plugin for disk) | **No** — Orama has no HNSW, no OPFS, no WAL, no Fjall, no capability graph, no TTL auto-expiry |
| **MiniSearch** | latest | ~22 KB | **5.9 KB** | ❌ (full-text only) | ❌ | ❌ | **No** — full-text only |
| **Lunr** | 2.3.9 | 28.5 KB | **8.1 KB** | ❌ (full-text only) | ❌ | ❌ | **No** — full-text only, no vectors |
| **VantaDB WASM** | 0.8.x | 1.77 MB | **~739 KB transfer** | ✅ HNSW | ✅ BM25 + RRF | ✅ OPFS / IndexedDB / in-mem | ✅ |

Sources:
- Orama gzipped: <https://bundlephobia.com/package/@orama/orama> (75.2 KB min, 23.8 KB gzipped, 2026-08-30)
- MiniSearch gzipped: <https://devpick.co/pkg/minisearch> (5.9 KB gzipped, 2026)
- Lunr gzipped: <https://bundlephobia.com/package/lunr> (28.5 KB min, 8.1 KB gzipped)
- VantaDB WASM gzipped: `vantadb-wasm/pkg/` measured via PowerShell `.NET GzipStream`, 2026-10-04 (fresh `wasm-pack build --release`, wasm-pack 0.15.0)

### Feature gap (what you lose if you switch to Orama for size)

VantaDB's ~739 KB gzipped transfer includes things Orama does not ship:

1. **OPFS persistence** — `connect_persistent`, `connect_idb`, `connect_worker` (worker-backed, durable across reloads)
2. **HNSW vector index** — sub-millisecond k-NN at 100K scale ([p99 441 µs, SIFT1M Balanced Cos](https://github.com/ness-e/Vantadb/blob/main/docs/user/operations/BENCHMARKS.md#-5-impact-of-loop-and-hnsw-distance-optimization-phase-2)), vs Orama's linear-scan `searchVector` (10.3 KB gzipped per bundlephobia export breakdown — for `search`/`searchVector` together)
3. **BM25 + RRF hybrid** — fused vector + text search, not "or", actually combined with reciprocal rank fusion
4. **Capability graphs** — typed nodes + edges (`BFS`, `DFS`, `topological_sort`)
5. **TTL auto-expiry** — records can expire automatically (`expires_at_ms`)
6. **WASM persistence parity with Node** — same API surface as `vantadb-node` (Node has 24 native methods including `compact_wal`, `purge_expired`, `similar_to_key`; WASM subset ships 47 methods)
7. **PITR + WAL** — `export`/`import` JSONL for round-trips; OPFS worker logs writes through a WAL before flushing

If you only need full-text + RAG in-memory with no persistence and ≤1k docs,
Orama at 23.8 KB gzipped is a better fit. If you need any of (1)–(7), the
size delta buys you real capability.

### When VantaDB is the right size

- Browser AI agents that need durable memory across reloads (OPFS)
- Embedded RAG where vector search must scale past linear scan (HNSW)
- Apps that need full-text AND vector in one fused query (RRF)
- Apps that want graph traversal alongside search (capability graph)

### When a JS-only engine is the right size

- Marketing-site search bars, ≤1k docs, ephemeral session data
- Apps already shipping 200+ KB of vendor JS — 23.8 KB is negligible
- Apps that do not need persistence, TTL, or graph queries

---

## 5. Migration / upgrade story

The WASM bundle is shipped from `vantadb-wasm/pkg/` as the npm
`vantadb-wasm` package (a dependency of `vantadb`). Consumers do not interact
with the `.wasm` directly. `pkg/README.md` and the npm metadata (keywords,
description) are copied from this file and `vantadb-wasm/Cargo.toml` by
`wasm-pack` at build time — edit the tracked sources, never `pkg/`:

- **Bundler users** — the wasm-bindgen glue handles it once
  `vite-plugin-wasm` (or equivalent) is installed.
- **Node users** — first call hits disk; subsequent calls in-process.
- **CDN users** — `esm.sh` serves a self-contained `.mjs`; `jsDelivr` does
  not work (Rollup limitation).
- **Self-hosting** — `wasm-pack build --release --target web` then serve the
  generated `vantadb_wasm.js` + `vantadb_wasm_bg.wasm` together.

---

## 6. Future size-reduction levers (not implemented, evaluated)

| Lever | Estimated saving | Status | Why we have not done it yet |
|-------|-----------------:|--------|------------------------------|
| `wasm-opt -Oz --strip-debug` (already on) | — | ✅ shipped | already on |
| Split engine into OPFS vs in-mem core | ~30% if user only needs in-mem | **deferred** | first-class API change, would break ABI |
| Lazy-imported WASM module init | zero — already lazy | ✅ shipped | first call only |
| Custom Rust allocator (mimalloc-rs / dlmalloc) | 5-15 KB | **deferred** | Requires benchmark against canonical P99 first (Regla 9) |
| LTO (`lto = true` in profile.release) | ~50-100 KB | **deferred** | Compile-time cost > benefit at 1.77 MB; revisit if size matters more than dev-loop speed |

Per **Regla 9** ("No optimize without measuring"), none of the deferred
levers ship until a baseline benchmark (`benches/canonical_p99.rs` or a
dedicated `wasm_size` bench) records the current 1.77 MB and the change
demonstrates a measured reduction without regressions.

---

## References

- `vantadb-wasm/Cargo.toml` — features, `wasm-opt = ["-Oz"]`, `crate-type = ["cdylib","lib"]`
- `vantadb-wasm/pkg/` — generated artifacts (gitignored; rebuild, never hand-edit —
  except via `node dev-tools/build-wasm-types.mjs` for the `.d.ts`)
- `vantadb-wasm/src/vantadb_wasm.d.ts` — hand-written TypeScript types
  (source of truth; sync into `pkg/` with the command in §1)
- [`vantadb-wasm/demo/`](https://github.com/ness-e/Vantadb/tree/main/vantadb-wasm/demo) — browser AI agent demo (Transformers.js + OPFS)
- `vantadb-ts/README.md` §"WASM bundle & lazy loading" — runtime-specific recipes
- `docs/user/QUICKSTART.md` §"4. Real Embeddings" — full walkthrough
- `docs/dev/research/research-vantadb-wasm-20260825.md` §H-17 — origin ticket
- `docs/dev/_templates/adr.md` — format reference for any future ADR on lazy-loading strategy

---

**Last reviewed:** 2026-10-04 (WSM-14 — vanta-worker). Numbers re-measured from
a fresh `pkg/` build of the same commit (see §1). Re-verify with bundlephobia
for JS-only competitors; WASM size shifts with code, toolchain, and
`Cargo.toml`/`Cargo.lock` changes.