"use client";

import { ArrowRight, Building2, Factory } from "lucide-react";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { Reveal } from "@/components/vanta/reveal";
import { CASE_STUDIES } from "@/components/vanta/vanta-data";
import { useLanguage } from "@/lib/language-provider";

export default function CaseStudiesPage() {
  const { tt } = useLanguage();
  return (
    <div className="animate-rise">
      <PageHeader
        badge="§CASE-STUDIES"
        title={tt("caseStudiesPage.title", "Case Studies")}
        subtitle={tt("caseStudiesPage.subtitle", "Three illustrative composite scenarios of VantaDB — agents with durable memory, air-gapped edge RAG, and in-process semantic code search. No servers, no clouds, no caveats.")}
        tag={tt("caseStudiesPage.tag", "3 deployments · Apache 2.0")}
      />

      <PageSection variant="cream">
        <Reveal direction="up">
          <div className="mb-8 flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <span className="inline-flex items-center gap-2 border-2 border-black bg-black px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-[#FF5500] ">
                <span className="h-1.5 w-1.5 bg-[#FF5500]" />
                {tt("caseStudiesPage.deploymentsTag", "Deployments")}
              </span>
              <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
                {tt("caseStudiesPage.deploymentsTitle", "In the wild")}
              </h2>
            </div>
          </div>
        </Reveal>

        <div className="grid grid-cols-1 gap-5 lg:grid-cols-3">
          {CASE_STUDIES.map((cs, i) => (
            <Reveal key={cs.slug} direction="up" delay={i * 80} as="article">
              <a
                href={`/case-studies/${cs.slug}`}
                className="press-lg group relative flex h-full flex-col border-4 border-black bg-[#F2EDE2] p-5 shadow-[6px_6px_0_0_#000] transition-transform hover:translate-x-[-3px] hover:translate-y-[-3px] hover:shadow-[9px_9px_0_0_#FF5500]   "
              >
                <span className="absolute -left-2 -top-3 rotate-[-6deg] border-2 border-black bg-black px-2 py-0.5 font-display text-xs uppercase text-[#FF5500]   ">
                  0{i + 1}
                </span>

                <div className="mb-4 flex items-center gap-3">
                  <span className="inline-flex h-12 w-12 items-center justify-center border-4 border-black bg-[#FF5500] text-black shadow-[3px_3px_0_0_#000] transition-transform group-hover:rotate-[-6deg]  ">
                    <Building2 className="h-6 w-6" strokeWidth={2.5} />
                  </span>
                  <div>
                    <span className="font-tech text-[10px] font-bold uppercase tracking-wider text-black/70 ">
                      {cs.company}
                    </span>
                    <span className="flex items-center gap-1 font-tech text-[10px] uppercase tracking-wider text-[#FF5500]">
                      <Factory className="h-3 w-3" strokeWidth={2.5} aria-hidden />
                      {tt(`caseStudiesData.${i}.industry`, cs.industry)}
                    </span>
                  </div>
                </div>

                <h3 className="glitch-hover font-display text-2xl uppercase leading-none text-black ">
                  {tt(`caseStudiesData.${i}.title`, cs.title)}
                </h3>

                <p className="mt-3 flex-1 font-tech text-xs leading-relaxed text-black/80 ">
                  {tt(`caseStudiesData.${i}.summary`, cs.summary)}
                </p>

                {/* Metrics preview */}
                <div className="mt-5 grid grid-cols-3 gap-2 border-t-4 border-black pt-4 ">
                  {cs.metrics.map((m, idx) => (
                    <div key={idx} className="text-center">
                      <span className="font-display text-lg uppercase leading-none text-[#FF5500]">
                        {m.value}
                      </span>
                      <p className="mt-1 font-tech text-[8px] uppercase leading-tight tracking-wider text-black/60 ">
                        {tt(`caseStudiesData.${i}.metricLabel.${idx}`, m.label)}
                      </p>
                    </div>
                  ))}
                </div>

                <div className="mt-5 flex items-center justify-between border-t-2 border-dashed border-black/30 pt-3 ">
                  <span className="font-tech text-[10px] font-bold uppercase tracking-[0.2em] text-black/60 ">
                    {tt("caseStudiesPage.readCaseStudy", "Read case study")}
                  </span>
                  <span className="inline-flex h-9 w-9 items-center justify-center border-2 border-black bg-[#FF5500] text-black transition-transform group-hover:translate-x-1 ">
                    <ArrowRight className="h-4 w-4" strokeWidth={2.5} aria-hidden />
                  </span>
                </div>

                <div className="mt-4 h-1 w-full speed-lines opacity-30" aria-hidden />
              </a>
            </Reveal>
          ))}
        </div>

        <Reveal direction="up" delay={120}>
          <div className="mt-10 border-4 border-dashed border-black/30 p-5 text-center ">
            <p className="font-tech text-xs leading-relaxed text-black/60 ">
              {tt("caseStudiesPage.note", "Composite scenarios based on typical usage patterns — not specific customers. Companies, quotes, and metrics are illustrative. Want to share your real VantaDB story? Join the Discord.")}
            </p>
          </div>
        </Reveal>
      </PageSection>
    </div>
  );
}
