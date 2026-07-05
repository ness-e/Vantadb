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
    <section className="nb-section" aria-label="FAQ">
      <div className="nb-inner">
        <span className="nb-mono-label">[QUESTIONS]</span>
        <h2 className="nb-section-headline">Frequently asked.</h2>

        <div className="nb-faq-list">
          {FAQ_ITEMS.map((item, i) => {
            const isOpen = activeIndex === i;
            return (
              <div key={i} className="nb-faq-item">
                <button
                  type="button"
                  className="nb-faq-q"
                  onClick={() => toggle(i)}
                  aria-expanded={isOpen}
                >
                  <span>{item.q}</span>
                  <span className={`nb-faq-toggle ${isOpen ? "nb-faq-toggle--open" : ""}`}>
                    {isOpen ? "−" : "+"}
                  </span>
                </button>
                {isOpen && (
                  <div className="nb-faq-a">
                    <p>{item.a}</p>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
}
