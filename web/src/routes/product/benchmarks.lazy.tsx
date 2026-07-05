import { createLazyRoute } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
import "../../styles/benchmarks.css";

export const Route = createLazyRoute("/product/benchmarks")({
  component: BenchmarksPage,
});

function BenchmarksPage() {
  const benchmarkRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(benchmarkRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
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
          sub="Raw Rust core performance on the SIFT1M dataset — 100K 128-dimensional vectors."
        />

        <div className="nb-engine-part">
          <table className="benchmarks-table">
            <thead>
              <tr>
                <th className="benchmarks-th">Engine</th>
                <th className="benchmarks-th-right">Insert (QPS)</th>
                <th className="benchmarks-th-right">Search (QPS)</th>
                <th className="benchmarks-th-right">p99 Latency (ms)</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td className="benchmarks-td">
                  <span className="benchmarks-amber-text">VantaDB (Rust)</span>
                </td>
                <td className="benchmarks-td-right">8,450</td>
                <td className="benchmarks-td-right">1,195</td>
                <td className="benchmarks-td-right">2.43</td>
              </tr>
              <tr>
                <td className="benchmarks-td">ChromaDB (Local)</td>
                <td className="benchmarks-td-right">1,200</td>
                <td className="benchmarks-td-right">450</td>
                <td className="benchmarks-td-right">12.50</td>
              </tr>
              <tr>
                <td className="benchmarks-td">Qdrant (Local Docker)</td>
                <td className="benchmarks-td-right">4,100</td>
                <td className="benchmarks-td-right">890</td>
                <td className="benchmarks-td-right">5.10</td>
              </tr>
            </tbody>
          </table>
        </div>

        <p className="benchmarks-footnote nb-engine-part">
          *Benchmarks run on AMD Ryzen 12-Core @ 3.5GHz with 100K 128-dimensional SIFT1M vectors.
          VantaDB uses BFS-compacted Memmapped HNSW and Fjall LSM-tree storage.
        </p>

        <div className="benchmarks-note-box nb-engine-part">
          <strong className="benchmarks-amber-text">Note:</strong> Benchmarks shown represent raw
          Rust core performance on the SIFT1M dataset. Real-world performance varies by dataset
          size, dimensionality, and hardware configuration. See full methodology in{" "}
          <code className="benchmarks-code-inline">docs/operations/BENCHMARKS.md</code>.
        </div>
      </NbSection>
    </div>
  );
}

export function PendingComponent() {
  return (
    <div className="benchmarks-pending">
      <div>Loading...</div>
    </div>
  );
}
