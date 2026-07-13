import { createLazyRoute } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbBlockAmber, NbSection, NbSectionHeader } from "@/components/nb";
import { PendingComponent } from "@/components/PendingComponent";
import { EngineBenchmark } from "@/components/EngineBenchmark";
import { EngineFeatureGrid } from "@/components/EngineFeatureGrid";
import { EngineGraphTopology } from "@/components/EngineGraphTopology";
import { EngineRRFWeightsSlider } from "@/components/EngineRRFWeightsSlider";
import { EngineWALSimulator } from "@/components/EngineWALSimulator";
import { EngineArchitecturePipeline } from "@/components/EngineArchitecturePipeline";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp } from "@/lib/motion-utils";
import "../styles/engine.css";

export const Route = createLazyRoute("/engine")({
  component: EnginePage,
  pendingComponent: PendingComponent,
});

function EnginePage() {
  const hybridRef = useRef<HTMLElement>(null);
  const graphRef = useRef<HTMLElement>(null);
  const walRef = useRef<HTMLElement>(null);
  const pipelineRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = hybridRef.current?.querySelectorAll<HTMLElement>(".");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, hybridRef);

  useAnimationSafe(() => {
    const parts = graphRef.current?.querySelectorAll<HTMLElement>(".");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, graphRef);

  useAnimationSafe(() => {
    const parts = walRef.current?.querySelectorAll<HTMLElement>(".");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, walRef);

  useAnimationSafe(() => {
    const parts = pipelineRef.current?.querySelectorAll<HTMLElement>(".");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, pipelineRef);

  return (
    <div className="nc-engine-crt">
      <NbSubpageHero
        pattern="p01"
        title={
          <span>
            Four modalities.
            <br />
            One atomic contract.
          </span>
        }
        sub="VantaDB consolidates lexical matching, HNSW vector search, local graph edges and transactional persistence in a zero-dependency Rust local-first library database."
      />

      <main>
        <NbSection ref={hybridRef} variant="lg" ariaLabel="Hybrid search">
          <NbSectionHeader
            monoLabel="[HYBRID SEARCH]"
            headline="BM25 + HNSW + RRF."
            sub="VantaDB query planner optimizes combined metadata filters, HNSW vector similarity, and BM25 full-text queries, synthesizing them into a single-pass execution plan."
          />

          <EngineBenchmark
            panelLabel="Instrument: Hybrid Fusion"
            description="Each query pass is fused through Reciprocal Rank Fusion, giving you the precision of keyword search with the semantic reach of vector embeddings — without managing separate infrastructure."
            items={[
              { value: "~1.2ms", label: "P50 Latency", unit: "Lexical BM25 search" },
              { value: "0.998", label: "Recall@10", unit: "Full-text recall rate" },
              { value: "M=16", label: "HNSW Connections", unit: "Graph density" },
              { value: "SQ8", label: "Quantization", unit: "Memory compression" },
            ]}
          />
        </NbSection>

        <NbSection ref={graphRef} ariaLabel="Graph topology">
          <NbSectionHeader
            monoLabel="[GRAPH QUERY]"
            headline="Knowledge topology."
            sub="Hover nodes to explore in-memory relations. VantaDB stores directed adjacency lists alongside vectors — supporting BFS, DFS, topological sort, and DAG cycle detection for graph-based agent memory."
          />

          <EngineFeatureGrid
            left={<EngineGraphTopology />}
            right={<EngineRRFWeightsSlider />}
            leftLabel="Live Topology · Hover to traverse"
          />
        </NbSection>

        <NbSection ref={walRef} ariaLabel="WAL durability">
          <NbSectionHeader
            monoLabel="[DURABILITY]"
            headline="Crash-safe WAL."
            sub="VantaDB guarantees complete transaction safety. Write-Ahead Logging forces log flushes before write acknowledgment, recovering state instantly on reboot."
          />

          <div className="nc-engine-section nc-engine-part">
            <p className="nc-engine-slider-desc">
              The WAL journal uses CRC32C integrity checksums with fsync-on-write semantics. On
              crash, automatic log replay detects the last consistent checkpoint and restores state
              in under 1ms.
            </p>
            <EngineWALSimulator />
          </div>
        </NbSection>

        <NbSection ref={pipelineRef} ariaLabel="Query pipeline">
          <NbSectionHeader
            monoLabel="[PIPELINE]"
            headline="End-to-end query execution."
            sub="A query travels through six stages — from parsing and hybrid search through graph traversal to a durable write confirmation."
          />

          <div className="nc-engine-section nc-engine-part">
            <EngineArchitecturePipeline />
          </div>
        </NbSection>

        <NbSection variant="dark" className="nb-bg-dot" ariaLabel="Get started">
          <NbBlockAmber as="div">
            <div className="nc-engine-cta">
              <div>
                <h2 className="nc-engine-cta-heading">Four modalities. One dependency.</h2>
                <p className="nc-engine-cta-sub">Install VantaDB in one command.</p>
              </div>
              <code className="nc-engine-cta-code">pip install vantadb-py</code>
            </div>
          </NbBlockAmber>
        </NbSection>
      </main>
    </div>
  );
}
