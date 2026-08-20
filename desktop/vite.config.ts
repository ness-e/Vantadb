import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import wasm from "vite-plugin-wasm";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// WASM-03: wasm-bindgen ships inline_js snippets (used by the IndexedDB
// bridge in vantadb-wasm/src/idb.rs) as side-effect IIFEs with no exports,
// but the generated wasm imports `__vanta_ensure_idb_bridge` from that
// module — the bundler must provide it in the import object or instantiation
// fails ("function import requires a callable"). The IIFE already registers
// globalThis.vantaIdbStorage when the module evaluates, so the export is an
// idempotent no-op (same contract as the WASM-01 no-modules require shim).
function wasmSnippetBridge(): Plugin {
  return {
    name: "vanta-wasm-snippet-bridge",
    transform(code, id) {
      if (id.includes("snippets/vantadb-wasm-") && id.endsWith("/inline0.js")) {
        return code + "\nexport function __vanta_ensure_idb_bridge() {}\n";
      }
      return code;
    },
  };
}

// https://vite.dev/config/
// WEB-05: `vite build --mode web` builds the embedded-server console (base
// `/dashboard/`, outDir `dist-web`); the default mode keeps the Tauri build
// (frontendDist `../dist`, base `/`) byte-identical. No Tauri plugin exists
// in this config, so "sin plugin Tauri en ese modo" is trivially satisfied.
//
// WASM-03: `vite build --mode wasm` builds the 100% browser standalone
// console (outDir `dist-wasm`): WASM engine + OPFS persistence, no server.
// Only this mode bundles the wasm-bindgen glue (vite-plugin-wasm transforms
// the `import * as wasm from "*.wasm"` into fetch+instantiate — never native
// ESM wasm, which stable Chrome/Edge reject, see WASM-01) and uses
// `build.target: "esnext"` (the plugin's documented alternative to
// vite-plugin-top-level-await). Tauri/web builds keep the glue externalized
// (WASM-02) so the lazy import never executes there.
export default defineConfig(({ mode }) => {
  const web = mode === "web";
  const wasmMode = mode === "wasm";
  return {
    plugins: [
      react(),
      tailwindcss(),
      ...(wasmMode ? [wasm(), wasmSnippetBridge()] : []),
    ],
    base: web ? "/dashboard/" : undefined,
    build: {
      ...(web ? { outDir: "dist-web" } : {}),
      ...(wasmMode ? { outDir: "dist-wasm", target: "esnext" } : {}),
      rollupOptions: {
        ...(wasmMode
          ? {}
          : {
              // WASM-02: the wasm-bindgen glue (vantadb-wasm/pkg) imports its
              // .wasm via ESM, which Vite 7 cannot bundle without a plugin.
              // The WASM backend is only reachable in `--mode wasm` (WASM-03
              // wires the standalone build); Tauri/web builds never execute
              // it, so the module is externalized — the lazy import stays as
              // a runtime import() that is never called in these modes.
              external: [/vantadb-wasm\/pkg\/vantadb_wasm\.js/],
            }),
      },
    },

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent Vite from obscuring rust errors
    clearScreen: false,
    // 2. tauri expects a fixed port, fail if that port is not available
    server: {
      port: 1420,
      strictPort: true,
      host: host || false,
      hmr: host
        ? {
            protocol: "ws",
            host,
            port: 1421,
          }
        : undefined,
      watch: {
        // 3. tell Vite to ignore watching `src-tauri`
        ignored: ["**/src-tauri/**"],
      },
    },
  };
});
