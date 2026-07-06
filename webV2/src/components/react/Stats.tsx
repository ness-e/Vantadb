import React, { useEffect, useRef } from "react";
import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

gsap.registerPlugin(ScrollTrigger);

const stats = [
  { value: "12k+", label: "GitHub Stars" },
  { value: "99.9%", label: "Uptime" },
  { value: "3x", label: "Faster Queries" },
  { value: "50k+", label: "Downloads" },
];

const Stats: React.FC = () => {
  const sectionRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const ctx = gsap.context(() => {
      gsap.from(".stat-item", {
        scrollTrigger: {
          trigger: sectionRef.current,
          start: "top 80%",
        },
        y: 30,
        opacity: 0,
        duration: 0.5,
        stagger: 0.1,
        ease: "power2.out",
      });
    }, sectionRef);

    return () => ctx.revert();
  }, []);

  return (
    <section ref={sectionRef} style={{ background: "var(--amber)", color: "var(--white)" }}>
      <div className="mx-auto px-6 py-16" style={{ maxWidth: "1280px" }}>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-8">
          {stats.map((stat) => (
            <div key={stat.label} className="stat-item text-center">
              <p className="font-display font-bold leading-none" style={{ fontSize: "var(--text-metric)" }}>
                {stat.value}
              </p>
              <p className="text-sm mt-2 opacity-80" style={{ fontFamily: "var(--font-mono)", textTransform: "uppercase", letterSpacing: "0.14em" }}>
                {stat.label}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default Stats;
