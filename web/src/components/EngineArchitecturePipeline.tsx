const PIPELINE_STAGES = [
  { name: "Query", desc: "Tokenizer", accent: "amber" },
  { name: "BM25", desc: "Lexical Score", accent: "steel" },
  { name: "HNSW", desc: "Vector Recall", accent: "steel" },
  { name: "RRF", desc: "Fused Ranker", accent: "amber" },
  { name: "Edges", desc: "Local Graph", accent: "steel" },
  { name: "WAL", desc: "Durable Write", accent: "steel" },
] as const;

export function EngineArchitecturePipeline() {
  return (
    <div className="nc-engine-conveyor">
      <div className="nc-engine-conveyor-track">
        {PIPELINE_STAGES.map((s, i) => (
          <div key={s.name} className="nc-engine-conveyor-stage">
            <div className="nc-engine-conveyor-card" data-accent={s.accent}>
              <div className="nc-engine-conveyor-card-name">{s.name}</div>
              <div className="nc-engine-conveyor-card-desc">{s.desc}</div>
            </div>
            {i < PIPELINE_STAGES.length - 1 && (
              <span className="nc-engine-conveyor-arrow" aria-hidden="true">
                →
              </span>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
