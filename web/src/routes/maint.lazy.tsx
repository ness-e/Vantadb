import { createLazyRoute } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";

export const Route = createLazyRoute("/maint")({
  component: MaintPage,
  pendingComponent: PendingComponent,
});

const LEGACY_OPS_PROBLEMS = [
  "Pinecone: monitor pod health, scale pods, watch quotas",
  "Redis: replication lag, OOM handling, failover testing",
  "S3: lifecycle policies, bucket versioning, access audits",
  "Network: DNS changes, TLS certs, firewall rules",
  "Alerting: 3+ dashboards, pager duty rotations",
];

const NO_OPS_LIST = [
  "No daemon to monitor — runs in your process",
  "No cluster scaling — uses your app's resources",
  "No network config — local file access only",
  "No dashboards — your app's observability is enough",
  "Upgrades: `pip install --upgrade vantadb-py`",
];

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

function MaintPage() {
  return (
    <div>
      <NbSubpageHero
        num="11"
        title={
          <span>
            Zero ops.
            <br />
            Just upgrade.
          </span>
        }
        sub="No daemons to monitor, no clusters to scale, no patches to schedule. VantaDB runs embedded in your process — the database is just another import."
      />

      <main>
        <section className="nb-section">
          <div className="nb-inner">
            <h2
              style={{
                fontFamily: "var(--font-display)",
                fontSize: "var(--text-title)",
                fontWeight: 700,
                color: "var(--foreground)",
                margin: "0 0 var(--space-md)",
              }}
            >
              Maintenance Comparison
            </h2>

            <div className="nb-grid nb-grid--cols-2" style={{ marginTop: "3rem" }}>
              <div className="nb-cell">
                <div
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "var(--text-label)",
                    color: "var(--steel)",
                    fontWeight: 700,
                    marginBottom: "var(--space-sm)",
                  }}
                >
                  LEGACY — 3 services to maintain
                </div>
                <ul
                  className="flex flex-col gap-3 mt-4"
                  style={{ listStyle: "none", margin: 0, padding: 0 }}
                >
                  {LEGACY_OPS_PROBLEMS.map((item) => (
                    <li key={item} className="flex gap-3 text-sm text-muted leading-relaxed">
                      <span
                        className="font-mono font-bold flex-shrink-0"
                        style={{ color: "var(--steel)", minWidth: "1rem" }}
                      >
                        ✗
                      </span>
                      {item}
                    </li>
                  ))}
                </ul>
              </div>
              <div className="nb-cell" style={{ borderLeft: "2px solid var(--amber)" }}>
                <div
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "var(--text-label)",
                    color: "var(--amber)",
                    fontWeight: 700,
                    marginBottom: "var(--space-sm)",
                  }}
                >
                  VANTADB — nothing to maintain
                </div>
                <ul
                  className="flex flex-col gap-3 mt-4"
                  style={{ listStyle: "none", margin: 0, padding: 0 }}
                >
                  {NO_OPS_LIST.map((item) => (
                    <li key={item} className="flex gap-3 text-sm text-foreground leading-relaxed">
                      <span
                        className="font-mono font-bold flex-shrink-0"
                        style={{ color: "var(--amber)", minWidth: "1rem" }}
                      >
                        ✓
                      </span>
                      {item}
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          </div>
        </section>

        <section className="nb-section">
          <div className="nb-inner">
            <h2
              style={{
                fontFamily: "var(--font-display)",
                fontSize: "var(--text-title)",
                fontWeight: 700,
                color: "var(--foreground)",
                margin: "0 0 var(--space-md)",
              }}
            >
              Weekly Ops Timeline
            </h2>
            <h2 className="font-display text-[clamp(1.5rem,3vw,2rem)] font-extrabold tracking-[-0.04em] mt-4 mb-12 leading-tight">
              From 4 hours to 30 seconds.
            </h2>

            <div className="nb-grid nb-grid--cols-2">
              <div className="nb-cell">
                <div className="nb-label" style={{ color: "var(--steel)" }}>
                  LEGACY WEEKLY OPS
                </div>
                {LEGACY_OPS.map((item, i) => (
                  <div
                    key={item.task}
                    className="grid grid-cols-[1fr_60px] gap-4 py-3 items-center"
                    style={{
                      borderBottom: i < LEGACY_OPS.length - 1 ? "1px solid var(--border)" : "none",
                    }}
                  >
                    <div className="flex gap-2 text-sm text-muted leading-relaxed">
                      <span
                        className="font-mono font-bold flex-shrink-0"
                        style={{ color: "var(--steel)" }}
                      >
                        ✗
                      </span>
                      {item.task}
                    </div>
                    <span
                      className="font-mono text-[0.65rem] text-right"
                      style={{ color: "var(--steel)" }}
                    >
                      {item.time}
                    </span>
                  </div>
                ))}
                <div
                  className="font-display text-2xl font-extrabold tracking-[-0.04em] mt-6"
                  style={{ color: "var(--steel)" }}
                >
                  ~4h / week
                </div>
              </div>
              <div className="nb-cell" style={{ borderLeft: "2px solid var(--amber)" }}>
                <div className="nb-label nb-label--amber">VANTADB WEEKLY OPS</div>
                {VANTA_OPS.map((item, i) => (
                  <div
                    key={item.task}
                    className="grid grid-cols-[1fr_60px] gap-4 py-3 items-center"
                    style={{
                      borderBottom: i < VANTA_OPS.length - 1 ? "1px solid var(--border)" : "none",
                    }}
                  >
                    <div className="flex gap-2 text-sm text-foreground leading-relaxed">
                      <span
                        className="font-mono font-bold flex-shrink-0"
                        style={{ color: "var(--amber)" }}
                      >
                        ✓
                      </span>
                      {item.task}
                    </div>
                    <span className="font-mono text-[0.65rem] text-amber text-right">
                      {item.time}
                    </span>
                  </div>
                ))}
                <div className="font-display text-2xl font-extrabold tracking-[-0.04em] mt-6 text-amber">
                  ~30s / week
                </div>
              </div>
            </div>

            <div className="nb-grid nb-grid--cols-2 mt-0">
              <div
                className="nb-cell"
                style={{
                  gridColumn: "1 / -1",
                  display: "grid",
                  gridTemplateColumns: "140px 1fr",
                  gap: "2rem",
                  alignItems: "center",
                }}
              >
                <span className="font-mono text-[0.6rem] text-amber uppercase tracking-[0.08em]">
                  KEY INSIGHT
                </span>
                <p className="text-sm text-muted leading-relaxed m-0">
                  Because VantaDB runs as an embedded library — not a separate server — there's
                  nothing to deploy, monitor, or scale independently. Your application's lifecycle{" "}
                  <em>is</em> the database lifecycle. No pager duty. No 2 AM wakeups.
                </p>
              </div>
            </div>
          </div>
        </section>

        <section className="nb-section nb-bg-dot">
          <div className="nb-inner">
            <div className="nb-block-amber">
              <div className="flex items-center justify-between gap-4 flex-wrap">
                <div>
                  <h2
                    className="font-display text-2xl font-extrabold"
                    style={{ color: "var(--text-on-amber)" }}
                  >
                    Zero ops. Ship and sleep.
                  </h2>
                  <p className="text-sm" style={{ color: "var(--text-on-amber)", opacity: 0.8 }}>
                    Install VantaDB in one command.
                  </p>
                </div>
                <code
                  className="font-mono text-lg font-bold"
                  style={{ color: "var(--text-on-amber)" }}
                >
                  pip install vantadb-py
                </code>
              </div>
            </div>
          </div>
        </section>
      </main>
    </div>
  );
}
