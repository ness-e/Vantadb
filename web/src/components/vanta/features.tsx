"use client";

import {
  Database,
  Search,
  Cpu,
  Layers,
  KeyRound,
  Workflow,
  type LucideIcon,
} from "lucide-react";
import { CORE_CAPABILITIES } from "./vanta-data";
import { Reveal } from "./reveal";
import { useLanguage } from "@/lib/language-provider";

const ICONS: Record<string, LucideIcon> = {
  Database,
  Search,
  Cpu,
  Layers,
  KeyRound,
  Workflow,
};

export function Features() {
  const { t, tt } = useLanguage();
  return (
    <section className="relative border-b-4 border-black bg-[#F2EDE2]  ">
      <div className="pointer-events-none absolute inset-0 grid-tech opacity-50" aria-hidden />
      {/* Section header */}
      <div className="relative mx-auto max-w-7xl px-4 pt-14 sm:px-6">
        <Reveal direction="up">
          <div className="flex flex-col gap-3 border-4 border-black bg-[#FBF9F5] p-6 shadow-[6px_6px_0_0_#000]    sm:flex-row sm:items-end sm:justify-between">
            <div>
              <h2 className="glitch-hover mt-3 font-display text-4xl uppercase leading-none text-black  sm:text-5xl">
                {t("features.title")}
              </h2>
              <p className="mt-2 max-w-2xl font-tech text-sm text-black/70 ">
                {t("features.subtitle")}
              </p>
            </div>
            <div className="flex items-center gap-2 font-tech text-[10px] uppercase tracking-wider text-black/70 ">
              <span className="h-2 w-2 animate-flicker bg-[#FF5500]" />
              {t("features.surfacesInfo")}
            </div>
          </div>
        </Reveal>
      </div>

      {/* Vignette grid */}
      <div className="relative mx-auto max-w-7xl px-4 py-10 sm:px-6">
        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3">
          {CORE_CAPABILITIES.map((cap, i) => {
            const Icon = ICONS[cap.icon] ?? Database;
            return (
              <Reveal key={cap.title} direction="up" delay={i * 60} as="article">
                <article
                  className="press-lg group relative flex h-full flex-col border-4 border-black bg-[#FBF9F5] p-5  "
                >
                {/* Panel number */}
                <span className="absolute -left-2 -top-3 rotate-[-6deg] border-2 border-black bg-black px-2 py-0.5 font-display text-xs uppercase text-[#FF5500]   ">
                  {String(i + 1).padStart(2, "0")}
                </span>

                {/* Icon + tag */}
                <div className="mb-4 flex items-start justify-between">
                  <span className="inline-flex h-12 w-12 items-center justify-center border-4 border-black bg-[#FF5500] text-black shadow-[3px_3px_0_0_#000] transition-transform group-hover:rotate-[-6deg]  ">
                    <Icon className="h-6 w-6" strokeWidth={2.5} />
                  </span>
                  <span className="border-2 border-black bg-[#FBF9F5] px-2 py-0.5 font-tech text-[9px] font-bold uppercase tracking-[0.2em] text-black   ">
                    {cap.tag}
                  </span>
                </div>

                {/* Title */}
                <h3 className="glitch-hover font-display text-2xl uppercase leading-none text-black ">
                  {tt(`features.capabilities.${i}.title`, cap.title)}
                </h3>

                {/* Mechanism (mono, boxed) */}
                <div className="mt-2 border-l-4 border-black bg-black/5 px-3 py-1.5  ">
                  <code className="font-tech text-[11px] font-bold text-black ">
                    {cap.mechanism}
                  </code>
                </div>

                {/* Detail */}
                <p className="mt-3 flex-1 font-tech text-xs leading-relaxed text-black/80 ">
                  {tt(`features.capabilities.${i}.detail`, cap.detail)}
                </p>

                {/* Bottom kinetic line */}
                <div className="mt-4 h-1 w-full speed-lines opacity-30" />
                </article>
              </Reveal>
            );
          })}
        </div>
      </div>
    </section>
  );
}
