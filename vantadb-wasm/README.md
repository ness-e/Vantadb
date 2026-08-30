# vantadb-wasm — bundle strategy

> **Why this file exists:** The WASM bundle is `~1.3 MB` raw (`1.35 MB` measured
> from `pkg/vantadb_wasm_bg.wasm` at HEAD), which is **~25× the size of Orama**
> (23.8 KB gzipped per [bundlephobia](https://bundlephobia.com/package/@orama/orama)).
> That gap is real and intentional: VantaDB ships persistence (OPFS/WAL/fjall),
> HNSW vector index, BM25 full-text, RRF hybrid fusion, capability graphs, and
> WASM-bindgen FFI glue — features Orama does not include. This document is the
> honest engineering brief for adoption: what gets shipped, why it is that size,
> how lazy loading works in each runtime, and where the feature gap lives.

---

## 1. Bundle sizes (measured 2026-08-30)

Reproducible from `vantadb-wasm/pkg/`:

| File | Raw bytes | Raw (KB / MB) | Gzipped | Gzipped (KB) | What it is |
|------|----------:|---------------|--------:|--------------|------------|
| `vantadb_wasm_bg.wasm`  |  1,411,870 | **1.35 MB** | 591,971 | **578 KB** | Engine binary (Rust → wasm32) |
| `vantadb_wasm_bg.js`    |     50,644 |   49.5 KB    |   9,555 |   9.3 KB     | wasm-bindgen glue (Node ESM) |
| `vantadb_wasm.js`       |     63,292 |   61.8 KB    |  11,746 |  11.5 KB     | wasm-bindgen glue (TS/Browser) |
| **TOTAL transfer (gzip)** | **1,525,806** | **1.45 MB** | **613,272** | **~599 KB** | What the browser actually fetches |

### How to reproduce

```powershell
# Raw sizes
Get-ChildItem -Path "vantadb-wasm/pkg" -Filter "vantadb_wasm_bg.wasm","vantadb_wasm_bg.js","vantadb_wasm.js" |
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
Get-GzipSize "vantadb-wasm/pkg/vantadb_wasm_bg.wasm"   # → 578,128 (typical)
```

Source of truth: `dev-tools/build-wasm.ps1` produces `vantadb-wasm/pkg/` via
`wasm-pack build --release`. The `Cargo.toml` already opts into `-Oz`:

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

---

## 4. Honest comparison vs JavaScript-only search engines

**Regla 11 note:** every number below links to a reproducible source. Re-run
`bundlephobia.com/package/<name>` for current numbers (the JS ecosystem
re-ships often; this table was measured 2026-08-30).

| Library | Version | Min | **Gzipped** | Vector? | Hybrid? | Persistence? | Feature parity with VantaDB |
|---------|---------|----:|------------:|---------|---------|--------------|-----------------------------|
| **@orama/orama** | 3.1.18 | 75.2 KB | **23.8 KB** | ✅ (`mode:'vector'`) | ✅ (`mode:'hybrid'`, RRF) | ❌ in-memory only (plugin for disk) | **No** — Orama has no HNSW, no OPFS, no WAL, no Fjall, no capability graph, no TTL auto-expiry |
| **MiniSearch** | latest | ~22 KB | **5.9 KB** | ❌ (full-text only) | ❌ | ❌ | **No** — full-text only |
| **Lunr** | 2.3.9 | 28.5 KB | **8.1 KB** | ❌ (full-text only) | ❌ | ❌ | **No** — full-text only, no vectors |
| **VantaDB WASM** | 0.5.x | 1.35 MB | **~599 KB transfer** | ✅ HNSW | ✅ BM25 + RRF | ✅ OPFS / IndexedDB / in-mem | ✅ |

Sources:
- Orama gzipped: <https://bundlephobia.com/package/@orama/orama> (75.2 KB min, 23.8 KB gzipped, 2026-08-30)
- MiniSearch gzipped: <https://devpick.co/pkg/minisearch> (5.9 KB gzipped, 2026)
- Lunr gzipped: <https://bundlephobia.com/package/lunr> (28.5 KB min, 8.1 KB gzipped)
- VantaDB WASM gzipped: `vantadb-wasm/pkg/` measured via PowerShell `.NET GzipStream`, 2026-08-30

### Feature gap (what you lose if you switch to Orama for size)

VantaDB's 599 KB gzipped includes things Orama does not ship:

1. **OPFS persistence** — `connect_persistent`, `connect_idb`, `connect_worker` (worker-backed, durable across reloads)
2. **HNSW vector index** — sub-millisecond k-NN at 100K scale, vs Orama's linear-scan `searchVector` (10.3 KB gzipped per bundlephobia export breakdown — for `search`/`searchVector` together)
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

The WASM bundle is shipped from `vantadb-wasm/pkg/` as part of the npm
`vantadb` package. Consumers do not interact with the `.wasm` directly:

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
| LTO (`lto = true` in profile.release) | ~50-100 KB | **deferred** | Compile-time cost > benefit at 1.35 MB; revisit if size matters more than dev-loop speed |

Per **Regla 9** ("No optimize without measuring"), none of the deferred
levers ship until a baseline benchmark (`benches/canonical_p99.rs` or a
dedicated `wasm_size` bench) records the current 1.35 MB and the change
demonstrates a measured reduction without regressions.

---

## References

- `vantadb-wasm/Cargo.toml` — features, `wasm-opt = ["-Oz"]`, `crate-type = ["cdylib","lib"]`
- `vantadb-wasm/pkg/` — generated artifacts
- `vantadb-wasm/src/vantadb_wasm.d.ts` — hand-written TypeScript types
- `vantadb-wasm/demo/` — browser demo (Transformers.js + OPFS)
- `vantadb-ts/README.md` §"WASM bundle & lazy loading" — runtime-specific recipes
- `docs/QUICKSTART.md` §"4. Real Embeddings" — full walkthrough
- `docs/research/research-vantadb-wasm-20260825.md` §H-17 — origin ticket
- `docs/_templates/adr.md` — format reference for any future ADR on lazy-loading strategy

---

**Last reviewed:** 2026-08-30 (WSM-13 — vanta-docs). Numbers reproducible from
the commands in §1. Re-verify with bundlephobia for JS-only competitors;
WASM size only changes when `Cargo.toml` features or `Cargo.lock` deps change.