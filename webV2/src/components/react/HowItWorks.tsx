import React, { useEffect, useRef } from "react";
import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

gsap.registerPlugin(ScrollTrigger);

const steps = [
  { num: "01", title: "Install", cmd: "curl -fsSL https://vantadb.com/install | sh", desc: "One command. No dependencies. Works on macOS, Linux, and Windows." },
  { num: "02", title: "Connect", cmd: "vantadb connect --engine=all", desc: "Connect your data sources. Vector, relational, and graph — all at once." },
  { num: "03", title: "Query", cmd: "vantadb query \"find similar to X\"", desc: "Query across all engines with a unified API. One query, any data shape." },
];

const HowItWorks: React.FC = () => {
  const sectionRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const ctx = gsap.context(() => {
      gsap.from(".step-card", {
        scrollTrigger: {
          trigger: sectionRef.current,
          start: "top 70%",
        },
        x: -30,
        opacity: 0,
        duration: 0.5,
        stagger: 0.15,
        ease: "power2.out",
      });
    }, sectionRef);

    return () => ctx.revert();
  }, []);

  return (
    <section id="how-it-works" ref={sectionRef} className="py-24">
      <div className="mx-auto px-6" style={{ maxWidth: "900px" }}>
        <div className="text-center mb-16">
          <p className="text-xs mb-3" style={{ fontFamily: "var(--font-mono)", color: "var(--amber)", textTransform: "uppercase", letterSpacing: "0.14em" }}>
            [ HOW IT WORKS ]
          </p>
          <h2 className="font-display font-bold" style={{ fontSize: "var(--text-display)" }}>
            From zero to production<br />
            <span style={{ color: "var(--amber)" }}>in minutes</span>
          </h2>
        </div>

        <div className="space-y-8">
          {steps.map((step, i) => (
            <div key={step.num} className="step-card">
              <div className="flex items-start gap-6">
                <span className="font-display font-bold text-4xl leading-none shrink-0" style={{ color: "var(--amber)" }}>
                  {step.num}
                </span>

                <div className="flex-1 min-w-0">
                  <h3 className="font-display font-semibold mb-3" style={{ fontSize: "var(--text-title)" }}>
                    {step.title}
                  </h3>

                  <div className="p-4 mb-3" style={{ background: "var(--terminal-bg)", border: "1px solid var(--border)" }}>
                    <code className="text-sm" style={{ fontFamily: "var(--font-mono)", color: "var(--amber)" }}>
                      $ {step.cmd}
                    </code>
                  </div>

                  <p className="text-sm" style={{ color: "var(--muted)" }}>
                    {step.desc}
                  </p>
                </div>
              </div>

              {i < steps.length - 1 && (
                <div className="ml-[2.3rem] my-6 w-px h-8" style={{ background: "var(--border)" }} />
              )}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default HowItWorks;
