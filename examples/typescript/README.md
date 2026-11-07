# VantaDB TypeScript / Browser examples

Local examples for the `vantadb` TypeScript SDK and the WASM OPFS build.
Everything runs in-process; no remote API calls, and demo data stays on
your machine (the browser demo uses a local HTTP server for the assets).

## Build from a fresh checkout (repo root)

```bash
# 1. WASM glue for the bundler target (needed by the TS SDK)
wasm-pack build vantadb-wasm --target bundler --out-dir pkg

# 2. TypeScript SDK
npm --prefix vantadb-ts ci
npm --prefix vantadb-ts run build

# 3. Node.js target for the Bun example (needed by its file: dep)
wasm-pack build vantadb-wasm --target nodejs --out-dir pkg-node

# 4. This example package (links vantadb via file: deps)
npm --prefix examples/typescript ci

# 5. Web target for the browser demo
wasm-pack build vantadb-wasm --target web --out-dir pkg-web
```

## Node.js quickstart (from repo root)

```bash
node examples/typescript/node-quickstart.mjs
```

In-memory engine (`Client.create()`): put, get by `{namespace, key}`,
hybrid `search` with fixture vectors (deterministic, not embeddings).

## Bun quickstart (from repo root)

```bash
bun run examples/typescript/bun-quickstart.ts
```

Same CRUD + hybrid-search flow executed by Bun, but against the
**lower-level official WASM binding** (`vantadb-wasm`, `nodejs` target —
`new Client({})`, positional `get(ns, key)`), not the higher-level
`vantadb` SDK wrapper used by the Node example. Requires step 3 above
(`--target nodejs`).

## Smoke tests (from this directory)

```bash
npm test             # node --test smoke.test.mjs
```

Stdlib `node:test` + `assert/strict`: CRUD lifecycle, hybrid search,
closed-client behavior, quickstart summary and CLI subprocess runs —
all against the real SDK.

## Browser (OPFS persistence)

Serve the repo root so the WASM asset resolves, then open the page:

```bash
python -m http.server 8766 --bind 127.0.0.1        # from repo root
# open http://127.0.0.1:8766/examples/typescript/browser-opfs.html
```

Buttons: **Open (persistent)** → **Put + Save** → **Close** →
**Reopen + Verify**. Requires a secure context (localhost counts) and a
browser that exposes `navigator.storage.getDirectory`.

## Limitations

- `Client.create()` is **in-memory** — Node records do not persist to
  disk automatically; browser persistence is an **OPFS snapshot**
  (`save()` before `close()`), not a WAL.
- Fixture vectors are deterministic placeholders, not embeddings.
- Verified on Windows with Node.js 26.8.1 and Bun 1.3.14; the OPFS flow was
  exercised in Chromium. Node emitted an experimental WASM-import warning.
  Other Node versions, Firefox and Safari were not validated in this run.
- The provider examples in `../../vantadb-ts/examples/` support
  `--check-imports` — an SDK import + CRUD smoke only, not an LLM
  integration certification.
