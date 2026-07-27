import { useState } from "react";

export function EngineRRFWeightsSlider() {
  const [bm25Weight, setBm25Weight] = useState(50);
  const hnswWeight = 100 - bm25Weight;

  const lexicalRecall = Math.round(bm25Weight * 0.7 + 10);
  const vectorRecall = Math.round(hnswWeight * 0.8 + 15);
  const fusedRecall = Math.round(100 - Math.abs(bm25Weight - 45) * 0.15);
  const queryLatency = (1.2 + (hnswWeight / 100) * 0.4).toFixed(2);

  return (
    <div className="nc-engine-slider-wrap">
      <div className="nc-engine-slider-header">
        <span className="nc-engine-slider-title">RRF Weights Planner</span>
        <span className="nc-engine-slider-latency">LATENCY: {queryLatency}ms</span>
      </div>

      <p className="nc-engine-slider-desc">
        Adjust the slider to coordinate keyword recall against vector space clustering.
      </p>

      <div className="nc-engine-slider-labels">
        <span>BM25: {bm25Weight}%</span>
        <span>HNSW: {hnswWeight}%</span>
      </div>

      <input
        type="range"
        min="0"
        max="100"
        value={bm25Weight}
        onChange={(e) => setBm25Weight(Number(e.target.value))}
        className="nc-engine-slider-input"
        aria-label="BM25 to HNSW fusion weight ratio"
      />

      <div className="nc-engine-slider-stats">
        <div className="nc-engine-slider-stat">
          <span className="nc-engine-slider-stat-label">LEXICAL RECALL</span>
          <span className="nc-engine-slider-stat-value">{lexicalRecall}%</span>
        </div>
        <div className="nc-engine-slider-stat">
          <span className="nc-engine-slider-stat-label">VECTOR RECALL</span>
          <span className="nc-engine-slider-stat-value">{vectorRecall}%</span>
        </div>
        <div className="nc-engine-slider-stat nc-engine-slider-stat--fused">
          <span className="nc-engine-slider-stat-label nc-engine-slider-stat-label--amber">
            FUSED @10
          </span>
          <span className="nc-engine-slider-stat-value nc-engine-slider-stat-value--amber">
            {fusedRecall}%
          </span>
        </div>
      </div>
    </div>
  );
}
