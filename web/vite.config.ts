/// <reference types="vitest" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import tsConfigPaths from "vite-tsconfig-paths";
import { TanStackRouterVite } from "@tanstack/router-plugin/vite";

export default defineConfig({
  plugins: [TanStackRouterVite(), react(), tailwindcss(), tsConfigPaths()],
  base: "/",
  optimizeDeps: {
    // Forzar que Vite pre-empaquete GSAP como una unidad cohesiva
    // sin tree-shaking agresivo que elimina el registerPlugin como side-effect
    include: [
      "gsap",
      "gsap/ScrollTrigger",
      "gsap/TextPlugin",
      "@gsap/react",
    ],
  },
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: "./src/test/setup.ts",
    css: true,
    exclude: ["e2e/**", "node_modules/**"],
  },
});
