import { createLazyRoute } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbBlockAmber, NbSection, NbSectionHeader } from "@/components/nb";
import { PendingComponent } from "@/components/PendingComponent";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp } from "@/lib/motion-utils";
import "../styles/config.css";

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
  const setupRef = useRef<HTMLElement>(null);
  const codeRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = setupRef.current?.querySelectorAll<HTMLElement>(".nc-cfg-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, setupRef);

  useAnimationSafe(() => {
    const parts = codeRef.current?.querySelectorAll<HTMLElement>(".nc-cfg-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, codeRef);

  return (
    <div>
      <NbSubpageHero
        pattern="p07"
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
        <NbSection ref={setupRef} ariaLabel="Setup comparison">
          <NbSectionHeader
            monoLabel="[SETUP]"
            headline="Setup comparison."
            sub="No API keys, no host/port/password, no cloud credentials — just a file path."
          />

          <div className="nc-cfg-grid nc-cfg-part">
            <div className="nc-cfg-col">
              <span className="nc-cfg-col-title nc-cfg-col-title--danger">
                LEGACY — Pages of config
              </span>
              <ul className="nc-cfg-list">
                {LEGACY_CONFIG.map((item) => (
                  <li key={item} className="nc-cfg-item">
                    {item}
                  </li>
                ))}
              </ul>
            </div>
            <div className="nc-cfg-col nc-cfg-col--vanta">
              <span className="nc-cfg-col-title nc-cfg-col-title--amber">VANTADB — Zero lines</span>
              <ul className="nc-cfg-list">
                {VANTA_CONFIG.map((item) => (
                  <li key={item} className="nc-cfg-item">
                    {item}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </NbSection>

        <NbSection ref={codeRef} ariaLabel="Code comparison">
          <NbSectionHeader
            monoLabel="[CODE]"
            headline="From 50 lines to 3."
            sub="Compare the configuration overhead of a traditional stack against VantaDB's zero-config approach."
          />

          <div className="nc-cfg-grid nc-cfg-part">
            <div className="nc-cfg-code-col">
              <div className="nc-cfg-code-header">
                <span className="nc-cfg-code-name">legacy_setup.py</span>
                <span className="nc-cfg-code-lines">50+ lines</span>
              </div>
              <pre className="nc-cfg-code-pre">
                <code>{LEGACY_CODE}</code>
              </pre>
            </div>
            <div className="nc-cfg-code-col nc-cfg-code-col--amber">
              <div className="nc-cfg-code-header">
                <span className="nc-cfg-code-name nc-cfg-code-name--amber">vantadb_setup.py</span>
                <span className="nc-cfg-code-lines nc-cfg-code-lines--amber">3 lines</span>
              </div>
              <pre className="nc-cfg-code-pre nc-cfg-code-pre--fg">
                <code>{VANTA_CODE}</code>
              </pre>
            </div>
          </div>
        </NbSection>

        <NbSection className="nb-bg-dot" ariaLabel="Get started">
          <NbBlockAmber as="div">
            <div className="nc-cfg-cta">
              <div>
                <h2 className="nc-cfg-cta-heading">Zero config. Ship faster.</h2>
                <p className="nc-cfg-cta-sub">Install VantaDB in one command.</p>
              </div>
              <code className="nc-cfg-cta-code">pip install vantadb-py</code>
            </div>
          </NbBlockAmber>
        </NbSection>
      </main>
    </div>
  );
}
