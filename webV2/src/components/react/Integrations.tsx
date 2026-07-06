import React, { useEffect, useRef } from "react";
import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

gsap.registerPlugin(ScrollTrigger);

const integrations = [
  { name: "PostgreSQL", short: "PG", color: "#336791" },
  { name: "Redis", short: "RD", color: "#DC382D" },
  { name: "Kafka", short: "KF", color: "#231F20" },
  { name: "Elasticsearch", short: "ES", color: "#00BFB3" },
  { name: "MongoDB", short: "MG", color: "#47A248" },
  { name: "MySQL", short: "MY", color: "#4479A1" },
  { name: "S3", short: "S3", color: "#FF9900" },
  { name: "BigQuery", short: "BQ", color: "#4285F4" },
];

const Integrations: React.FC = () => {
  const sectionRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const ctx = gsap.context(() => {
      gsap.from(".int-card", {
        scrollTrigger: {
          trigger: sectionRef.current,
          start: "top 75%",
        },
        y: 20,
        opacity: 0,
        duration: 0.4,
        stagger: 0.05,
        ease: "power2.out",
      });
    }, sectionRef);

    return () => ctx.revert();
  }, []);

  return (
    <section id="integrations" ref={sectionRef} className="py-24">
      <div className="mx-auto px-6" style={{ maxWidth: "900px" }}>
        <div className="text-center mb-12">
          <p className="text-xs mb-3" style={{ fontFamily: "var(--font-mono)", color: "var(--amber)", textTransform: "uppercase", letterSpacing: "0.14em" }}>
            [ INTEGRATIONS ]
          </p>
          <h2 className="font-display font-bold" style={{ fontSize: "var(--text-display)" }}>
            Works with <span style={{ color: "var(--amber)" }}>your stack</span>
          </h2>
        </div>

        <div className="grid grid-cols-4 gap-4">
          {integrations.map((int) => (
            <div
              key={int.name}
              className="int-card flex flex-col items-center justify-center p-6 text-center"
              style={{
                border: "1px solid var(--border)",
                background: "var(--surface)",
                transition: "border-color 0.15s var(--ease-brutal)",
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.borderColor = int.color;
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.borderColor = "var(--border)";
              }}
            >
              <div className="w-12 h-12 flex items-center justify-center mb-2 font-bold text-lg" style={{ fontFamily: "var(--font-mono)", color: "var(--amber)" }}>
                {int.short}
              </div>
              <p className="text-xs" style={{ fontFamily: "var(--font-mono)", color: "var(--muted)", textTransform: "uppercase", letterSpacing: "0.1em" }}>
                {int.name}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default Integrations;
