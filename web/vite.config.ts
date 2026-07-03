import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import tsConfigPaths from "vite-tsconfig-paths";
import { TanStackRouterVite } from "@tanstack/router-plugin/vite";

export default defineConfig({
  plugins: [TanStackRouterVite(), react(), tailwindcss(), tsConfigPaths()],
  base: "/",
  build: {
    rollupOptions: {
      output: {
        manualChunks(id: string) {
          if (id.includes("node_modules/react-dom") || id.includes("node_modules/react/")) return "vendor-react";
          if (id.includes("@tanstack/react-router") || id.includes("@tanstack/react-query")) return "vendor-router";
          if (id.includes("node_modules/three")) return "vendor-three";
          if (id.includes("node_modules/gsap") || id.includes("@gsap/react")) return "vendor-gsap";
          if (id.includes("node_modules/motion")) return "vendor-motion";
        },
      },
    },
  },
  optimizeDeps: {
    // Forzar que Vite pre-empaquete GSAP como una unidad cohesiva
    // sin tree-shaking agresivo que elimina el registerPlugin como side-effect
    include: ["gsap", "gsap/ScrollTrigger", "gsap/TextPlugin", "@gsap/react"],
  },
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: "./src/test/setup.ts",
    css: true,
    exclude: ["e2e/**", "node_modules/**"],
  },
});
