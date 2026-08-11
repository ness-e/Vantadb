import { defineConfig } from "vitest/config";

// vite-plugin-wasm is intentionally absent: it rewrites `import ... from
// "*.wasm"` into the virtual module `__vite-plugin-wasm-helper`, which
// vitest >=4 cannot resolve in Node (vitest-dev/vitest#6723). The wasm
// dependency (`vantadb-wasm`) is instead loaded natively by Node (>=22
// supports ESM wasm imports), so vitest must NOT inline or transform it.
export default defineConfig({
  test: {
    include: ["src/**/__tests__/**/*.test.ts"],
    testTimeout: 30000,
    server: {
      deps: {
        external: [/vantadb-wasm/],
      },
    },
    coverage: {
      provider: "v8",
      include: ["src/**/*.ts"],
      exclude: ["src/**/__tests__/**"],
      reporter: ["text", "json"],
      reportOnFailure: true,
    },
  },
});
