"use client";

import {
  Bot,
  BookOpen,
  Code2,
  ArrowRight,
  type LucideIcon,
} from "lucide-react";
import Link from "next/link";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { Reveal } from "@/components/vanta/reveal";
import { USE_CASES_DETAIL } from "@/components/vanta/vanta-data";
import { useLanguage } from "@/lib/language-provider";

const ICONS: Record<string, LucideIcon> = {
  Bot,
  BookOpen,
  Code2,
};

export default function UseCasesPage() {
  const { t, tt } = useLanguage();
  return (
    <div className="animate-rise">
      <PageHeader
        badge="§USE-CASES"
        title={tt("useCasesPage.title", "Use Cases")}
        subtitle={tt("useCasesPage.subtitle", "Three domains where local-first hybrid memory changes the rules. AI agents that remember between sessions. Local RAG without the cloud. IDE tooling with semantic code search — all in-process.")}
        tag={tt("useCasesPage.tag", "3 domains · in-process")}
      />

      <PageSection variant="cream">
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-3">
          {USE_CASES_DETAIL.map((uc, i) => {
            const Icon = ICONS[uc.icon] ?? Bot;
            return (
              <Reveal key={uc.slug} direction="up" delay={i * 80} as="article">
                <article className="press-lg group relative flex h-full flex-col border-4 border-black bg-[#F2EDE2] p-5  ">
                  <span className="absolute -left-2 -top-3 rotate-[-6deg] border-2 border-black bg-black px-2 py-0.5 font-display text-xs uppercase text-[#FF5500]   ">
                    0{i + 1}
                  </span>

                  <div className="mb-4 flex items-start justify-between">
                    <span className="inline-flex h-14 w-14 items-center justify-center border-4 border-black bg-[#FF5500] text-black shadow-[3px_3px_0_0_#000] transition-transform group-hover:rotate-[-6deg]  ">
                      <Icon className="h-7 w-7" strokeWidth={2.5} />
                    </span>
                  </div>

                  <h3 className="glitch-hover font-display text-2xl uppercase leading-none text-black ">
                    {tt(`useCasesPage.item.${i}.title`, uc.title)}
                  </h3>
                  <p className="mt-2 font-tech text-[11px] uppercase tracking-wider text-[#FF5500]">
                    {tt(`useCasesPage.item.${i}.tagline`, uc.tagline)}
                  </p>

                  <div className="mt-4 border-l-4 border-black bg-black/5 px-3 py-2  ">
                    <span className="font-tech text-[10px] font-bold uppercase tracking-wider text-black/70 ">
                      {tt("useCasesPage.painLabel", "The pain")}
                    </span>
                    <p className="mt-1 font-tech text-xs leading-relaxed text-black/80 ">
                      {tt(`solutions${uc.slug === "ai-agents" ? "Agents" : uc.slug === "local-rag" ? "LocalRag" : "AiIde"}.pain`, uc.pain)}
                    </p>
                  </div>

                  <div className="mt-3 border-l-4 border-[#FF5500] bg-[#FF5500]/5 px-3 py-2">
                    <span className="font-tech text-[10px] font-bold uppercase tracking-wider text-[#FF5500]">
                      {tt("useCasesPage.solutionLabel", "VantaDB solution")}
                    </span>
                    <p className="mt-1 font-tech text-xs leading-relaxed text-black/80 ">
                      {tt(`solutions${uc.slug === "ai-agents" ? "Agents" : uc.slug === "local-rag" ? "LocalRag" : "AiIde"}.solution`, uc.solution)}
                    </p>
                  </div>

                  <div className="mt-auto pt-5">
                    <Link
                      href={`/solutions/${uc.slug}`}
                      className="press inline-flex w-full items-center justify-between gap-2 border-4 border-black bg-[#1A1A1A] px-3 py-2.5 font-tech text-xs font-bold uppercase tracking-wider text-[#FBF9F5] transition-colors hover:bg-[#FF5500] hover:text-black     "
                    >
                      <span>{tt("useCasesPage.explore", "Explore")}</span>
                      <ArrowRight className="h-3.5 w-3.5" strokeWidth={2.5} />
                    </Link>
                  </div>

                  <div className="mt-4 h-1 w-full speed-lines opacity-30" />
                </article>
              </Reveal>
            );
          })}
        </div>
      </PageSection>
    </div>
  );
}
