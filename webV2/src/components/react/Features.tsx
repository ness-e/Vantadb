import React, { useEffect, useRef } from "react";
import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

gsap.registerPlugin(ScrollTrigger);

const features = [
  { index: "01", title: "Vector Engine", desc: "Semantic search and embeddings at light speed. Built-in ANN indexing with HNSW." },
  { index: "02", title: "Relational Engine", desc: "Full SQL support with ACID transactions. Join across vector and relational data." },
  { index: "03", title: "Graph Engine", desc: "Native graph traversals. Property graph model with Cypher-compatible queries." },
  { index: "04", title: "Hybrid Queries", desc: "Combine vector, relational, and graph in a single query. One pipeline, any shape." },
  { index: "05", title: "Real-time Streaming", desc: "Built-in pub/sub and change data capture. React to data as it changes." },
  { index: "06", title: "Zero-ops Deploy", desc: "Single binary. No dependencies. Deploy anywhere — laptop to Kubernetes." },
];

const Features: React.FC = () => {
  const sectionRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const ctx = gsap.context(() => {
      gsap.from(".feature-card", {
        scrollTrigger: {
          trigger: sectionRef.current,
          start: "top 75%",
        },
        y: 40,
        opacity: 0,
        duration: 0.5,
        stagger: 0.08,
        ease: "power2.out",
      });
    }, sectionRef);

    return () => ctx.revert();
  }, []);

  return (
    <section id="features" ref={sectionRef} className="py-24">
      <div className="mx-auto px-6" style={{ maxWidth: "1280px" }}>
        <div className="text-center mb-16">
          <p className="text-xs mb-3" style={{ fontFamily: "var(--font-mono)", color: "var(--amber)", textTransform: "uppercase", letterSpacing: "0.14em" }}>
            [ FEATURES ]
          </p>
          <h2 className="font-display font-bold" style={{ fontSize: "var(--text-display)" }}>
            Everything you need<br />
            <span style={{ color: "var(--amber)" }}>to build with data</span>
          </h2>
        </div>

        <div className="grid md:grid-cols-3 gap-4">
          {features.map((f) => (
            <div
              key={f.index}
              className="feature-card p-6"
              style={{
                border: "1px solid var(--border)",
                background: "var(--surface)",
                transition: "border-color 0.15s var(--ease-brutal), transform 0.15s var(--ease-brutal)",
                cursor: "default",
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.borderColor = "var(--amber)";
                e.currentTarget.style.transform = "translateY(-4px)";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.borderColor = "var(--border)";
                e.currentTarget.style.transform = "translateY(0)";
              }}
            >
              <p className="text-xs mb-2" style={{ fontFamily: "var(--font-mono)", color: "var(--amber)", textTransform: "uppercase", letterSpacing: "0.14em" }}>
                &gt; [{f.index}]
              </p>
              <h3 className="font-display font-semibold mb-2" style={{ fontSize: "var(--text-title)" }}>
                {f.title}
              </h3>
              <p className="text-sm" style={{ color: "var(--muted)", lineHeight: "1.65" }}>
                {f.desc}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default Features;
