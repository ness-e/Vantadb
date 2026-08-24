"use client";

import { Zap, ArrowRight } from "lucide-react";
import Link from "next/link";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { Reveal } from "@/components/vanta/reveal";
import { useLanguage } from "@/lib/language-provider";

interface LatencyRow {
  system: string;
  p50: string;
  p99: string;
  note: string;
  pct: number; // bar width 0-100 (lower latency = wider bar)
  vantage: boolean;
}

export default function LatencyPage() {
  const { t, tt } = useLanguage();

  const rows: LatencyRow[] = [
    {
      system: "VantaDB Rust Core",
      p50: "1.2ms",
      p99: "—",
      note: tt("latencyPage.rows.0.note", "In-process · zero network · BENCHMARKS §1"),
      pct: 100,
      vantage: true,
    },
    {
      system: "VantaDB Python SDK",
      p50: "39.74ms",
      p99: "58.2ms",
      note: tt("latencyPage.rows.1.note", "PyO3 boundary · 1 thread · §7"),
      pct: 88,
      vantage: false,
    },
    {
      system: "Chroma (local)",
      p50: "0.94ms",
      p99: "3.35ms",
      note: tt("latencyPage.rows.2.note", "Measured by competitive_bench.py · §7"),
      pct: 76,
      vantage: false,
    },
    {
      system: "Weaviate (Docker)",
      p50: "—",
      p99: "—",
      note: tt("latencyPage.rows.3.note", "Not measured locally · no number published"),
      pct: 58,
      vantage: false,
    },
    {
      system: "Pinecone (cloud)",
      p50: "—",
      p99: "—",
      note: tt("latencyPage.rows.4.note", "Managed service · not measured by harness"),
      pct: 30,
      vantage: false,
    },
    {
      system: "Pinecone (cloud, cold)",
      p50: "—",
      p99: "—",
      note: tt("latencyPage.rows.5.note", "Managed service · not measured by harness"),
      pct: 12,
      vantage: false,
    },
  ];

  return (
    <div className="animate-rise">
      <PageHeader
        badge="§LATENCY"
        title={tt("latencyPage.title", "In-Process Latency")}
        subtitle={tt(
          "latencyPage.subtitle",
          "VantaDB Rust Core: 1.2ms p50 (HNSW · 10K). Python SDK: 39.74ms p50. Cloud DBs: not measured by the harness."
        )}
        tag={tt("latencyPage.tag", "1.2ms p50 · 0 network hops")}
      />

      {/* Comparison table */}
      <PageSection variant="cream">
        <Reveal direction="up">
          <div className="mb-8 flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <span className="inline-flex items-center gap-2 border-2 border-black bg-[#FF5500] px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-black ">
                <span className="h-1.5 w-1.5 bg-black" />
                {tt("latencyPage.tableTag", "Contextual")}
              </span>
              <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
                {tt("latencyPage.tableTitle", "p50 / p99 head-to-head")}
              </h2>
              <p className="mt-2 max-w-2xl font-tech text-xs text-black/70 ">
                {tt(
                  "latencyPage.tableSubtitle",
                  "VantaDB Rust Core p50 from BENCHMARKS §1 (HNSW · 10K). SDK and Chroma cells from §7 competitive benchmark. Cloud DBs not measured by the local harness — marked —."
                )}
              </p>
            </div>
          </div>
        </Reveal>

        <Reveal direction="up" delay={80}>
          <div className="overflow-x-auto border-4 border-black bg-[#FBF9F5] shadow-[6px_6px_0_0_#000]   ">
            <table className="w-full min-w-[640px] border-collapse font-tech text-xs">
              <thead>
                <tr className="border-b-4 border-black ">
                  <th className="px-4 py-3 text-left font-bold uppercase tracking-wider text-black ">
                    {tt("latencyPage.thSystem", "System")}
                  </th>
                  <th className="px-4 py-3 text-right font-bold uppercase tracking-wider text-black ">
                    {tt("latencyPage.thP50", "p50")}
                  </th>
                  <th className="px-4 py-3 text-right font-bold uppercase tracking-wider text-black ">
                    {tt("latencyPage.thP99", "p99")}
                  </th>
                  <th className="px-4 py-3 text-left font-bold uppercase tracking-wider text-black ">
                    {tt("latencyPage.thNote", "Notes")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {rows.map((r, i) => (
                  <tr
                    key={r.system}
                    className={`border-b-2 border-black/20  ${i % 2 === 0 ? "bg-[#F2EDE2]/40 " : ""} ${r.vantage ? "bg-[#FF5500]/10" : ""}`}
                  >
                    <td className="px-4 py-3 font-bold uppercase tracking-wider text-black ">
                      <span className="inline-flex items-center gap-2">
                        {r.vantage && <Zap className="h-3 w-3 text-[#FF5500]" strokeWidth={3} />}
                        {r.system}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-right font-bold text-black ">
                      {r.vantage ? (
                        <span className="text-[#FF5500]">{r.p50}</span>
                      ) : (
                        r.p50
                      )}
                    </td>
                    <td className="px-4 py-3 text-right text-black/70 ">
                      {r.p99}
                    </td>
                    <td className="px-4 py-3 text-black/60 ">
                      {r.note}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Reveal>
      </PageSection>

      {/* CSS bar chart */}
      <PageSection variant="paper">
        <Reveal direction="up">
          <div className="mb-8 flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <span className="inline-flex items-center gap-2 border-2 border-black bg-black px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-[#FF5500] ">
                <span className="h-1.5 w-1.5 bg-[#FF5500]" />
                {tt("latencyPage.chartTag", "Visualized")}
              </span>
              <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
                {tt("latencyPage.chartTitle", "Lower is faster")}
              </h2>
              <p className="mt-2 max-w-2xl font-tech text-xs text-black/70 ">
                {tt(
                  "latencyPage.chartSubtitle",
                  "Cada barra representa p50 latency relativa. VantaDB Rust Core en neon — el resto en escala de grises."
                )}
              </p>
            </div>
          </div>
        </Reveal>

        <Reveal direction="up" delay={80}>
          <div className="space-y-3 border-4 border-black bg-[#FBF9F5] p-6 shadow-[6px_6px_0_0_#000]   ">
            {rows.map((r, i) => (
              <div key={r.system} className="flex items-center gap-3">
                <div className="w-32 shrink-0 font-tech text-[10px] font-bold uppercase tracking-wider text-black/70  sm:w-48">
                  {r.system}
                </div>
                <div className="relative h-8 flex-1 border-2 border-black bg-[#F2EDE2]  ">
                  <div
                    className={`relative h-full ${r.vantage ? "bg-[#FF5500]" : "bg-black/70 "}`}
                    style={{
                      width: `${r.pct}%`,
                      boxShadow: r.vantage ? "4px 4px 0 0 #000" : "none",
                    }}
                  >
                    <span className="absolute right-2 top-1/2 -translate-y-1/2 font-tech text-[10px] font-bold uppercase text-[#FBF9F5] mix-blend-difference">
                      {r.p50}
                    </span>
                  </div>
                </div>
                <span className="hidden w-8 shrink-0 text-right font-tech text-[10px] text-black/40  sm:inline">
                  {String(i + 1).padStart(2, "0")}
                </span>
              </div>
            ))}
          </div>
        </Reveal>
      </PageSection>

      {/* Argument */}
      <PageSection variant="ink">
        <Reveal direction="up">
          <div className="mx-auto max-w-3xl text-center">
            <span className="inline-flex items-center gap-2 border-2 border-[#FF5500] bg-[#FF5500] px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-black">
              <span className="h-1.5 w-1.5 bg-black" />
              {tt("latencyPage.argumentTag", "The argument")}
            </span>
            <h2 className="glitch-hover mt-4 font-display text-4xl uppercase leading-[0.9] text-[#FBF9F5] sm:text-5xl">
              {tt("latencyPage.argumentTitle", "Zero network round-trips.")}
            </h2>
            <p className="mt-5 font-tech text-sm leading-relaxed text-[#FBF9F5]/80 sm:text-base">
              {tt(
                "latencyPage.argumentBody",
                "Una query a una DB cloud hace TCP handshake → TLS → auth → routing → query → response → close. Mínimo 4 saltos. VantaDB ejecuta en el mismo proceso que tu código Python: un FFI call, un memcmp, un return. No es optimization — es arquitectura. La latencia del cloud no es un bug de Pinecone. Es el costo inevitable de poner la DB en otra máquina."
              )}
            </p>
            <div className="mt-8">
              <Link
                href="/benchmarks"
                className="press-neon btn-neon-glow inline-flex items-center gap-2 border-4 border-[#FBF9F5] bg-[#FF5500] px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black"
              >
                {tt("latencyPage.ctaBtn", "See full benchmarks")}
                <ArrowRight className="h-4 w-4" strokeWidth={2.5} />
              </Link>
            </div>
          </div>
        </Reveal>
      </PageSection>
    </div>
  );
}
