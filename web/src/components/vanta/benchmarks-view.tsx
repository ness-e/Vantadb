"use client";

import {
  Activity,
  Cpu,
  Gauge,
  Zap,
  Terminal,
  ArrowRight,
  TrendingDown,
  Boxes,
} from "lucide-react";
import { BENCH01, SIFT1M, VANTA, COMPETITIVE_TABLE } from "./vanta-data";
import type { View } from "./vanta-data";
import { LatencyComparator } from "./latency-comparator";
import { Reveal } from "./reveal";

export function BenchmarksView({ onNavigate }: { onNavigate: (v: View) => void }) {
  return (
    <div className="animate-rise">
      {/* Header */}
      <section className="relative overflow-hidden border-b-4 border-black bg-[#FBF9F5]  ">
        <div className="pointer-events-none absolute inset-0 grid-tech" aria-hidden />
        <div className="pointer-events-none absolute -right-20 top-0 h-72 w-72 halftone halftone-fade opacity-25" aria-hidden />
        <div className="relative mx-auto max-w-7xl px-4 py-12 sm:px-6 sm:py-16">
          <div className="flex flex-wrap items-center gap-3">
            <span className="inline-flex items-center gap-2 border-4 border-black bg-[#FF5500] px-3 py-1 font-display text-sm uppercase text-black shadow-[4px_4px_0_0_#000]  ">
              <Activity className="h-4 w-4" strokeWidth={2.5} />
              Performance
            </span>
            <span className="font-tech text-[10px] uppercase tracking-[0.3em] text-black/50 ">
              BENCH-01 · SIFT1M · certification hardware
            </span>
          </div>
          <h1 className="glitch-hover mt-5 font-display text-6xl uppercase leading-[0.85] text-black  sm:text-8xl">
            Bench
            <br />
            <span className="text-outline-neon glow-neon">marks</span>
          </h1>
          <p className="mt-5 max-w-2xl border-l-4 border-[#FF5500] pl-4 font-tech text-sm leading-relaxed text-black/80  sm:text-base">
            A formal Python-native performance benchmark suite captures ingestion
            throughput and query latency under realistic single-threaded workloads. No
            network. No warm-up tricks. In-process, on bare metal.
          </p>

          {/* Quick stats */}
          <Reveal direction="up" delay={100}>
          <div className="mt-8 grid grid-cols-2 gap-3 sm:grid-cols-4">
            {[
              { v: "95", l: "ingestion ops/sec", s: "SDK · 10K · 128d", icon: Boxes },
              { v: "1.2ms", l: "HNSW p50 · 10K", s: "Rust Core · 128d", icon: Gauge },
              { v: "2.80x", l: "best speedup", s: "SIFT1M Balanced L2", icon: TrendingDown },
              { v: "3,636", l: "QPS peak", s: "Balanced Cos 100K", icon: Zap },
            ].map((s, i) => {
              const Icon = s.icon;
              return (
                <Reveal key={s.l} direction="scale" delay={150 + i * 60}>
                <div
                  className="press flex h-full flex-col border-4 border-black bg-[#FBF9F5] p-4  "
                >
                  <Icon className="h-5 w-5 text-[#FF5500]" strokeWidth={2.5} />
                  <span className="mt-2 font-display text-3xl leading-none text-black ">
                    {s.v}
                  </span>
                  <span className="font-tech text-[10px] font-bold uppercase tracking-wider text-black ">
                    {s.l}
                  </span>
                  <span className="font-tech text-[9px] uppercase tracking-wider text-black/50 ">
                    {s.s}
                  </span>
                </div>
                </Reveal>
              );
            })}
          </div>
          </Reveal>
        </div>
      </section>

      {/* BENCH-01 Table */}
      <section className="relative border-b-4 border-black bg-[#F2EDE2]">
        <div className="pointer-events-none absolute inset-0 grid-tech opacity-50" aria-hidden />
        <div className="relative mx-auto max-w-7xl px-4 py-14 sm:px-6">
          <Reveal direction="up">
          <SectionHeader
            tag="§01"
            title={BENCH01.title}
            subtitle={BENCH01.subtitle}
          />
          </Reveal>

          {/* Table */}
          <div className="mt-6 overflow-x-auto border-4 border-black bg-[#FBF9F5] shadow-[8px_8px_0_0_#000]">
            <table className="w-full min-w-[720px] border-collapse font-tech">
              <thead>
                <tr className="border-b-4 border-black bg-black text-[#FBF9F5]">
                  <th className="border-r-2 border-[#FBF9F5]/20 px-4 py-3 text-left text-[11px] font-bold uppercase tracking-wider">
                    Metric
                  </th>
                  <th className="border-r-2 border-[#FBF9F5]/20 px-4 py-3 text-right text-[11px] font-bold uppercase tracking-wider">
                    p50
                  </th>
                  <th className="border-r-2 border-[#FBF9F5]/20 px-4 py-3 text-right text-[11px] font-bold uppercase tracking-wider">
                    p99
                  </th>
                  <th className="px-4 py-3 text-right text-[11px] font-bold uppercase tracking-wider text-[#FF5500]">
                    Throughput
                  </th>
                </tr>
              </thead>
              <tbody>
                {BENCH01.rows.map((r, i) => (
                  <tr
                    key={r.metric}
                    className={`border-b-2 border-black/15 transition-colors hover:bg-[#FF5500]/10 ${
                      r.highlight ? "bg-[#FF5500]/15" : i % 2 ? "bg-[#F2EDE2]/40" : ""
                    }`}
                  >
                    <td className="border-r-2 border-black/10 px-4 py-3 text-xs font-bold text-black">
                      <div className="flex items-center gap-2">
                        {r.highlight && (
                          <span className="h-2 w-2 shrink-0 animate-pulse-ring bg-[#FF5500]" />
                        )}
                        {r.metric}
                      </div>
                    </td>
                    <td className="border-r-2 border-black/10 px-4 py-3 text-right font-mono text-sm font-bold text-black">
                      {r.p50}
                    </td>
                    <td className="border-r-2 border-black/10 px-4 py-3 text-right font-mono text-sm text-black/70">
                      {r.p99}
                    </td>
                    <td className="px-4 py-3 text-right font-mono text-sm font-bold text-[#FF5500]">
                      {r.throughput}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {/* Latency bar visualization */}
          <div className="mt-6 border-4 border-black bg-[#FBF9F5] p-5 shadow-[6px_6px_0_0_#000]">
            <div className="mb-4 flex items-center justify-between">
              <h3 className="font-display text-xl uppercase text-black">
                Query latency · p50 vs p99
              </h3>
              <span className="font-tech text-[10px] uppercase tracking-wider text-black/50">
                lower is faster →
              </span>
            </div>
            <div className="space-y-4">
              {[
                { label: "BM25 (Lexical)", p50: 115.334, p99: 137.539, color: "bg-black" },
                { label: "HNSW (Vector)", p50: 61.996, p99: 71.893, color: "bg-[#1A1A1A]" },
                { label: "Hybrid Fusion", p50: 179.810, p99: 211.059, color: "bg-[#FF5500]" },
              ].map((b) => {
                const max = 220.0;
                return (
                  <div key={b.label}>
                    <div className="mb-1 flex items-center justify-between font-tech text-[11px]">
                      <span className="font-bold uppercase tracking-wider text-black">
                        {b.label}
                      </span>
                      <span className="text-black/60">
                        p50 {b.p50}ms · p99 {b.p99}ms
                      </span>
                    </div>
                    <div className="relative h-7 border-2 border-black bg-[#F2EDE2]">
                      <div
                        className={`absolute left-0 top-0 h-1/2 ${b.color}`}
                        style={{ width: `${(b.p50 / max) * 100}%` }}
                        title={`p50 ${b.p50}ms`}
                      />
                      <div
                        className={`absolute left-0 bottom-0 h-1/2 ${b.color} opacity-50`}
                        style={{ width: `${(b.p99 / max) * 100}%` }}
                        title={`p99 ${b.p99}ms`}
                      />
                      <div
                        className="absolute top-0 h-full w-0.5 bg-black"
                        style={{ left: `${(b.p50 / max) * 100}%` }}
                      />
                    </div>
                  </div>
                );
              })}
            </div>
            <div className="mt-3 flex items-center gap-4 font-tech text-[10px] uppercase tracking-wider text-black/60">
              <span className="flex items-center gap-1.5">
                <span className="h-2 w-4 bg-black" /> p50
              </span>
              <span className="flex items-center gap-1.5">
                <span className="h-2 w-4 bg-black opacity-50" /> p99
              </span>
              <span className="ml-auto">scale 0–220ms</span>
            </div>
          </div>

          {/* Hardware note */}
          <div className="mt-4 flex items-start gap-3 border-l-4 border-black bg-[#FBF9F5] px-4 py-3">
            <Cpu className="mt-0.5 h-4 w-4 shrink-0 text-[#FF5500]" strokeWidth={2.5} />
            <p className="font-tech text-[11px] text-black/70">
              <span className="font-bold uppercase tracking-wider">Hardware profile:</span>{" "}
              {BENCH01.hardware}
            </p>
          </div>
        </div>
      </section>

      {/* Interactive Latency Explorer */}
      <LatencyComparator />

      {/* SIFT1M Table */}
      <section className="relative border-b-4 border-black bg-[#FBF9F5]">
        <div className="pointer-events-none absolute inset-0 grid-tech opacity-50" aria-hidden />
        <div className="relative mx-auto max-w-7xl px-4 py-14 sm:px-6">
          <Reveal direction="up">
          <SectionHeader tag="§02" title={SIFT1M.title} subtitle={SIFT1M.subtitle} />
          </Reveal>

          <div className="mt-6 overflow-x-auto border-4 border-black bg-[#FBF9F5] shadow-[8px_8px_0_0_#000]">
            <table className="w-full min-w-[860px] border-collapse font-tech">
              <thead>
                <tr className="border-b-4 border-black bg-black text-[#FBF9F5]">
                  <th className="border-r-2 border-[#FBF9F5]/20 px-3 py-3 text-left text-[10px] font-bold uppercase tracking-wider">
                    Scale
                  </th>
                  <th className="border-r-2 border-[#FBF9F5]/20 px-3 py-3 text-left text-[10px] font-bold uppercase tracking-wider">
                    Config
                  </th>
                  <th className="border-r-2 border-[#FBF9F5]/20 px-3 py-3 text-left text-[10px] font-bold uppercase tracking-wider">
                    Metric
                  </th>
                  <th className="border-r-2 border-[#FBF9F5]/20 px-3 py-3 text-right text-[10px] font-bold uppercase tracking-wider">
                    Before
                  </th>
                  <th className="border-r-2 border-[#FBF9F5]/20 px-3 py-3 text-right text-[10px] font-bold uppercase tracking-wider">
                    Now
                  </th>
                  <th className="border-r-2 border-[#FBF9F5]/20 px-3 py-3 text-right text-[10px] font-bold uppercase tracking-wider text-[#FF5500]">
                    Speedup
                  </th>
                  <th className="border-r-2 border-[#FBF9F5]/20 px-3 py-3 text-right text-[10px] font-bold uppercase tracking-wider">
                    p99
                  </th>
                  <th className="px-3 py-3 text-right text-[10px] font-bold uppercase tracking-wider">
                    QPS
                  </th>
                </tr>
              </thead>
              <tbody>
                {SIFT1M.rows.map((r, i) => (
                  <tr
                    key={r.config}
                    className={`border-b-2 border-black/15 transition-colors hover:bg-[#FF5500]/10 ${
                      i % 2 ? "bg-[#F2EDE2]/40" : ""
                    }`}
                  >
                    <td className="border-r-2 border-black/10 px-3 py-3 text-xs font-bold text-black">
                      {r.scale}
                    </td>
                    <td className="border-r-2 border-black/10 px-3 py-3 text-xs text-black">
                      {r.config}
                    </td>
                    <td className="border-r-2 border-black/10 px-3 py-3 text-[10px] uppercase tracking-wider text-black/60">
                      {r.metricType}
                    </td>
                    <td className="border-r-2 border-black/10 px-3 py-3 text-right font-mono text-xs text-black/50 line-through decoration-[#FF5500]/60">
                      {r.before}
                    </td>
                    <td className="border-r-2 border-black/10 px-3 py-3 text-right font-mono text-xs font-bold text-black">
                      {r.after}
                    </td>
                    <td className="border-r-2 border-black/10 px-3 py-3 text-right">
                      <span className="inline-flex items-center gap-1 border-2 border-black bg-[#FF5500] px-2 py-0.5 font-mono text-xs font-bold text-black">
                        <TrendingDown className="h-3 w-3" strokeWidth={3} />
                        {r.speedup}
                      </span>
                    </td>
                    <td className="border-r-2 border-black/10 px-3 py-3 text-right font-mono text-xs text-black/70">
                      {r.p99}
                    </td>
                    <td className="px-3 py-3 text-right font-mono text-xs font-bold text-black">
                      {r.qps}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {/* Speedup viz */}
          <div className="mt-6 border-4 border-black bg-black p-5 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]">
            <div className="mb-4 flex items-center justify-between">
              <h3 className="font-display text-xl uppercase text-[#FF5500]">
                Phase 2 construction speedups
              </h3>
              <span className="font-tech text-[10px] uppercase tracking-wider text-[#FBF9F5]/50">
                SIFT1M · 100K vectors
              </span>
            </div>
            <div className="space-y-3">
              {SIFT1M.rows.map((r) => {
                const mult = parseFloat(r.speedup);
                const pct = Math.min((mult / 3) * 100, 100);
                return (
                  <div key={r.config} className="flex items-center gap-3">
                    <span className="w-40 shrink-0 font-tech text-[10px] uppercase tracking-wider text-[#FBF9F5]/70">
                      {r.config}
                    </span>
                    <div className="relative h-5 flex-1 border-2 border-[#FBF9F5]/30 bg-[#1A1A1A]">
                      <div
                        className="absolute left-0 top-0 h-full bg-[#FF5500]"
                        style={{ width: `${pct}%` }}
                      />
                      <span className="absolute right-2 top-1/2 -translate-y-1/2 font-mono text-[10px] font-bold text-black">
                        {r.speedup}
                      </span>
                    </div>
                  </div>
                );
              })}
            </div>
            <p className="mt-4 border-t-2 border-[#FBF9F5]/15 pt-3 font-tech text-[10px] leading-relaxed text-[#FBF9F5]/60">
              Optimizations: static prefetch · elimination of Euclidean square root in hot
              graph traversal · pure SIMD cosine similarity ·{" "}
              <span className="text-[#FF5500]">O(M²) select_neighbors</span> (caches
              references to eradicate HashMap queries during the diversity loop).
            </p>
          </div>

          {/* Hardware note */}
          <div className="mt-4 flex items-start gap-3 border-l-4 border-black bg-[#FBF9F5] px-4 py-3">
            <Cpu className="mt-0.5 h-4 w-4 shrink-0 text-[#FF5500]" strokeWidth={2.5} />
            <p className="font-tech text-[11px] text-black/70">
              <span className="font-bold uppercase tracking-wider">
                Certification hardware:
              </span>{" "}
              {SIFT1M.hardware}
            </p>
          </div>
        </div>
      </section>

      {/* Competitive benchmark */}
      <section className="relative border-b-4 border-black bg-[#FBF9F5]">
        <div className="pointer-events-none absolute inset-0 grid-tech opacity-50" aria-hidden />
        <div className="relative mx-auto max-w-7xl px-4 py-14 sm:px-6">
          <Reveal direction="up">
          <SectionHeader
            tag="§03"
            title={COMPETITIVE_TABLE.title}
            subtitle={COMPETITIVE_TABLE.subtitle}
          />
          </Reveal>

          <div className="mt-6 overflow-x-auto border-4 border-black bg-[#FBF9F5] shadow-[8px_8px_0_0_#000]">
            <table className="w-full min-w-[900px] border-collapse font-tech">
              <thead>
                <tr className="border-b-4 border-black bg-black text-[#FBF9F5]">
                  <th className="border-r-2 border-[#FBF9F5]/20 px-4 py-3 text-left text-[10px] font-bold uppercase tracking-wider">
                    Metric
                  </th>
                  {["VantaDB", "LanceDB", "ChromaDB", "Pinecone", "Weaviate"].map((h) => (
                    <th
                      key={h}
                      className={`border-r-2 border-[#FBF9F5]/20 px-4 py-3 text-right text-[10px] font-bold uppercase tracking-wider ${
                        h === "VantaDB" ? "text-[#FF5500]" : ""
                      }`}
                    >
                      {h}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {COMPETITIVE_TABLE.rows.map((r) => (
                  <tr
                    key={r.metric}
                    className={`border-b-2 border-black/15 transition-colors hover:bg-[#FF5500]/10 ${
                      r.highlight ? "bg-[#FF5500]/15" : ""
                    }`}
                  >
                    <td className="border-r-2 border-black/10 px-4 py-3 text-xs font-bold text-black">
                      <div className="flex items-center gap-2">
                        {r.highlight && (
                          <span className="h-2 w-2 shrink-0 animate-pulse-ring bg-[#FF5500]" />
                        )}
                        {r.metric}
                      </div>
                    </td>
                    <td className="border-r-2 border-black/10 px-4 py-3 text-right font-mono text-sm font-bold text-[#FF5500]">
                      {r.vanta}
                    </td>
                    <td className="border-r-2 border-black/10 px-4 py-3 text-right font-mono text-sm text-black/80">
                      {r.lance}
                    </td>
                    <td className="border-r-2 border-black/10 px-4 py-3 text-right font-mono text-sm text-black/80">
                      {r.chroma}
                    </td>
                    <td className="border-r-2 border-black/10 px-4 py-3 text-right font-mono text-[11px] uppercase tracking-wider text-black/50">
                      {r.pinecone}
                    </td>
                    <td className="px-4 py-3 text-right font-mono text-[11px] uppercase tracking-wider text-black/50">
                      {r.weaviate}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {/* Honesty note */}
          <div className="mt-4 flex items-start gap-3 border-l-4 border-black bg-[#FBF9F5] px-4 py-3">
            <Cpu className="mt-0.5 h-4 w-4 shrink-0 text-[#FF5500]" strokeWidth={2.5} />
            <p className="font-tech text-[10px] leading-relaxed text-black/60">
              {COMPETITIVE_TABLE.note}
            </p>
          </div>

          {/* Source */}
          <div className="mt-2 flex items-start gap-3 px-4 py-2 font-tech text-[10px] text-black/50">
            <span className="uppercase tracking-wider">Source:</span>
            <a
              href={COMPETITIVE_TABLE.sourceLink}
              target="_blank"
              rel="noreferrer"
              className="underline decoration-[#FF5500]/60 underline-offset-2 hover:text-[#FF5500]"
            >
              docs/blog/benchmarks_vs_lancedb_chroma.md
            </a>
          </div>
        </div>
      </section>

      {/* Run locally */}
      <section className="relative border-b-4 border-black bg-[#F2EDE2]">
        <div className="pointer-events-none absolute inset-0 grid-tech opacity-50" aria-hidden />
        <div className="relative mx-auto max-w-7xl px-4 py-14 sm:px-6">
          <Reveal direction="up">
          <SectionHeader
            tag="§04"
            title="Run the benchmark locally"
            subtitle="Measure the baseline on your own hardware in three commands"
          />
          </Reveal>
          <div className="mt-6 grid grid-cols-1 gap-4 md:grid-cols-3">
            {[
              {
                step: "01",
                title: "Install python bindings",
                code: "pip install maturin\nmaturin develop --release",
              },
              {
                step: "02",
                title: "Execute the benchmark",
                code: "python benchmarks/vantadb_local_bench.py \\\n  --size 10000 --dim 128 --queries 1000",
              },
              {
                step: "03",
                title: "Read the report",
                code: "cat vanta_benchmark_report.json",
              },
            ].map((c) => (
              <div
                key={c.step}
                className="press-lg flex flex-col border-4 border-black bg-[#FBF9F5] p-5"
              >
                <div className="mb-3 flex items-center justify-between">
                  <span className="font-display text-2xl text-[#FF5500]">{c.step}</span>
                  <Terminal className="h-5 w-5 text-black" strokeWidth={2.5} />
                </div>
                <h3 className="mb-2 font-tech text-xs font-bold uppercase tracking-wider text-black">
                  {c.title}
                </h3>
                <pre className="flex-1 overflow-x-auto border-2 border-black bg-black p-3 font-tech text-[11px] leading-relaxed text-[#FBF9F5]">
                  {c.code}
                </pre>
              </div>
            ))}
          </div>

          {/* CTA */}
          <div className="mt-8 flex flex-col items-center justify-between gap-4 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500] sm:flex-row">
            <div>
              <h3 className="font-display text-2xl uppercase leading-none text-[#FBF9F5]">
                Ready to build?
              </h3>
              <p className="mt-1 font-tech text-xs text-[#FBF9F5]/60">
                Install vantadb-py and run your first hybrid search in 5 minutes.
              </p>
            </div>
            <button
              onClick={() => onNavigate("docs")}
              className="press-neon inline-flex shrink-0 items-center gap-2 border-4 border-black bg-[#FF5500] px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black"
            >
              Open Quickstart
              <ArrowRight className="h-4 w-4" strokeWidth={2.5} />
            </button>
          </div>
        </div>
      </section>
    </div>
  );
}

function SectionHeader({
  tag,
  title,
  subtitle,
}: {
  tag: string;
  title: string;
  subtitle: string;
}) {
  return (
    <div className="flex flex-col gap-3 border-4 border-black bg-[#FBF9F5] p-5 shadow-[6px_6px_0_0_#000] sm:flex-row sm:items-end sm:justify-between">
      <div>
        <span className="inline-flex items-center gap-2 border-2 border-black bg-[#FF5500] px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-black">
          <span className="h-1.5 w-1.5 bg-black" />
          {tag}
        </span>
        <h2 className="mt-3 font-display text-3xl uppercase leading-none text-black sm:text-4xl">
          {title}
        </h2>
        <p className="mt-2 max-w-2xl font-tech text-xs text-black/70">{subtitle}</p>
      </div>
      <div className="hidden items-center gap-2 font-tech text-[10px] uppercase tracking-wider text-black/50 sm:flex">
        <span className="h-2 w-2 animate-flicker bg-[#FF5500]" />
        certified
      </div>
    </div>
  );
}
