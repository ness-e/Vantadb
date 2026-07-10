import { createLazyRoute } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader } from "@/components/nb";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp } from "@/lib/motion-utils";
import "../../styles/benchmarks.css";

export const Route = createLazyRoute("/product/benchmarks")({
  component: BenchmarksPage,
});

function BenchmarksPage() {
  const benchmarkRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = benchmarkRef.current?.querySelectorAll<HTMLElement>(".nc-bm-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, benchmarkRef);

  return (
    <div className="nb-page">
      <NbSubpageHero
        pattern="p22"
        title={
          <span>
            Performance
            <br />
            Metrics.
          </span>
        }
        sub="VantaDB is engineered for extreme low latency on edge devices. See how we compare against the industry."
      />

      <NbSection ref={benchmarkRef} ariaLabel="Benchmark metrics">
        <NbSectionHeader
          monoLabel="[BENCHMARKS]"
          headline="Throughput vs Latency."
          sub="Raw Rust core performance on the SIFT1M dataset \u2014 100K 128-dimensional vectors."
        />

        <div className="nc-bm-part">
          <table className="nc-bm-table">
            <thead>
              <tr>
                <th className="nc-bm-th">Engine</th>
                <th className="nc-bm-th-right">Insert (QPS)</th>
                <th className="nc-bm-th-right">Search (QPS)</th>
                <th className="nc-bm-th-right">p99 Latency (ms)</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td className="nc-bm-td">
                  <span className="nc-bm-amber">VantaDB (Rust)</span>
                </td>
                <td className="nc-bm-td-right nc-bm-amber">8,450</td>
                <td className="nc-bm-td-right nc-bm-amber">1,195</td>
                <td className="nc-bm-td-right nc-bm-amber">2.43</td>
              </tr>
              <tr>
                <td className="nc-bm-td">ChromaDB (Local)</td>
                <td className="nc-bm-td-right">1,200</td>
                <td className="nc-bm-td-right">450</td>
                <td className="nc-bm-td-right">12.50</td>
              </tr>
              <tr>
                <td className="nc-bm-td">Qdrant (Local Docker)</td>
                <td className="nc-bm-td-right">4,100</td>
                <td className="nc-bm-td-right">890</td>
                <td className="nc-bm-td-right">5.10</td>
              </tr>
            </tbody>
          </table>
        </div>

        <p className="nc-bm-footnote nc-bm-part">
          *Benchmarks run on AMD Ryzen 12-Core @ 3.5GHz with 100K 128-dimensional SIFT1M vectors.
          VantaDB uses BFS-compacted Memmapped HNSW and Fjall LSM-tree storage.
        </p>

        <div className="nc-bm-note nc-bm-part">
          <strong className="nc-bm-amber">Note:</strong> Benchmarks shown represent raw Rust core
          performance on the SIFT1M dataset. Real-world performance varies by dataset size,
          dimensionality, and hardware configuration. See full methodology in{" "}
          <code className="nc-bm-code">docs/operations/BENCHMARKS.md</code>.
        </div>
      </NbSection>
    </div>
  );
}

export function PendingComponent() {
  return (
    <div className="nc-bm-pending">
      <div>Loading...</div>
    </div>
  );
}
