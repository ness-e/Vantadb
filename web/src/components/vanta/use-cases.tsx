"use client";

import {
  Bot,
  BookOpen,
  Code2,
  ArrowRight,
  MemoryStick,
  Cpu,
  Sparkles,
  type LucideIcon,
} from "lucide-react";
import { Reveal } from "./reveal";
import { useLanguage } from "@/lib/language-provider";

// §09 — Use Cases
// Three horizontal cards: AI Agents, Local RAG, IDE Tooling. Each with a
// neon icon stamp, font-display title, font-tech description, capability
// tags, and an "Explore →" affordance (button — no destination yet, raises a
// press effect and triggers toast via parent wiring if desired).

type UseCase = {
  icon: LucideIcon;
  title: string;
  tag: string;
  desc: string;
  capabilities: string[];
  metric: string;
  metricLabel: string;
};

const USE_CASES: UseCase[] = [
  {
    icon: Bot,
    title: "AI Agents",
    tag: "memory",
    desc: "Memoria persistente para agentes. put/get/search con namespaces, vectores opcionales y metadata escalar. Tus agentes recuerdan entre sesiones — sin Redis, sin Postgres, sin red.",
    capabilities: ["namespace", "vector", "metadata", "versions"],
    metric: "<2ms",
    metricLabel: "recall por turno",
  },
  {
    icon: BookOpen,
    title: "Local RAG",
    tag: "retrieval",
    desc: "Pipelines RAG 100% locales. Chunk, embed, almacena en VantaDB, hybrid search con RRF, alimenta contexto a tu LLM. Compatible con Ollama, LlamaIndex, LangChain y Haystack.",
    capabilities: ["BM25", "HNSW", "RRF", "JSONL"],
    metric: "100%",
    metricLabel: "Recall@10",
  },
  {
    icon: Code2,
    title: "IDE Tooling",
    tag: "code",
    desc: "Memoria para asistentes de código. Indexa repos, símbolos y snippets como embeddings. Búsqueda semántica sub-millisecond sobre tu codebase, in-process junto al LSP.",
    capabilities: ["CLI", "SDK", "embeddings", "export"],
    metric: "0",
    metricLabel: "network hops",
  },
];

export function UseCases() {
  const { t } = useLanguage();
  const tt = (key: string, fallback: string) => {
    const v = t(key);
    return v === key ? fallback : v;
  };

  return (
    <section
      aria-label={tt("useCases.ariaLabel", "Casos de uso de VantaDB")}
      className="relative border-b-4 border-black bg-[#FBF9F5]  "
    >
      {/* Subtle grid + halftone accents */}
      <div className="pointer-events-none absolute inset-0 grid-tech opacity-50" aria-hidden />
      <div
        className="pointer-events-none absolute -left-20 top-20 h-72 w-72 halftone halftone-fade opacity-15 "
        aria-hidden
      />

      <div className="relative mx-auto max-w-7xl px-4 py-14 sm:px-6">
        {/* Header */}
        <Reveal direction="up">
          <div className="mb-10 flex flex-col gap-3 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:flex-row sm:items-end sm:justify-between">
            <div>
              <h2 className="glitch-hover mt-3 font-display text-4xl uppercase leading-none sm:text-5xl">
                {tt("useCases.title", "Casos de uso")}
              </h2>
              <p className="mt-2 max-w-2xl font-tech text-sm text-[#FBF9F5]/70">
                {tt(
                  "useCases.subtitle",
                  "Tres dominios donde la memoria local híbrida cambia las reglas."
                )}
              </p>
            </div>
            <div className="flex items-center gap-2 font-tech text-[10px] uppercase tracking-wider text-[#FF5500]">
              <MemoryStick className="h-3.5 w-3.5" strokeWidth={2.5} />
              {tt("useCases.tag", "In-process · zero network")}
            </div>
          </div>
        </Reveal>

        {/* Cards grid: 1 col mobile, 3 cols desktop */}
        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3">
          {USE_CASES.map((uc, i) => {
            const Icon = uc.icon;
            return (
              <Reveal key={uc.title} direction="up" delay={i * 80} as="article">
                <article className="press-lg group relative flex h-full flex-col border-4 border-black bg-[#FBF9F5] p-5  ">
                  {/* Panel number */}
                  <span className="absolute -left-2 -top-3 rotate-[-6deg] border-2 border-black bg-black px-2 py-0.5 font-display text-xs uppercase text-[#FF5500]   ">
                    0{i + 1}
                  </span>

                  {/* Icon + tag */}
                  <div className="mb-4 flex items-start justify-between">
                    <span className="inline-flex h-14 w-14 items-center justify-center border-4 border-black bg-[#FF5500] text-black shadow-[3px_3px_0_0_#000] transition-transform group-hover:rotate-[-6deg]  ">
                      <Icon className="h-7 w-7" strokeWidth={2.5} />
                    </span>
                    <span className="border-2 border-black bg-[#FBF9F5] px-2 py-0.5 font-tech text-[9px] font-bold uppercase tracking-[0.2em] text-black   ">
                      {uc.tag}
                    </span>
                  </div>

                  {/* Title */}
                  <h3 className="glitch-hover font-display text-2xl uppercase leading-none text-black ">
                    {tt(`useCases.items.${i}.title`, uc.title)}
                  </h3>

                  {/* Metric strip */}
                  <div className="mt-3 flex items-baseline gap-2 border-l-4 border-[#FF5500] bg-black/5 px-3 py-2 ">
                    <span className="font-display text-xl uppercase text-black ">
                      {uc.metric}
                    </span>
                    <span className="font-tech text-[10px] uppercase tracking-wider text-black/70 ">
                      {tt(`useCases.items.${i}.metricLabel`, uc.metricLabel)}
                    </span>
                  </div>

                  {/* Description */}
                  <p className="mt-3 flex-1 font-tech text-xs leading-relaxed text-black/80 ">
                    {tt(`useCases.items.${i}.desc`, uc.desc)}
                  </p>

                  {/* Capability tags */}
                  <div className="mt-4 flex flex-wrap gap-1.5">
                    {uc.capabilities.map((cap) => (
                      <span
                        key={cap}
                        className="border-2 border-black bg-black px-1.5 py-0.5 font-tech text-[9px] font-bold uppercase tracking-wider text-[#FF5500]   "
                      >
                        {cap}
                      </span>
                    ))}
                  </div>

                  {/* Explore affordance */}
                  <button
                    type="button"
                    className="press mt-5 inline-flex items-center justify-between gap-2 border-4 border-black bg-[#1A1A1A] px-3 py-2.5 font-tech text-xs font-bold uppercase tracking-wider text-[#FBF9F5] transition-colors hover:bg-[#FF5500] hover:text-black     "
                    aria-label={tt("useCases.exploreAria", "Explorar caso de uso")}
                  >
                    <span className="flex items-center gap-1.5">
                      <Cpu className="h-3.5 w-3.5" strokeWidth={2.5} />
                      {tt("useCases.explore", "Explorar")}
                    </span>
                    <ArrowRight className="h-3.5 w-3.5" strokeWidth={2.5} />
                  </button>

                  {/* Bottom kinetic line */}
                  <div className="mt-4 h-1 w-full speed-lines opacity-30" />
                </article>
              </Reveal>
            );
          })}
        </div>
      </div>
    </section>
  );
}

// i18n keys usadas (añadir a dictionaries.ts en Fase 5):
// useCases.ariaLabel, useCases.title, useCases.subtitle, useCases.tag,
// useCases.explore, useCases.exploreAria,
// useCases.items.0.title, useCases.items.0.desc, useCases.items.0.metricLabel,
// useCases.items.1.title, useCases.items.1.desc, useCases.items.1.metricLabel,
// useCases.items.2.title, useCases.items.2.desc, useCases.items.2.metricLabel
