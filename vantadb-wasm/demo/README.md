# VantaDB WASM — Browser AI Agent Demo

A self-contained browser demo showing an AI Agent using VantaDB WASM for persistent vector memory.

## How it works

- **VantaDB WASM** stores messages as vector embeddings in-memory with OPFS persistence
- **Transformers.js** generates embeddings on-device in the browser (no server needed)
- The agent stores each message, searches for similar past memories, and displays them

## Run

```bash
# From this directory:
npx serve .

# Or with Python:
python3 -m http.server 8080
# Then open http://localhost:8080
```

Serve the entire `vantadb-wasm/` directory so the demo can import `../pkg/vantadb_wasm.js`.

## Requirements

- A browser that supports WASM and OPFS (Chrome 86+, Edge 86+, Firefox 111+, Safari 15.2+)
- ~100MB free memory for the Transformers.js model
- First load downloads the embedding model (~23MB)

## Files

- `index.html` — Chat interface with dark theme
- `app.js` — Main application logic
- `package.json` — `npm run dev` script

## Bundle size & lazy loading

The WASM engine binary (`../pkg/vantadb_wasm_bg.wasm`) is **~1.35 MB raw
/ ~578 KB gzipped** (measured 2026-08-30). It is **not loaded on page
load** — the wasm-bindgen glue lazy-imports it on the first method call
(`VantaDB.create()`).

For production self-hosting outside this demo:

```bash
# Rebuild for plain HTML/JS browsers (uses fetch() + WebAssembly.instantiateStreaming)
wasm-pack build --release --target web --out-dir pkg

# Or for bundlers (Vite/Webpack/esbuild) — needs vite-plugin-wasm
wasm-pack build --release --target bundler --out-dir pkg
```

Full bundle strategy (feature flags, comparison with Orama / MiniSearch / Lunr,
CDN vs self-host tradeoffs): see [`../README.md`](../README.md).

| Asset this demo fetches | Size |
|--------------------------|-----:|
| `vantadb_wasm_bg.wasm` (engine) | ~1.35 MB raw / ~578 KB gzipped |
| `vantadb_wasm.js` (glue)        | ~62 KB raw / ~12 KB gzipped |
| Transformers.js model (Xenova/all-MiniLM-L6-v2) | ~23 MB (first load, cached after) |
