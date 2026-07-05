import { createLazyRoute } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";

export const Route = createLazyRoute("/config")({
  component: ConfigPage,
  pendingComponent: PendingComponent,
});

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
    <div>
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

      <main>
        <section className="nb-section">
          <div className="nb-inner">
            <div className="nb-label">01 / 02 — Setup Comparison</div>

            <div className="nb-grid nb-grid--cols-2" style={{ marginTop: "3rem" }}>
              <div className="nb-cell">
                <div className="nb-label" style={{ color: "var(--steel)" }}>
                  LEGACY — Pages of config
                </div>
                <ul className="nb-list mt-4">
                  {LEGACY_CONFIG.map((item) => (
                    <li key={item}>{item}</li>
                  ))}
                </ul>
              </div>
              <div className="nb-cell" style={{ borderLeft: "2px solid var(--amber)" }}>
                <div className="nb-label nb-label--amber">VANTADB — Zero lines</div>
                <ul className="nb-list mt-4">
                  {VANTA_CONFIG.map((item) => (
                    <li key={item}>{item}</li>
                  ))}
                </ul>
              </div>
            </div>
          </div>
        </section>

        <section className="nb-section">
          <div className="nb-inner">
            <div className="nb-label">02 / 02 — Code: From 50 Lines to 1</div>

            <div className="nb-grid nb-grid--cols-2" style={{ marginTop: "3rem" }}>
              <div className="nb-cell" style={{ padding: 0, background: "var(--black)" }}>
                <div
                  className="flex items-center justify-between px-5 py-3"
                  style={{ borderBottom: "1px solid var(--border)" }}
                >
                  <span className="font-mono text-[0.6rem] text-steel uppercase tracking-[0.08em]">
                    legacy_setup.py
                  </span>
                  <span className="font-mono text-[0.55rem] text-muted">50+ lines</span>
                </div>
                <pre className="m-0 p-6 font-mono text-[0.72rem] leading-relaxed text-muted overflow-x-auto whitespace-pre">
                  <code>{LEGACY_CODE}</code>
                </pre>
              </div>
              <div
                className="nb-cell"
                style={{
                  padding: 0,
                  background: "var(--black)",
                  borderLeft: "2px solid var(--amber)",
                }}
              >
                <div
                  className="flex items-center justify-between px-5 py-3"
                  style={{ borderBottom: "1px solid var(--border)" }}
                >
                  <span className="font-mono text-[0.6rem] text-amber uppercase tracking-[0.08em]">
                    vantadb_setup.py
                  </span>
                  <span className="font-mono text-[0.55rem] text-amber">3 lines</span>
                </div>
                <pre className="m-0 p-6 font-mono text-[0.72rem] leading-relaxed text-foreground overflow-x-auto whitespace-pre">
                  <code>{VANTA_CODE}</code>
                </pre>
              </div>
            </div>
          </div>
        </section>

        <section className="nb-section nb-bg-dot">
          <div className="nb-inner">
            <div className="nb-block-amber">
              <div className="flex items-center justify-between gap-4 flex-wrap">
                <div>
                  <div className="nb-label" style={{ color: "var(--text-on-amber)" }}>
                    GET STARTED
                  </div>
                  <h2
                    className="font-display text-2xl font-extrabold"
                    style={{ color: "var(--text-on-amber)" }}
                  >
                    Zero config. Ship faster.
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
