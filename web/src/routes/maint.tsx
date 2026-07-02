import { createFileRoute } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";

export const Route = createFileRoute("/maint")({
  head: () => ({
    meta: [
      { title: "VantaDB — Zero Maintenance Operations" },
      {
        name: "description",
        content:
          "No daemons to monitor, no clusters to scale, no patches to schedule. VantaDB runs as an embedded library — your app IS the database server.",
      },
    ],
  }),
  component: MaintPage,
});

// ── Data ─────────────────────────────────────────────────────────────────────
const LEGACY_OPS = [
  { task: "Review 3 monitoring dashboards", time: "30m" },
  { task: "Check Pinecone pod utilization", time: "15m" },
  { task: "Rotate Redis credentials", time: "20m" },
  { task: "Review S3 access logs", time: "15m" },
  { task: "Patch/update 3 services", time: "2h" },
  { task: "Respond to 2–3 alerts", time: "45m" },
];

const VANTA_OPS = [
  { task: "Check for new version on PyPI", time: "5s" },
  { task: "Run pip install --upgrade", time: "10s" },
  { task: "Verify app still works", time: "15s" },
  { task: "Done.", time: "" },
];

const NO_OPS_LIST = [
  "No daemon to monitor — runs in your process",
  "No cluster scaling — uses your app's resources",
  "No network config — local file access only",
  "No dashboards — your app's observability is enough",
  "Upgrades: `pip install --upgrade vantadb-py`",
];

const LEGACY_OPS_PROBLEMS = [
  "Pinecone: monitor pod health, scale pods, watch quotas",
  "Redis: replication lag, OOM handling, failover testing",
  "S3: lifecycle policies, bucket versioning, access audits",
  "Network: DNS changes, TLS certs, firewall rules",
  "Alerting: 3+ dashboards, pager duty rotations",
];

function MaintPage() {
  return (
    <div className="engine-page">
      <SwissSubpageHero
        num="11"
        eyebrow="Operations & Maintenance"
        title={
          <span>
            Zero ops.
            <br />
            Just upgrade.
          </span>
        }
        sub="No daemons to monitor, no clusters to scale, no patches to schedule. VantaDB runs embedded in your process — the database is just another import."
      />

      <main className="engine-main">
        {/* Section 1: Comparison */}
        <section className="engine-section engine-section--bordered">
          <span className="swiss-eyebrow">01 / 02 — Maintenance Comparison</span>
          <div
            className="swiss-grid-12"
            style={{ alignItems: "start", marginTop: "3rem", gap: "1px" }}
          >
            <div
              className="col-span-6"
              style={{ border: "1px solid var(--border)", padding: "2.5rem" }}
            >
              <h2
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "1.1rem",
                  fontWeight: 800,
                  letterSpacing: "-0.03em",
                  color: "var(--steel)",
                  marginBottom: "2rem",
                  textTransform: "uppercase",
                }}
              >
                Legacy — 3 services to maintain
              </h2>
              <ul
                style={{
                  listStyle: "none",
                  margin: 0,
                  padding: 0,
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.9rem",
                }}
              >
                {LEGACY_OPS_PROBLEMS.map((item, i) => (
                  <li
                    key={i}
                    style={{
                      display: "flex",
                      gap: "0.75rem",
                      fontFamily: "var(--font-sans)",
                      fontSize: "0.82rem",
                      color: "var(--muted)",
                      lineHeight: 1.5,
                    }}
                  >
                    <span
                      style={{
                        color: "#ff3b30",
                        fontWeight: 700,
                        minWidth: "1rem",
                        fontFamily: "var(--font-mono)",
                        flexShrink: 0,
                      }}
                    >
                      ✗
                    </span>
                    {item}
                  </li>
                ))}
              </ul>
            </div>
            <div
              className="col-span-6"
              style={{
                border: "1px solid var(--border)",
                borderLeft: "2px solid var(--amber)",
                padding: "2.5rem",
                background: "var(--surface)",
              }}
            >
              <h2
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "1.1rem",
                  fontWeight: 800,
                  letterSpacing: "-0.03em",
                  color: "var(--amber)",
                  marginBottom: "2rem",
                  textTransform: "uppercase",
                }}
              >
                VantaDB — nothing to maintain
              </h2>
              <ul
                style={{
                  listStyle: "none",
                  margin: 0,
                  padding: 0,
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.9rem",
                }}
              >
                {NO_OPS_LIST.map((item, i) => (
                  <li
                    key={i}
                    style={{
                      display: "flex",
                      gap: "0.75rem",
                      fontFamily: "var(--font-sans)",
                      fontSize: "0.82rem",
                      color: "var(--foreground)",
                      lineHeight: 1.5,
                    }}
                  >
                    <span
                      style={{
                        color: "var(--amber)",
                        fontWeight: 700,
                        minWidth: "1rem",
                        fontFamily: "var(--font-mono)",
                        flexShrink: 0,
                      }}
                    >
                      ✓
                    </span>
                    {item}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </section>

        {/* Section 2: Weekly Ops Timeline */}
        <section className="engine-section">
          <span className="swiss-eyebrow">02 / 02 — Weekly Ops Timeline</span>
          <h2
            style={{
              fontFamily: "var(--font-display)",
              fontSize: "clamp(1.5rem, 3vw, 2rem)",
              fontWeight: 800,
              letterSpacing: "-0.04em",
              margin: "1.25rem 0 3rem",
              lineHeight: 1.05,
            }}
          >
            From 4 hours to 30 seconds.
          </h2>

          <div
            style={{
              display: "grid",
              gridTemplateColumns: "1fr 1fr",
              gap: "1px",
              background: "var(--border)",
              border: "1px solid var(--border)",
            }}
          >
            {/* Legacy */}
            <div
              style={{
                background: "var(--background)",
                padding: "2.5rem",
                display: "flex",
                flexDirection: "column",
                gap: "0",
              }}
            >
              <div
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "0.6rem",
                  color: "var(--steel)",
                  textTransform: "uppercase",
                  letterSpacing: "0.08em",
                  marginBottom: "1.5rem",
                }}
              >
                Legacy Weekly Ops
              </div>
              {LEGACY_OPS.map((item, i) => (
                <div
                  key={i}
                  style={{
                    display: "grid",
                    gridTemplateColumns: "1fr 60px",
                    gap: "1rem",
                    padding: "0.75rem 0",
                    borderBottom: i < LEGACY_OPS.length - 1 ? "1px solid var(--border)" : "none",
                    alignItems: "center",
                  }}
                >
                  <div
                    style={{
                      display: "flex",
                      gap: "0.6rem",
                      fontFamily: "var(--font-sans)",
                      fontSize: "0.8rem",
                      color: "var(--muted)",
                      lineHeight: 1.4,
                    }}
                  >
                    <span
                      style={{
                        color: "#ff3b30",
                        fontFamily: "var(--font-mono)",
                        fontWeight: 700,
                        flexShrink: 0,
                      }}
                    >
                      ✗
                    </span>
                    {item.task}
                  </div>
                  <span
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.65rem",
                      color: "#ff3b30",
                      textAlign: "right",
                    }}
                  >
                    {item.time}
                  </span>
                </div>
              ))}
              <div
                style={{
                  marginTop: "2rem",
                  fontFamily: "var(--font-display)",
                  fontSize: "1.4rem",
                  fontWeight: 800,
                  letterSpacing: "-0.04em",
                  color: "#ff3b30",
                }}
              >
                ~4h / week
              </div>
            </div>

            {/* VantaDB */}
            <div
              style={{
                background: "var(--surface-raised)",
                padding: "2.5rem",
                borderLeft: "2px solid var(--amber)",
                display: "flex",
                flexDirection: "column",
                gap: "0",
              }}
            >
              <div
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "0.6rem",
                  color: "var(--amber)",
                  textTransform: "uppercase",
                  letterSpacing: "0.08em",
                  marginBottom: "1.5rem",
                }}
              >
                VantaDB Weekly Ops
              </div>
              {VANTA_OPS.map((item, i) => (
                <div
                  key={i}
                  style={{
                    display: "grid",
                    gridTemplateColumns: "1fr 60px",
                    gap: "1rem",
                    padding: "0.75rem 0",
                    borderBottom: i < VANTA_OPS.length - 1 ? "1px solid var(--border)" : "none",
                    alignItems: "center",
                  }}
                >
                  <div
                    style={{
                      display: "flex",
                      gap: "0.6rem",
                      fontFamily: "var(--font-sans)",
                      fontSize: "0.8rem",
                      color: "var(--foreground)",
                      lineHeight: 1.4,
                    }}
                  >
                    <span
                      style={{
                        color: "var(--amber)",
                        fontFamily: "var(--font-mono)",
                        fontWeight: 700,
                        flexShrink: 0,
                      }}
                    >
                      ✓
                    </span>
                    {item.task}
                  </div>
                  <span
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.65rem",
                      color: "var(--amber)",
                      textAlign: "right",
                    }}
                  >
                    {item.time}
                  </span>
                </div>
              ))}
              <div
                style={{
                  marginTop: "2rem",
                  fontFamily: "var(--font-display)",
                  fontSize: "1.4rem",
                  fontWeight: 800,
                  letterSpacing: "-0.04em",
                  color: "var(--amber)",
                }}
              >
                ~30s / week
              </div>
            </div>
          </div>

          {/* Closing note */}
          <div
            style={{
              border: "1px solid var(--border)",
              borderTop: "none",
              background: "var(--surface)",
              padding: "1.5rem 2.5rem",
              display: "grid",
              gridTemplateColumns: "140px 1fr",
              gap: "2rem",
              alignItems: "center",
            }}
          >
            <span
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "0.6rem",
                color: "var(--amber)",
                textTransform: "uppercase",
                letterSpacing: "0.08em",
              }}
            >
              KEY INSIGHT
            </span>
            <p
              style={{
                fontFamily: "var(--font-sans)",
                fontSize: "0.82rem",
                color: "var(--muted)",
                lineHeight: 1.7,
                margin: 0,
              }}
            >
              Because VantaDB runs as an embedded library — not a separate server — there's nothing
              to deploy, monitor, or scale independently. Your application's lifecycle <em>is</em>{" "}
              the database lifecycle. No pager duty. No 2 AM wakeups.
            </p>
          </div>
        </section>
      </main>
    </div>
  );
}
