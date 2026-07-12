import { useCallback, useEffect, useRef, useState } from "react";
import { useReducedMotion } from "../hooks/useReducedMotion";
import { NbArrow, NbNoise } from "./nb";

export function NbTerminalHero() {
  const [copied, setCopied] = useState(false);
  const scopeRef = useRef<HTMLDivElement>(null);
  const waveRef = useRef<SVGPathElement>(null);
  const blipRef = useRef<SVGCircleElement>(null);
  const frameRef = useRef(0);
  const reducedMotion = useReducedMotion();

  useEffect(() => {
    if (!waveRef.current || !blipRef.current) return;
    const wave = waveRef.current;
    const blip = blipRef.current;
    let t = 0;
    let cancelled = false;

    const draw = () => {
      if (cancelled) return;
      t += 0.025;
      const w = 360;
      const h = 140;
      const cx = w / 2;
      const cy = h / 2;
      let d = `M 0 ${cy}`;
      let bx = 0;
      let by = cy;
      for (let x = 0; x <= w; x += 2) {
        const y =
          cy +
          Math.sin((x / w) * Math.PI * 6 + t) * 32 +
          Math.sin((x / w) * Math.PI * 2 + t * 0.6) * 16 +
          Math.sin((x / w) * Math.PI * 14 + t * 1.4) * 8 +
          Math.sin((x / w) * Math.PI * 30 + t * 2.1) * 3;
        d += ` L ${x} ${y}`;
        if (x === Math.round(cx + Math.sin(t * 0.7) * 80)) {
          bx = x;
          by = y;
        }
      }
      wave.setAttribute("d", d);
      blip.setAttribute("cx", `${bx}`);
      blip.setAttribute("cy", `${by}`);
      frameRef.current = requestAnimationFrame(draw);
    };

    const onVisibility = () => {
      if (document.hidden) {
        cancelAnimationFrame(frameRef.current);
      } else if (!reducedMotion) {
        draw();
      }
    };
    document.addEventListener("visibilitychange", onVisibility);

    if (!reducedMotion) draw();

    return () => {
      cancelled = true;
      cancelAnimationFrame(frameRef.current);
      document.removeEventListener("visibilitychange", onVisibility);
    };
  }, [reducedMotion]);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText("pip install vantadb-py");
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      console.error("Failed to copy");
    }
  }, []);

  return (
    <section className="nb-osc-hero" aria-label="Hero">
      <NbNoise />

      {/* Background grid */}
      <svg
        className="nb-osc-hero-bg"
        viewBox="0 0 1280 800"
        preserveAspectRatio="none"
        aria-hidden="true"
      >
        <defs>
          <pattern id="osc-dot" width="40" height="40" patternUnits="userSpaceOnUse">
            <circle cx="1" cy="1" r="1" fill="var(--border)" />
          </pattern>
        </defs>
        <rect width="100%" height="100%" fill="url(#osc-dot)" />
      </svg>

      <div className="nb-osc-hero-inner" ref={scopeRef}>
        {/* ── Left column: content ── */}
        <div className="nb-osc-content">
          <div className="nb-osc-pills">
            <span className="nb-osc-pill">OPEN SOURCE</span>
            <span className="nb-osc-pill">RUST-NATIVE</span>
            <span className="nb-osc-pill">IN-PROCESS</span>
            <span className="nb-osc-pill">MIT</span>
          </div>

          <h1 className="nb-osc-title">
            <span className="nb-osc-title-line1">Embedded Memory</span>
            <span className="nb-osc-title-line2">
              Engine<span className="nb-osc-title-muted"> for AI</span>
            </span>
          </h1>

          <p className="nb-osc-desc">
            HNSW vector search, BM25 full-text, and hybrid RRF in a single Rust binary.
            <br />
            <span className="nb-osc-desc-sub">
              [ zero servers &middot; zero ops &middot; sub-millisecond ]
            </span>
          </p>

          <div className="nb-osc-actions">
            <button onClick={handleCopy} className="nb-osc-cta" aria-label="Copy install command">
              <span className="nb-osc-cta-prefix">$</span>
              <span>{copied ? "Copied!" : "pip install vantadb-py"}</span>
              <svg
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
            <NbArrow href="/docs">READ DOCS</NbArrow>
          </div>

          {/* Status line */}
          <div className="nb-osc-status">
            <span className="nb-osc-status-dot" aria-hidden="true" />
            <span className="nb-osc-status-label">ALL SYSTEMS OPERATIONAL</span>
            <span className="nb-osc-status-divider" aria-hidden="true">
              |
            </span>
            <span className="nb-osc-status-ver">v0.1.5</span>
          </div>
        </div>

        {/* ── Right column: oscilloscope ── */}
        <div className="nb-osc-scope" role="img" aria-label="Live query signal oscilloscope">
          <div className="nb-osc-scope-label">CH1: QUERY SIGNAL</div>

          <svg viewBox="0 0 360 160" className="nb-osc-scope-screen" aria-hidden="true">
            {/* Grid */}
            {[0, 40, 80, 120, 160].map((y) => (
              <line
                key={`hg-${y}`}
                x1="0"
                y1={y}
                x2="360"
                y2={y}
                stroke="var(--border-strong)"
                strokeWidth="0.5"
                opacity="0.6"
              />
            ))}
            {[0, 40, 80, 120, 160, 200, 240, 280, 320, 360].map((x) => (
              <line
                key={`vg-${x}`}
                x1={x}
                y1="0"
                x2={x}
                y2="160"
                stroke="var(--border-strong)"
                strokeWidth="0.5"
                opacity="0.6"
              />
            ))}

            {/* Center crosshair */}
            <line
              x1="0"
              y1="80"
              x2="360"
              y2="80"
              stroke="var(--border-visible)"
              strokeWidth="0.5"
              strokeDasharray="4 4"
            />
            <line
              x1="180"
              y1="0"
              x2="180"
              y2="160"
              stroke="var(--border-visible)"
              strokeWidth="0.5"
              strokeDasharray="4 4"
            />

            {/* Signal trace */}
            <path
              ref={waveRef}
              d="M 0 80 L 360 80"
              fill="none"
              stroke="var(--amber)"
              strokeWidth="2"
              strokeLinecap="round"
            />

            {/* Floating blip */}
            <circle ref={blipRef} cx="180" cy="80" r="5" fill="var(--amber)" opacity="0.7" />

            {/* Sweep line */}
            <line
              className="nb-osc-sweep"
              x1="0"
              y1="0"
              x2="0"
              y2="160"
              stroke="var(--amber)"
              strokeWidth="1"
              opacity="0.25"
            >
              <animate attributeName="x" from="0" to="360" dur="2s" repeatCount="indefinite" />
            </line>
          </svg>

          {/* Scope controls */}
          <div className="nb-osc-scope-controls">
            <span className="nb-osc-scope-ctrl">TIME/DIV: 0.3ms</span>
            <span className="nb-osc-scope-ctrl">VOLTS/DIV: 500mV</span>
            <span className="nb-osc-scope-ctrl">COUPLING: DC</span>
          </div>

          {/* Animated data packets */}
          <div className="nb-osc-packets" aria-hidden="true">
            <div className="nb-osc-packet" style={{ animationDelay: "0s" }} />
            <div className="nb-osc-packet" style={{ animationDelay: "0.6s" }} />
            <div className="nb-osc-packet" style={{ animationDelay: "1.2s" }} />
            <div className="nb-osc-packet" style={{ animationDelay: "1.8s" }} />
          </div>

          <div className="nb-osc-scope-legend">
            <span className="nb-osc-legend-dot" style={{ background: "var(--amber)" }} />
            SIGNAL
            <span className="nb-osc-legend-dot" style={{ background: "var(--steel)" }} />
            TRIG
          </div>
        </div>
      </div>
    </section>
  );
}
