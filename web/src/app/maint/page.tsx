"use client";

import {
  Server,
  RefreshCw,
  HardDrive,
  Gauge,
  ArrowRight,
  type LucideIcon,
} from "lucide-react";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { Reveal } from "@/components/vanta/reveal";
import { MAINTENANCE_PILLARS, VANTA } from "@/components/vanta/vanta-data";
import { useLanguage } from "@/lib/language-provider";

const ICONS: Record<string, LucideIcon> = {
  Server,
  RefreshCw,
  HardDrive,
  Gauge,
};

export default function MaintPage() {
  const { t, tt } = useLanguage();
  return (
    <div className="animate-rise">
      <PageHeader
        badge="§MAINT"
        title={tt("maintPage.title", "Zero Maintenance")}
        subtitle={tt("maintPage.subtitle", "There is no database server to operate. No provisioning, no scaling, no patching, no 3am pages. Four pillars that keep operational cost at zero.")}
        tag={tt("maintPage.tag", "Embedded · self-healing")}
      />

      <PageSection variant="cream">
        {/* Intro */}
        <Reveal direction="up">
          <div className="mb-10 border-4 border-black bg-[#F2EDE2] p-6 shadow-[6px_6px_0_0_#000]   ">
            <div className="flex flex-col gap-4 sm:flex-row sm:items-center">
              <Server className="h-10 w-10 shrink-0 text-[#FF5500]" strokeWidth={2.5} />
              <div>
                <h2 className="glitch-hover font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
                  {tt("maintPage.introTitle", "What 'zero maintenance' means")}
                </h2>
                <p className="mt-2 max-w-3xl font-tech text-sm leading-relaxed text-black/80 ">
                  {tt("maintPage.introBody", "VantaDB is a Rust library linked into your process. There is no daemon, no cluster, no control plane, no upgrades to coordinate. Backups are file copies. Recovery is automatic via WAL replay. Indexes repair themselves from canonical records. The engine bounds its own memory. You ship it, you forget it.")}
                </p>
              </div>
            </div>
          </div>
        </Reveal>

        {/* Pillars grid */}
        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2">
          {MAINTENANCE_PILLARS.map((pillar, i) => {
            const Icon = ICONS[pillar.icon] ?? Server;
            return (
              <Reveal key={pillar.title} direction="up" delay={i * 80} as="article">
                <article className="press-lg group relative flex h-full flex-col border-4 border-black bg-[#FBF9F5] p-5  ">
                  <span className="absolute -left-2 -top-3 rotate-[-6deg] border-2 border-black bg-black px-2 py-0.5 font-display text-xs uppercase text-[#FF5500]   ">
                    0{i + 1}
                  </span>

                  <div className="mb-4 flex items-start justify-between">
                    <span className="inline-flex h-12 w-12 items-center justify-center border-4 border-black bg-[#FF5500] text-black shadow-[3px_3px_0_0_#000] transition-transform group-hover:rotate-[-6deg]  ">
                      <Icon className="h-6 w-6" strokeWidth={2.5} />
                    </span>
                    <span className="border-2 border-black bg-[#F2EDE2] px-2 py-0.5 font-tech text-[9px] font-bold uppercase tracking-[0.2em] text-black   ">
                      {tt(`maintPage.pillars.${i}.tag`, pillar.tag)}
                    </span>
                  </div>

                  <h3 className="glitch-hover font-display text-2xl uppercase leading-none text-black ">
                    {tt(`maintPage.pillars.${i}.title`, pillar.title)}
                  </h3>

                  <p className="mt-3 flex-1 font-tech text-xs leading-relaxed text-black/80 ">
                    {tt(`maintPage.pillars.${i}.body`, pillar.body)}
                  </p>

                  <div className="mt-4 h-1 w-full speed-lines opacity-30" />
                </article>
              </Reveal>
            );
          })}
        </div>

        <Reveal direction="up" delay={120}>
          <div className="mt-10 flex flex-col items-center justify-between gap-4 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:flex-row">
            <div>
              <h3 className="font-display text-2xl uppercase leading-none text-[#FBF9F5]">
                {tt("maintPage.ctaTitle", "Ship it, forget it")}
              </h3>
              <p className="mt-1 font-tech text-xs text-[#FBF9F5]/60">
                {tt("maintPage.ctaBody", "No DBA required. No on-call rotation. Just a library that works.")}
              </p>
            </div>
            <a
              href={VANTA.quickstart}
              target="_blank"
              rel="noopener noreferrer"
              className="press-neon btn-neon-glow inline-flex shrink-0 items-center gap-2 border-4 border-[#FBF9F5] bg-[#FF5500] px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black"
            >
              {tt("maintPage.ctaButton", "Read the quickstart")}
              <ArrowRight className="h-4 w-4" strokeWidth={2.5} />
            </a>
          </div>
        </Reveal>
      </PageSection>
    </div>
  );
}
