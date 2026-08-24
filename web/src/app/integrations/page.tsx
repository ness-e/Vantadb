"use client";

import { ArrowUpRight } from "lucide-react";
import Link from "next/link";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { Reveal } from "@/components/vanta/reveal";
import { useLanguage } from "@/lib/language-provider";

type Status = "native" | "experimental" | "coming";



interface Integration {
  name: string;
  desc: string;
  status: Status;
  url?: string;
}

export default function IntegrationsPage() {
  const { tt } = useLanguage();

  const integrations: Integration[] = [
    {
      name: "OpenAI",
      desc: tt(
        "integrationsPage.items.0.desc",
        "Embeddings GPT-4 + vectores 1536/3072d. Pipeline directo: text → embedding → VantaDB.put(). Sin adapter, sin glue code."
      ),
      status: "native",
    },
    {
      name: "Ollama",
      desc: tt(
        "integrationsPage.items.1.desc",
        "Modelos locales (llama3, mistral, nomic-embed) via Ollama API. Inference + retrieval 100% local-first, sin API key."
      ),
      status: "native",
    },
    {
      name: "CrewAI",
      desc: tt(
        "integrationsPage.items.2.desc",
        "Memoria persistente para agentes CrewAI. Cada agente tiene su namespace en VantaDB — shared memory across tasks sin cloud."
      ),
      status: "native",
    },
    {
      name: "Haystack",
      desc: tt(
        "integrationsPage.items.3.desc",
        "Document store compatible con pipelines Haystack 2.x. Drop-in replacement para Elasticsearch/OpenSearch stores."
      ),
      status: "native",
    },
    {
      name: "DSPy",
      desc: tt(
        "integrationsPage.items.4.desc",
        "Retriever module para DSPy. Indexá tu corpus en VantaDB y usalo como retrieval backend en pipelines DSPy sin configuración extra."
      ),
      status: "experimental",
    },
    {
      name: "LiteLLM",
      desc: tt(
        "integrationsPage.items.5.desc",
        "Proxy unificado de LLMs. Combiná LiteLLM para inference + VantaDB para retrieval. Ambos son drop-in Python — sin setup adicional."
      ),
      status: "native",
    },
    {
      name: "Mem0",
      desc: tt(
        "integrationsPage.items.6.desc",
        "Capa de memoria para agentes. VantaDB como backend persistente para Mem0 — recuerdos long-term en tu filesystem."
      ),
      status: "experimental",
    },
    {
      name: "Letta",
      desc: tt(
        "integrationsPage.items.7.desc",
        "Agentes con memoria stateful (MemGPT). VantaDB reemplaza el storage backend por defecto, manteniendo states in-process."
      ),
      status: "experimental",
    },
    {
      name: "MCP",
      desc: tt(
        "integrationsPage.items.8.desc",
        "Model Context Protocol server experimental. Expone VantaDB como tool consumible por Claude, Cursor y otros MCP clients."
      ),
      status: "coming",
    },
  ];

  const statusMeta: Record<Status, { label: string; className: string }> = {
    native: {
      label: tt("integrationsPage.status.native", "Native"),
      className: "border-2 border-black bg-[#FF5500] text-black ",
    },
    experimental: {
      label: tt("integrationsPage.status.experimental", "Experimental"),
      className: "border-2 border-black bg-black text-[#FF5500]   ",
    },
    coming: {
      label: tt("integrationsPage.status.coming", "Coming soon"),
      className: "border-2 border-black bg-[#F2EDE2] text-black/70   ",
    },
  };

  return (
    <div className="animate-rise">
      <PageHeader
        badge="§ECOSYSTEM"
        title={tt("integrationsPage.title", "Ecosystem & Integrations")}
        subtitle={tt(
          "integrationsPage.subtitle",
          "VantaDB se integra con el stack de IA que ya usás. Native, experimental y coming soon. Sin glue code, sin adapters pesados."
        )}
        tag={tt("integrationsPage.tag", "OpenAI · Ollama · CrewAI · MCP")}
      />

      <PageSection variant="cream">
        <Reveal direction="up">
          <div className="mb-8 flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <span className="inline-flex items-center gap-2 border-2 border-black bg-[#FF5500] px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-black ">
                <span className="h-1.5 w-1.5 bg-black" />
                {tt("integrationsPage.gridTag", "Integration matrix")}
              </span>
              <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
                {tt("integrationsPage.gridTitle", "Plays well with your stack")}
              </h2>
              <p className="mt-2 max-w-2xl font-tech text-xs text-black/70 ">
                {tt(
                  "integrationsPage.gridSubtitle",
                  "Cada integración es código Python. Native = soporte first-class. Experimental = API puede cambiar. Coming soon = en roadmap activo."
                )}
              </p>
            </div>
          </div>
        </Reveal>

        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3">
          {integrations.map((it, i) => {
            const meta = statusMeta[it.status];
            return (
              <Reveal key={it.name} direction="up" delay={i * 60} as="article">
                <article className="press-lg group relative flex h-full flex-col border-4 border-black bg-[#F2EDE2] p-5  ">
                  <div className="mb-4 flex items-start justify-between gap-3">
                    <span className="inline-flex h-11 w-11 shrink-0 items-center justify-center border-4 border-black bg-[#FBF9F5] font-display text-lg uppercase text-black shadow-[3px_3px_0_0_#000]    ">
                      {it.name.charAt(0)}
                    </span>
                    <span className={`inline-flex items-center gap-1.5 px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.2em] ${meta.className}`}>
                      <span className="h-1.5 w-1.5 bg-current" />
                      {meta.label}
                    </span>
                  </div>
                  <h3 className="glitch-hover font-display text-2xl uppercase leading-none text-black ">
                    {it.name}
                  </h3>
                  <p className="mt-3 flex-1 font-tech text-xs leading-relaxed text-black/80 ">
                    {it.desc}
                  </p>
                  <div className="mt-4 flex items-center justify-between">
                    <div className="h-1 flex-1 speed-lines opacity-30" />
                    {it.url && (
                      <Link
                        href={it.url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="ml-3 inline-flex items-center gap-1 font-tech text-[10px] font-bold uppercase tracking-wider text-black/60 transition-colors group-hover:text-[#FF5500] "
                      >
                        {tt("integrationsPage.viewDocs", "Docs")}
                        <ArrowUpRight className="h-3 w-3" strokeWidth={2.5} />
                      </Link>
                    )}
                  </div>
                </article>
              </Reveal>
            );
          })}
        </div>
      </PageSection>

      <PageSection variant="paper">
        <Reveal direction="up">
          <div className="border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:p-8">
            <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <h3 className="font-display text-2xl uppercase leading-none text-[#FBF9F5]">
                  {tt("integrationsPage.ctaTitle", "Missing an integration?")}
                </h3>
                <p className="mt-2 max-w-xl font-tech text-xs text-[#FBF9F5]/70">
                  {tt(
                    "integrationsPage.ctaBody",
                    "VantaDB es Python puro — cualquier framework que hable Python puede usarlo. Abrí un issue en GitHub con tu caso de uso, o mandá un PR."
                  )}
                </p>
              </div>
              <Link
                href="https://github.com/ness-e/Vantadb/issues/new"
                target="_blank"
                rel="noopener noreferrer"
                className="press-neon btn-neon-glow inline-flex shrink-0 items-center gap-2 border-4 border-[#FBF9F5] bg-[#FF5500] px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black"
              >
                {tt("integrationsPage.ctaBtn", "Open an issue")}
                <ArrowUpRight className="h-4 w-4" strokeWidth={2.5} />
              </Link>
            </div>
          </div>
        </Reveal>
      </PageSection>
    </div>
  );
}
