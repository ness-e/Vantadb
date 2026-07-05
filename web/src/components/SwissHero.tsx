import { useState, useCallback } from "react";
import { Link } from "@tanstack/react-router";
import { HeroBackground } from "./HeroBackground";

export function SwissHero() {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText("pip install vantadb-py");
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      console.error("Failed to copy command");
    }
  }, []);

  return (
    <section className="hero" aria-label="Hero">
      <HeroBackground />
      <div className="hero-scanline" aria-hidden="true" />
      <div className="hero-grid" aria-hidden="true">
        <svg
          viewBox="0 0 1280 800"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
          preserveAspectRatio="none"
        >
          {Array.from({ length: 12 }, (_, i) => (
            <line
              key={`v${i}`}
              x1={`${(i / 12) * 100}%`}
              y1="0"
              x2={`${(i / 12) * 100}%`}
              y2="100%"
              stroke="var(--border)"
              strokeWidth="0.5"
            />
          ))}
          {Array.from({ length: 8 }, (_, i) => (
            <line
              key={`h${i}`}
              x1="0"
              y1={`${(i / 7) * 100}%`}
              x2="100%"
              y2={`${(i / 7) * 100}%`}
              stroke="var(--border)"
              strokeWidth="0.5"
            />
          ))}
        </svg>
      </div>

      <div className="hero-inner">
        <div className="nb-telemetry" aria-hidden="true">
          <span>[ VANTADB MONITOR ]</span>
          <span>STATUS: READY</span>
          <span>CONN: 0</span>
          <span>MODE: EMBEDDED</span>
        </div>

        <div className="hero-pills">
          <span className="nb-pill-status nb-pill-status--amber">RUST-NATIVE</span>
          <span className="nb-pill-status">IN-PROCESS</span>
          <span className="nb-pill-status">ZERO-SERVERS</span>
        </div>

        <h1 className="hero-title">
          <span
            className="hero-title-glitch"
            data-text="Embedded Vector"
          >
            Embedded Vector
          </span>
          <br />
          <span
            className="hero-title-glitch hero-title-glitch--amber"
            data-text="Database"
          >
            Database
          </span>
          <span className="hero-title-muted"> for AI Agents</span>
        </h1>

        <div className="hero-desc-group">
          <p className="hero-desc">
            HNSW vector search, BM25 full-text, and hybrid RRF in a single Rust
            binary.
          </p>
          <p className="hero-desc-sub">
            <span className="hero-bracket">[</span>
            zero servers &#xB7; zero ops &#xB7; sub-millisecond
            <span className="hero-bracket">]</span>
          </p>
        </div>

        <div className="hero-actions">
          <button
            onClick={handleCopy}
            className="hero-cta"
            aria-label="Copy pip install command"
          >
            <span className="hero-cta-prefix">{">"}</span>
            <span className="hero-cta-text">
              {copied ? "Copied!" : "pip install vantadb-py"}
            </span>
            {copied ? (
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
                <polyline points="20 6 9 17 4 12" />
              </svg>
            ) : (
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
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
              </svg>
            )}
          </button>
          <Link to="/docs" className="hero-link">
            READ DOCS
            <span className="hero-link-arrows">&gt;&gt;&gt;</span>
          </Link>
        </div>
      </div>

      <div className="hero-marquee" aria-hidden="true">
        <div className="hero-marquee-inner">
          <span>sub-millisecond hybrid queries</span>
          <span>zero infrastructure</span>
          <span>HNSW + BM25 + RRF</span>
          <span>WAL durability</span>
          <span>Apache 2.0</span>
          <span>Rust native</span>
          <span>embedded in-process</span>
          <span>sub-millisecond hybrid queries</span>
          <span>zero infrastructure</span>
          <span>HNSW + BM25 + RRF</span>
          <span>WAL durability</span>
          <span>Apache 2.0</span>
          <span>Rust native</span>
          <span>embedded in-process</span>
        </div>
      </div>
    </section>
  );
}
