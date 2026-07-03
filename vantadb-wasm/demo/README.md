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
