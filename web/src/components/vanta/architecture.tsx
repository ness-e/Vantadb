"use client";

import {
  ArrowRight,
  ArrowDown,
  Database,
  FileStack,
  ShieldCheck,
  Sigma,
  Layers3,
  Cpu,
} from "lucide-react";
import type { View } from "./vanta-data";
import { Reveal } from "./reveal";
import { useLanguage } from "@/lib/language-provider";

export function Architecture({ onNavigate }: { onNavigate: (v: View) => void }) {
  const { t } = useLanguage();
  const tt = (key: string, fallback: string) => {
    const v = t(key);
    return v === key ? fallback : v;
  };
  return (
    <section
      aria-label={tt("architecture.ariaLabel", "Pipeline de retrieval")}
      className="relative border-b-4 border-black bg-[#FBF9F5]  "
    >
      <div className="pointer-events-none absolute inset-0 grid-tech opacity-50" aria-hidden />
      <div className="relative mx-auto max-w-7xl px-4 py-14 sm:px-6">
        {/* Header */}
        <Reveal direction="up">
          <div className="mb-10 flex flex-col gap-3 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:flex-row sm:items-end sm:justify-between">
            <div>
            <h2 className="glitch-hover mt-3 font-display text-4xl uppercase leading-none sm:text-5xl">
              {tt("architecture.title", "Retrieval Pipeline")}
            </h2>
            <p className="mt-2 max-w-2xl font-tech text-sm text-[#FBF9F5]/70">
              {tt(
                "architecture.subtitle",
                "Cómo un query se convierte en resultados ordenados. El motor planifica y ejecuta las rutas léxica y vectorial en paralelo, luego las fusiona con Reciprocal Rank Fusion."
              )}
            </p>
          </div>
          <span className="font-tech text-[10px] uppercase tracking-wider text-[#FF5500]">
            {tt("architecture.tag", "BM25 ∥ HNSW → RRF")}
          </span>
        </div>
        </Reveal>

        {/* Flow diagram */}
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-12 lg:items-stretch">
          {/* Input query */}
          <FlowNode
            className="lg:col-span-2"
            icon={<Database className="h-6 w-6" strokeWidth={2.5} />}
            title={tt("architecture.node.query.title", "Query")}
            sub={tt("architecture.node.query.sub", "namespace + key")}
            tone="ink"
          >
            <code className="font-tech text-[10px] text-[#FF5500]">
              db.search(...)
            </code>
          </FlowNode>

          <FlowArrow className="hidden lg:flex" />

          {/* Parallel paths */}
          <div className="lg:col-span-5 grid grid-cols-1 gap-4">
            <FlowNode
              icon={<FileStack className="h-6 w-6" strokeWidth={2.5} />}
              title={tt("architecture.node.bm25.title", "BM25 · Lexical Path")}
              sub={tt("architecture.node.bm25.sub", "inverted index · phrase positions")}
              tone="cream"
            >
              <p className="font-tech text-[11px] text-black/70 ">
                {tt(
                  "architecture.node.bm25.body",
                  "Term-frequency scoring over UTF-8 payloads. Derived text index, repairable from canonical records."
                )}
              </p>
            </FlowNode>
            <FlowNode
              icon={<Layers3 className="h-6 w-6" strokeWidth={2.5} />}
              title={tt("architecture.node.hnsw.title", "HNSW · Vector Path")}
              sub={tt("architecture.node.hnsw.sub", "cosine · M · ef_search")}
              tone="cream"
            >
              <p className="font-tech text-[11px] text-black/70 ">
                {tt(
                  "architecture.node.hnsw.body",
                  "Native hierarchical small-world graph. Cosine similarity with configurable construction & search parameters."
                )}
              </p>
            </FlowNode>
          </div>

          <FlowArrow className="hidden lg:flex" />

          {/* RRF Fusion */}
          <FlowNode
            className="lg:col-span-2"
            icon={<Sigma className="h-6 w-6" strokeWidth={2.5} />}
            title={tt("architecture.node.rrf.title", "RRF Fusion")}
            sub={tt("architecture.node.rrf.sub", "reciprocal rank")}
            tone="neon"
          >
            <p className="font-tech text-[10px] font-bold text-black">
              score = Σ 1/(k + rank)
            </p>
          </FlowNode>

          {/* Output */}
          <div className="lg:col-span-1 flex items-center justify-center">
            <div className="hidden lg:block">
              <ArrowRight className="h-8 w-8 text-black " strokeWidth={3} />
            </div>
          </div>
        </div>

        {/* Second row: WAL + output */}
        <div className="mt-4 grid grid-cols-1 gap-4 lg:grid-cols-12">
          <FlowNode
            className="lg:col-span-5"
            icon={<ShieldCheck className="h-6 w-6" strokeWidth={2.5} />}
            title={tt("architecture.node.wal.title", "WAL Recovery · CRC32C")}
            sub={tt("architecture.node.wal.sub", "crash-safe · append-only")}
            tone="ink"
          >
            <p className="font-tech text-[11px] text-[#FBF9F5]/80 ">
              {tt(
                "architecture.node.wal.body",
                "Every mutation hits the Write-Ahead Log with CRC32C checksums before commit. On restart, the log replays canonical mutations in order — durable across crashes, power loss, and process kills."
              )}
            </p>
          </FlowNode>

          <FlowArrow className="hidden lg:flex" />

          <FlowNode
            className="lg:col-span-3"
            icon={<Cpu className="h-6 w-6" strokeWidth={2.5} />}
            title={tt("architecture.node.surface.title", "Embedded Surface")}
            sub={tt("architecture.node.surface.sub", "Rust + PyO3")}
            tone="cream"
          >
            <p className="font-tech text-[11px] text-black/70 ">
              {tt(
                "architecture.node.surface.body",
                "Stable src/sdk.rs boundary. Zero network hops, in-process execution."
              )}
            </p>
          </FlowNode>

          <FlowArrow className="hidden lg:flex" />

          <FlowNode
            className="lg:col-span-3"
            icon={<ArrowDown className="h-6 w-6" strokeWidth={2.5} />}
            title={tt("architecture.node.hits.title", "Ranked Hits")}
            sub={tt("architecture.node.hits.sub", "top_k · 1.2ms")}
            tone="neon"
          >
            <p className="font-tech text-[11px] font-bold text-black">
              {tt(
                "architecture.node.hits.body",
                "100% Recall@10 on validated 10K–100K synthetic sets."
              )}
            </p>
          </FlowNode>
        </div>

        {/* CTA strip */}
        <div className="mt-10 flex flex-col items-center justify-between gap-4 border-4 border-black bg-[#FF5500] p-6 shadow-[6px_6px_0_0_#000]   sm:flex-row">
          <div>
            <h3 className="font-display text-2xl uppercase leading-none text-black sm:text-3xl">
              {tt("architecture.cta.title", "Read the full architecture")}
            </h3>
            <p className="mt-1 font-tech text-xs text-black/80">
              {tt(
                "architecture.cta.body",
                "Durability model, retrieval mechanisms, and the SDK boundary — in the docs."
              )}
            </p>
          </div>
          <button
            onClick={() => onNavigate("docs")}
            className="press inline-flex shrink-0 items-center gap-2 border-4 border-black bg-black px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-[#FBF9F5]"
          >
            {tt("architecture.cta.button", "Open Quickstart")}
            <ArrowRight className="h-4 w-4" strokeWidth={2.5} />
          </button>
        </div>
      </div>
    </section>
  );
}

function FlowNode({
  icon,
  title,
  sub,
  tone,
  className,
  children,
}: {
  icon: React.ReactNode;
  title: string;
  sub: string;
  tone: "ink" | "cream" | "neon";
  className?: string;
  children?: React.ReactNode;
}) {
  const styles =
    tone === "ink"
      ? "bg-black text-[#FBF9F5] border-black   "
      : tone === "neon"
        ? "bg-[#FF5500] text-black border-black "
        : "bg-[#FBF9F5] text-black border-black   ";
  return (
    <div
      className={`press-lg relative flex flex-col border-4 p-4 ${styles} ${className ?? ""}`}
    >
      <div className="mb-2 flex items-center gap-2">
        <span
          className={`inline-flex h-9 w-9 items-center justify-center border-4 border-black ${
            tone === "ink"
              ? "bg-[#FF5500] text-black "
              : "bg-black text-[#FF5500]   "
          }`}
        >
          {icon}
        </span>
        <div className="leading-none">
          <p className="font-display text-lg uppercase">{title}</p>
          <p
            className={`font-tech text-[9px] uppercase tracking-wider ${
              tone === "ink" ? "text-[#FBF9F5]/60 " : "text-black/60 "
            }`}
          >
            {sub}
          </p>
        </div>
      </div>
      <div className="flex-1">{children}</div>
    </div>
  );
}

function FlowArrow({ className }: { className?: string }) {
  return (
    <div className={`items-center justify-center ${className ?? ""}`}>
      <ArrowRight className="h-8 w-8 text-black " strokeWidth={3} />
    </div>
  );
}
