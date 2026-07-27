import wasm from "vite-plugin-wasm";
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { TanStackRouterVite } from "@tanstack/router-plugin/vite";
// rollup-plugin-visualizer: install with `npm i -D rollup-plugin-visualizer`
import { visualizer } from "rollup-plugin-visualizer";

export default defineConfig({
  resolve: {
    tsconfigPaths: true,
  },
  plugins: [
    TanStackRouterVite(),
    react(),
    tailwindcss(),
    wasm(),
    ...(process.env.ANALYZE ? [visualizer({ filename: "dist/stats.html", open: true })] : []),
  ],
  server: {
    fs: {
      allow: [".."],
    },
  },
  base: "/",
  build: {
    cssMinify: "esbuild",
    rollupOptions: {
      output: {
        manualChunks(id: string) {
          if (id.includes("node_modules/react-dom") || id.includes("node_modules/react/"))
            return "vendor-react";
          if (id.includes("@tanstack/react-router") || id.includes("@tanstack/react-query"))
            return "vendor-router";
        },
      },
    },
  },
  optimizeDeps: {
    include: [],
  },
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: "./src/test/setup.ts",
    css: true,
    exclude: ["e2e/**", "node_modules/**"],
    coverage: { provider: "v8", reporter: ["text", "html"] },
  },
});
