interface SEOProps {
  title: string;
  description: string;
  ogTitle?: string;
  ogDescription?: string;
  ogImage?: string;
  canonical?: string;
  noindex?: boolean;
}

type MetaTag =
  | { title: string; name?: never; property?: never; content?: never }
  | { title?: never; name: string; content: string }
  | { title?: never; property: string; content: string };

export function seo({
  title,
  description,
  ogTitle,
  ogDescription,
  ogImage,
  canonical,
  noindex,
}: SEOProps) {
  const tags: MetaTag[] = [
    { title },
    { name: "description", content: description },
    { property: "og:title", content: ogTitle || title },
    { property: "og:description", content: ogDescription || description },
    { name: "twitter:title", content: ogTitle || title },
    { name: "twitter:description", content: ogDescription || description },
  ];

  if (ogImage) {
    tags.push({ property: "og:image", content: ogImage });
    tags.push({ name: "twitter:image", content: ogImage });
  }

  const links: Array<{ rel: string; href: string }> = [];

  if (canonical) {
    links.push({ rel: "canonical", href: canonical });
  }

  if (noindex) {
    tags.push({ name: "robots", content: "noindex, nofollow" });
  }

  return { meta: tags, links };
}
