import { createLazyRoute, Link } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader, NbBlockAmber } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
import "../../styles/ai-agents.css";

export const Route = createLazyRoute("/solutions/ai-agents")({
  component: AiAgentsPage,
});

const PROBLEMS = [
  { icon: "✗", text: "OpenAI assistants: context window limited to 128K tokens" },
  { icon: "✗", text: "Redis + embedding: two services to manage for persisted memory" },
  { icon: "✗", text: "Flat files: no semantic search, no structured querying" },
  { icon: "✗", text: "Cloud vector DB: adds 100ms+ latency per memory access" },
];

const SOLUTIONS_LIST = [
  { icon: "✓", text: "Store agent memories as typed vector + metadata records" },
  { icon: "✓", text: 'Semantic recall: "What did the user say about pricing?"' },
  { icon: "✓", text: 'Structured queries: "Show tool calls from last 24h with errors"' },
  { icon: "✓", text: "In-process: no net call, no serialization, 1.2ms recall" },
];

const PRIMITIVES = [
  {
    num: "01",
    title: "Conversation Log",
    desc: "Append-only log of every turn. Query by semantic similarity, time range, or metadata tags.",
  },
  {
    num: "02",
    title: "Tool Result Cache",
    desc: "Store tool call outputs keyed by input hash. Avoid redundant LLM invocations.",
  },
  {
    num: "03",
    title: "User Preferences",
    desc: "Persistent key-value with vector embeddings for preference matching across sessions.",
  },
  {
    num: "04",
    title: "Ephemeral State",
    desc: "In-memory WAL for active session state. Flushed to durable storage on checkpoint.",
  },
];

const MEMORY_CODE = `import vantadb_py as vantadb

db = vantadb.VantaDB("./agent_memory")

# Store a memory with vector + metadata
db.put(key="msg-1", vector=embedding, metadata={"role": "user", "session": session_id, "text": message})

# Semantic recall — no API call, no network
results = db.search_memory(query=embedding, top_k=5)`;

function AiAgentsPage() {
  const problemRef = useRef<HTMLElement>(null);
  const primitivesRef = useRef<HTMLElement>(null);
  const implRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nc-aa-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(problemRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, problemRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nc-aa-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(primitivesRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, primitivesRef);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nc-aa-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(implRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, implRef);

  return (
    <div className="nb-page">
      <NbSubpageHero
        pattern="p23"
        title={
          <span>
            Memory that
            <br />
            doesn't forget.
          </span>
        }
        sub="Give your AI agent persistent memory — conversation history, tool call results, learned preferences, and ephemeral state — all in one embedded database that lives inside your agent process."
      />

      <NbSection ref={problemRef} ariaLabel="The Problem">
        <NbSectionHeader
          monoLabel="[THE PROBLEM]"
          headline="Stateless agents can't remember."
          sub="Every conversation starts from zero. VantaDB gives agents persistent memory that lives inside the process."
        />

        <div className="nc-aa-part">
          <div className="nc-aa-compare">
            <div className="nc-aa-col">
              <span className="nc-aa-col-label nc-aa-col-label--danger">Stateless agents</span>
              <ul className="nc-aa-list">
                {PROBLEMS.map((p) => (
                  <li key={p.text} className="nc-aa-item">
                    <span className="nc-aa-icon nc-aa-icon--danger">
                      {p.icon}
                    </span>
                    {p.text}
                  </li>
                ))}
              </ul>
            </div>
            <div className="nc-aa-col nc-aa-col--amber">
              <span className="nc-aa-col-label nc-aa-col-label--amber">Embedded memory</span>
              <ul className="nc-aa-list">
                {SOLUTIONS_LIST.map((s) => (
                  <li key={s.text} className="nc-aa-item nc-aa-item--fg">
                    <span className="nc-aa-icon nc-aa-icon--amber">{s.icon}</span>
                    {s.text}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </div>
      </NbSection>

      <NbSection ref={primitivesRef} className="nb-bg-cross--faint" ariaLabel="Memory Primitives">
        <NbSectionHeader
          monoLabel="[PRIMITIVES]"
          headline="Four fundamental memory patterns."
          sub="Structured, searchable, and purpose-built for agent architectures."
        />

        <div className="nc-aa-part">
          <div className="nc-aa-primitives">
            {PRIMITIVES.map((p) => (
              <div key={p.num} className="nc-aa-pcard">
                <span className="nc-aa-pnum">{p.num}</span>
                <h3 className="nc-aa-ptitle">{p.title}</h3>
                <p className="nc-aa-pdesc">{p.desc}</p>
              </div>
            ))}
          </div>
        </div>
      </NbSection>

      <NbSection ref={implRef} ariaLabel="Implementation">
        <NbSectionHeader
          monoLabel="[IMPLEMENTATION]"
          headline="One import. One database."
          sub="Embed persistent memory into your agent with three lines of Python."
        />

        <div className="nc-aa-part">
          <pre className="nc-aa-code">
            <code>{MEMORY_CODE}</code>
          </pre>
        </div>

        <div className="nc-aa-part">
          <NbBlockAmber>
            <div className="nb-text-center">
              <span className="nb-mono-label">BUILD YOUR AGENT</span>
              <p className="nb-section-sub">Get started with the docs.</p>
              <Link to="/docs" className="nb-btn nb-btn--ghost">
                DOCS
              </Link>
            </div>
          </NbBlockAmber>
        </div>
      </NbSection>
    </div>
  );
}

export function PendingComponent() {
  return (
    <div className="ai-agents-pending">
      <span>Loading...</span>
    </div>
  );
}
