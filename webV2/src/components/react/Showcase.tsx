import React, { useEffect, useRef, useState } from "react";
import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

gsap.registerPlugin(ScrollTrigger);

const screenshots = [
  { id: 1, label: "Query Console", desc: "Execute hybrid queries across all three engines" },
  { id: 2, label: "Dashboard", desc: "Real-time metrics and cluster monitoring" },
  { id: 3, label: "Schema Designer", desc: "Visual data modeling with zero config" },
];

const Showcase: React.FC = () => {
  const sectionRef = useRef<HTMLDivElement>(null);
  const [active, setActive] = useState(0);

  useEffect(() => {
    const ctx = gsap.context(() => {
      gsap.from(".showcase-content", {
        scrollTrigger: {
          trigger: sectionRef.current,
          start: "top 75%",
        },
        y: 30,
        opacity: 0,
        duration: 0.6,
        ease: "power2.out",
      });
    }, sectionRef);

    return () => ctx.revert();
  }, []);

  return (
    <section id="showcase" ref={sectionRef} className="py-24">
      <div className="mx-auto px-6" style={{ maxWidth: "1280px" }}>
        <div className="text-center mb-12">
          <p className="text-xs mb-3" style={{ fontFamily: "var(--font-mono)", color: "var(--amber)", textTransform: "uppercase", letterSpacing: "0.14em" }}>
            [ SHOWCASE ]
          </p>
          <h2 className="font-display font-bold" style={{ fontSize: "var(--text-display)" }}>
            See VantaDB <span style={{ color: "var(--amber)" }}>in action</span>
          </h2>
        </div>

        <div className="showcase-content">
          {/* Terminal window frame */}
          <div style={{ border: "1px solid var(--border-strong)", background: "var(--terminal-bg)", maxWidth: "800px", margin: "0 auto" }}>
            {/* Title bar */}
            <div className="flex items-center gap-2 px-4 py-2.5" style={{ borderBottom: "1px solid var(--border)", background: "var(--surface)" }}>
              <span className="w-3 h-3 rounded-full" style={{ background: "#ff5f56" }} />
              <span className="w-3 h-3 rounded-full" style={{ background: "#ffbd2e" }} />
              <span className="w-3 h-3 rounded-full" style={{ background: "#27c93f" }} />
              <span className="ml-3 text-xs" style={{ fontFamily: "var(--font-mono)", color: "var(--muted)" }}>
                {screenshots[active].label} — vantadb console
              </span>
            </div>

            {/* Screenshot area (placeholder) */}
            <div className="flex items-center justify-center" style={{ height: "400px", background: "var(--background)" }}>
              <div className="text-center">
                <p className="text-4xl mb-3" style={{ color: "var(--amber)" }}>~$</p>
                <p className="text-sm" style={{ fontFamily: "var(--font-mono)", color: "var(--muted)" }}>
                  {screenshots[active].desc}
                </p>
                <p className="text-xs mt-4" style={{ fontFamily: "var(--font-mono)", color: "var(--steel)" }}>
                  Screenshot coming soon
                </p>
              </div>
            </div>
          </div>

          {/* Navigation dots */}
          <div className="flex justify-center gap-2 mt-6">
            {screenshots.map((s, i) => (
              <button
                key={s.id}
                onClick={() => setActive(i)}
                className="w-3 h-3 transition-all duration-150"
                style={{
                  background: i === active ? "var(--amber)" : "var(--border-strong)",
                  border: "none",
                  cursor: "pointer",
                }}
                aria-label={`View ${s.label}`}
              />
            ))}
          </div>
        </div>
      </div>
    </section>
  );
};

export default Showcase;
