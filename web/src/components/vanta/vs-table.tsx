"use client";

import { Check, ArrowRight } from "lucide-react";
import Link from "next/link";
import { Reveal } from "./reveal";
import { VANTA } from "./vanta-data";
import { useLanguage } from "@/lib/language-provider";

interface VsRow {
  featureKey: string;
  featureFallback: string;
  vantadb: string;
  pinecone: string;
  weaviate: string;
  chroma: string;
  vantadbHighlight?: boolean;
}

const ROWS: VsRow[] = [
  {
    featureKey: "vsTable.row.0.feature",
    featureFallback: "Latency",
    vantadb: "1.2ms",
    pinecone: "~50-150ms",
    weaviate: "~20-80ms",
    chroma: "~5-30ms",
    vantadbHighlight: true,
  },
  {
    featureKey: "vsTable.row.1.feature",
    featureFallback: "Network hops",
    vantadb: "0",
    pinecone: "1+",
    weaviate: "1+",
    chroma: "0-1",
    vantadbHighlight: true,
  },
  {
    featureKey: "vsTable.row.2.feature",
    featureFallback: "Deployment",
    vantadb: "pip install",
    pinecone: "Cloud account",
    weaviate: "Docker cluster",
    chroma: "pip install",
    vantadbHighlight: true,
  },
  {
    featureKey: "vsTable.row.3.feature",
    featureFallback: "Crash recovery",
    vantadb: "WAL + CRC32C",
    pinecone: "Managed",
    weaviate: "WAL",
    chroma: "Limited",
    vantadbHighlight: true,
  },
  {
    featureKey: "vsTable.row.4.feature",
    featureFallback: "Hybrid search",
    vantadb: "BM25 + HNSW · RRF",
    pinecone: "Vector only*",
    weaviate: "BM25 + HNSW",
    chroma: "Vector only",
    vantadbHighlight: true,
  },
  {
    featureKey: "vsTable.row.5.feature",
    featureFallback: "Data egress",
    vantadb: "None",
    pinecone: "Cloud",
    weaviate: "Self-host or cloud",
    chroma: "None",
    vantadbHighlight: true,
  },
  {
    featureKey: "vsTable.row.6.feature",
    featureFallback: "Cost @ 1M vectors",
    vantadb: "$0",
    pinecone: "$1,800/mo",
    weaviate: "$600/mo",
    chroma: "$0",
    vantadbHighlight: true,
  },
];

export function VsTable() {
  const { t } = useLanguage();
  const tt = (key: string, fallback: string) => {
    const v = t(key);
    return v === key ? fallback : v;
  };

  return (
    <section
      aria-label={tt("vsTable.tagHeader", "Comparison · Head-to-head")}
      className="relative border-b-4 border-black bg-[#F2EDE2]  "
    >
      <div className="pointer-events-none absolute inset-0 grid-tech opacity-30" aria-hidden />
      <div className="relative mx-auto max-w-7xl px-4 py-14 sm:px-6">
        <Reveal direction="up">
          <div className="mb-8 flex flex-col gap-3 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:flex-row sm:items-end sm:justify-between">
            <div>
              <span className="inline-flex items-center gap-2 border-2 border-[#FF5500] bg-[#FF5500] px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-black">
                <span className="h-1.5 w-1.5 bg-black" />
                {tt("vsTable.tagHeader", "Comparison · Head-to-head")}
              </span>
              <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-[0.9] sm:text-4xl lg:text-5xl">
                {tt("vsTable.title", "VantaDB vs Cloud DBs")}
              </h2>
              <p className="mt-3 max-w-2xl font-tech text-xs text-[#FBF9F5]/70 sm:text-sm">
                {tt("vsTable.subtitle", "Same dataset, same queries. VantaDB column in neon — the rest in grayscale. The difference isn't marginal.")}
              </p>
            </div>
            <span className="flex items-center gap-2 font-tech text-[10px] uppercase tracking-wider text-[#FF5500]">
              <span className="h-2 w-2 animate-flicker bg-[#FF5500]" />
              {tt("vsTable.tag", "Side-by-side")}
            </span>
          </div>
        </Reveal>

        <Reveal direction="up" delay={80}>
          <div className="overflow-x-auto border-4 border-black bg-[#FBF9F5] shadow-[6px_6px_0_0_#000]   ">
            <table className="w-full min-w-[640px] border-collapse font-tech text-xs">
              <thead>
                <tr className="border-b-4 border-black ">
                  <th className="px-4 py-4 text-left font-bold uppercase tracking-wider text-black ">
                    {tt("vsTable.thFeature", "Feature")}
                  </th>
                  <th className="border-l-4 border-[#FF5500] bg-[#FF5500] px-4 py-4 text-left font-bold uppercase tracking-wider text-black">
                    <span className="flex items-center gap-2">
                      <span className="h-1.5 w-1.5 bg-black" />
                      {tt("vsTable.thVantadb", "VantaDB")}
                    </span>
                  </th>
                  <th className="border-l-2 border-black px-4 py-4 text-left font-bold uppercase tracking-wider text-black/70  ">
                    {tt("vsTable.thPinecone", "Pinecone")}
                  </th>
                  <th className="border-l-2 border-black px-4 py-4 text-left font-bold uppercase tracking-wider text-black/70  ">
                    {tt("vsTable.thWeaviate", "Weaviate")}
                  </th>
                  <th className="border-l-2 border-black px-4 py-4 text-left font-bold uppercase tracking-wider text-black/70  ">
                    {tt("vsTable.thChroma", "Chroma")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {ROWS.map((row, i) => (
                  <tr
                    key={row.featureKey}
                    className={`border-b-2 border-black/20 transition-colors hover:bg-[#FF5500]/5   ${i % 2 === 0 ? "bg-[#F2EDE2]/40 " : ""}`}
                  >
                    <td className="px-4 py-3 font-bold uppercase tracking-wider text-black ">
                      {tt(row.featureKey, row.featureFallback)}
                    </td>
                    <td className="border-l-4 border-[#FF5500] bg-[#FF5500]/10 px-4 py-3 font-bold text-black ">
                      <span className="inline-flex items-center gap-2">
                        <Check className="h-3 w-3 text-[#FF5500]" strokeWidth={3} />
                        {row.vantadb}
                      </span>
                    </td>
                    <td className="border-l-2 border-black/20 px-4 py-3 text-black/70  ">
                      {row.pinecone}
                    </td>
                    <td className="border-l-2 border-black/20 px-4 py-3 text-black/70  ">
                      {row.weaviate}
                    </td>
                    <td className="border-l-2 border-black/20 px-4 py-3 text-black/70  ">
                      {row.chroma}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Reveal>

        <Reveal direction="up" delay={120}>
          <div className="mt-8 flex flex-col items-center justify-between gap-4 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:flex-row">
            <div className="flex items-center gap-3">
              <Check className="h-5 w-5 shrink-0 text-[#FF5500]" strokeWidth={3} />
              <div>
                <h3 className="font-display text-xl uppercase leading-none text-[#FBF9F5]">
                  {tt("vsTable.ctaTitle", "Try it in 30 seconds")}
                </h3>
                <p className="mt-1 font-tech text-xs text-[#FBF9F5]/60">
                  {tt("vsTable.ctaBody", "pip install vantadb-py — in-process, no account, no API key.")}
                </p>
              </div>
            </div>
            <Link
              href={VANTA.pypi}
              target="_blank"
              rel="noopener noreferrer"
              className="press-neon btn-neon-glow inline-flex shrink-0 items-center gap-2 border-4 border-[#FBF9F5] bg-[#FF5500] px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black"
            >
              pip install vantadb-py
              <ArrowRight className="h-4 w-4" strokeWidth={2.5} />
            </Link>
          </div>
        </Reveal>
      </div>
    </section>
  );
}
