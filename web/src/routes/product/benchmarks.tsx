import { createFileRoute } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";

export const Route = createFileRoute("/product/benchmarks")({
  head: () => ({
    meta: [
      { title: "VantaDB Benchmarks" },
      {
        name: "description",
        content: "Performance benchmarks for VantaDB: Latency, Throughput, and Memory usage compared to competitors.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/product/benchmarks" }],
  }),
  component: BenchmarksPage,
});

function BenchmarksPage() {
  return (
    <div className="engine-page">
      <SwissSubpageHero
        num="01"
        eyebrow="Benchmarks"
        title={
          <span>
            Performance
            <br />
            Metrics.
          </span>
        }
        sub="VantaDB is engineered for extreme low latency on edge devices. See how we compare against the industry."
      />

      <main className="engine-main">
        <section className="engine-section">
          <span className="swiss-eyebrow">Throughput vs Latency</span>
          
          <div style={{ marginTop: "3rem" }}>
            <table style={{ width: "100%", borderCollapse: "collapse", border: "1px solid var(--border)", fontFamily: "var(--font-mono)", fontSize: "0.9rem" }}>
              <thead>
                <tr style={{ borderBottom: "1px solid var(--border)", background: "var(--surface-raised)", color: "var(--steel)" }}>
                  <th style={{ padding: "1rem", textAlign: "left" }}>Engine</th>
                  <th style={{ padding: "1rem", textAlign: "right" }}>Insert (QPS)</th>
                  <th style={{ padding: "1rem", textAlign: "right" }}>Search (QPS)</th>
                  <th style={{ padding: "1rem", textAlign: "right" }}>p99 Latency (ms)</th>
                </tr>
              </thead>
              <tbody>
                <tr style={{ borderBottom: "1px solid var(--border)" }}>
                  <td style={{ padding: "1rem" }}><span style={{ color: "var(--amber)", fontWeight: "bold" }}>VantaDB (Rust)</span></td>
                  <td style={{ padding: "1rem", textAlign: "right" }}>8,450</td>
                  <td style={{ padding: "1rem", textAlign: "right" }}>1,195</td>
                  <td style={{ padding: "1rem", textAlign: "right" }}>2.43</td>
                </tr>
                <tr style={{ borderBottom: "1px solid var(--border)" }}>
                  <td style={{ padding: "1rem" }}>ChromaDB (Local)</td>
                  <td style={{ padding: "1rem", textAlign: "right" }}>1,200</td>
                  <td style={{ padding: "1rem", textAlign: "right" }}>450</td>
                  <td style={{ padding: "1rem", textAlign: "right" }}>12.50</td>
                </tr>
                <tr>
                  <td style={{ padding: "1rem" }}>Qdrant (Local Docker)</td>
                  <td style={{ padding: "1rem", textAlign: "right" }}>4,100</td>
                  <td style={{ padding: "1rem", textAlign: "right" }}>890</td>
                  <td style={{ padding: "1rem", textAlign: "right" }}>5.10</td>
                </tr>
              </tbody>
            </table>
          </div>

          <p style={{ marginTop: "2rem", color: "var(--steel)", lineHeight: "1.6" }}>
            *Benchmarks run on an Apple M2 Pro (16GB RAM) with 100K 1536-dimensional vectors. VantaDB uses BFS-compacted Memmapped HNSW and Fjall LSM-tree storage.
          </p>
        </section>
      </main>
    </div>
  );
}
