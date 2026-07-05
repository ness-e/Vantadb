import { createLazyRoute } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader, NbBlockAmber } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
import { PendingComponent } from "@/components/PendingComponent";
import "../styles/security.css";

export const Route = createLazyRoute("/security")({
  component: SecurityPage,
  pendingComponent: PendingComponent,
});

function SecurityPage() {
  const principlesRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(principlesRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, principlesRef);

  return (
    <div>
      <NbSubpageHero
        pattern="p10"
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
            sub="Every layer of VantaDB is built with a strict security boundary — no cloud pings, no telemetry, no data exfiltration."
          />

          <div className="nb-engine-part">
            <div className="nb-grid nb-grid--cols-3 security-grid">
              <div className="nb-card-frame">
                <span className="nb-mono-label">NO TELEMETRY</span>
                <p className="nb-card-frame-desc">
                  We do not track your usage. The VantaDB core library contains zero analytics,
                  tracking pixels, or outbound HTTP requests. Your data never leaves your
                  environment.
                </p>
              </div>
              <div className="nb-card-frame">
                <span className="nb-mono-label">AES-256-GCM ENCRYPTION</span>
                <p className="nb-card-frame-desc">
                  Enterprise at-rest encryption is currently on the roadmap for Phase 5. This will
                  ensure that WAL and LSM-tree SSTables are fully encrypted on disk.
                </p>
              </div>
              <div className="nb-card-frame">
                <span className="nb-mono-label">MEMORY SAFETY</span>
                <p className="nb-card-frame-desc">
                  Written entirely in Rust, the engine is immune to buffer overflows, use-after-free
                  vulnerabilities, and memory leaks that plague traditional C/C++ vector databases.
                </p>
              </div>
            </div>
          </div>
        </NbSection>

        <NbSection className="nb-bg-dot" ariaLabel="Get started">
          <NbBlockAmber as="div">
            <div className="security-cta-row">
              <div>
                <h2 className="security-cta-heading">Built on Rust. Safe by default.</h2>
                <p className="security-cta-sub">Install VantaDB in one command.</p>
              </div>
              <code className="security-cta-code">pip install vantadb-py</code>
            </div>
          </NbBlockAmber>
        </NbSection>
      </main>
    </div>
  );
}
