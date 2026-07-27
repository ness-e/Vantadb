"use client";

import {
  Code2,
  Boxes,
  Binary,
  Terminal,
  Plug,
  type LucideIcon,
} from "lucide-react";
import { Reveal } from "./reveal";
import { useLanguage } from "@/lib/language-provider";

// §10 — Ecosystem
// Grid of integration chips grouped by category: Languages, Storage,
// Algorithms, Interfaces. Compact, hover-glow badges. Cream background.

type Chip = string;

type Category = {
  title: string;
  icon: LucideIcon;
  chips: Chip[];
};

const CATEGORIES: Category[] = [
  {
    title: "Languages",
    icon: Code2,
    chips: ["Python 3.11+", "Rust 1.94+", "PyO3", "Fjall", "RocksDB"],
  },
  {
    title: "Algorithms",
    icon: Binary,
    chips: ["HNSW", "BM25", "RRF", "Cosine", "SIMD"],
  },
  {
    title: "Storage",
    icon: Boxes,
    chips: ["WAL", "CRC32C", "JSONL", "VantaFile", "namespace"],
  },
  {
    title: "Interfaces",
    icon: Terminal,
    chips: ["CLI", "Server", "Python SDK", "Rust crate", "PyPI wheel"],
  },
];

export function Ecosystem() {
  const { t } = useLanguage();
  const tt = (key: string, fallback: string) => {
    const v = t(key);
    return v === key ? fallback : v;
  };

  return (
    <section
      aria-label={tt("ecosystem.ariaLabel", "Ecosistema e integraciones")}
      className="relative border-b-4 border-black bg-[#FBF9F5]  "
    >
      {/* Technical grid overlay */}
      <div className="pointer-events-none absolute inset-0 grid-tech opacity-50" aria-hidden />

      <div className="relative mx-auto max-w-7xl px-4 py-14 sm:px-6">
        {/* Header */}
        <Reveal direction="up">
          <div className="mb-10 flex flex-col gap-3 border-4 border-black bg-[#FBF9F5] p-6 shadow-[6px_6px_0_0_#000]    sm:flex-row sm:items-end sm:justify-between">
            <div>
              <span className="inline-flex items-center gap-2 glow-box-neon border-2 border-black bg-[#FF5500] px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-black ">
                <Plug className="h-3 w-3" strokeWidth={3} />
                §10
              </span>
              <h2 className="glitch-hover mt-3 font-display text-4xl uppercase leading-none text-black  sm:text-5xl">
                {tt("ecosystem.title", "Ecosistema")}
              </h2>
              <p className="mt-2 max-w-2xl font-tech text-sm text-black/70 ">
                {tt(
                  "ecosystem.subtitle",
                  "Las piezas que componen VantaDB y las superficies que expone a tu stack."
                )}
              </p>
            </div>
            <div className="flex items-center gap-2 font-tech text-[10px] uppercase tracking-wider text-black/50 ">
              <span className="h-2 w-2 animate-flicker bg-[#FF5500]" />
              {tt("ecosystem.tag", "Apache 2.0 · open source")}
            </div>
          </div>
        </Reveal>

        {/* Categories grid: 1 col mobile, 2 cols tablet, 4 cols desktop */}
        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
          {CATEGORIES.map((cat, i) => {
            const Icon = cat.icon;
            return (
              <Reveal key={cat.title} direction="up" delay={i * 80} as="article">
                <article className="group flex h-full flex-col border-4 border-black bg-[#FBF9F5] p-4 transition-all hover:-translate-y-1  ">
                  {/* Category header */}
                  <div className="mb-3 flex items-center gap-2 border-b-2 border-black/15 pb-3 ">
                    <span className="inline-flex h-9 w-9 items-center justify-center border-4 border-black bg-[#FF5500] text-black shadow-[3px_3px_0_0_#000] transition-transform group-hover:rotate-[-6deg]  ">
                      <Icon className="h-4 w-4" strokeWidth={2.5} />
                    </span>
                    <div className="leading-none">
                      <p className="font-display text-base uppercase text-black ">
                        {tt(`ecosystem.categories.${i}.title`, cat.title)}
                      </p>
                      <p className="mt-0.5 font-tech text-[9px] uppercase tracking-wider text-[#FF5500]">
                        {String(cat.chips.length).padStart(2, "0")} chips
                      </p>
                    </div>
                  </div>

                  {/* Chips list */}
                  <ul className="flex flex-wrap gap-1.5">
                    {cat.chips.map((chip, j) => (
                      <li key={chip}>
                        <span
                          className="inline-flex items-center gap-1 border-2 border-black bg-black px-2 py-1 font-tech text-[10px] font-bold uppercase tracking-wider text-[#FBF9F5] transition-colors hover:border-[#FF5500] hover:bg-[#FF5500] hover:text-black     "
                        >
                          <span className="h-1 w-1 bg-[#FF5500]" aria-hidden />
                          {tt(`ecosystem.categories.${i}.chips.${j}`, chip)}
                        </span>
                      </li>
                    ))}
                  </ul>

                  {/* Bottom kinetic line */}
                  <div className="mt-auto pt-3">
                    <div className="h-1 w-full speed-lines opacity-25" />
                  </div>
                </article>
              </Reveal>
            );
          })}
        </div>

        {/* Footer strip: distribution summary */}
        <Reveal direction="up" delay={120}>
          <div className="mt-6 flex flex-wrap items-center justify-between gap-4 border-4 border-black bg-black p-4 shadow-[6px_6px_0_0_#FF5500] ">
            <div className="flex items-center gap-3">
              <Plug className="h-5 w-5 text-[#FF5500]" strokeWidth={2.5} />
              <p className="font-display text-lg uppercase text-[#FBF9F5]">
                {tt("ecosystem.distribution", "Distribución multi-superficie")}
              </p>
            </div>
            <div className="flex flex-wrap items-center gap-3 font-tech text-[10px] uppercase tracking-wider text-[#FBF9F5]/60">
              <span className="border-2 border-[#FF5500] px-2 py-0.5 text-[#FF5500]">pip</span>
              <span className="border-2 border-[#FBF9F5]/30 px-2 py-0.5">cargo</span>
              <span className="border-2 border-[#FBF9F5]/30 px-2 py-0.5">binary</span>
              <span className="border-2 border-[#FBF9F5]/30 px-2 py-0.5">wheel · win/mac/lin</span>
            </div>
          </div>
        </Reveal>
      </div>
    </section>
  );
}

// i18n keys usadas (añadir a dictionaries.ts en Fase 5):
// ecosystem.ariaLabel, ecosystem.title, ecosystem.subtitle, ecosystem.tag,
// ecosystem.distribution,
// ecosystem.categories.0.title,
// ecosystem.categories.0.chips.0..4,
// ecosystem.categories.1.title,
// ecosystem.categories.1.chips.0..4,
// ecosystem.categories.2.title,
// ecosystem.categories.2.chips.0..4,
// ecosystem.categories.3.title,
// ecosystem.categories.3.chips.0..4
