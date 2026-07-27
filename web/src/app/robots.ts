import type { MetadataRoute } from "next";

/**
 * robots.ts — Next.js native robots.txt generator.
 * Allows all crawlers, points to sitemap.
 */
export default function robots(): MetadataRoute.Robots {
  return {
    rules: [
      {
        userAgent: "*",
        allow: "/",
        disallow: ["/api/", "/_next/"],
      },
    ],
    sitemap: "https://vantadb.dev/sitemap.xml",
    host: "https://vantadb.dev",
  };
}
