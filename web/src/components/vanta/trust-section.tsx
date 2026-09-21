"use client";

import { Shield, GitBranch, Package, Cpu, Users, Zap, Globe } from "lucide-react";
import { Reveal } from "./reveal";
import { VANTA } from "./vanta-data";
import { useLanguage } from "@/lib/language-provider";
import { CountUpStat } from "@/hooks/count-up";

const TRUST_METRICS = [
  { value: "Apache 2.0", label: "Open Source", sub: "Permissive license", icon: Shield },
  { value: "Rust", label: "Memory Safe", sub: "No GC, no data races", icon: Cpu },
  { value: "Embedded", label: "No Server", sub: "Self-contained engine — no server, no cloud", icon: Package },
  { value: "PyO3", label: "Python Bindings", sub: "Native speed bridge", icon: Zap },
];

const STACK_ITEMS = [
  { name: "Rust Core", desc: "Engine, WAL, HNSW, BM25, Fjall/RocksDB" },
  { name: "PyO3 Bridge", desc: "Stable SDK boundary, zero-copy where possible" },
  { name: "Python Wheels", desc: "Pre-compiled for Windows, macOS, Linux" },
  { name: "Embedded CLI", desc: "vanta-cli: put, list, export, audit, repair" },
];

export function TrustSection() {
  const { t, tt } = useLanguage();
  return (
    <section className="relative border-b-4 border-black bg-[#FBF9F5]  ">
      <div className="pointer-events-none absolute inset-0 halftone opacity-[0.04]" aria-hidden />
      <div className="relative mx-auto max-w-7xl px-4 py-14 sm:px-6">
        <Reveal direction="up">
          <div className="mb-8 flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <h2 className="glitch-hover mt-3 font-display text-4xl uppercase leading-none text-black  sm:text-5xl">
                {t("trust.title")}
              </h2>
              <p className="mt-2 max-w-2xl font-tech text-sm text-black/70 ">
                {t("trust.subtitle")}
              </p>
            </div>
            <div className="flex items-center gap-2 font-tech text-[10px] uppercase tracking-wider text-black/70 ">
              <Globe className="h-3.5 w-3.5" />
              {t("trust.localFirst")}
            </div>
          </div>
        </Reveal>

        <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
          {TRUST_METRICS.map((m, i) => {
            const Icon = m.icon;
            return (
              <Reveal key={m.label} direction="up" delay={i * 80}>
                <article className="group relative flex h-full flex-col border-4 border-black bg-[#FBF9F5] p-5 transition-all hover:-translate-y-1  ">
                  <div className="animate-float mb-3 inline-flex h-10 w-10 items-center justify-center border-4 border-black bg-black text-[#FF5500] shadow-[3px_3px_0_0_#000] transition-transform group-hover:rotate-[8deg]  ">
                    <Icon className="h-5 w-5" strokeWidth={2.5} />
                  </div>
                  <span className="font-display text-2xl uppercase leading-none text-black ">
                    <CountUpStat value={m.value} />
                  </span>
                  <span className="mt-1 font-tech text-xs font-bold uppercase tracking-wider text-black ">
                    {tt(`trust.metrics.${i}.label`, m.label)}
                  </span>
                  <span className="mt-0.5 font-tech text-[10px] uppercase tracking-wider text-[#FF5500]">
                    {tt(`trust.metrics.${i}.sub`, m.sub)}
                  </span>
                  <div className="mt-auto pt-3">
                    <div className="h-1 w-full speed-lines opacity-20" />
                  </div>
                </article>
              </Reveal>
            );
          })}
        </div>

        <Reveal direction="up" delay={100}>
          <div className="mt-8 border-4 border-black bg-black p-6 shadow-[6px_6px_0_0_#000]  ">
            <div className="mb-4 flex items-center gap-3">
              <GitBranch className="h-5 w-5 text-[#FF5500]" strokeWidth={2.5} />
              <h3 className="glitch-hover font-display text-2xl uppercase text-[#FBF9F5]">
                {t("trust.techStack")}
              </h3>
              <span className="ml-auto font-tech text-[9px] uppercase tracking-wider text-[#FBF9F5]/40">
                {VANTA.rustVersion} + {VANTA.pythonVersion}
              </span>
            </div>
            <div className="stagger-children grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
              {STACK_ITEMS.map((item, idx) => (
                <div
                  key={item.name}
                  className="border-2 border-[#FBF9F5]/15 bg-[#1A1A1A] p-4 transition-colors hover:border-[#FF5500]/50"
                >
                  <h4 className="font-tech text-xs font-bold uppercase tracking-wider text-[#FF5500]">
                    {tt(`trust.stack.${idx}.name`, item.name)}
                  </h4>
                  <p className="mt-1 font-tech text-[11px] leading-relaxed text-[#FBF9F5]/60">
                    {tt(`trust.stack.${idx}.desc`, item.desc)}
                  </p>
                </div>
              ))}
            </div>
          </div>
        </Reveal>

        <Reveal direction="up" delay={150}>
          <div className="mt-8 flex flex-wrap items-center justify-center gap-4 font-tech text-[10px] uppercase tracking-wider text-black/70 ">
            <span className="flex items-center gap-1">
              <Users className="h-3 w-3" />
              {t("trust.openSource")}
            </span>
            <span className="flex items-center gap-1">
              <Shield className="h-3 w-3" />
              {VANTA.license}
            </span>
            <span className="flex items-center gap-1">
              <Zap className="h-3 w-3" />
              {t("trust.inProcess")}
            </span>
            <span className="flex items-center gap-1">
              <Globe className="h-3 w-3" />
              {VANTA.rustVersion} · {VANTA.pythonVersion}
            </span>
          </div>
        </Reveal>
      </div>
    </section>
  );
}
