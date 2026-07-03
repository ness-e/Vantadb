import { createFileRoute } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";

export const Route = createFileRoute("/about/roadmap")({
  head: () => ({
    meta: [
      { title: "VantaDB Roadmap" },
      {
        name: "description",
        content: "The technical and strategic roadmap for VantaDB.",
      },
    ],
    links: [{ rel: "canonical", href: "https://vantadb.dev/about/roadmap" }],
  }),
  component: RoadmapPage,
});

function RoadmapPage() {
  return (
    <div className="engine-page">
      <SwissSubpageHero
        num="03"
        eyebrow="Strategic Roadmap"
        title={
          <span>
            The Path
            <br />
            Forward.
          </span>
        }
        sub="VantaDB is built transparently. This is our trajectory toward becoming the standard memory engine for local AI."
      />

      <main className="engine-main">
        <section className="engine-section">
          <span className="swiss-eyebrow">Milestones</span>
          
          <div style={{ marginTop: "3rem", display: "grid", gap: "2rem" }}>
            <div style={{ borderLeft: "2px solid var(--amber)", paddingLeft: "1.5rem" }}>
              <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.8rem", color: "var(--steel)" }}>PHASE 1-3 • COMPLETED</span>
              <h3 style={{ margin: "0.5rem 0", color: "var(--foreground)", fontSize: "1.2rem" }}>Core Engine & Integrations</h3>
              <p style={{ color: "var(--steel)", margin: 0, lineHeight: "1.6" }}>
                HNSW Vector Search, LSM-Tree WAL durability, BM25 text search, Reciprocal Rank Fusion (RRF), Python PyO3 SDK, and HTTP/MCP Servers.
              </p>
            </div>

            <div style={{ borderLeft: "2px solid var(--border)", paddingLeft: "1.5rem" }}>
              <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.8rem", color: "var(--amber)" }}>PHASE 4 • CURRENT</span>
              <h3 style={{ margin: "0.5rem 0", color: "var(--foreground)", fontSize: "1.2rem" }}>Code Health & Security</h3>
              <p style={{ color: "var(--steel)", margin: 0, lineHeight: "1.6" }}>
                Deprecating unmaintained dependencies, increasing test coverage, unifying documentation, and preparing the web presence.
              </p>
            </div>

            <div style={{ borderLeft: "2px solid var(--border)", paddingLeft: "1.5rem" }}>
              <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.8rem", color: "var(--steel)" }}>PHASE 5 • UPCOMING</span>
              <h3 style={{ margin: "0.5rem 0", color: "var(--foreground)", fontSize: "1.2rem" }}>Enterprise Readiness & Cloud</h3>
              <p style={{ color: "var(--steel)", margin: 0, lineHeight: "1.6" }}>
                AES-256 at-rest encryption, async WAL shipping, Point-in-Time Recovery (PITR), dynamic quantization, and VantaDB Cloud Beta.
              </p>
            </div>
          </div>
        </section>
      </main>
    </div>
  );
}
