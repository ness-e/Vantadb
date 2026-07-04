import { createLazyRoute } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";

export const Route = createLazyRoute("/config")({
  component: ConfigPage,
  pendingComponent: PendingComponent,
});

// ── Data ─────────────────────────────────────────────────────────────────────
const LEGACY_CONFIG = [
  "Pinecone: API key, environment, pod type, index config",
  "Redis: host, port, password, TLS, cluster mode",
  "S3: region, bucket, credentials, IAM roles, CORS",
  "Schema migrations: define, version, migrate, rollback",
  "Connection pooling: tune pool size, timeouts, retries",
];

const VANTA_CONFIG = [
  "No API keys to configure or rotate",
  "No host/port/password — connect to a file path",
  "No cloud credentials or IAM policies",
  "Schema-free: insert data, DB infers types",
  "Auto-indexing: vectors indexed automatically",
];

const LEGACY_CODE = `# Set up 3 services + auth + schema
import pinecone
import redis
import boto3

pinecone.init(api_key=os.environ["PINECONE_KEY"],
              environment="us-east-1-aws")

r = redis.Redis(host=os.environ["REDIS_HOST"],
                port=6379,
                password=os.environ["REDIS_PW"],
                ssl=True)

s3 = boto3.client("s3",
                  region_name="us-east-1",
                  aws_access_key_id=...,
                  aws_secret_access_key=...)

# Define schema, create index, set up cache...
# (50+ lines of config)`;

const VANTA_CODE = `import vantadb_py

db = vantadb_py.VantaDB("./my_db.vdb")

# Ready. No config, no schema, no cloud.`;

function ConfigPage() {
  return (
    <div className="engine-page">
      <SwissSubpageHero
        num="10"
        eyebrow="Configuration"
        title={
          <span>
            Zero config.
            <br />
            Just connect.
          </span>
        }
        sub="No YAML, no .env, no migration scripts. VantaDB is schema-free and self-configuring. Point it at a file path and start querying."
      />

      <main className="engine-main">
        {/* Section 1: Comparison */}
        <section className="engine-section engine-section--bordered">
          <span className="swiss-eyebrow">01 / 02 — Setup Comparison</span>
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
                Legacy — Pages of config
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
                {LEGACY_CONFIG.map((item, i) => (
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
                        color: "var(--steel)",
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
                VantaDB — Zero lines
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
                {VANTA_CONFIG.map((item, i) => (
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

        {/* Section 2: Code comparison */}
        <section className="engine-section">
          <span className="swiss-eyebrow">02 / 02 — Code: From 50 Lines to 1</span>
          <div
            className="swiss-grid-12"
            style={{ alignItems: "start", marginTop: "3rem", gap: "1px" }}
          >
            {/* Legacy code */}
            <div
              className="col-span-6"
              style={{ border: "1px solid var(--border)", background: "var(--block-dark-bg)" }}
            >
              <div
                style={{
                  padding: "0.65rem 1.25rem",
                  borderBottom: "1px solid var(--block-dark-border)",
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                }}
              >
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.6rem",
                    color: "var(--steel)",
                    textTransform: "uppercase",
                    letterSpacing: "0.08em",
                  }}
                >
                  legacy_setup.py
                </span>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.55rem",
                    color: "var(--block-dark-muted)",
                  }}
                >
                  50+ lines
                </span>
              </div>
              <pre
                style={{
                  margin: 0,
                  padding: "1.5rem",
                  fontFamily: "var(--font-mono)",
                  fontSize: "0.72rem",
                  lineHeight: 1.65,
                  color: "var(--block-dark-muted)",
                  overflowX: "auto",
                  whiteSpace: "pre",
                }}
              >
                <code>{LEGACY_CODE}</code>
              </pre>
            </div>

            {/* VantaDB code */}
            <div
              className="col-span-6"
              style={{
                border: "1px solid var(--border)",
                borderLeft: "2px solid var(--amber)",
                background: "var(--block-dark-bg)",
              }}
            >
              <div
                style={{
                  padding: "0.65rem 1.25rem",
                  borderBottom: "1px solid var(--block-dark-border)",
                  display: "flex",
                  justifyContent: "space-between",
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
                  vantadb_setup.py
                </span>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.55rem",
                    color: "var(--amber)",
                  }}
                >
                  3 lines
                </span>
              </div>
              <pre
                style={{
                  margin: 0,
                  padding: "1.5rem",
                  fontFamily: "var(--font-mono)",
                  fontSize: "0.72rem",
                  lineHeight: 1.65,
                  color: "var(--block-dark-text)",
                  overflowX: "auto",
                  whiteSpace: "pre",
                }}
              >
                <code>{VANTA_CODE}</code>
              </pre>
            </div>
          </div>
        </section>
      </main>

      <style>{`
        @media (max-width: 768px) {
          .swiss-grid-12 { grid-template-columns: 1fr !important; }
        }
      `}</style>
    </div>
  );
}
