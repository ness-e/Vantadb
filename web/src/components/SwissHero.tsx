import { useRef, useState, useCallback } from "react";
import { Link } from "@tanstack/react-router";
import { gsap, useGSAP } from "../lib/gsap";

/* ── Telemetry data ── */
const METRICS = [
  { label: "P99 LATENCY", value: 0.8, suffix: "ms", accent: true, decimals: 1 },
  { label: "RECALL@10", value: 0.998, suffix: "", accent: false, decimals: 3 },
  { label: "EXTERNAL DEPS", value: 0, suffix: "", accent: false, decimals: 0 },
  { label: "BINARY COUNT", value: 1, suffix: "", accent: false, decimals: 0 },
] as const;

export function SwissHero() {
  const containerRef = useRef<HTMLElement>(null);
  const [copied, setCopied] = useState(false);

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      const ease = "cubic-bezier(0.25, 1, 0.5, 1)";

      mm.add("(prefers-reduced-motion: no-preference)", () => {
        // 1. Draw SVG grid hairlines
        gsap.to(".swiss-hero-grid-line", {
          strokeDashoffset: 0,
          duration: 1.2,
          stagger: 0.04,
          ease: "power2.inOut",
        });

        // 2. Divider line scales in
        gsap.to(".swiss-hero-divider", {
          scaleY: 1,
          duration: 0.8,
          delay: 0.2,
          ease,
        });

        // 3. Tactical labels flash orange → foreground
        gsap.fromTo(
          ".swiss-hero-label span",
          { opacity: 0, color: "var(--amber)" },
          {
            opacity: 1,
            color: "var(--foreground)",
            duration: 0.3,
            stagger: 0.08,
            ease: "power1.in",
            delay: 0.4,
          }
        );

        // 4. Title reveal via clip-path mask
        gsap.fromTo(
          ".swiss-hero-title-line",
          { y: "110%", clipPath: "inset(0% 0% 100% 0%)" },
          {
            y: "0%",
            clipPath: "inset(0% 0% 0% 0%)",
            duration: 0.8,
            stagger: 0.1,
            ease: "power3.out",
            delay: 0.6,
          }
        );

        // 5. Tagline fade in
        gsap.fromTo(
          ".swiss-hero-tagline",
          { opacity: 0, y: 15 },
          { opacity: 1, y: 0, duration: 0.6, ease: "power2.out", delay: 0.9 }
        );

        // 6. Description fade in
        gsap.fromTo(
          ".swiss-hero-description",
          { opacity: 0, y: 15 },
          { opacity: 1, y: 0, duration: 0.5, ease: "power2.out", delay: 1.1 }
        );

        // 7. CTAs fade in
        gsap.fromTo(
          ".swiss-hero-actions",
          { opacity: 0, y: 15 },
          { opacity: 1, y: 0, duration: 0.5, ease: "power2.out", delay: 1.3 }
        );

        // 8. Telemetry labels appear
        gsap.fromTo(
          ".swiss-telemetry-label",
          { opacity: 0 },
          {
            opacity: 1,
            duration: 0.15,
            stagger: 0.06,
            ease: "power1.in",
            delay: 0.9,
          }
        );

        // 9. Telemetry numeric count-up
        const valueEls = document.querySelectorAll<HTMLElement>(".swiss-telemetry-value");
        valueEls.forEach((el) => {
          const target = parseFloat(el.dataset.target || "0");
          const decimals = parseInt(el.dataset.decimals || "0", 10);
          const counter = { val: 0 };
          gsap.to(counter, {
            val: target,
            duration: 0.4,
            delay: 1.0,
            ease: "power2.out",
            onUpdate() {
              el.textContent = counter.val.toFixed(decimals);
            },
          });
        });

        // 10. Telemetry unit labels
        gsap.fromTo(
          ".swiss-telemetry-unit",
          { opacity: 0 },
          {
            opacity: 1,
            duration: 0.2,
            stagger: 0.06,
            ease: "power1.in",
            delay: 1.2,
          }
        );
      });
    },
    { scope: containerRef }
  );

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText("pip install vantadb-py");
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy command: ", err);
    }
  }, []);

  return (
    <section className="swiss-hero-section" ref={containerRef}>
      {/* Grid SVG Hairlines */}
      <svg
        className="swiss-hero-grid-svg"
        viewBox="0 0 1200 800"
        preserveAspectRatio="none"
        aria-hidden="true"
      >
        {Array.from({ length: 13 }).map((_, i) => {
          const x = (i / 12) * 1200;
          return (
            <line
              key={i}
              x1={x}
              y1="0"
              x2={x}
              y2="800"
              className="swiss-hero-grid-line"
            />
          );
        })}
      </svg>

      <div className="swiss-hero-layout">
        {/* ── Left Zone: Content ── */}
        <div className="swiss-hero-content">
          <div className="swiss-hero-label">
            <span>[RUST-NATIVE]</span>
            <span>[IN-PROCESS]</span>
            <span>[ZERO-SERVERS]</span>
          </div>

          <h1 className="swiss-hero-title">
            <span className="swiss-hero-title-wrapper">
              <span className="swiss-hero-title-line">VantaDB</span>
            </span>
          </h1>

          <h2 className="swiss-hero-tagline">
            The database that thinks with you.
          </h2>

          <p className="swiss-hero-description">
            One pip install. Vector search, SQL, and full-text search in a single
            binary. Zero servers. Zero ops. Sub-millisecond.
          </p>

          <div className="swiss-hero-actions">
            <button
              onClick={handleCopy}
              className="swiss-button-primary"
              aria-label="Copy pip install command"
            >
              <span>{copied ? "Copied!" : "pip install vantadb-py"}</span>
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2.5"
                strokeLinecap="round"
                strokeLinejoin="round"
                aria-hidden="true"
              >
                {copied ? (
                  <polyline points="20 6 9 17 4 12" />
                ) : (
                  <>
                    <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                    <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                  </>
                )}
              </svg>
            </button>
            <Link to="/docs" className="swiss-button-ghost">
              Read Docs
            </Link>
          </div>
        </div>

        {/* ── Right Zone: OLED Telemetry Panel ── */}
        <div className="swiss-hero-telemetry">
          <div className="swiss-hero-divider" />

          <div className="swiss-telemetry-stack">
            {METRICS.map((m) => (
              <div key={m.label} className="swiss-telemetry-cell">
                <div className="swiss-telemetry-label">{m.label}</div>
                <div
                  className={`swiss-telemetry-value${m.accent ? " swiss-telemetry-value--accent" : ""}`}
                  data-target={m.value}
                  data-decimals={m.decimals}
                >
                  0
                </div>
                {m.suffix && (
                  <div className="swiss-telemetry-unit">{m.suffix}</div>
                )}
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}
