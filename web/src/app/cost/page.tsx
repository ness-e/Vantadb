"use client";

import { ArrowRight, TrendingDown, Server, Cloud } from "lucide-react";
import Link from "next/link";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { Reveal } from "@/components/vanta/reveal";
import { TCO_COMPARISON, VANTA } from "@/components/vanta/vanta-data";
import { useLanguage } from "@/lib/language-provider";

export default function CostPage() {
  const { t } = useLanguage();
  const tt = (key: string, fallback: string) => {
    const v = t(key);
    return v === key ? fallback : v;
  };
  return (
    <div className="animate-rise">
      <PageHeader
        badge="§COST"
        title={tt("costPage.title", "Total Cost of Ownership")}
        subtitle={tt("costPage.subtitle", "VantaDB runs in-process on hardware you already own. Cloud vector DBs charge for instances, pods, replicas, and egress. The gap widens with scale.")}
        tag={tt("costPage.tag", "$0 forever · self-hosted")}
      />

      <PageSection variant="cream">
        <Reveal direction="up">
          <div className="mb-8 grid grid-cols-1 gap-4 sm:grid-cols-3">
            <div className="border-4 border-black bg-[#FF5500] p-4 shadow-[4px_4px_0_0_#000]  ">
              <Server className="h-6 w-6 text-black" strokeWidth={2.5} />
              <p className="mt-2 font-display text-3xl uppercase leading-none text-black">$0</p>
              <p className="mt-1 font-tech text-[10px] uppercase tracking-wider text-black/70">
                {tt("costPage.statVantadb", "VantaDB at any scale")}
              </p>
            </div>
            <div className="border-4 border-black bg-[#F2EDE2] p-4 shadow-[4px_4px_0_0_#000]   ">
              <Cloud className="h-6 w-6 text-black " strokeWidth={2.5} />
              <p className="mt-2 font-display text-3xl uppercase leading-none text-black ">
                $1,800+
              </p>
              <p className="mt-1 font-tech text-[10px] uppercase tracking-wider text-black/70 ">
                {tt("costPage.statPinecone", "Pinecone at 1M vectors")}
              </p>
            </div>
            <div className="border-4 border-black bg-[#F2EDE2] p-4 shadow-[4px_4px_0_0_#000]   ">
              <TrendingDown className="h-6 w-6 text-[#FF5500]" strokeWidth={2.5} />
              <p className="mt-2 font-display text-3xl uppercase leading-none text-black ">
                ∞
              </p>
              <p className="mt-1 font-tech text-[10px] uppercase tracking-wider text-black/70 ">
                {tt("costPage.statEgress", "Egress fees avoided")}
              </p>
            </div>
          </div>
        </Reveal>

        <Reveal direction="up" delay={80}>
          <div className="mb-8 flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <span className="inline-flex items-center gap-2 border-2 border-black bg-black px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-[#FF5500] ">
                <span className="h-1.5 w-1.5 bg-[#FF5500]" />
                {tt("costPage.scenariosTag", "Scenarios")}
              </span>
              <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
                {tt("costPage.scenariosTitle", "Cost across scales")}
              </h2>
              <p className="mt-2 max-w-2xl font-tech text-xs text-black/70 ">
                {tt("costPage.scenariosSubtitle", "Illustrative monthly costs. VantaDB column highlighted — it stays at zero because the engine is embedded and uses your existing CPU/RAM.")}
              </p>
            </div>
          </div>
        </Reveal>

        <Reveal direction="up" delay={120}>
          <div className="overflow-x-auto border-4 border-black bg-[#FBF9F5] shadow-[6px_6px_0_0_#000]   ">
            <table className="w-full min-w-[760px] border-collapse font-tech text-xs">
              <thead>
                <tr className="border-b-4 border-black ">
                  <th className="px-3 py-3 text-left font-bold uppercase tracking-wider text-black ">
                    {tt("costPage.thScenario", "Scenario")}
                  </th>
                  <th className="border-l-4 border-[#FF5500] bg-[#FF5500] px-3 py-3 text-left font-bold uppercase tracking-wider text-black">
                    {tt("costPage.thVantadb", "VantaDB")}
                  </th>
                  <th className="border-l-2 border-black px-3 py-3 text-left font-bold uppercase tracking-wider text-black/70  ">
                    {tt("costPage.thPinecone", "Pinecone")}
                  </th>
                  <th className="border-l-2 border-black px-3 py-3 text-left font-bold uppercase tracking-wider text-black/70  ">
                    {tt("costPage.thWeaviate", "Weaviate Cloud")}
                  </th>
                  <th className="border-l-2 border-black px-3 py-3 text-left font-bold uppercase tracking-wider text-black/70  ">
                    {tt("costPage.thNote", "Note")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {TCO_COMPARISON.map((row, i) => (
                  <tr
                    key={row.scenario}
                    className={i % 2 === 0 ? "bg-[#F2EDE2]/50 " : ""}
                  >
                    <td className="px-3 py-3 font-bold uppercase tracking-wider text-black ">
                      {tt(`costPage.tco.${i}.scenario`, row.scenario)}
                    </td>
                    <td className="border-l-4 border-[#FF5500] bg-[#FF5500]/10 px-3 py-3 font-bold text-black ">
                      {row.vantadb}
                    </td>
                    <td className="border-l-2 border-black px-3 py-3 text-black/70  ">
                      {row.cloudPinecone}
                    </td>
                    <td className="border-l-2 border-black px-3 py-3 text-black/70  ">
                      {row.cloudWeaviate}
                    </td>
                    <td className="border-l-2 border-black px-3 py-3 text-black/60  ">
                      {tt(`costPage.tco.${i}.note`, row.note)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Reveal>

        <Reveal direction="up" delay={150}>
          <div className="mt-8 border-4 border-dashed border-black/30 p-5 text-center ">
            <p className="font-tech text-xs leading-relaxed text-black/60 ">
              {tt("costPage.note", "Cost figures are illustrative, drawn from public cloud vector DB pricing as of 2025. VantaDB Enterprise adds support and compliance artifacts — see /pricing.")}
            </p>
          </div>
        </Reveal>

        <Reveal direction="up" delay={180}>
          <div className="mt-8 flex flex-col items-center justify-between gap-4 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:flex-row">
            <div>
              <h3 className="font-display text-2xl uppercase leading-none text-[#FBF9F5]">
                {tt("costPage.ctaTitle", "Stop paying for vectors")}
              </h3>
              <p className="mt-1 font-tech text-xs text-[#FBF9F5]/60">
                {tt("costPage.ctaBody", "pip install vantadb-py and own your retrieval layer forever.")}
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
