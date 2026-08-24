"use client";

import { Database, FileStack, Layers3, Cpu, ArrowRight, Check, type LucideIcon } from "lucide-react";
import Link from "next/link";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { Reveal } from "@/components/vanta/reveal";
import { VANTA } from "@/components/vanta/vanta-data";
import { useLanguage } from "@/lib/language-provider";

const ICONS: Record<string, LucideIcon> = {
  Database,
  FileStack,
  Layers3,
  Cpu,
};

export default function StoragePage() {
  const { tt } = useLanguage();

  const archLayers = [
    {
      icon: "FileStack",
      label: tt("storagePage.arch.0.label", "LSM-tree"),
      desc: tt("storagePage.arch.0.desc", "Log-structured merge tree. Writes secuenciales, compaction en background. Memtable → SSTables → bloom filters."),
      tone: "vanilla" as const,
    },
    {
      icon: "Database",
      label: tt("storagePage.arch.1.label", "WAL + CRC32C"),
      desc: tt("storagePage.arch.1.desc", "Write-ahead log con checksums CRC32C por entrada. Cada mutación se persiste antes de confirmar. Crash-safe by design."),
      tone: "neon" as const,
    },
    {
      icon: "Layers3",
      label: tt("storagePage.arch.2.label", "HNSW index"),
      desc: tt("storagePage.arch.2.desc", "Hierarchical Navigable Small World graph. Búsqueda vectorial approximada con recall 0.998@10. Derivado — rebuildable desde WAL."),
      tone: "vanilla" as const,
    },
    {
      icon: "Cpu",
      label: tt("storagePage.arch.3.label", "SDK boundary (PyO3)"),
      desc: tt("storagePage.arch.3.desc", "FFI boundary Rust ↔ Python via PyO3. Sin leaks de raw pointers. Stable ABI entre releases. Python SDK expose, Rust core persists."),
      tone: "ink" as const,
    },
  ];

  const components = [
    {
      icon: "FileStack",
      title: tt("storagePage.components.0.title", "LSM-tree storage"),
      body: tt(
        "storagePage.components.0.body",
        "Reemplaza B-tree de SQLite/Postgres para workloads write-heavy. Sequential I/O, no random seeks. Compaction tiered — sin write amplification excesivo."
      ),
    },
    {
      icon: "Database",
      title: tt("storagePage.components.1.title", "WAL durability"),
      body: tt(
        "storagePage.components.1.body",
        "Reemplaza Redis AOF + Postgres WAL. Una sola log structure, no dos. CRC32C por entrada — detecta corruption bit-level. Recovery en milisegundos."
      ),
    },
    {
      icon: "Layers3",
      title: tt("storagePage.components.2.title", "HNSW vector index"),
      body: tt(
        "storagePage.components.2.body",
        "Reemplaza el ANN index de Pinecone/Qdrant. In-process, sin serialization. M=32, efConstruction=200. Recall 0.998@10 con 1M vectores."
      ),
    },
    {
      icon: "Cpu",
      title: tt("storagePage.components.3.title", "PyO3 SDK boundary"),
      body: tt(
        "storagePage.components.3.body",
        "Reemplaza HTTP/gRPC SDK de cloud DBs. Zero network. Una llamada FFI, no una request. Python objects → Rust structs sin copy cuando es posible."
      ),
    },
  ];

  return (
    <div className="animate-rise">
      <PageHeader
        badge="§STORAGE"
        title={tt("storagePage.title", "Single-Binary Storage")}
        subtitle={tt(
          "storagePage.subtitle",
          "LSM-tree → WAL con CRC32C → HNSW index → SDK boundary PyO3. Un binario Rust reemplaza Pinecone + Redis + S3. Sin containers, sin sidecars."
        )}
        tag={tt("storagePage.tag", "Rust binary · PyO3 boundary")}
      />

      {/* Architecture diagram */}
      <PageSection variant="cream">
        <Reveal direction="up">
          <div className="mb-8 flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <span className="inline-flex items-center gap-2 border-2 border-black bg-[#FF5500] px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-black ">
                <span className="h-1.5 w-1.5 bg-black" />
                {tt("storagePage.diagramTag", "Architecture")}
              </span>
              <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
                {tt("storagePage.diagramTitle", "One binary. Four layers.")}
              </h2>
              <p className="mt-2 max-w-2xl font-tech text-xs text-black/70 ">
                {tt(
                  "storagePage.diagramSubtitle",
                  "Datos fluyen top → bottom: write entra por LSM, se persiste al WAL, actualiza el HNSW, se expone via PyO3. Lectura va directo a los índices."
                )}
              </p>
            </div>
          </div>
        </Reveal>

        <Reveal direction="up" delay={80}>
          <div className="border-4 border-black bg-[#FBF9F5] p-4 shadow-[6px_6px_0_0_#000]    sm:p-6">
            <div className="flex flex-col items-stretch gap-3">
              {archLayers.map((layer, i) => {
                const Icon = ICONS[layer.icon] ?? Database;
                const toneClass =
                  layer.tone === "neon"
                    ? "border-4 border-black bg-[#FF5500] text-black shadow-[6px_6px_0_0_#000]  "
                    : layer.tone === "ink"
                      ? "border-4 border-black bg-black text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500] "
                      : "border-4 border-black bg-[#F2EDE2] text-black shadow-[6px_6px_0_0_#000]    ";
                return (
                  <div key={layer.label} className="flex flex-col items-stretch gap-3 sm:flex-row sm:items-center">
                    <div className="flex items-center gap-3 sm:w-72 sm:shrink-0">
                      <span className="inline-flex h-10 w-10 shrink-0 items-center justify-center border-2 border-current">
                        <Icon className="h-5 w-5" strokeWidth={2.5} />
                      </span>
                      <span className="font-tech text-[10px] uppercase tracking-wider opacity-70">
                        {String(i + 1).padStart(2, "0")} / {String(archLayers.length).padStart(2, "0")}
                      </span>
                    </div>
                    <div className={`flex flex-1 flex-col gap-1 p-4 ${toneClass}`}>
                      <div className="flex items-baseline justify-between gap-3">
                        <h3 className="font-display text-xl uppercase leading-none sm:text-2xl">
                          {layer.label}
                        </h3>
                        <span className="font-tech text-[10px] uppercase tracking-wider opacity-70">
                          {tt("storagePage.layer", "Layer")} {i + 1}
                        </span>
                      </div>
                      <p className="font-tech text-xs leading-relaxed opacity-90">{layer.desc}</p>
                    </div>
                    {i < archLayers.length - 1 && (
                      <span className="hidden h-6 w-6 shrink-0 items-center justify-center font-display text-2xl text-black/40  sm:flex sm:-my-2">
                        ↓
                      </span>
                    )}
                  </div>
                );
              })}
            </div>
            <div className="mt-4 flex items-center justify-between border-t-2 border-black/10 pt-3 font-tech text-[10px] uppercase tracking-wider text-black/50  ">
              <span>{tt("storagePage.flowDown", "Write path ↓")}</span>
              <span>{tt("storagePage.flowUp", "Read path ↑ (direct to index)")}</span>
            </div>
          </div>
        </Reveal>
      </PageSection>

      {/* Components grid */}
      <PageSection variant="paper">
        <Reveal direction="up">
          <div className="mb-8 flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <span className="inline-flex items-center gap-2 border-2 border-black bg-black px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-[#FF5500] ">
                <span className="h-1.5 w-1.5 bg-[#FF5500]" />
                {tt("storagePage.componentsTag", "Replaces")}
              </span>
              <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
                {tt("storagePage.componentsTitle", "Pinecone + Redis + S3 → one binary")}
              </h2>
            </div>
          </div>
        </Reveal>

        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2">
          {components.map((c, i) => {
            const Icon = ICONS[c.icon] ?? Database;
            return (
              <Reveal key={c.title} direction="up" delay={i * 80} as="article">
                <article className="press-lg group relative flex h-full flex-col border-4 border-black bg-[#FBF9F5] p-5  ">
                  <span className="absolute -left-2 -top-3 rotate-[-6deg] border-2 border-black bg-black px-2 py-0.5 font-display text-xs uppercase text-[#FF5500]   ">
                    0{i + 1}
                  </span>
                  <div className="mb-4 flex items-center gap-3">
                    <span className="inline-flex h-11 w-11 shrink-0 items-center justify-center border-4 border-black bg-[#FF5500] text-black shadow-[3px_3px_0_0_#000] transition-transform group-hover:rotate-[-6deg]  ">
                      <Icon className="h-5 w-5" strokeWidth={2.5} />
                    </span>
                    <h3 className="glitch-hover font-display text-2xl uppercase leading-none text-black ">
                      {c.title}
                    </h3>
                  </div>
                  <p className="flex-1 font-tech text-xs leading-relaxed text-black/80 ">
                    {c.body}
                  </p>
                  <div className="mt-4 h-1 w-full speed-lines opacity-30" />
                </article>
              </Reveal>
            );
          })}
        </div>
      </PageSection>

      {/* Argument */}
      <PageSection variant="cream">
        <Reveal direction="up">
          <div className="flex flex-col items-center justify-between gap-4 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:flex-row">
            <div className="flex items-center gap-3">
              <Check className="h-5 w-5 shrink-0 text-[#FF5500]" strokeWidth={3} />
              <div>
                <h3 className="font-display text-xl uppercase leading-none text-[#FBF9F5]">
                  {tt("storagePage.ctaTitle", "One binary. Zero moving parts.")}
                </h3>
                <p className="mt-1 font-tech text-xs text-[#FBF9F5]/60">
                  {tt("storagePage.ctaBody", "pip install vantadb-py — Pinecone, Redis y S3 ya no son necesarios.")}
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
      </PageSection>
    </div>
  );
}
