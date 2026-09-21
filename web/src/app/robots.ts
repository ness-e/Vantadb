import type { MetadataRoute } from "next";
import { SITE_URL } from "@/lib/site-config";

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
    sitemap: `${SITE_URL}/sitemap.xml`,
    host: SITE_URL,
  };
}
