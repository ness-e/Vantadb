import { defineConfig } from "vitest/config";
import wasm from "vite-plugin-wasm";

export default defineConfig({
  plugins: [wasm()],
  test: {
    include: ["src/**/__tests__/**/*.test.ts"],
    server: {
      deps: {
        inline: ["vantadb-wasm"],
      },
    },
  },
});
