import { createLazyRoute } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
import { PendingComponent } from "@/components/PendingComponent";
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
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(patternsRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, patternsRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(pipelineRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
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

      <NbSection ref={patternsRef} ariaLabel="Production patterns">
        <NbSectionHeader
          monoLabel="[PATTERNS]"
          headline="Production patterns."
          sub="Eight proven architectures for embedded vector search in real-world applications."
        />
        <div className="use-cases-bento nb-engine-part">
          {CASES.map((c) => (
            <div key={c.title} className="use-cases-pattern-card">
              <h3 className="nb-card-frame-title">{c.title}</h3>
              <p className="nb-card-frame-desc">{c.desc}</p>
              <div className="use-cases-tags-wrap">
                {c.tags.map((tag) => (
                  <span key={tag} className="use-cases-tag">
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
        <div className="use-cases-pipeline-grid nb-engine-part">
          {PIPELINE_STEPS.map((step) => (
            <div key={step.title} className="use-cases-pipeline-card">
              <h3 className="nb-card-frame-title">{step.title}</h3>
              <p className="nb-card-frame-desc">{step.desc}</p>
              <div className="use-cases-tags-wrap">
                {step.tags.map((tag) => (
                  <span key={tag} className="use-cases-tag-amber">
                    {tag}
                  </span>
                ))}
              </div>
            </div>
          ))}
        </div>
      </NbSection>
    </div>
  );
}
