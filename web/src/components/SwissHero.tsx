import { useState, useCallback } from "react";
import { Link } from "@tanstack/react-router";

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
    <section className="swiss-hero" aria-label="Hero">
      <div className="swiss-hero-grid-bg" aria-hidden="true">
        <svg
          viewBox="0 0 1280 800"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
          preserveAspectRatio="none"
        >
          {Array.from({ length: 12 }, (_, i) => (
            <line
              key={`v${i}`}
              x1={(i / 12) * 100 + "%"}
              y1="0"
              x2={(i / 12) * 100 + "%"}
              y2="100%"
              stroke="var(--border)"
              strokeWidth="0.5"
            />
          ))}
          {Array.from({ length: 8 }, (_, i) => (
            <line
              key={`h${i}`}
              x1="0"
              y1={(i / 7) * 100 + "%"}
              x2="100%"
              y2={(i / 7) * 100 + "%"}
              stroke="var(--border)"
              strokeWidth="0.5"
            />
          ))}
        </svg>
      </div>

      <div className="swiss-hero-geo swiss-hero-geo--circle" aria-hidden="true" />
      <div className="swiss-hero-geo swiss-hero-geo--circle2" aria-hidden="true" />
      <div className="swiss-hero-geo swiss-hero-geo--bar" aria-hidden="true" />
      <div className="swiss-hero-geo swiss-hero-geo--bar2" aria-hidden="true" />

      <div className="swiss-hero-inner">
        <div className="swiss-hero-eyebrow">
          <span>RUST-NATIVE</span>
          <span>IN-PROCESS</span>
          <span>ZERO-SERVERS</span>
          <span>APACHE 2.0</span>
        </div>

        <h1 className="swiss-hero-title">
          Embedded Vector
          <br />
          <span className="word-highlight">Database</span> for AI Agents
        </h1>

        <div className="swiss-hero-desc-group">
          <p className="swiss-hero-desc">
            HNSW vector search, BM25 full-text, and hybrid RRF in a single Rust binary.
          </p>
          <p className="swiss-hero-desc-sub">Zero servers. Zero ops. Sub-millisecond.</p>
        </div>

        <div className="swiss-hero-actions">
          <button
            onClick={handleCopy}
            className="swiss-hero-copy-btn"
            aria-label="Copy pip install command"
          >
            <span>{copied ? "Copied!" : "pip install vantadb-py"}</span>
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
          <Link to="/docs" className="swiss-hero-link">
            READ DOCS
            <svg
              width="12"
              height="12"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2.5"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-hidden="true"
            >
              <line x1="5" y1="12" x2="19" y2="12" />
              <polyline points="12 5 19 12 12 19" />
            </svg>
          </Link>
        </div>
      </div>

      <div className="swiss-hero-marquee" aria-hidden="true">
        <div className="swiss-hero-marquee-inner">
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
