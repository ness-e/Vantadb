import { createLazyRoute } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import "../../styles/benchmarks.css";

export const Route = createLazyRoute("/product/benchmarks")({
  component: BenchmarksPage,
});

function BenchmarksPage() {
  return (
    <div className="nb-page">
      <NbSubpageHero
        num="01"
        title={
          <span>
            Performance
            <br />
            Metrics.
          </span>
        }
        sub="VantaDB is engineered for extreme low latency on edge devices. See how we compare against the industry."
      />

      <main className="nb-main">
        <section className="nb-section">
          <h2 className="benchmarks-hero-title">Throughput vs Latency</h2>

          <div className="benchmarks-mt-3rem">
            <table className="benchmarks-table">
              <thead>
                <tr className="benchmarks-thead-tr">
                  <th className="benchmarks-th">Engine</th>
                  <th className="benchmarks-th-right">Insert (QPS)</th>
                  <th className="benchmarks-th-right">Search (QPS)</th>
                  <th className="benchmarks-th-right">p99 Latency (ms)</th>
                </tr>
              </thead>
              <tbody>
                <tr className="benchmarks-tbody-tr">
                  <td className="benchmarks-td">
                    <span className="benchmarks-amber-text">VantaDB (Rust)</span>
                  </td>
                  <td className="benchmarks-td-right">8,450</td>
                  <td className="benchmarks-td-right">1,195</td>
                  <td className="benchmarks-td-right">2.43</td>
                </tr>
                <tr className="benchmarks-tbody-tr">
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

          <p className="benchmarks-footnote">
            *Benchmarks run on AMD Ryzen 12-Core @ 3.5GHz with 100K 128-dimensional SIFT1M vectors.
            VantaDB uses BFS-compacted Memmapped HNSW and Fjall LSM-tree storage.
          </p>

          <div className="benchmarks-note-box">
            <strong className="benchmarks-amber-text">Note:</strong> Benchmarks shown represent raw
            Rust core performance on the SIFT1M dataset (100K vectors, 128-dim, Balanced Cos
            configuration). Real-world performance varies by dataset size, dimensionality, and
            hardware configuration. See full methodology in{" "}
            <code className="benchmarks-code-inline">docs/operations/BENCHMARKS.md</code>.
          </div>
        </section>
      </main>

      <style>{`
        @media (max-width: 640px) {
          table { font-size: 0.75rem !important; }
          th, td { padding: 0.65rem !important; }
        }
      `}</style>
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
