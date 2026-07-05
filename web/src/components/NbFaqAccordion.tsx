import { useState } from "react";
import "../styles/faq-accordion.css";

const FAQ_ITEMS = [
  {
    q: "When should I use VantaDB vs Chroma?",
    a: "VantaDB is best when you need embedded vector search without infrastructure. Unlike Chroma, VantaDB runs in-process, requires zero servers, and delivers sub-millisecond hybrid search. Chrome is great for prototyping; VantaDB is built for production embedding.",
  },
  {
    q: "Can I use VantaDB in production?",
    a: "Yes. VantaDB is used in production by teams building AI agents, local RAG systems, and edge applications. It features a Write-Ahead Log for durability, configurable memory limits, and zero external dependencies.",
  },
  {
    q: "How does VantaDB compare to SQLite + vec0?",
    a: "Both are embedded databases, but VantaDB adds native HNSW vector indexing, BM25 full-text search, hybrid query fusion, and PyO3 bindings — all in a single 2MB binary. SQLite + vec0 requires extension wrangling and lacks hybrid search.",
  },
  {
    q: "Do I need a server?",
    a: "No. VantaDB is an embedded database. You embed it like SQLite — no daemons, no containers, no separate processes, no cloud infrastructure. One binary, one file, zero servers.",
  },
];

export function NbFaqAccordion() {
  const [activeIndex, setActiveIndex] = useState<number | null>(null);

  const toggle = (index: number) => {
    setActiveIndex((prev) => (prev === index ? null : index));
  };

  return (
    <section className="nb-section" aria-label="Frequently Asked Questions">
      <div className="nb-inner">
        <h2
          style={{
            fontFamily: "var(--font-display)",
            fontSize: "1.25rem",
            fontWeight: 700,
            color: "var(--amber)",
          }}
        >
          FAQ
        </h2>
        <div className="nb-divider" />

        <div className="faq-accordion" style={{ marginTop: "var(--space-xl)" }}>
          {FAQ_ITEMS.map((item, index) => {
            const isOpen = activeIndex === index;
            return (
              <div key={index} className="faq-item">
                <button
                  type="button"
                  className="faq-question"
                  onClick={() => toggle(index)}
                  aria-expanded={isOpen}
                >
                  <span>{item.q}</span>
                  <span
                    className="faq-chevron"
                    aria-hidden="true"
                    style={{ transform: isOpen ? "rotate(180deg)" : "rotate(0deg)" }}
                  >
                    ▾
                  </span>
                </button>
                <div
                  className="faq-answer-wrapper"
                  style={{
                    maxHeight: isOpen ? "var(--faq-answer-h, 300px)" : "0",
                  }}
                >
                  <div className="faq-answer">{item.a}</div>
                </div>
                <div className="nb-divider" />
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
}
