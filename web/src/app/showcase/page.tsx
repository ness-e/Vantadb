"use client";

import { ArrowUpRight, Sparkles, type LucideIcon } from "lucide-react";
import Link from "next/link";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { Reveal } from "@/components/vanta/reveal";
import { useLanguage } from "@/lib/language-provider";

const ICONS: Record<string, LucideIcon> = {
  Sparkles,
  ArrowUpRight,
};

interface ShowcaseItem {
  title: string;
  desc: string;
  author: string;
  tags: string[];
  url: string;
  emoji: string;
}

export default function ShowcasePage() {
  const { t } = useLanguage();
  const tt = (key: string, fallback: string) => {
    const v = t(key);
    return v === key ? fallback : v;
  };

  const items: ShowcaseItem[] = [
    {
      title: tt("showcasePage.items.0.title", "LangGraph Checkpoint Store"),
      desc: tt(
        "showcasePage.items.0.desc",
        "Persistent checkpoint storage for LangGraph agents. Namespace-scoped memory with hybrid search, GIL-released batch queries, and sub-ms recall on conversation history."
      ),
      author: "@ness-e",
      tags: ["agents", "langgraph", "memory"],
      url: "https://github.com/ness-e/Vantadb/blob/main/examples/python/langgraph_checkpoint.py",
      emoji: "🧠",
    },
    {
      title: tt("showcasePage.items.1.title", "AutoGen Multi-Agent Memory"),
      desc: tt(
        "showcasePage.items.1.desc",
        "Persistent memory backend for AutoGen conversational agents. Six agents, shared knowledge base, topic-scoped isolation via namespaces."
      ),
      author: "@ness-e",
      tags: ["agents", "autogen", "multi-agent"],
      url: "https://github.com/ness-e/Vantadb/blob/main/examples/python/autogen_memory.py",
      emoji: "🤖",
    },
    {
      title: tt("showcasePage.items.2.title", "Haystack DocumentStore"),
      desc: tt(
        "showcasePage.items.2.desc",
        "VantaDB as a DocumentStore for Haystack pipelines. Hybrid vector + text search backend for RAG pipelines with metadata filtering."
      ),
      author: "@ness-e",
      tags: ["rag", "haystack", "pipeline"],
      url: "https://github.com/ness-e/Vantadb/blob/main/examples/python/haystack_documentstore.py",
      emoji: "📚",
    },
    {
      title: tt("showcasePage.items.3.title", "CrewAI Agent Memory"),
      desc: tt(
        "showcasePage.items.3.desc",
        "Persistent memory for CrewAI agent crews. Shared context across tasks, role-scoped namespaces, automatic vector indexing of agent outputs."
      ),
      author: "@ness-e",
      tags: ["agents", "crewai", "memory"],
      url: "https://github.com/ness-e/Vantadb/blob/main/examples/python/crewai_memory.py",
      emoji: "👥",
    },
    {
      title: tt("showcasePage.items.4.title", "Rust Hybrid Search"),
      desc: tt(
        "showcasePage.items.4.desc",
        "Core hybrid search engine: BM25 + HNSW + RRF fusion in native Rust. Full CBO planner pushing filters before graph traversal. Sub-ms search on 10k+ nodes."
      ),
      author: "@ness-e",
      tags: ["rust", "hybrid", "engine"],
      url: "https://github.com/ness-e/Vantadb/blob/main/examples/rust/hybrid.rs",
      emoji: "⚙️",
    },
    {
      title: tt("showcasePage.items.5.title", "GraphRAG Pipeline"),
      desc: tt(
        "showcasePage.items.5.desc",
        "End-to-end GraphRAG in Rust: seed entities, expand relationships, retrieve subgraphs, generate context. KB construction with incremental indexing."
      ),
      author: "@ness-e",
      tags: ["graphrag", "rust", "pipeline"],
      url: "https://github.com/ness-e/Vantadb/blob/main/examples/rust/graphrag.rs",
      emoji: "🔬",
    },
  ];

  return (
    <div className="animate-rise">
      <PageHeader
        badge="§SHOWCASE"
        title={tt("showcasePage.title", "Community Showcase")}
          subtitle={tt(
          "showcasePage.subtitle",
          "Proyectos construidos con VantaDB por la comunidad. Agentes, RAG, devtools — todos local-first, todos sin cloud tax."
        )}
        tag={tt("showcasePage.tag", "Community · local-first")}
      />

      <PageSection variant="cream">
        <Reveal direction="up">
          <div className="mb-8 flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <span className="inline-flex items-center gap-2 border-2 border-black bg-[#FF5500] px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-black ">
                <span className="h-1.5 w-1.5 bg-black" />
                {tt("showcasePage.gridTag", "Built with VantaDB")}
              </span>
              <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
                {tt("showcasePage.gridTitle", "Shipped, not slideshows")}
              </h2>
              <p className="mt-2 max-w-2xl font-tech text-xs text-black/70 ">
                {tt(
                  "showcasePage.gridSubtitle",
                  "Cada proyecto es código abierto y corre en tu máquina. Click para ver el repo, cloná, ejecutá."
                )}
              </p>
            </div>
          </div>
        </Reveal>

        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3">
          {items.map((it, i) => (
            <Reveal key={it.title} direction="up" delay={i * 60} as="article">
              <article className="press-lg group relative flex h-full flex-col border-4 border-black bg-[#F2EDE2] p-5  ">
                <div className="mb-4 flex items-start justify-between gap-3">
                  <span className="inline-flex h-12 w-12 shrink-0 items-center justify-center border-4 border-black bg-[#FBF9F5] text-2xl shadow-[3px_3px_0_0_#000]   ">
                    {it.emoji}
                  </span>
                  <ArrowUpRight className="h-4 w-4 text-black/40 transition-all group-hover:-translate-y-0.5 group-hover:translate-x-0.5 group-hover:text-[#FF5500] " strokeWidth={2.5} />
                </div>
                <h3 className="glitch-hover font-display text-2xl uppercase leading-none text-black ">
                  {it.title}
                </h3>
                <p className="mt-3 flex-1 font-tech text-xs leading-relaxed text-black/80 ">
                  {it.desc}
                </p>
                <div className="mt-4 flex flex-wrap items-center gap-1.5">
                  {it.tags.map((tag) => (
                    <span
                      key={tag}
                      className="inline-flex items-center border border-black/40 px-1.5 py-0.5 font-tech text-[10px] uppercase tracking-wider text-black/60  "
                    >
                      {tag}
                    </span>
                  ))}
                </div>
                <div className="mt-4 flex items-center justify-between border-t-2 border-black/10 pt-3 ">
                  <span className="font-tech text-[10px] uppercase tracking-wider text-black/60 ">
                    {it.author}
                  </span>
                  <Link
                    href={it.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="font-tech text-[10px] font-bold uppercase tracking-wider text-black/60 transition-colors group-hover:text-[#FF5500] "
                  >
                    {tt("showcasePage.viewSource", "View source")}
                  </Link>
                </div>
              </article>
            </Reveal>
          ))}
        </div>
      </PageSection>

      <PageSection variant="paper">
        <Reveal direction="up">
          <div className="border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:p-8">
            <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
              <div className="flex items-center gap-4">
                <span className="inline-flex h-12 w-12 shrink-0 items-center justify-center border-4 border-[#FF5500] bg-[#FF5500] text-black shadow-[4px_4px_0_0_#FBF9F5]">
                  <Sparkles className="h-6 w-6" strokeWidth={2.5} />
                </span>
                <div>
                  <h3 className="font-display text-2xl uppercase leading-none text-[#FBF9F5]">
                    {tt("showcasePage.ctaTitle", "Built something with VantaDB?")}
                  </h3>
                  <p className="mt-2 max-w-xl font-tech text-xs text-[#FBF9F5]/70">
                    {tt(
                      "showcasePage.ctaBody",
                      "Mandanos un PR con tu proyecto al README, o abrí un issue con el link. Lo agregamos al showcase."
                    )}
                  </p>
                </div>
              </div>
              <Link
                href="https://github.com/ness-e/Vantadb"
                target="_blank"
                rel="noopener noreferrer"
                className="press-neon btn-neon-glow inline-flex shrink-0 items-center gap-2 border-4 border-[#FBF9F5] bg-[#FF5500] px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black"
              >
                {tt("showcasePage.ctaBtn", "Submit your project")}
                <ArrowUpRight className="h-4 w-4" strokeWidth={2.5} />
              </Link>
            </div>
          </div>
        </Reveal>
      </PageSection>
    </div>
  );
}
