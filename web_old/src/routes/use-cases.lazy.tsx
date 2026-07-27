import { createLazyRoute } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader } from "@/components/nb";
import { PendingComponent } from "@/components/PendingComponent";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp } from "@/lib/motion-utils";
import "../styles/use-cases.css";

export const Route = createLazyRoute("/use-cases")({
  component: UseCasesPage,
  pendingComponent: PendingComponent,
});

const CASES = [
  {
    title: "AI Agent Memory",
    tags: ["persistent", "≤1ms read", "crash-safe"],
    desc: "Store conversational history, agent thoughts, and user preferences locally. Context survives restarts with WAL crash safety.",
  },
  {
    title: "Local-First RAG",
    tags: ["hybrid search", "zero deps", "on-device"],
    desc: "Run BM25 lexical + HNSW vector fusion in-process. No external server, no network overhead.",
  },
  {
    title: "Codebase Intelligence",
    tags: ["graph edges", "AST-aware", "30K loc/s"],
    desc: "Map function definitions, imports, and caller relations in a local knowledge graph.",
  },
  {
    title: "Multi-Agent Orchestration",
    tags: ["namespaces", "isolation", "concurrent"],
    desc: "Run hundreds of independent agents on a single DB file. Namespace-level isolation.",
  },
  {
    title: "E-Commerce Semantic Search",
    tags: ["vector", "metadata filter", "real-time"],
    desc: "Serve personalized product recommendations using vector similarity on behavior embeddings.",
  },
  {
    title: "Edge / IoT Inference",
    tags: ["embedded", "ARM/RISC-V", "WAL-safe"],
    desc: "Persist device state and sensor telemetry on embedded hardware with WAL crash protection.",
  },
  {
    title: "Healthcare RAG",
    tags: ["PHI-safe", "audit log", "zero-server"],
    desc: "Run private medical RAG on-device with full audit trails. No PHI leaves the network.",
  },
  {
    title: "Financial Document Processing",
    tags: ["compliance", "high-throughput", "WAL"],
    desc: "Parse, index, and search documents with crash-safe durability at thousands per second.",
  },
];

const PIPELINE_STEPS = [
  {
    title: "Memory",
    desc: "Embeddings and metadata are written to the LSM-tree engine with immediate durability. WAL guarantees crash recovery.",
    tags: ["write-ahead log", "CRC32C", "O(1) append"],
  },
  {
    title: "Search",
    desc: "HNSW vector index + BM25 lexical index fused via RRF in a single call. Sub-millisecond in-process latency.",
    tags: ["HNSW", "BM25", "RRF fusion"],
  },
  {
    title: "Persist",
    desc: "Everything lives in a single portable DB file. Backup via SCP, move across machines, survive kills.",
    tags: ["single file", "portable", "zero reindex"],
  },
];

function UseCasesPage() {
  const patternsRef = useRef<HTMLElement>(null);
  const pipelineRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = patternsRef.current?.querySelectorAll<HTMLElement>(".nc-uc-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, patternsRef);

  useAnimationSafe(() => {
    const parts = pipelineRef.current?.querySelectorAll<HTMLElement>(".nc-uc-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, pipelineRef);

  return (
    <div className="nb-page">
      <NbSubpageHero
        pattern="p14"
        title={
          <span>
            Built for agents
            <br />
            that need context.
          </span>
        }
        sub="Eight production-tested patterns for persistent memory, hybrid search, and agentic data."
      />

      <main>
        <NbSection ref={patternsRef} ariaLabel="Production patterns">
          <NbSectionHeader
            monoLabel="[PATTERNS]"
            headline="Production patterns."
            sub="Eight proven architectures for embedded vector search in real-world applications."
          />
          <div className="nc-uc-grid nc-uc-part">
            {CASES.map((c) => (
              <div key={c.title} className="nc-uc-card nc-uc-card--amber">
                <h3 className="nc-uc-title">{c.title}</h3>
                <p className="nc-uc-desc">{c.desc}</p>
                <div className="nc-uc-tags">
                  {c.tags.map((tag) => (
                    <span key={tag} className="nc-uc-tag nc-uc-tag--amber">
                      {tag}
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </NbSection>

        <NbSection
          ref={pipelineRef}
          variant="lg"
          className="nb-bg-cross--faint"
          ariaLabel="Core pipeline"
        >
          <NbSectionHeader
            monoLabel="[PIPELINE]"
            headline="Memory → Search → Persist."
            sub="Three stages from ingestion to durable storage, all in-process."
          />
          <div className="nc-uc-pipeline nc-uc-part">
            {PIPELINE_STEPS.map((step) => (
              <div key={step.title} className="nc-uc-step nc-uc-step--amber">
                <h3 className="nc-uc-step-title">{step.title}</h3>
                <p className="nc-uc-step-desc">{step.desc}</p>
                <div className="nc-uc-tags">
                  {step.tags.map((tag) => (
                    <span key={tag} className="nc-uc-tag nc-uc-tag--amber">
                      {tag}
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </NbSection>
      </main>
    </div>
  );
}
