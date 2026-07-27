import { createLazyRoute } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbBlockAmber, NbSection, NbSectionHeader } from "@/components/nb";
import { PendingComponent } from "@/components/PendingComponent";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp } from "@/lib/motion-utils";
import "../styles/security.css";

export const Route = createLazyRoute("/security")({
  component: SecurityPage,
  pendingComponent: PendingComponent,
});

function SecurityPage() {
  const principlesRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = principlesRef.current?.querySelectorAll<HTMLElement>(".nc-sec-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, principlesRef);

  return (
    <div>
      <NbSubpageHero
        pattern="p12"
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
        <NbSection ref={principlesRef} ariaLabel="Core principles">
          <NbSectionHeader
            monoLabel="[PRINCIPLES]"
            headline="Security by design."
            sub="Every layer of VantaDB is built with a strict security boundary \u2014 no cloud pings, no telemetry, no data exfiltration."
          />

          <div className="nc-sec-grid">
            <div className="nc-sec-card nc-sec-part">
              <span className="nc-sec-stamp">No Telemetry</span>
              <p className="nc-sec-body">
                We do not track your usage. The VantaDB core library contains zero analytics,
                tracking pixels, or outbound HTTP requests. Your data never leaves your environment.
              </p>
            </div>
            <div className="nc-sec-card nc-sec-part">
              <span className="nc-sec-stamp">AES-256-GCM</span>
              <p className="nc-sec-body">
                Enterprise at-rest encryption is currently on the roadmap for Phase 5. This will
                ensure that WAL and LSM-tree SSTables are fully encrypted on disk.
              </p>
            </div>
            <div className="nc-sec-card nc-sec-part">
              <span className="nc-sec-stamp">Memory Safe</span>
              <p className="nc-sec-body">
                Written entirely in Rust, the engine is immune to buffer overflows, use-after-free
                vulnerabilities, and memory leaks that plague traditional C/C++ vector databases.
              </p>
            </div>
          </div>
        </NbSection>

        <NbSection className="nb-bg-dot" ariaLabel="Get started">
          <NbBlockAmber as="div">
            <div className="nc-sec-cta-row">
              <div>
                <h2 className="nc-sec-cta-heading">Built on Rust. Safe by default.</h2>
                <p className="nc-sec-cta-sub">Install VantaDB in one command.</p>
              </div>
              <code className="nc-sec-cta-code">pip install vantadb-py</code>
            </div>
          </NbBlockAmber>
        </NbSection>
      </main>
    </div>
  );
}
