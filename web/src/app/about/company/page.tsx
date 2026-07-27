"use client";

import { ArrowRight, Github, Target, Compass, Shield, Unlock } from "lucide-react";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { Reveal } from "@/components/vanta/reveal";
import { COMPANY_INFO } from "@/components/vanta/vanta-data";
import { type LucideIcon } from "lucide-react";
import { useLanguage } from "@/lib/language-provider";

const PRINCIPLE_ICONS: LucideIcon[] = [Target, Compass, Shield, Unlock];

export default function CompanyPage() {
  const { t } = useLanguage();
  const tt = (key: string, fallback: string) => {
    const v = t(key);
    return v === key ? fallback : v;
  };
  return (
    <div className="animate-rise">
      <PageHeader
        badge="§ABOUT"
        title={tt("aboutCompanyPage.title", "Company")}
        subtitle={tt("aboutCompanyPage.subtitle", "VantaDB is built by a distributed open-source team. No VC, no servers, no telemetry — just an embedded Rust engine and a license that lets you fork it tomorrow.")}
        tag={`${COMPANY_INFO.founded} · ${COMPANY_INFO.location}`}
      />

      {/* Mission */}
      <PageSection variant="cream">
        <Reveal direction="up">
          <div className="border-4 border-black bg-[#F2EDE2] p-6 shadow-[6px_6px_0_0_#000]    sm:p-8">
            <div className="flex flex-col gap-4 sm:flex-row sm:items-start">
              <span className="inline-flex h-14 w-14 shrink-0 items-center justify-center border-4 border-black bg-[#FF5500] text-black shadow-[3px_3px_0_0_#000]  ">
                <Target className="h-7 w-7" strokeWidth={2.5} />
              </span>
              <div>
                <span className="inline-flex items-center gap-2 border-2 border-black bg-black px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-[#FF5500] ">
                  {tt("aboutCompany.missionTag", "Mission")}
                </span>
                <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
                  {tt("aboutCompany.missionTitle", "Make local-first the default")}
                </h2>
                <p className="mt-3 max-w-3xl font-tech text-sm leading-relaxed text-black/80 ">
                  {tt("aboutCompany.mission", COMPANY_INFO.mission)}
                </p>
              </div>
            </div>
          </div>
        </Reveal>

        {/* Stats grid */}
        <div className="mt-6 grid grid-cols-2 gap-4 lg:grid-cols-4">
          {COMPANY_INFO.stats.map((stat, i) => (
            <Reveal key={i} direction="up" delay={i * 60} as="article">
              <article className="press h-full border-4 border-black bg-[#FBF9F5] p-4  ">
                <span className="font-display text-2xl uppercase leading-none text-black  sm:text-3xl">
                  {stat.value}
                </span>
                <p className="mt-2 font-tech text-[10px] uppercase tracking-wider text-black/60 ">
                  {tt(`aboutCompany.stats.${i}.label`, stat.label)}
                </p>
                <div className="mt-3 h-1 w-full speed-lines opacity-30" aria-hidden />
              </article>
            </Reveal>
          ))}
        </div>
      </PageSection>

      {/* Principles */}
      <PageSection variant="paper">
        <Reveal direction="up">
          <div className="mb-8">
            <span className="inline-flex items-center gap-2 border-2 border-black bg-black px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-[#FF5500] ">
              <span className="h-1.5 w-1.5 bg-[#FF5500]" />
              {tt("aboutCompany.principlesTag", "Principles")}
            </span>
            <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
              {tt("aboutCompany.principlesTitle", "Four things we don't compromise on")}
            </h2>
          </div>
        </Reveal>

        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2">
          {COMPANY_INFO.principles.map((p, i) => {
            const Icon = PRINCIPLE_ICONS[i % PRINCIPLE_ICONS.length];
            return (
              <Reveal key={i} direction="up" delay={i * 80} as="article">
                <article className="press-lg group relative flex h-full flex-col border-4 border-black bg-[#FBF9F5] p-5  ">
                  <span className="absolute -left-2 -top-3 rotate-[-6deg] border-2 border-black bg-black px-2 py-0.5 font-display text-xs uppercase text-[#FF5500]   ">
                    0{i + 1}
                  </span>
                  <div className="mb-4">
                    <span className="inline-flex h-12 w-12 items-center justify-center border-4 border-black bg-[#FF5500] text-black shadow-[3px_3px_0_0_#000] transition-transform group-hover:rotate-[-6deg]  ">
                      <Icon className="h-6 w-6" strokeWidth={2.5} />
                    </span>
                  </div>
                  <h3 className="glitch-hover font-display text-xl uppercase leading-none text-black  sm:text-2xl">
                    {tt(`aboutCompany.principles.${i}.title`, p.title)}
                  </h3>
                  <p className="mt-3 flex-1 font-tech text-xs leading-relaxed text-black/80 ">
                    {tt(`aboutCompany.principles.${i}.body`, p.body)}
                  </p>
                  <div className="mt-4 h-1 w-full speed-lines opacity-30" aria-hidden />
                </article>
              </Reveal>
            );
          })}
        </div>

        {/* Repo CTA */}
        <Reveal direction="up" delay={120}>
          <div className="mt-10 flex flex-col items-center justify-between gap-4 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:flex-row">
            <div>
              <h3 className="font-display text-2xl uppercase leading-none text-[#FBF9F5]">
                {tt("aboutCompany.ctaTitle", "Read the source")}
              </h3>
              <p className="mt-1 font-tech text-xs text-[#FBF9F5]/60">
                {tt("aboutCompany.ctaBody", "Apache 2.0 · no CLA · no telemetry · audit every line.")}
              </p>
            </div>
            <a
              href={COMPANY_INFO.repo}
              target="_blank"
              rel="noopener noreferrer"
              className="press-neon btn-neon-glow inline-flex shrink-0 items-center gap-2 border-4 border-[#FBF9F5] bg-[#FF5500] px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black"
            >
              <Github className="h-4 w-4" strokeWidth={2.5} aria-hidden />
              {tt("aboutCompany.viewOnGithub", "View on GitHub")}
              <ArrowRight className="h-4 w-4" strokeWidth={2.5} aria-hidden />
            </a>
          </div>
        </Reveal>
      </PageSection>
    </div>
  );
}
