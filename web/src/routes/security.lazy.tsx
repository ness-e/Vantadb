import { createLazyRoute } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";

export const Route = createLazyRoute("/security")({
  component: SecurityPage,
  pendingComponent: PendingComponent,
});

function SecurityPage() {
  return (
    <div className="engine-page">
      <SwissSubpageHero
        num="02"
        eyebrow="Security Posture"
        title={
          <span>
            Zero Trust.
            <br />
            100% Local.
          </span>
        }
        sub="VantaDB is designed with a strict security boundary. No cloud pings, no telemetry, no data exfiltration."
      />

      <main className="engine-main">
        <section className="engine-section">
          <span className="swiss-eyebrow">Core Principles</span>

          <div style={{ marginTop: "3rem", display: "grid", gap: "2rem" }}>
            <div
              style={{
                padding: "2rem",
                border: "1px solid var(--border)",
                background: "var(--surface-raised)",
              }}
            >
              <h3
                style={{
                  margin: "0 0 1rem 0",
                  color: "var(--amber)",
                  fontFamily: "var(--font-mono)",
                  fontSize: "1rem",
                }}
              >
                [01] No Telemetry
              </h3>
              <p style={{ color: "var(--steel)", lineHeight: "1.6", margin: 0 }}>
                We do not track your usage. The VantaDB core library contains zero analytics,
                tracking pixels, or outbound HTTP requests. Your data never leaves your environment.
              </p>
            </div>

            <div
              style={{
                padding: "2rem",
                border: "1px solid var(--border)",
                background: "var(--surface-raised)",
              }}
            >
              <h3
                style={{
                  margin: "0 0 1rem 0",
                  color: "var(--amber)",
                  fontFamily: "var(--font-mono)",
                  fontSize: "1rem",
                }}
              >
                [02] AES-256-GCM Encryption (Roadmap)
              </h3>
              <p style={{ color: "var(--steel)", lineHeight: "1.6", margin: 0 }}>
                Enterprise at-rest encryption is currently on the roadmap for Phase 5. This will
                ensure that WAL and LSM-tree SSTables are fully encrypted on disk.
              </p>
            </div>

            <div
              style={{
                padding: "2rem",
                border: "1px solid var(--border)",
                background: "var(--surface-raised)",
              }}
            >
              <h3
                style={{
                  margin: "0 0 1rem 0",
                  color: "var(--amber)",
                  fontFamily: "var(--font-mono)",
                  fontSize: "1rem",
                }}
              >
                [03] Memory Safety
              </h3>
              <p style={{ color: "var(--steel)", lineHeight: "1.6", margin: 0 }}>
                Written entirely in Rust, the engine is immune to buffer overflows, use-after-free
                vulnerabilities, and memory leaks that plague traditional C/C++ vector databases.
              </p>
            </div>
          </div>
        </section>
      </main>
    </div>
  );
}

export function PendingComponent() {
  return (
    <div style={{ display: "flex", justifyContent: "center", alignItems: "center", height: "60vh", color: "var(--muted)" }}>
      <div>Loading...</div>
    </div>
  );
}
