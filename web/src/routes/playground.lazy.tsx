import { useState } from "react";
import { createLazyRoute } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";
import "../styles/playground.css";

export const Route = createLazyRoute("/playground")({
  component: PlaygroundPage,
  pendingComponent: PendingComponent,
});

const SAMPLE_QUERIES = [
  "search similar to 'quantum computing'",
  "hybrid search 'machine learning papers 2024'",
  "knn 5 nearest 'transformer architecture'",
];

const FAKE_RESULTS: Record<string, { id: string; score: number; text: string }[]> = {
  "search similar to 'quantum computing'": [
    {
      id: "doc_042",
      score: 0.94,
      text: "Quantum circuit optimization using variational algorithms",
    },
    { id: "doc_117", score: 0.89, text: "Topological qubits: a survey of error-correcting codes" },
    { id: "doc_203", score: 0.81, text: "Quantum machine learning: beyond the HHL algorithm" },
    {
      id: "doc_056",
      score: 0.76,
      text: "Entanglement distillation for distributed quantum computing",
    },
    {
      id: "doc_331",
      score: 0.72,
      text: "Noisy intermediate-scale quantum (NISQ) device benchmarking",
    },
  ],
  "hybrid search 'machine learning papers 2024'": [
    { id: "doc_189", score: 0.96, text: "LoRA: Low-Rank Adaptation of Large Language Models" },
    {
      id: "doc_044",
      score: 0.91,
      text: "Scalable diffusion models for high-resolution image synthesis",
    },
    { id: "doc_278", score: 0.87, text: "Efficient RLHF with preference model distillation" },
    { id: "doc_102", score: 0.79, text: "Multi-modal embeddings for cross-modal retrieval" },
    { id: "doc_410", score: 0.73, text: "Self-supervised learning in computer vision: a review" },
  ],
  "knn 5 nearest 'transformer architecture'": [
    { id: "doc_001", score: 0.98, text: "Attention Is All You Need — original Transformer paper" },
    { id: "doc_022", score: 0.93, text: "BERT: Pre-training of Deep Bidirectional Transformers" },
    { id: "doc_067", score: 0.88, text: "GPT-3: Scaling Language Models with Few-Shot Learning" },
    {
      id: "doc_155",
      score: 0.82,
      text: "TransformerXL: Attentive Language Models Beyond a Fixed-Length Context",
    },
    {
      id: "doc_290",
      score: 0.77,
      text: "Efficient Transformers: A Survey of Efficient Attention Mechanisms",
    },
  ],
};

const CMD_LINE_STYLE: React.CSSProperties = {
  fontFamily: "var(--font-mono)",
  fontSize: "var(--text-code)",
  lineHeight: 1.6,
};

function PlaygroundPage() {
  const [input, setInput] = useState("");
  const [results, setResults] = useState<{ id: string; score: number; text: string }[] | null>(
    null,
  );
  const [simulating, setSimulating] = useState(false);

  function fillQuery(q: string) {
    setInput(q);
  }

  function runQuery() {
    if (!input.trim()) return;
    setSimulating(true);
    setResults(null);
    setTimeout(() => {
      const hit =
        FAKE_RESULTS[input.trim()] ??
        FAKE_RESULTS[Object.keys(FAKE_RESULTS)[0] as keyof typeof FAKE_RESULTS];
      setResults(hit ?? []);
      setSimulating(false);
    }, 800);
  }

  return (
    <div className="nb-page">
      <NbSubpageHero
        num="02"
        title="Run VantaDB in your browser."
        sub="Full vector search engine compiled to WASM. No install, no server, no excuses."
      />

      <section className="nb-section nb-bg-cross--faint">
        <div className="nb-inner">
          <div className="playground-terminal">
            <div className="playground-terminal-header">
              <span className="playground-dot-red" />
              <span className="playground-dot-yellow" />
              <span className="playground-dot-green" />
              <span
                style={{
                  ...CMD_LINE_STYLE,
                  color: "var(--steel)",
                  marginLeft: "auto",
                  fontSize: "0.6rem",
                }}
              >
                vantadb@playground
              </span>
            </div>

            <div className="playground-terminal-content">
              <div className="playground-prompt-row">
                <span style={{ ...CMD_LINE_STYLE, color: "var(--amber)" }}>
                  vantadb@playground:~$
                </span>
                <input
                  value={input}
                  onChange={(e) => setInput(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && runQuery()}
                  placeholder="Type a query or select a sample..."
                  style={{
                    ...CMD_LINE_STYLE,
                    flex: 1,
                    background: "transparent",
                    border: "none",
                    outline: "none",
                    color: "#e0e0e0",
                    caretColor: "var(--amber)",
                  }}
                />
                <button
                  onClick={runQuery}
                  disabled={simulating}
                  style={{
                    ...CMD_LINE_STYLE,
                    background: "var(--amber)",
                    color: "var(--text-on-amber)",
                    border: "none",
                    padding: "4px 12px",
                    cursor: "pointer",
                    fontWeight: 700,
                    letterSpacing: "0.1em",
                    textTransform: "uppercase",
                    fontSize: "0.65rem",
                    opacity: simulating ? 0.5 : 1,
                  }}
                >
                  Run
                </button>
              </div>

              <div className="playground-sample-row">
                {SAMPLE_QUERIES.map((q) => (
                  <button
                    key={q}
                    onClick={() => fillQuery(q)}
                    style={{
                      ...CMD_LINE_STYLE,
                      background: "transparent",
                      border: "1px solid var(--border-visible)",
                      padding: "4px 10px",
                      color: "var(--steel)",
                      cursor: "pointer",
                      fontSize: "0.65rem",
                    }}
                  >
                    {q}
                  </button>
                ))}
              </div>

              <div className="playground-results-area">
                {simulating && (
                  <span style={{ ...CMD_LINE_STYLE, color: "var(--amber)" }}>
                    simulating query...
                  </span>
                )}

                {results && !simulating && (
                  <div>
                    <span
                      style={{
                        ...CMD_LINE_STYLE,
                        color: "var(--steel)",
                        display: "block",
                        marginBottom: "var(--space-sm)",
                        fontVariantNumeric: "tabular-nums",
                      }}
                    >
                      {results.length} results ({results[0]?.score.toFixed(3)} max score)
                    </span>
                    {results.map((r) => (
                      <div key={r.id} className="playground-result-row">
                        <span
                          style={{
                            ...CMD_LINE_STYLE,
                            color: "var(--amber)",
                            width: "10ch",
                            flexShrink: 0,
                            fontSize: "0.6rem",
                            letterSpacing: "0.04em",
                          }}
                        >
                          {r.id}
                        </span>
                        <div className="playground-score-bar">
                          <div
                            style={{
                              width: `${r.score * 100}%`,
                              height: "100%",
                              background: "var(--amber)",
                            }}
                          />
                        </div>
                        <span
                          style={{
                            ...CMD_LINE_STYLE,
                            color: "#e0e0e0",
                            flex: 1,
                            whiteSpace: "nowrap",
                            overflow: "hidden",
                            textOverflow: "ellipsis",
                            fontSize: "0.65rem",
                          }}
                        >
                          {r.text}
                        </span>
                        <span
                          style={{
                            ...CMD_LINE_STYLE,
                            color: "var(--steel)",
                            fontSize: "0.6rem",
                            width: 30,
                            textAlign: "right",
                            fontVariantNumeric: "tabular-nums",
                          }}
                        >
                          {r.score.toFixed(2)}
                        </span>
                      </div>
                    ))}
                  </div>
                )}

                {!results && !simulating && (
                  <span style={{ ...CMD_LINE_STYLE, color: "var(--steel)" }}>
                    ready. select a sample query or type your own.
                  </span>
                )}
              </div>
            </div>
          </div>
        </div>
      </section>

      <section className="nb-section">
        <div className="nb-inner">
          <div className="nb-grid nb-grid--cols-4">
            {[
              { label: "Engine", value: "VantaDB WASM" },
              { label: "Status", value: "Ready (simulated)" },
              { label: "Mode", value: "HNSW + BM25" },
              { label: "Dims", value: "1536d" },
            ].map((s) => (
              <div key={s.label} className="playground-stat-card">
                <span className="playground-stat-value">{s.value}</span>
                <span className="playground-stat-label">{s.label}</span>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section nb-bg-cross--faint">
        <div className="nb-inner">
          <h2 className="playground-heading">Quick Start</h2>
          <div className="nb-divider" />
          <div className="playground-code-block">
            <span className="playground-syntax-steel">
              // Embed VantaDB in any JS runtime{"\n"}
            </span>
            <span className="playground-syntax-amber">import</span>{" "}
            <span className="playground-syntax-default">{`{ VantaDB }`}</span>{" "}
            <span className="playground-syntax-amber">from</span>{" "}
            <span className="playground-syntax-steel">"@vantadb/wasm"</span>
            <span className="playground-syntax-default">;</span>
            {"\n\n"}
            <span className="playground-syntax-amber">const</span>{" "}
            <span className="playground-syntax-default">db = </span>
            <span className="playground-syntax-amber">await</span>{" "}
            <span className="playground-syntax-default">VantaDB.init();</span>
            {"\n\n"}
            <span className="playground-syntax-amber">await</span>{" "}
            <span className="playground-syntax-default">db.insert({"{"}</span>
            {"\n"}
            <span className="playground-syntax-default"> id: </span>
            <span className="playground-syntax-steel">"doc1"</span>
            <span className="playground-syntax-default">,</span>
            {"\n"}
            <span className="playground-syntax-default"> vector: [...],</span>
            {"\n"}
            <span className="playground-syntax-default"> text: </span>
            <span className="playground-syntax-steel">"..."</span>
            <span className="playground-syntax-default">,</span>
            {"\n"}
            <span className="playground-syntax-default">{"}"});</span>
            {"\n\n"}
            <span className="playground-syntax-amber">const</span>{" "}
            <span className="playground-syntax-default">results = </span>
            <span className="playground-syntax-amber">await</span>{" "}
            <span className="playground-syntax-default">db.hybridSearch(</span>
            <span className="playground-syntax-steel">"query"</span>
            <span className="playground-syntax-default">, {"{ topK: 10 }"});</span>
          </div>
        </div>
      </section>

      <section className="nb-section">
        <div className="nb-inner">
          <div className="playground-install-row">
            <div className="playground-install-command">
              <span className="playground-syntax-amber">$</span>
              npm install @vantadb/wasm
            </div>
          </div>
        </div>
      </section>
    </div>
  );
}
