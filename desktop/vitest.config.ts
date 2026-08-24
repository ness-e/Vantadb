// DESKTOP-26: config de vitest. jsdom para tests de componentes (RTL);
// tests node-only declaran `// @vitest-environment node` (projection.worker).
// globals:true → RTL auto-cleanup corre sin setup file.
import react from "@vitejs/plugin-react";
import { defaultExclude, defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    globals: true,
    // Los *.test.ts que importan "node:test" corren con `node --test`
    // (convención previa a DESKTOP-26) — vitest no los descubre.
    exclude: [
      ...defaultExclude,
      "src/consolidate-core.test.ts",
      "src/importDrop.test.ts",
      "src/vanta-wasm-map.test.ts",
      "src/vanta-http-map.test.ts",
      "src/vanta-deep-link.test.ts",
      "src/retrieval-core.test.ts",
      "src/indices-core.test.ts",
      "src/components/export/status-report.test.ts",
      "src/components/export/export-jsonl.test.ts",
    ],
  },
});
