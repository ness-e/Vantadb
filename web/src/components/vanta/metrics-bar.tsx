"use client";

import { Activity, Gauge, Crosshair, Network, TrendingUp, type LucideIcon } from "lucide-react";
import { Reveal } from "./reveal";
import { CountUpStat } from "@/hooks/count-up";
import { useLanguage } from "@/lib/language-provider";

// §03 — Metrics Bar
// High-impact animated stats strip on cream "paper" background. Five bold
// metrics pulled straight from BENCH-01 + SIFT1M (vanta-data.ts). Each metric
// counts up from 0 when scrolled into view.

type Metric = {
  value: string;
  label: string;
  sub: string;
  icon: LucideIcon;
};

const METRICS: Metric[] = [
  { value: "1.2ms", label: "HNSW p50 · 10K", sub: "Rust Core · 128d", icon: Activity },
  { value: "3,636", label: "Peak QPS", sub: "SIFT1M Balanced Cos · 100K", icon: Gauge },
  { value: "100%", label: "Recall@10", sub: "Validated 10K–100K", icon: Crosshair },
  { value: "0", label: "Network hops", sub: "In-process · embedded", icon: Network },
  { value: "2.80x", label: "SIFT1M speedup", sub: "Balanced L2 · 100K", icon: TrendingUp },
];

export function MetricsBar() {
  const { t } = useLanguage();
  const tt = (key: string, fallback: string) => {
    const v = t(key);
    return v === key ? fallback : v;
  };

  return (
    <section
      aria-label={tt("metricsBar.ariaLabel", "Métricas de rendimiento")}
      className="relative border-b-4 border-black bg-[#F2EDE2]  "
    >
      {/* Technical drafting grid overlay */}
      <div className="pointer-events-none absolute inset-0 grid-tech opacity-50" aria-hidden />

      <div className="relative mx-auto max-w-7xl px-4 py-12 sm:px-6 sm:py-14">
        {/* Compact header */}
        <Reveal direction="up">
          <div className="mb-8 flex flex-col gap-3 border-4 border-black bg-[#FBF9F5] p-5 shadow-[6px_6px_0_0_#000]    sm:flex-row sm:items-end sm:justify-between">
            <div>
              <span className="inline-flex items-center gap-2 glow-box-neon border-2 border-black bg-[#FF5500] px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-black ">
                <span className="h-1.5 w-1.5 bg-black" />
                §03
              </span>
              <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
                {tt("metricsBar.title", "Métricas en vivo")}
              </h2>
              <p className="mt-2 max-w-xl font-tech text-xs text-black/70  sm:text-sm">
                {tt(
                  "metricsBar.subtitle",
                  "Mediciones reales de BENCH-01 y SIFT1M en hardware de referencia."
                )}
              </p>
            </div>
            <div className="flex items-center gap-2 font-tech text-[10px] uppercase tracking-wider text-black/50 ">
              <span className="h-2 w-2 animate-flicker bg-[#FF5500]" />
              {tt("metricsBar.tag", "In-process · single-thread")}
            </div>
          </div>
        </Reveal>

        {/* Metrics grid: 2 cols mobile, 4 cols tablet+, 5 cols xl */}
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-4 sm:gap-4 lg:grid-cols-5">
          {METRICS.map((m, i) => {
            const Icon = m.icon;
            return (
              <Reveal key={m.label} direction="up" delay={i * 70} as="article">
                <article className="group relative flex h-full flex-col border-4 border-black bg-[#FBF9F5] p-4 transition-all hover:-translate-y-1  ">
                  {/* Icon row */}
                  <div className="mb-3 flex items-center justify-between">
                    <span className="inline-flex h-9 w-9 items-center justify-center border-4 border-black bg-[#FF5500] text-black shadow-[3px_3px_0_0_#000] transition-transform group-hover:rotate-[-6deg]  ">
                      <Icon className="h-4 w-4" strokeWidth={2.5} />
                    </span>
                    <span className="font-tech text-[9px] uppercase tracking-[0.2em] text-black/40 ">
                      {String(i + 1).padStart(2, "0")}
                    </span>
                  </div>

                  {/* Number */}
                  <span className="font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
                    <CountUpStat value={m.value} />
                  </span>

                  {/* Label + sub */}
                  <span className="mt-2 font-tech text-[11px] font-bold uppercase tracking-wider text-black ">
                    {tt(`metricsBar.metrics.${i}.label`, m.label)}
                  </span>
                  <span className="mt-0.5 font-tech text-[9px] uppercase tracking-wider text-[#FF5500]">
                    {tt(`metricsBar.metrics.${i}.sub`, m.sub)}
                  </span>

                  {/* Kinetic baseline */}
                  <div className="mt-auto pt-3">
                    <div className="h-1 w-full speed-lines opacity-25" />
                  </div>
                </article>
              </Reveal>
            );
          })}
        </div>

        {/* Footnote: hardware profile */}
        <Reveal direction="up" delay={120}>
          <p className="mt-6 flex flex-wrap items-center gap-2 font-tech text-[10px] uppercase tracking-wider text-black/50 ">
            <span className="h-1.5 w-1.5 bg-[#FF5500]" />
            {tt(
              "metricsBar.footnote",
              "Hardware: 12-core CPU @ 3.5GHz, AVX2, Windows 11 / Ubuntu 22.04 LTS."
            )}
          </p>
        </Reveal>
      </div>
    </section>
  );
}

// i18n keys usadas (añadir a dictionaries.ts en Fase 5):
// metricsBar.ariaLabel, metricsBar.title, metricsBar.subtitle, metricsBar.tag,
// metricsBar.footnote,
// metricsBar.metrics.0.label, metricsBar.metrics.0.sub,
// metricsBar.metrics.1.label, metricsBar.metrics.1.sub,
// metricsBar.metrics.2.label, metricsBar.metrics.2.sub,
// metricsBar.metrics.3.label, metricsBar.metrics.3.sub,
// metricsBar.metrics.4.label, metricsBar.metrics.4.sub
