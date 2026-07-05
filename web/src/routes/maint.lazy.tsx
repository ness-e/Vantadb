import { createLazyRoute } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader, NbBlockAmber } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
import { PendingComponent } from "@/components/PendingComponent";
import "../styles/maint.css";

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
  const overviewRef = useRef<HTMLElement>(null);
  const timelineRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(overviewRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, overviewRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(timelineRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, timelineRef);

  return (
    <div>
      <NbSubpageHero
        pattern="p08"
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
        <NbSection ref={overviewRef} ariaLabel="Maintenance overview">
          <NbSectionHeader
            monoLabel="[OVERVIEW]"
            headline="Maintenance comparison."
            sub="Three managed services become one embedded library — nothing to deploy, monitor, or scale."
          />

          <div className="nb-engine-part">
            <div className="nb-grid nb-grid--cols-2 maint-grid">
              <div className="nb-cell">
                <div className="maint-label-legacy">LEGACY — 3 services to maintain</div>
                <ul className="maint-list">
                  {LEGACY_OPS_PROBLEMS.map((item) => (
                    <li key={item} className="maint-list-item">
                      <span className="maint-icon-steel">✗</span>
                      {item}
                    </li>
                  ))}
                </ul>
              </div>
              <div className="nb-cell maint-cell-border">
                <div className="maint-label-amber">VANTADB — nothing to maintain</div>
                <ul className="maint-list">
                  {NO_OPS_LIST.map((item) => (
                    <li key={item} className="maint-list-item--fg">
                      <span className="maint-icon-amber">✓</span>
                      {item}
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          </div>
        </NbSection>

        <NbSection ref={timelineRef} ariaLabel="Weekly ops timeline">
          <NbSectionHeader
            monoLabel="[TIMELINE]"
            headline="Weekly ops timeline."
            sub="From 4 hours to 30 seconds — see exactly what each ops cycle looks like."
          />

          <div className="nb-engine-part">
            <h2 className="maint-timeline-subhead">From 4 hours to 30 seconds.</h2>

            <div className="nb-grid nb-grid--cols-2">
              <div className="nb-cell">
                <div className="nb-mono-label">LEGACY WEEKLY OPS</div>
                {LEGACY_OPS.map((item, i) => (
                  <div key={item.task} className="maint-timeline-row">
                    <div className="maint-timeline-row-content">
                      <span className="maint-icon-steel">✗</span>
                      {item.task}
                    </div>
                    <span className="maint-timeline-time maint-timeline-time--steel">
                      {item.time}
                    </span>
                  </div>
                ))}
                <div className="maint-timeline-total maint-label-steel">~4h / week</div>
              </div>
              <div className="nb-cell maint-cell-border">
                <div className="nb-mono-label">VANTADB WEEKLY OPS</div>
                {VANTA_OPS.map((item, i) => (
                  <div key={item.task} className="maint-timeline-row">
                    <div className="maint-timeline-row-content--fg">
                      <span className="maint-icon-amber">✓</span>
                      {item.task}
                    </div>
                    <span className="maint-timeline-time maint-timeline-time--amber">
                      {item.time}
                    </span>
                  </div>
                ))}
                <div className="maint-timeline-total maint-timeline-total--amber">~30s / week</div>
              </div>
            </div>
          </div>

          <div className="nb-engine-part">
            <div className="nb-grid nb-grid--cols-2">
              <div className="nb-cell maint-cell-cross">
                <span className="nb-mono-label">KEY INSIGHT</span>
                <p className="maint-insight-text">
                  Because VantaDB runs as an embedded library — not a separate server — there's
                  nothing to deploy, monitor, or scale independently. Your application's lifecycle{" "}
                  <em>is</em> the database lifecycle. No pager duty. No 2 AM wakeups.
                </p>
              </div>
            </div>
          </div>
        </NbSection>

        <NbSection className="nb-bg-dot" ariaLabel="Get started">
          <NbBlockAmber as="div">
            <div className="maint-cta-row">
              <div>
                <h2 className="maint-cta-heading">Zero ops. Ship and sleep.</h2>
                <p className="maint-cta-sub">Install VantaDB in one command.</p>
              </div>
              <code className="maint-cta-code">pip install vantadb-py</code>
            </div>
          </NbBlockAmber>
        </NbSection>
      </main>
    </div>
  );
}
