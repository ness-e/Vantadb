"use client";

import { Bot, ArrowRight, AlertTriangle, Check, Workflow, Gauge, Code2 } from "lucide-react";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { Reveal } from "@/components/vanta/reveal";
import { USE_CASES_DETAIL, VANTA } from "@/components/vanta/vanta-data";
import { useLanguage } from "@/lib/language-provider";

const UC = USE_CASES_DETAIL[0]; // ai-agents

export default function AiAgentsPage() {
  const { tt } = useLanguage();
  return (
    <div className="animate-rise">
      <PageHeader
        badge="§SOLUTION"
        title={tt("useCasesPage.item.0.title", UC.title)}
        subtitle={tt("useCasesPage.item.0.tagline", UC.tagline)}
        tag={tt("solutionAiAgentsPage.tag", "Persistent agent memory · 1.2ms recall")}
      >
        <div className="flex flex-wrap items-center gap-3">
          <span className="inline-flex h-12 w-12 items-center justify-center border-4 border-black bg-[#FF5500] text-black shadow-[3px_3px_0_0_#000]  ">
            <Bot className="h-6 w-6" strokeWidth={2.5} />
          </span>
          <span className="font-tech text-xs uppercase tracking-wider text-black/60 ">
            {tt("solutionsAgents.headerNote", "Namespace-scoped memory · survives process restarts · zero re-embedding")}
          </span>
        </div>
      </PageHeader>

      {/* Pain + Solution */}
      <PageSection variant="cream">
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          <Reveal direction="right" as="article">
            <article className="press-lg h-full border-4 border-black bg-[#F2EDE2] p-6  ">
              <div className="mb-3 flex items-center gap-2">
                <AlertTriangle className="h-5 w-5 text-black " strokeWidth={2.5} />
                <span className="font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-black/70 ">
                  {tt("solutionPage.painLabel", "The pain")}
                </span>
              </div>
              <h2 className="glitch-hover font-display text-2xl uppercase leading-none text-black ">
                {tt("solutionsAgents.painTitle", "Memory that forgets")}
              </h2>
              <p className="mt-3 font-tech text-sm leading-relaxed text-black/80 ">
                {tt("solutionsAgents.pain", UC.pain)}
              </p>
            </article>
          </Reveal>

          <Reveal direction="left" as="article">
            <article className="press-lg h-full border-4 border-[#FF5500] bg-[#FBF9F5] p-6 shadow-[6px_6px_0_0_#FF5500] ">
              <div className="mb-3 flex items-center gap-2">
                <Check className="h-5 w-5 text-[#FF5500]" strokeWidth={2.5} />
                <span className="font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-[#FF5500]">
                  {tt("solutionPage.solutionLabel", "VantaDB solution")}
                </span>
              </div>
              <h2 className="glitch-hover font-display text-2xl uppercase leading-none text-black ">
                {tt("solutionsAgents.solutionTitle", "Durable local recall")}
              </h2>
              <p className="mt-3 font-tech text-sm leading-relaxed text-black/80 ">
                {tt("solutionsAgents.solution", UC.solution)}
              </p>
            </article>
          </Reveal>
        </div>
      </PageSection>

      {/* Flow */}
      <PageSection variant="paper">
        <Reveal direction="up">
          <div className="mb-8 flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <span className="inline-flex items-center gap-2 border-2 border-black bg-black px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-[#FF5500] ">
                <Workflow className="h-3 w-3" strokeWidth={3} />
                {tt("solutionPage.flowTag", "Flow")}
              </span>
              <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
                {tt("solutionPage.flowTitle", "How it works end-to-end")}
              </h2>
            </div>
          </div>
        </Reveal>

        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {UC.flow.map((step, i) => (
            <Reveal key={i} direction="up" delay={i * 80} as="article">
              <article className="press h-full border-4 border-black bg-[#FBF9F5] p-4  ">
                <span className="inline-flex h-8 w-8 items-center justify-center border-2 border-black bg-[#FF5500] font-display text-sm text-black ">
                  {i + 1}
                </span>
                <p className="mt-3 font-tech text-xs leading-relaxed text-black/80 ">
                  {tt(`solutionsAgents.flow.${i}`, step)}
                </p>
                {i < UC.flow.length - 1 && (
                  <ArrowRight
                    className="mt-3 hidden h-4 w-4 text-[#FF5500] lg:block"
                    strokeWidth={2.5}
                    aria-hidden
                  />
                )}
              </article>
            </Reveal>
          ))}
        </div>
      </PageSection>

      {/* Metrics */}
      <PageSection variant="ink">
        <Reveal direction="up">
          <div className="mb-8 flex items-center gap-3">
            <Gauge className="h-6 w-6 text-[#FF5500]" strokeWidth={2.5} />
            <h2 className="glitch-hover font-display text-3xl uppercase leading-none text-[#FBF9F5] sm:text-4xl">
              {tt("solutionPage.metricsTitle", "Metrics")}
            </h2>
          </div>
        </Reveal>

        <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
          {UC.metrics.map((m, i) => (
            <Reveal key={i} direction="scale" delay={i * 80} as="article">
              <article className="border-4 border-[#FBF9F5] bg-[#1A1A1A] p-6">
                <span className="font-display text-5xl uppercase leading-none text-[#FF5500]">
                  {m.value}
                </span>
                <p className="mt-2 font-tech text-xs uppercase tracking-wider text-[#FBF9F5]/60">
                  {tt(`solutionsAgents.metricLabel.${i}`, m.label)}
                </p>
              </article>
            </Reveal>
          ))}
        </div>
      </PageSection>

      {/* Code block */}
      <PageSection variant="cream">
        <Reveal direction="up">
          <div className="mb-6 flex items-center gap-3">
            <Code2 className="h-6 w-6 text-black " strokeWidth={2.5} />
            <h2 className="glitch-hover font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
              {tt("solutionPage.codeTitle", "Code")}
            </h2>
          </div>
        </Reveal>

        <Reveal direction="up" delay={80}>
          <div className="border-4 border-black bg-black shadow-[6px_6px_0_0_#000]  ">
            <div className="flex items-center justify-between border-b-2 border-[#FBF9F5]/20 bg-[#1A1A1A] px-3 py-2">
              <span className="inline-flex items-center gap-1.5 font-tech text-[11px] uppercase tracking-wider text-[#FBF9F5]/70">
                <Code2 className="h-3 w-3 text-[#FF5500]" />
                {UC.slug}.py
              </span>
              <span className="font-tech text-[9px] uppercase tracking-wider text-[#FBF9F5]/30">
                {tt("solutionPage.codeLang", "Python · vantadb")}
              </span>
            </div>
            <pre className="scanlines overflow-x-auto bg-black p-4 font-tech text-[12px] leading-relaxed text-[#FBF9F5]">
              <code>{UC.code}</code>
            </pre>
          </div>
        </Reveal>

        <Reveal direction="up" delay={120}>
          <div className="mt-8 flex flex-col items-center justify-between gap-4 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:flex-row">
            <div>
              <h3 className="font-display text-2xl uppercase leading-none text-[#FBF9F5]">
                {tt("solutionsAgents.ctaTitle", "Give your agent a memory")}
              </h3>
              <p className="mt-1 font-tech text-xs text-[#FBF9F5]/60">
                {tt("solutionsAgents.ctaBody", "pip install vantadb-py and put() your first observation in 30 seconds.")}
              </p>
            </div>
            <a
              href={VANTA.pypi}
              target="_blank"
              rel="noopener noreferrer"
              className="press-neon btn-neon-glow inline-flex shrink-0 items-center gap-2 border-4 border-[#FBF9F5] bg-[#FF5500] px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black"
            >
              pip install vantadb-py
              <ArrowRight className="h-4 w-4" strokeWidth={2.5} />
            </a>
          </div>
        </Reveal>
      </PageSection>
    </div>
  );
}
