import type { Metadata } from "next";
import { CASE_STUDIES } from "@/components/vanta/vanta-data";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ slug: string }>;
}): Promise<Metadata> {
  const { slug } = await params;
  const cs = CASE_STUDIES.find((c) => c.slug === slug);
  if (!cs) {
    return {
      title: "Not Found · VantaDB",
      description:
        "El caso de estudio que buscas no existe. Explora casos reales de VantaDB: agentes, edge, IDE.",
      robots: { index: false, follow: false },
    };
  }
  const url = `https://vantadb.dev/case-studies/${cs.slug}`;
  return {
    title: `${cs.title} · VantaDB Case Study`,
    description: cs.summary,
    openGraph: {
      title: `${cs.title} · VantaDB Case Study`,
      description: cs.summary,
      url,
      siteName: "VantaDB",
      type: "article",
    },
    alternates: {
      canonical: url,
    },
  };
}

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
