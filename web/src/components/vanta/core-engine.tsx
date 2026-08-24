"use client";

import {
  ArrowRight,
  ArrowDown,
  Database,
  FileStack,
  Layers3,
  Sigma,
  ShieldCheck,
  Cpu,
  TerminalSquare,
  Crosshair,
} from "lucide-react";
import { Reveal } from "./reveal";
import { useLanguage } from "@/lib/language-provider";

// §05 — Core Engine
// Visual pipeline of the VantaDB engine: Input Query → Query Planner →
// (BM25 ∥ HNSW) → RRF Fusion → Ranked Results, with a side rail showing the
// WAL/CRC32C durability layer. All in CSS — no canvas, no images.
// Manga/linocut boxes with rigid black shadows and neon accents.

export function CoreEngine() {
  const { t, tt } = useLanguage();

  return (
    <section
      aria-label={tt("coreEngine.ariaLabel", "Núcleo del motor VantaDB")}
      className="relative border-b-4 border-black bg-[#FBF9F5]  "
    >
      {/* Technical grid overlay */}
      <div className="pointer-events-none absolute inset-0 grid-tech opacity-50" aria-hidden />
      {/* Halftone top-right corner accent */}
      <div
        className="pointer-events-none absolute -right-20 -top-20 h-72 w-72 halftone halftone-fade opacity-20 "
        aria-hidden
      />

      <div className="relative mx-auto max-w-7xl px-4 py-14 sm:px-6">
        {/* Header */}
        <Reveal direction="up">
          <div className="mb-10 flex flex-col gap-3 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:flex-row sm:items-end sm:justify-between">
            <div>
              <h2 className="glitch-hover mt-3 font-display text-4xl uppercase leading-none sm:text-5xl">
                {tt("coreEngine.title", "El motor")}
              </h2>
              <p className="mt-2 max-w-2xl font-tech text-sm text-[#FBF9F5]/70">
                {tt(
                  "coreEngine.subtitle",
                  "Un query viaja por planner, rutas paralelas y fusión RRF — todo en proceso, sin saltos de red."
                )}
              </p>
            </div>
            <span className="font-tech text-[10px] uppercase tracking-wider text-[#FF5500]">
              <span className="mr-1 animate-blink">▌</span>
              Rust 1.94+ · PyO3
            </span>
          </div>
        </Reveal>

        {/* Pipeline diagram */}
        <Reveal direction="up" delay={60}>
          <div className="border-4 border-black bg-[#FBF9F5] p-4 shadow-[6px_6px_0_0_#000]    sm:p-6">
            {/* Stage 1: Input Query */}
            <PipelineStage
              index="01"
              title={tt("coreEngine.stage1.title", "Input Query")}
              tag="db.search_memory(...)"
              tone="cream"
              icon={<TerminalSquare className="h-5 w-5" strokeWidth={2.5} />}
              body={
                <code className="font-tech text-[11px] text-black/80 ">
                  db.search_memory("agent/main", query_vector=[...], top_k=5)
                </code>
              }
            />

            <Connector label="PLANNER" />

            {/* Stage 2: Query Planner */}
            <PipelineStage
              index="02"
              title={tt("coreEngine.stage2.title", "Query Planner")}
              tag="route · plan · dispatch"
              tone="ink"
              icon={<Cpu className="h-5 w-5" strokeWidth={2.5} />}
              body={
                <p className="font-tech text-[11px] text-[#FBF9F5]/80">
                  {tt(
                    "coreEngine.stage2.body",
                    "Inspecciona el query. Si hay vector + texto, lanza ambas rutas en paralelo. Si solo texto, ruta BM25. Si solo vector, ruta HNSW."
                  )}
                </p>
              }
            />

            {/* Branching connector: planner → two parallel paths */}
            <div className="my-3 flex flex-col items-center gap-1 sm:flex-row sm:justify-center sm:gap-12">
              <BranchArrow
                label="LEXICAL"
                className="sm:-rotate-3"
                icon={<FileStack className="h-3.5 w-3.5" strokeWidth={2.5} />}
              />
              <BranchArrow
                label="VECTOR"
                className="sm:rotate-3"
                icon={<Layers3 className="h-3.5 w-3.5" strokeWidth={2.5} />}
              />
            </div>

            {/* Stage 3: Parallel paths */}
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
              <PipelineStage
                index="03a"
                title="BM25"
                tag="inverted index · phrase"
                tone="cream"
                icon={<FileStack className="h-5 w-5" strokeWidth={2.5} />}
                body={
                  <div className="space-y-1.5">
                    <p className="font-tech text-[11px] text-black/80 ">
                      {tt(
                        "coreEngine.stage3a.body",
                        "Term-frequency scoring sobre UTF-8. Índice invertido derivado, reparable desde registros canónicos."
                      )}
                    </p>
                    <div className="flex flex-wrap gap-1">
                      <Tag>BM25</Tag>
                      <Tag>TF·IDF</Tag>
                      <Tag>phrase</Tag>
                    </div>
                  </div>
                }
              />
              <PipelineStage
                index="03b"
                title="HNSW"
                tag="cosine · M · ef_search"
                tone="cream"
                icon={<Layers3 className="h-5 w-5" strokeWidth={2.5} />}
                body={
                  <div className="space-y-1.5">
                    <p className="font-tech text-[11px] text-black/80 ">
                      {tt(
                        "coreEngine.stage3b.body",
                        "Hierarchical small-world graph. Cosine similarity, parámetros M / ef_construction / ef_search. SIMD donde aplica."
                      )}
                    </p>
                    <div className="flex flex-wrap gap-1">
                      <Tag>HNSW</Tag>
                      <Tag>cosine</Tag>
                      <Tag>SIMD</Tag>
                    </div>
                  </div>
                }
              />
            </div>

            {/* Converging connector: two paths → RRF */}
            <div className="my-3 flex flex-col items-center gap-1 sm:flex-row sm:justify-center sm:gap-12">
              <BranchArrow
                label="FUSE"
                className="sm:rotate-3"
                pointing="down"
                icon={<Sigma className="h-3.5 w-3.5" strokeWidth={2.5} />}
              />
              <BranchArrow
                label="FUSE"
                className="sm:-rotate-3"
                pointing="down"
                icon={<Sigma className="h-3.5 w-3.5" strokeWidth={2.5} />}
              />
            </div>

            {/* Stage 4: RRF Fusion */}
            <PipelineStage
              index="04"
              title="RRF Fusion"
              tag="reciprocal rank"
              tone="neon"
              icon={<Sigma className="h-5 w-5" strokeWidth={2.5} />}
              body={
                <div className="space-y-1.5">
                  <p className="font-tech text-[11px] font-bold text-black">
                    score = Σ 1/(k + rank)
                  </p>
                  <p className="font-tech text-[10px] text-black/80">
                    {tt(
                      "coreEngine.stage4.body",
                      "Combina rankings léxicos y vectoriales sin necesidad de scores comparables."
                    )}
                  </p>
                </div>
              }
            />

            <Connector label="RANK" />

            {/* Stage 5: Ranked Results */}
            <PipelineStage
              index="05"
              title={tt("coreEngine.stage5.title", "Ranked Hits")}
              tag="top_k · HNSW p50 1.2ms · 99.8% Recall@10"
              tone="cream"
              icon={<Crosshair className="h-5 w-5" strokeWidth={2.5} />}
              body={
                <div className="flex flex-wrap items-center gap-2">
                  <code className="font-tech text-[11px] text-black/80 ">
                    [hit, hit, hit, ...]
                  </code>
                  <span className="border-2 border-black bg-[#FF5500] px-2 py-0.5 font-tech text-[9px] font-bold uppercase tracking-wider text-black ">
                    1.2ms p50
                  </span>
                </div>
              }
            />
          </div>
        </Reveal>

        {/* Side rail: durability layer (WAL + CRC32C) */}
        <Reveal direction="up" delay={120}>
          <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-3">
            <DurabilityCard
              icon={<ShieldCheck className="h-5 w-5" strokeWidth={2.5} />}
              title="WAL · CRC32C"
              tag="crash-safe"
              body={tt(
                "coreEngine.wal.body",
                "Cada mutación se escribe al Write-Ahead Log con checksums CRC32C antes del commit. Recovery real tras crash, corte de energía o kill del proceso."
              )}
            />
            <DurabilityCard
              icon={<Database className="h-5 w-5" strokeWidth={2.5} />}
              title="Fjall / RocksDB"
              tag="storage backend"
              body={tt(
                "coreEngine.storage.body",
                "Fjall por defecto, RocksDB como fallback. Storage backend configurable con VantaFile abstraction."
              )}
            />
            <DurabilityCard
              icon={<Cpu className="h-5 w-5" strokeWidth={2.5} />}
              title="PyO3 · In-process"
              tag="zero network"
              body={tt(
                "coreEngine.surface.body",
                "Stable src/sdk.rs boundary. Cero hops de red, ejecución in-process. Latencia determinista."
              )}
            />
          </div>
        </Reveal>
      </div>
    </section>
  );
}

// ---------- Subcomponents ----------

function PipelineStage({
  index,
  title,
  tag,
  tone,
  icon,
  body,
}: {
  index: string;
  title: string;
  tag: string;
  tone: "ink" | "cream" | "neon";
  icon: React.ReactNode;
  body: React.ReactNode;
}) {
  const styles =
    tone === "ink"
      ? "bg-black text-[#FBF9F5] border-black   "
      : tone === "neon"
        ? "bg-[#FF5500] text-black border-black "
        : "bg-[#FBF9F5] text-black border-black   ";

  const iconWrap =
    tone === "ink"
      ? "bg-[#FF5500] text-black border-black "
      : "bg-black text-[#FF5500] border-black   ";

  return (
    <div className={`press-lg relative flex flex-col border-4 p-4 ${styles}`}>
      {/* Stage index stamp */}
      <span className="absolute -left-2 -top-3 rotate-[-6deg] border-2 border-black bg-black px-2 py-0.5 font-display text-xs uppercase text-[#FF5500]   ">
        {index}
      </span>
      <div className="mb-2 flex items-center gap-3">
        <span className={`inline-flex h-9 w-9 items-center justify-center border-4 ${iconWrap}`}>
          {icon}
        </span>
        <div className="leading-none">
          <p className="font-display text-lg uppercase">{title}</p>
          <p
            className={`mt-0.5 font-tech text-[9px] uppercase tracking-wider ${
              tone === "ink" ? "text-[#FBF9F5]/60 " : "text-black/60 "
            }`}
          >
            {tag}
          </p>
        </div>
      </div>
      <div className="flex-1">{body}</div>
    </div>
  );
}

function Connector({ label }: { label: string }) {
  return (
    <div className="flex items-center justify-center py-2" aria-hidden>
      <div className="flex flex-col items-center gap-1">
        <span className="font-tech text-[9px] uppercase tracking-[0.3em] text-black/70 ">
          {label}
        </span>
        <span className="flex flex-col items-center text-black ">
          <ArrowDown className="h-4 w-4" strokeWidth={3} />
          <ArrowDown className="-mt-2 h-4 w-4" strokeWidth={3} />
        </span>
      </div>
    </div>
  );
}

function BranchArrow({
  label,
  className = "",
  icon,
  pointing = "down",
}: {
  label: string;
  className?: string;
  icon: React.ReactNode;
  pointing?: "down" | "right";
}) {
  return (
    <div
      className={`flex items-center gap-1.5 font-tech text-[9px] uppercase tracking-[0.2em] text-[#FF5500] ${className}`}
      aria-hidden
    >
      {icon}
      <span>{label}</span>
      {pointing === "down" ? (
        <ArrowDown className="h-3.5 w-3.5 text-black " strokeWidth={3} />
      ) : (
        <ArrowRight className="h-3.5 w-3.5 text-black " strokeWidth={3} />
      )}
    </div>
  );
}

function Tag({ children }: { children: React.ReactNode }) {
  return (
    <span className="border-2 border-black bg-black px-1.5 py-0.5 font-tech text-[9px] font-bold uppercase tracking-wider text-[#FF5500]   ">
      {children}
    </span>
  );
}

function DurabilityCard({
  icon,
  title,
  tag,
  body,
}: {
  icon: React.ReactNode;
  title: string;
  tag: string;
  body: string;
}) {
  return (
    <article className="press group relative flex h-full flex-col border-4 border-black bg-[#FBF9F5] p-4  ">
      <div className="mb-2 flex items-center justify-between">
        <span className="inline-flex h-9 w-9 items-center justify-center border-4 border-black bg-black text-[#FF5500] transition-transform group-hover:rotate-[-6deg]   ">
          {icon}
        </span>
        <span className="border-2 border-black bg-[#FF5500] px-2 py-0.5 font-tech text-[9px] font-bold uppercase tracking-[0.2em] text-black ">
          {tag}
        </span>
      </div>
      <h3 className="font-display text-lg uppercase leading-none text-black ">
        {title}
      </h3>
      <p className="mt-2 font-tech text-[11px] leading-relaxed text-black/80 ">
        {body}
      </p>
    </article>
  );
}

// i18n keys usadas (añadir a dictionaries.ts en Fase 5):
// coreEngine.ariaLabel, coreEngine.title, coreEngine.subtitle,
// coreEngine.stage1.title, coreEngine.stage2.title, coreEngine.stage2.body,
// coreEngine.stage3a.body, coreEngine.stage3b.body, coreEngine.stage4.body,
// coreEngine.stage5.title,
// coreEngine.wal.body, coreEngine.storage.body, coreEngine.surface.body
