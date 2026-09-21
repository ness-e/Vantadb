"use client";

import { COMPETITIVE_BENCHMARK } from "@/lib/vanta-data";
import { Reveal } from "./reveal";

/** Number → locale string with fixed decimals, no trailing zero inflation. */
const fmt = (n: number, digits = 1): string =>
  n.toLocaleString("en-US", {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
  });

/**
 * Competitive benchmark table against LanceDB & ChromaDB (INV-007-B).
 * Renders data from the versioned JSON contract
 * (`web/src/lib/data/competitive-benchmark.json`), NOT hardcoded numbers.
 * Mounted below <BenchmarkRace /> on /benchmarks.
 */
export function CompetitiveTable() {
  const data = COMPETITIVE_BENCHMARK;
  const { dataset } = data;

  return (
    <section
      aria-label="Competitive benchmark table"
      className="relative border-b-4 border-black bg-[#FBF9F5]  "
    >
      <div className="pointer-events-none absolute inset-0 grid-tech opacity-40" aria-hidden />
      <div className="relative mx-auto max-w-7xl px-4 py-14 sm:px-6">
        <Reveal direction="up">
          <div className="mb-8 flex flex-col gap-3 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:flex-row sm:items-end sm:justify-between">
            <div>
              <span className="inline-flex items-center gap-2 border-2 border-[#FF5500] bg-[#FF5500] px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-black">
                <span className="h-1.5 w-1.5 bg-black" />
                Reproducible · JSON contract
              </span>
              <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-[0.9] sm:text-4xl lg:text-5xl">
                VantaDB vs LanceDB &amp; ChromaDB
              </h2>
              <p className="mt-3 max-w-2xl font-tech text-xs text-[#FBF9F5]/70 sm:text-sm">
                {dataset.name} · {dataset.vectors.toLocaleString()} vectors ·{" "}
                {dataset.queries.toLocaleString()} queries · top-{dataset.top_k} ·{" "}
                {dataset.metric} · {dataset.ingest_mode}
              </p>
            </div>
            <span className="flex items-center gap-2 font-tech text-[10px] uppercase tracking-wider text-[#FF5500]">
              <span className="h-2 w-2 animate-flicker bg-[#FF5500]" />
              run {data.generated_at} · real
            </span>
          </div>
        </Reveal>

        <Reveal direction="up" delay={60}>
          <div className="overflow-x-auto border-4 border-black bg-[#FBF9F5] shadow-[8px_8px_0_0_#000]  ">
            <table className="w-full min-w-[900px] border-collapse font-tech">
              <thead>
                <tr className="border-b-4 border-black bg-black text-[#FBF9F5]">
                  <th className="border-r-2 border-[#FBF9F5]/20 px-4 py-3 text-left text-[10px] font-bold uppercase tracking-wider">
                    Engine
                  </th>
                  <th className="border-r-2 border-[#FBF9F5]/20 px-4 py-3 text-right text-[10px] font-bold uppercase tracking-wider">
                    Ingest QPS
                  </th>
                  <th className="border-r-2 border-[#FBF9F5]/20 px-4 py-3 text-right text-[10px] font-bold uppercase tracking-wider">
                    Index (ms)
                  </th>
                  <th className="border-r-2 border-[#FBF9F5]/20 px-4 py-3 text-right text-[10px] font-bold uppercase tracking-wider">
                    Query QPS
                  </th>
                  <th className="border-r-2 border-[#FBF9F5]/20 px-4 py-3 text-right text-[10px] font-bold uppercase tracking-wider">
                    p50 (ms)
                  </th>
                  <th className="border-r-2 border-[#FBF9F5]/20 px-4 py-3 text-right text-[10px] font-bold uppercase tracking-wider">
                    p99 (ms)
                  </th>
                  <th className="border-r-2 border-[#FBF9F5]/20 px-4 py-3 text-right text-[10px] font-bold uppercase tracking-wider text-[#FF5500]">
                    Recall@10
                  </th>
                  <th className="border-r-2 border-[#FBF9F5]/20 px-4 py-3 text-right text-[10px] font-bold uppercase tracking-wider">
                    Peak RSS (MB)
                  </th>
                  <th className="px-4 py-3 text-right text-[10px] font-bold uppercase tracking-wider">
                    Δ RSS (MB)
                  </th>
                </tr>
              </thead>
              <tbody>
                {data.results.map((r, i) => {
                  const isVanta = r.engine === "VantaDB";
                  return (
                    <tr
                      key={r.engine}
                      className={`border-b-2 border-black/15 transition-colors hover:bg-[#FF5500]/10 ${
                        isVanta ? "bg-[#FF5500]/15" : i % 2 ? "bg-[#F2EDE2]/40" : ""
                      }`}
                    >
                      <td className="border-r-2 border-black/10 px-4 py-3 text-xs font-bold text-black">
                        <div className="flex items-center gap-2">
                          {isVanta && (
                            <span className="h-2 w-2 shrink-0 animate-pulse-ring bg-[#FF5500]" />
                          )}
                          {r.engine}
                        </div>
                      </td>
                      <td className="border-r-2 border-black/10 px-4 py-3 text-right font-mono text-sm text-black">
                        {fmt(r.ingest_qps)}
                      </td>
                      <td className="border-r-2 border-black/10 px-4 py-3 text-right font-mono text-sm text-black/70">
                        {r.index_time_ms == null ? "N/A (inc)" : fmt(r.index_time_ms)}
                      </td>
                      <td className="border-r-2 border-black/10 px-4 py-3 text-right font-mono text-sm text-black">
                        {fmt(r.query_qps)}
                      </td>
                      <td className="border-r-2 border-black/10 px-4 py-3 text-right font-mono text-sm text-black/70">
                        {r.query_p50_ms.toFixed(3)}
                      </td>
                      <td className="border-r-2 border-black/10 px-4 py-3 text-right font-mono text-sm text-black/70">
                        {r.query_p99_ms.toFixed(3)}
                      </td>
                      <td className="border-r-2 border-black/10 px-4 py-3 text-right font-mono text-sm font-bold text-[#FF5500]">
                        {(r.recall_at_k * 100).toFixed(2)}%
                      </td>
                      <td className="border-r-2 border-black/10 px-4 py-3 text-right font-mono text-sm text-black/70">
                        {fmt(r.mem_peak_rss_mb)}
                      </td>
                      <td className="px-4 py-3 text-right font-mono text-sm text-black/70">
                        {fmt(r.mem_delta_rss_mb)}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </Reveal>

        <Reveal direction="up" delay={100}>
          <div className="mt-4 flex items-start gap-3 border-l-4 border-[#FF5500] bg-[#FBF9F5] px-4 py-3">
            <p className="font-tech text-[10px] leading-relaxed text-black/60">
              Readings are real numbers from the certified run{" "}
              <span className="font-bold">({data.generated_at})</span> — {data.dataset.name},{" "}
              {data.dataset.vectors.toLocaleString()} vectors, {data.dataset.queries} queries,
              top-{data.dataset.top_k}. VantaDB runs through its PyO3 bindings over the mmap Rust
              core; LanceDB and ChromaDB use their native C/C++ wrappers. Regenerate:{" "}
              <code className="bg-[#F2EDE2] px-1">
                python benchmarks/competitive_bench.py --dataset {data.dataset.name} --size{" "}
                {data.dataset.vectors} --queries {data.dataset.queries}
              </code>
            </p>
          </div>
          <p className="mt-2 px-4 font-tech text-[10px] leading-relaxed text-black/70">
            <span className="uppercase tracking-wider">Source:</span>{" "}
            <a
              href="https://vantadb.vercel.app/blog/benchmarks-vs-lancedb-chroma"
              target="_blank"
              rel="noreferrer"
              className="underline decoration-[#FF5500]/60 underline-offset-2 hover:text-[#FF5500]"
            >
              docs/blog/benchmarks_vs_lancedb_chroma.md
            </a>{" "}
            — repo-tracked JSON:{" "}
            <code className="bg-[#F2EDE2] px-1">web/src/lib/data/competitive-benchmark.json</code>
          </p>
        </Reveal>
      </div>
    </section>
  );
}