import { useState, useRef } from "react";
import { createLazyRoute } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
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

function PlaygroundPage() {
  const [input, setInput] = useState("");
  const [results, setResults] = useState<{ id: string; score: number; text: string }[] | null>(
    null,
  );
  const [simulating, setSimulating] = useState(false);

  const terminalRef = useRef<HTMLElement>(null);
  const statsRef = useRef<HTMLElement>(null);
  const codeRef = useRef<HTMLElement>(null);
  const installRef = useRef<HTMLElement>(null);

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

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({
      scrollTrigger: scrollTriggerConfig(terminalRef.current, 60),
    });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, terminalRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({
      scrollTrigger: scrollTriggerConfig(statsRef.current, 60),
    });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, statsRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({
      scrollTrigger: scrollTriggerConfig(codeRef.current, 60),
    });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, codeRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({
      scrollTrigger: scrollTriggerConfig(installRef.current, 60),
    });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, installRef);

  return (
    <div>
      <NbSubpageHero
        pattern="p01"
        title="Run VantaDB in your browser."
        sub="Full vector search engine compiled to WASM. No install, no server, no excuses."
      />

      <main>
        <NbSection ref={terminalRef} className="nb-bg-cross--faint" ariaLabel="Playground terminal">
          <NbSectionHeader
            monoLabel="[PLAYGROUND]"
            headline="Run VantaDB in your browser."
            sub="Full vector search engine compiled to WASM. No install, no server, no excuses."
          />

          <div className="nb-engine-part">
            <div className="playground-terminal">
              <div className="playground-terminal-header">
                <span className="playground-dot-red" />
                <span className="playground-dot-yellow" />
                <span className="playground-dot-green" />
                <span className="playground-terminal-label">vantadb@playground</span>
              </div>

              <div className="playground-terminal-content">
                <div className="playground-prompt-row">
                  <span className="playground-prompt-label">vantadb@playground:~$</span>
                  <input
                    value={input}
                    onChange={(e) => setInput(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && runQuery()}
                    placeholder="Type a query or select a sample..."
                    className="playground-prompt-input"
                  />
                  <button onClick={runQuery} disabled={simulating} className="playground-run-btn">
                    Run
                  </button>
                </div>

                <div className="playground-sample-row">
                  {SAMPLE_QUERIES.map((q) => (
                    <button key={q} onClick={() => fillQuery(q)} className="playground-sample-btn">
                      {q}
                    </button>
                  ))}
                </div>

                <div className="playground-results-area">
                  {simulating && <span className="playground-simulating">simulating query...</span>}

                  {results && !simulating && (
                    <div>
                      <span className="playground-results-summary">
                        {results.length} results ({results[0]?.score.toFixed(3)} max score)
                      </span>
                      {results.map((r) => (
                        <div key={r.id} className="playground-result-row">
                          <span className="playground-result-id">{r.id}</span>
                          <div className="playground-score-bar">
                            <div
                              className="playground-score-bar-fill"
                              style={{ width: `${r.score * 100}%` }}
                            />
                          </div>
                          <span className="playground-result-text">{r.text}</span>
                          <span className="playground-result-score">{r.score.toFixed(2)}</span>
                        </div>
                      ))}
                    </div>
                  )}

                  {!results && !simulating && (
                    <span className="playground-empty">
                      ready. select a sample query or type your own.
                    </span>
                  )}
                </div>
              </div>
            </div>
          </div>
        </NbSection>

        <NbSection ref={statsRef} ariaLabel="Engine stats">
          <NbSectionHeader
            monoLabel="[ENGINE]"
            headline="Engine configuration."
            sub="Current WASM runtime environment."
          />

          <div className="nb-grid nb-grid--cols-4 nb-engine-part">
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
        </NbSection>

        <NbSection ref={codeRef} className="nb-bg-cross--faint" ariaLabel="Quick start code">
          <NbSectionHeader
            monoLabel="[QUICK START]"
            headline="Get started in seconds."
            sub="Embed VantaDB in any JS runtime."
          />

          <div className="nb-engine-part">
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
              <span className="playground-syntax-default">db.insert({"{"})</span>
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
        </NbSection>

        <NbSection ref={installRef} ariaLabel="Install command">
          <NbSectionHeader
            monoLabel="[INSTALL]"
            headline="One command."
            sub="Add VantaDB to your project."
          />

          <div className="nb-engine-part">
            <div className="playground-install-row">
              <div className="playground-install-command">
                <span className="playground-syntax-amber">$</span>
                npm install @vantadb/wasm
              </div>
            </div>
          </div>
        </NbSection>
      </main>
    </div>
  );
}
