"use client";

import { ArrowLeft, ArrowRight, Building2, Factory, AlertTriangle, Check, Quote } from "lucide-react";
import { useParams, useRouter } from "next/navigation";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { Reveal } from "@/components/vanta/reveal";
import { CASE_STUDIES } from "@/components/vanta/vanta-data";
import { useLanguage } from "@/lib/language-provider";

export default function CaseStudyPage() {
  const params = useParams();
  const router = useRouter();
  const { t, tt } = useLanguage();
  const slug = params.slug as string;
  const csIndex = CASE_STUDIES.findIndex((c) => c.slug === slug);
  const cs = csIndex >= 0 ? CASE_STUDIES[csIndex] : undefined;

  if (!cs) {
    return (
      <div className="animate-rise">
        <PageHeader
          badge="§404"
          title={tt("caseStudyPage.title", "Case Study Not Found")}
          subtitle={tt("caseStudyPage.subtitle", "This case study doesn't exist or hasn't been published yet.")}
          tag={tt("caseStudyPage.tag", "Case Studies · missing")}
        />
        <PageSection variant="cream">
          <div className="mx-auto max-w-2xl">
            <button
              onClick={() => router.push("/case-studies")}
              className="press inline-flex items-center gap-2 border-4 border-black bg-[#FF5500] px-5 py-3 font-tech text-xs font-bold uppercase tracking-wider text-black shadow-[6px_6px_0_0_#000] transition-colors hover:bg-black hover:text-[#FF5500]  "
            >
              <ArrowLeft className="h-4 w-4" strokeWidth={2.5} aria-hidden />
              {tt("caseStudy.backToList", "Back to Case Studies")}
            </button>
          </div>
        </PageSection>
      </div>
    );
  }

  return (
    <div className="animate-rise">
      <PageHeader
        badge={`§${tt(`caseStudiesData.${csIndex}.industry`, cs.industry).toUpperCase()}`}
        title={tt(`caseStudiesData.${csIndex}.title`, cs.title)}
        subtitle={tt(`caseStudiesData.${csIndex}.summary`, cs.summary)}
        tag={`${cs.company} · ${tt(`caseStudiesData.${csIndex}.industry`, cs.industry)}`}
      >
        <div className="flex flex-wrap items-center gap-3 font-tech text-[11px] uppercase tracking-wider text-black/70 ">
          <span className="inline-flex items-center gap-1.5 border-2 border-black bg-[#FBF9F5] px-2.5 py-1 text-black   ">
            <Building2 className="h-3 w-3 text-[#FF5500]" strokeWidth={2.5} aria-hidden />
            {cs.company}
          </span>
          <span className="inline-flex items-center gap-1.5 border-2 border-black bg-[#FBF9F5] px-2.5 py-1 text-black   ">
            <Factory className="h-3 w-3 text-[#FF5500]" strokeWidth={2.5} aria-hidden />
            {tt(`caseStudiesData.${csIndex}.industry`, cs.industry)}
          </span>
        </div>
      </PageHeader>

      {/* Composite disclaimer — these scenarios are illustrative, not real customers */}
      <div className="border-b-4 border-black bg-[#F2EDE2] px-4 py-2.5 text-center sm:px-6">
        <p className="font-tech text-[10px] uppercase tracking-[0.2em] text-black/70">
          {tt(
            "caseStudy.compositeDisclaimer",
            "Composite scenario based on typical usage patterns — not a specific customer"
          )}
        </p>
      </div>

      {/* Metrics grid */}
      <PageSection variant="ink">
        <Reveal direction="up">
          <div className="mb-8 flex items-center gap-3">
            <span className="inline-flex h-2 w-2 animate-flicker bg-[#FF5500]" aria-hidden />
            <span className="font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-[#FBF9F5]/60">
              {tt("caseStudy.resultsLabel", "Results")}
            </span>
          </div>
        </Reveal>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
          {cs.metrics.map((m, i) => (
            <Reveal key={i} direction="scale" delay={i * 80} as="article">
              <article className="press-lg h-full border-4 border-[#FBF9F5] bg-[#1A1A1A] p-6 shadow-[6px_6px_0_0_#FF5500]">
                <span className="font-display text-4xl uppercase leading-none text-[#FF5500] sm:text-5xl">
                  {m.value}
                </span>
                <p className="mt-3 font-tech text-xs uppercase tracking-wider text-[#FBF9F5]/70">
                  {tt(`caseStudiesData.${csIndex}.metricLabel.${i}`, m.label)}
                </p>
              </article>
            </Reveal>
          ))}
        </div>
      </PageSection>

      {/* Challenge + Solution */}
      <PageSection variant="cream">
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          <Reveal direction="right" as="article">
            <article className="press-lg h-full border-4 border-black bg-[#F2EDE2] p-6 shadow-[6px_6px_0_0_#000]   ">
              <div className="mb-3 flex items-center gap-2">
                <AlertTriangle className="h-5 w-5 text-black " strokeWidth={2.5} />
                <span className="font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-black/70 ">
                  {tt("caseStudy.challengeLabel", "The challenge")}
                </span>
              </div>
              <h2 className="glitch-hover font-display text-2xl uppercase leading-none text-black ">
                {tt("caseStudy.challengeTitle", "What they faced")}
              </h2>
              <p className="mt-3 font-tech text-sm leading-relaxed text-black/80 ">
                {tt(`caseStudiesData.${csIndex}.challenge`, cs.challenge)}
              </p>
            </article>
          </Reveal>

          <Reveal direction="left" as="article">
            <article className="press-lg h-full border-4 border-[#FF5500] bg-[#FBF9F5] p-6 shadow-[6px_6px_0_0_#FF5500] ">
              <div className="mb-3 flex items-center gap-2">
                <Check className="h-5 w-5 text-[#FF5500]" strokeWidth={2.5} />
                <span className="font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-[#FF5500]">
                  {tt("caseStudy.solutionLabel", "The solution")}
                </span>
              </div>
              <h2 className="glitch-hover font-display text-2xl uppercase leading-none text-black ">
                {tt("caseStudy.solutionTitle", "How VantaDB helped")}
              </h2>
              <p className="mt-3 font-tech text-sm leading-relaxed text-black/80 ">
                {tt(`caseStudiesData.${csIndex}.solution`, cs.solution)}
              </p>
            </article>
          </Reveal>
        </div>
      </PageSection>

      {/* Quote */}
      <PageSection variant="paper">
        <Reveal direction="up">
          <figure className="relative mx-auto max-w-4xl border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:p-10">
            <Quote
              className="absolute -left-4 -top-4 h-12 w-12 rotate-[-8deg] border-4 border-black bg-[#FF5500] p-2 text-black shadow-[3px_3px_0_0_#000]  "
              strokeWidth={2.5}
              aria-hidden
            />
            <div
              className="pointer-events-none absolute inset-0 grid-tech opacity-20"
              aria-hidden
            />
            <blockquote className="relative">
              <p className="font-display text-2xl uppercase leading-tight text-[#FBF9F5] sm:text-3xl">
                <span className="text-[#FF5500]" aria-hidden>
                  “
                </span>
                {tt(`caseStudiesData.${csIndex}.quote`, cs.quote)}
                <span className="text-[#FF5500]" aria-hidden>
                  ”
                </span>
              </p>
              <figcaption className="mt-6 flex items-center gap-3 border-t-2 border-[#FBF9F5]/20 pt-4">
                <span className="inline-flex h-8 w-8 items-center justify-center border-2 border-[#FBF9F5] bg-[#FF5500] text-black">
                  <ArrowRight className="h-4 w-4 rotate-0" strokeWidth={2.5} aria-hidden />
                </span>
                <span className="font-tech text-xs uppercase tracking-wider text-[#FBF9F5]/70">
                  {tt(`caseStudiesData.${csIndex}.quoteAuthor`, cs.quoteAuthor)}
                </span>
              </figcaption>
            </blockquote>
          </figure>
        </Reveal>

        <Reveal direction="up" delay={120}>
          <div className="mx-auto mt-10 flex max-w-4xl flex-col items-center justify-between gap-4 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:flex-row">
            <div>
              <h3 className="font-display text-2xl uppercase leading-none text-[#FBF9F5]">
                {tt("caseStudy.ctaTitle", "Build your own story")}
              </h3>
              <p className="mt-1 font-tech text-xs text-[#FBF9F5]/60">
                {tt("caseStudy.ctaBody", "pip install vantadb-py and ship in minutes — no cloud, no servers.")}
              </p>
            </div>
            <button
              onClick={() => router.push("/case-studies")}
              className="press-neon btn-neon-glow inline-flex shrink-0 items-center gap-2 border-4 border-[#FBF9F5] bg-[#FF5500] px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black"
            >
              <ArrowLeft className="h-4 w-4" strokeWidth={2.5} aria-hidden />
              {tt("caseStudy.backToList", "Back to Case Studies")}
            </button>
          </div>
        </Reveal>
      </PageSection>
    </div>
  );
}
