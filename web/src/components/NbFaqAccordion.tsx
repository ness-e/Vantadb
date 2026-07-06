import { useState, useCallback } from "react";
import { NbSection, NbSectionHeader } from "../components/nb";
import "../styles/faq-accordion.css";

const FAQ_ITEMS = [
  {
    q: "When should I use VantaDB vs Chroma?",
    a: "VantaDB is best when you need embedded vector search without infrastructure. Unlike Chroma, VantaDB runs in-process, zero servers, sub-millisecond hybrid search. Chroma is great for prototyping; VantaDB for production.",
    severity: "INFO",
  },
  {
    q: "Can I use VantaDB in production?",
    a: "Yes. VantaDB is used in production by teams building AI agents, local RAG, and edge applications. WAL durability, configurable memory limits, zero external dependencies.",
    severity: "INFO",
  },
  {
    q: "How does VantaDB compare to SQLite + vec0?",
    a: "Both embedded databases. VantaDB adds native HNSW vector indexing, BM25 FTS, hybrid query fusion, and PyO3 bindings — all in a 2MB binary. SQLite+vec0 needs extension wrangling and lacks hybrid search.",
    severity: "COMPARE",
  },
  {
    q: "Do I need a server?",
    a: "No. VantaDB embeds like SQLite — no daemons, no containers, no separate processes. One binary, one file, zero servers.",
    severity: "CRITICAL",
  },
];

const SEV_COLORS: Record<string, string> = {
  INFO: "var(--steel)",
  COMPARE: "var(--amber)",
  CRITICAL: "var(--success)",
};

export function NbFaqAccordion() {
  const [open, setOpen] = useState<number | null>(null);
  const toggle = useCallback((i: number) => setOpen((p) => (p === i ? null : i)), []);

  return (
    <NbSection ariaLabel="FAQ">
      <NbSectionHeader
        monoLabel="[DEBUG LOG]"
        headline="Frequently asked questions."
        sub="Expand each entry for the full stack trace."
      />

      <div className="faq-console">
        {/* Console header */}
        <div className="faq-console-bar">
          <span className="faq-console-bar-dot" />
          <span className="faq-console-bar-dot" />
          <span className="faq-console-bar-dot" />
          <span className="faq-console-bar-label">vantadb@faq:~/debug</span>
        </div>

        <div className="faq-console-body">
          {FAQ_ITEMS.map((item, i) => {
            const isOpen = open === i;
            const sevColor = SEV_COLORS[item.severity];

            return (
              <div key={i} className={`faq-entry ${isOpen ? "faq-entry--open" : ""}`}>
                <button
                  type="button"
                  className="faq-entry-btn"
                  onClick={() => toggle(i)}
                  aria-expanded={isOpen}
                >
                  {/* Timestamp */}
                  <span className="faq-entry-ts">
                    {new Date().toLocaleTimeString("en-US", { hour12: false, hour: "2-digit", minute: "2-digit", second: "2-digit" })}
                  </span>

                  {/* Severity badge */}
                  <span className="faq-entry-sev" style={{ color: sevColor }}>
                    [{item.severity}]
                  </span>

                  {/* Question */}
                  <span className="faq-entry-q">{item.q}</span>

                  {/* Toggle icon */}
                  <span className={`faq-entry-icon ${isOpen ? "faq-entry-icon--open" : ""}`} aria-hidden="true">
                    <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                      <path d="M6 1v10M1 6h10" stroke={isOpen ? "var(--amber)" : "var(--steel)"} strokeWidth="2" strokeLinecap="square" />
                    </svg>
                  </span>
                </button>

                {/* Answer as stack trace */}
                <div className="faq-entry-reveal" role="region">
                  <div className="faq-entry-trace">
                    <span className="faq-entry-trace-prefix">#</span>
                    <p className="faq-entry-a">{item.a}</p>
                  </div>
                  <div className="faq-entry-trace-end">
                    {isOpen && <span className="faq-entry-exit">[exit 0]</span>}
                  </div>
                </div>
              </div>
            );
          })}
        </div>

        <div className="faq-console-footer">
          <span className="faq-console-footer-prompt">&gt;&gt;&gt;</span>
          <span className="faq-console-footer-text">Need more help? </span>
          <a href="/docs" className="faq-console-footer-link">Read the docs →</a>
        </div>
      </div>
    </NbSection>
  );
}
