import type { MetadataRoute } from "next";

/**
 * sitemap.ts — Next.js native sitemap generator.
 * Lists all live App Router routes for SEO indexing.
 */
export default function sitemap(): MetadataRoute.Sitemap {
  const baseUrl = "https://vantadb.dev";
  const lastModified = new Date();

  // Static routes
  const staticRoutes: MetadataRoute.Sitemap = [
    { url: `${baseUrl}/`, lastModified, changeFrequency: "weekly", priority: 1.0 },
    { url: `${baseUrl}/benchmarks`, lastModified, changeFrequency: "monthly", priority: 0.9 },
    { url: `${baseUrl}/docs`, lastModified, changeFrequency: "weekly", priority: 0.9 },
    { url: `${baseUrl}/engine`, lastModified, changeFrequency: "monthly", priority: 0.8 },
    { url: `${baseUrl}/architecture`, lastModified, changeFrequency: "monthly", priority: 0.8 },
    { url: `${baseUrl}/playground`, lastModified, changeFrequency: "monthly", priority: 0.7 },
    { url: `${baseUrl}/changelog`, lastModified, changeFrequency: "monthly", priority: 0.6 },
    { url: `${baseUrl}/why-vantadb`, lastModified, changeFrequency: "monthly", priority: 0.8 },
    { url: `${baseUrl}/pricing`, lastModified, changeFrequency: "monthly", priority: 0.8 },
    { url: `${baseUrl}/security`, lastModified, changeFrequency: "monthly", priority: 0.7 },
    { url: `${baseUrl}/use-cases`, lastModified, changeFrequency: "monthly", priority: 0.7 },
    { url: `${baseUrl}/cost`, lastModified, changeFrequency: "monthly", priority: 0.6 },
    { url: `${baseUrl}/maint`, lastModified, changeFrequency: "monthly", priority: 0.6 },
    { url: `${baseUrl}/solutions/ai-agents`, lastModified, changeFrequency: "monthly", priority: 0.8 },
    { url: `${baseUrl}/solutions/local-rag`, lastModified, changeFrequency: "monthly", priority: 0.8 },
    { url: `${baseUrl}/solutions/ai-ide-tooling`, lastModified, changeFrequency: "monthly", priority: 0.8 },
    { url: `${baseUrl}/blog`, lastModified, changeFrequency: "weekly", priority: 0.7 },
    { url: `${baseUrl}/case-studies`, lastModified, changeFrequency: "monthly", priority: 0.7 },
    { url: `${baseUrl}/about/company`, lastModified, changeFrequency: "yearly", priority: 0.5 },
    { url: `${baseUrl}/about/team`, lastModified, changeFrequency: "yearly", priority: 0.5 },
    { url: `${baseUrl}/about/community`, lastModified, changeFrequency: "monthly", priority: 0.5 },
    { url: `${baseUrl}/about/contact`, lastModified, changeFrequency: "yearly", priority: 0.5 },
  ];

  // Dynamic blog posts
  const blogSlugs = [
    "introducing-vantadb",
    "how-hybrid-search-works",
    "sqlite-for-ai-agents",
    "why-i-built-vantadb-local-memory-engine",
  ];
  const blogRoutes: MetadataRoute.Sitemap = blogSlugs.map((slug) => ({
    url: `${baseUrl}/blog/${slug}`,
    lastModified,
    changeFrequency: "yearly",
    priority: 0.6,
  }));

  // Dynamic case studies
  const caseSlugs = [
    "agent-local-memory-ollama",
    "rag-edge-device",
    "ide-semantic-search",
  ];
  const caseRoutes: MetadataRoute.Sitemap = caseSlugs.map((slug) => ({
    url: `${baseUrl}/case-studies/${slug}`,
    lastModified,
    changeFrequency: "yearly",
    priority: 0.6,
  }));

  return [...staticRoutes, ...blogRoutes, ...caseRoutes];
}
