import type { Metadata } from "next";
import { BLOG_POSTS } from "@/components/vanta/vanta-data";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ slug: string }>;
}): Promise<Metadata> {
  const { slug } = await params;
  const post = BLOG_POSTS.find((p) => p.slug === slug);
  if (!post) {
    return {
      title: "Not Found · VantaDB",
      description:
        "El artículo de blog que buscas no existe. Explora el blog de VantaDB sobre hybrid search, local-first y Rust.",
      robots: { index: false, follow: false },
    };
  }
  const url = `https://vantadb.dev/blog/${post.slug}`;
  return {
    title: `${post.title} · VantaDB Blog`,
    description: post.excerpt,
    openGraph: {
      title: `${post.title} · VantaDB Blog`,
      description: post.excerpt,
      url,
      siteName: "VantaDB",
      type: "article",
      publishedTime: post.date,
      authors: [post.author],
    },
    alternates: {
      canonical: url,
    },
  };
}

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
