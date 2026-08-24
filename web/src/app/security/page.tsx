"use client";

import {
  ShieldCheck,
  Lock,
  FileStack,
  Package,
  ScanSearch,
  Eye,
  ArrowRight,
  type LucideIcon,
} from "lucide-react";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { Reveal } from "@/components/vanta/reveal";
import { SECURITY_PILLARS } from "@/components/vanta/vanta-data";
import { useLanguage } from "@/lib/language-provider";

const ICONS: Record<string, LucideIcon> = {
  ShieldCheck,
  Lock,
  FileStack,
  Package,
  ScanSearch,
  Eye,
};

export default function SecurityPage() {
  const { t, tt } = useLanguage();
  return (
    <div className="animate-rise">
      <PageHeader
        badge="§SECURITY"
        title={tt("securityPage.title", "Security")}
        subtitle={tt("securityPage.subtitle", "Six pillars that make VantaDB safe to ship in production. Crash-safe WAL, zero network surface, canonical records, memory-safe Rust, supply-chain hygiene, and zero telemetry.")}
        tag={tt("securityPage.tag", "Defense in depth · Local-first")}
      />

      <PageSection variant="cream">
        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3">
          {SECURITY_PILLARS.map((pillar, i) => {
            const Icon = ICONS[pillar.icon] ?? ShieldCheck;
            return (
              <Reveal key={pillar.title} direction="up" delay={i * 70} as="article">
                <article className="press-lg group relative flex h-full flex-col border-4 border-black bg-[#F2EDE2] p-5  ">
                  <span className="absolute -left-2 -top-3 rotate-[-6deg] border-2 border-black bg-black px-2 py-0.5 font-display text-xs uppercase text-[#FF5500]   ">
                    0{i + 1}
                  </span>

                  <div className="mb-4 flex items-start justify-between">
                    <span className="inline-flex h-12 w-12 items-center justify-center border-4 border-black bg-[#FF5500] text-black shadow-[3px_3px_0_0_#000] transition-transform group-hover:rotate-[-6deg]  ">
                      <Icon className="h-6 w-6" strokeWidth={2.5} />
                    </span>
                    <span className="border-2 border-black bg-[#FBF9F5] px-2 py-0.5 font-tech text-[9px] font-bold uppercase tracking-[0.2em] text-black   ">
                      {tt(`securityPage.pillars.${i}.tag`, pillar.tag)}
                    </span>
                  </div>

                  <h3 className="glitch-hover font-display text-2xl uppercase leading-none text-black ">
                    {tt(`securityPage.pillars.${i}.title`, pillar.title)}
                  </h3>

                  <p className="mt-3 flex-1 font-tech text-xs leading-relaxed text-black/80 ">
                    {tt(`securityPage.pillars.${i}.body`, pillar.body)}
                  </p>

                  <div className="mt-4 h-1 w-full speed-lines opacity-30" />
                </article>
              </Reveal>
            );
          })}
        </div>

        <Reveal direction="up" delay={120}>
          <div className="mt-10 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500] ">
            <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
              <div className="flex items-center gap-3">
                <ShieldCheck className="h-8 w-8 shrink-0 text-[#FF5500]" strokeWidth={2.5} />
                <div>
                  <h3 className="font-display text-2xl uppercase leading-none text-[#FBF9F5]">
                    {tt("securityPage.verifyTitle", "Verify it yourself")}
                  </h3>
                  <p className="mt-1 font-tech text-xs text-[#FBF9F5]/60">
                    {tt("securityPage.verifyBody", "The source is open. Every line of the Rust core, the WAL replay logic, and the PyO3 bindings is auditable.")}
                  </p>
                </div>
              </div>
              <a
                href="https://github.com/ness-e/Vantadb"
                target="_blank"
                rel="noopener noreferrer"
                className="press-neon btn-neon-glow inline-flex shrink-0 items-center gap-2 border-4 border-[#FBF9F5] bg-[#FF5500] px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black"
              >
                {tt("securityPage.readSource", "Read the source")}
                <ArrowRight className="h-4 w-4" strokeWidth={2.5} />
              </a>
            </div>
          </div>
        </Reveal>
      </PageSection>
    </div>
  );
}
