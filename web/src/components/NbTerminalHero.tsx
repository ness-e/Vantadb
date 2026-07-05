import { useState, useCallback, useEffect, useRef } from "react";
import { Link } from "@tanstack/react-router";
import { NbCursor } from "./nb/NbCursor";
import { NbLogLine } from "./nb/NbLogLine";
import { NbNoise } from "./nb/NbNoise";

const BOOT_LOG = [
  { level: "info" as const, msg: "VantaDB Engine v0.1.5 (commit 4a2f1b9)" },
  { level: "ok" as const, msg: "Rust core initialized in 3.2ms" },
  { level: "info" as const, msg: "PyO3 bridge ready — zero-copy FFI active" },
  { level: "ok" as const, msg: "HNSW index: 128d, ef_construction=200" },
  { level: "ok" as const, msg: "BM25 engine: tokenizer=unicode, k1=1.2, b=0.75" },
  { level: "ok" as const, msg: "WAL: write-ahead logging enabled, fsync=default" },
  { level: "info" as const, msg: "Memory-mapped storage: ./vantadb.mem" },
  { level: "ok" as const, msg: "Hybrid search (RRF) ready" },
  { level: "warn" as const, msg: "No external servers detected — running in embedded mode" },
  { level: "ok" as const, msg: "System ready. Waiting for queries..." },
];

const DEMO_COMMANDS = [
  '>>> db.search("quantum computing", top_k=3)',
  '  1. "Quantum computing basics" — score: 0.94',
  '  2. "Introduction to qubits" — score: 0.87',
  '  3. "Quantum algorithms" — score: 0.81',
  "  Time: 0.47ms (zero network)",
];

export function NbTerminalHero() {
  const [visibleLogs, setVisibleLogs] = useState(1);
  const [showPrompt, setShowPrompt] = useState(false);
  const [cmdIndex, setCmdIndex] = useState(0);
  const [copied, setCopied] = useState(false);
  const logRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (visibleLogs < BOOT_LOG.length) {
      const t = setTimeout(() => setVisibleLogs((p) => p + 1), 80 + Math.random() * 120);
      return () => clearTimeout(t);
    }
    const t = setTimeout(() => setShowPrompt(true), 400);
    return () => clearTimeout(t);
  }, [visibleLogs]);

  useEffect(() => {
    if (!showPrompt || cmdIndex >= DEMO_COMMANDS.length) return;
    const t = setTimeout(() => setCmdIndex((p) => p + 1), 600 + Math.random() * 400);
    return () => clearTimeout(t);
  }, [showPrompt, cmdIndex]);

  useEffect(() => {
    if (logRef.current) {
      logRef.current.scrollTop = logRef.current.scrollHeight;
    }
  }, [visibleLogs, cmdIndex]);

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
    <section className="nb-terminal-hero" aria-label="Hero">
      <NbNoise />
      <div className="nb-terminal-hero-grid" aria-hidden="true">
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

      <div className="nb-terminal-hero-inner">
        <div className="nb-terminal-hero-pills">
          <span className="nb-terminal-hero-pill">OPEN SOURCE</span>
          <span className="nb-terminal-hero-pill">RUST-NATIVE</span>
          <span className="nb-terminal-hero-pill">IN-PROCESS</span>
          <span className="nb-terminal-hero-pill">APACHE 2.0</span>
        </div>

        <h1 className="nb-terminal-hero-title">
          <span className="nb-terminal-hero-glitch" data-text="Embedded Memory">
            Embedded Memory
          </span>
          <br />
          <span
            className="nb-terminal-hero-glitch nb-terminal-hero-glitch--amber"
            data-text="Engine"
          >
            Engine
          </span>
          <span className="nb-terminal-hero-title-muted"> for AI</span>
        </h1>

        <p className="nb-terminal-hero-desc">
          HNSW vector search, BM25 full-text, and hybrid RRF in a single Rust binary.
          <br />
          <span className="nb-terminal-hero-desc-sub">
            [ zero servers &middot; zero ops &middot; sub-millisecond ]
          </span>
        </p>

        <div className="nb-terminal-hero-actions">
          <button
            onClick={handleCopy}
            className="nb-terminal-hero-cta"
            aria-label="Copy install command"
          >
            <span className="nb-terminal-hero-cta-prefix">{">"}</span>
            <span className="nb-terminal-hero-cta-text">
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
              >
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
              </svg>
            )}
          </button>
          <Link to="/docs" className="nb-arrow">
            READ DOCS
          </Link>
        </div>
      </div>

      <div className="nb-terminal-window" role="region" aria-label="Terminal boot sequence">
        <div className="nb-terminal-window-bar">
          <span className="nb-terminal-window-dot" />
          <span className="nb-terminal-window-dot" />
          <span className="nb-terminal-window-dot" />
          <span className="nb-terminal-window-label">vantadb@engine:~/boot</span>
        </div>
        <div ref={logRef} className="nb-terminal-window-body">
          {BOOT_LOG.slice(0, visibleLogs).map((line, i) => (
            <NbLogLine key={i} level={line.level} message={line.msg} />
          ))}
          {showPrompt && (
            <div className="nb-terminal-window-prompt">
              <span className="nb-terminal-window-prompt-symbol">&gt;&gt;&gt;</span>
            </div>
          )}
          {DEMO_COMMANDS.slice(0, cmdIndex).map((line, i) => (
            <div key={`cmd-${i}`} className="nb-terminal-window-output">
              {line.startsWith("  ") ? (
                <span className="nb-terminal-window-result">{line}</span>
              ) : (
                <span className="nb-terminal-window-cmd">{line}</span>
              )}
            </div>
          ))}
          {cmdIndex < DEMO_COMMANDS.length && showPrompt && (
            <span className="nb-terminal-window-prompt-ready">
              &gt;&gt;&gt; <NbCursor />
            </span>
          )}
          {cmdIndex >= DEMO_COMMANDS.length && (
            <div className="nb-terminal-window-prompt">
              <span className="nb-terminal-window-prompt-symbol">&gt;&gt;&gt;</span>
              <NbCursor />
            </div>
          )}
        </div>
      </div>
    </section>
  );
}
