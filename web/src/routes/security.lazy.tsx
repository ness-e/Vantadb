import { createLazyRoute } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";
import "../styles/security.css";

export const Route = createLazyRoute("/security")({
  component: SecurityPage,
  pendingComponent: PendingComponent,
});

function SecurityPage() {
  return (
    <div>
      <NbSubpageHero
        num="13"
        title={
          <span>
            Zero Trust.
            <br />
            100% Local.
          </span>
        }
        sub="VantaDB is designed with a strict security boundary. No cloud pings, no telemetry, no data exfiltration."
      />

      <main>
        <section className="nb-section">
          <div className="nb-inner">
            <h2 className="security-heading">
              CORE PRINCIPLES
            </h2>

            <div className="nb-grid nb-grid--cols-3 security-grid">
              <div className="nb-cell">
                <div className="security-cell-label">
                  NO TELEMETRY
                </div>
                <p className="text-sm text-muted leading-relaxed m-0 mt-4">
                  We do not track your usage. The VantaDB core library contains zero analytics,
                  tracking pixels, or outbound HTTP requests. Your data never leaves your
                  environment.
                </p>
              </div>
              <div className="nb-cell">
                <div className="security-cell-label">
                  AES-256-GCM ENCRYPTION
                </div>
                <p className="text-sm text-muted leading-relaxed m-0 mt-4">
                  Enterprise at-rest encryption is currently on the roadmap for Phase 5. This will
                  ensure that WAL and LSM-tree SSTables are fully encrypted on disk.
                </p>
              </div>
              <div className="nb-cell">
                <div className="security-cell-label">
                  MEMORY SAFETY
                </div>
                <p className="text-sm text-muted leading-relaxed m-0 mt-4">
                  Written entirely in Rust, the engine is immune to buffer overflows, use-after-free
                  vulnerabilities, and memory leaks that plague traditional C/C++ vector databases.
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
                    className="font-display text-2xl font-extrabold security-block-heading"
                  >
                    Built on Rust. Safe by default.
                  </h2>
                  <p className="text-sm security-block-text">
                    Install VantaDB in one command.
                  </p>
                </div>
                <code className="font-mono text-lg font-bold security-block-code">
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
