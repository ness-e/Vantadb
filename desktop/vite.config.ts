import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
// WEB-05: `vite build --mode web` builds the embedded-server console (base
// `/dashboard/`, outDir `dist-web`); the default mode keeps the Tauri build
// (frontendDist `../dist`, base `/`) byte-identical. No Tauri plugin exists
// in this config, so "sin plugin Tauri en ese modo" is trivially satisfied.
export default defineConfig(({ mode }) => {
  const web = mode === "web";
  return {
    plugins: [react(), tailwindcss()],
    base: web ? "/dashboard/" : undefined,
    build: {
      ...(web ? { outDir: "dist-web" } : {}),
      rollupOptions: {
        // WASM-02: the wasm-bindgen glue (vantadb-wasm/pkg) imports its .wasm
        // via ESM, which Vite 7 cannot bundle without a plugin. The WASM
        // backend is only reachable in `--mode wasm` (WASM-03 wires the
        // standalone build); Tauri/web builds never execute it, so the module
        // is externalized — the lazy import stays as a runtime import() that
        // is never called in these modes.
        external: [/vantadb-wasm\/pkg\/vantadb_wasm\.js/],
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
