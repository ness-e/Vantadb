import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "standalone",
  /* config options here */
  reactStrictMode: true,
  // REVIEW-18: fija el workspace root para Turbopack — evita warning de
  // package-lock stray fuera del repo (p.ej. C:\Users\Eros\package-lock.json).
  turbopack: {
    root: __dirname,
  },
};

export default nextConfig;
