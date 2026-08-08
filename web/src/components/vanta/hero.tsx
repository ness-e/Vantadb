"use client";

import { Github, Terminal, Zap, ArrowDown, Copy, Check, ChevronRight } from "lucide-react";
import { useState } from "react";
import { VANTA, HERO_STATS } from "./vanta-data";
import type { View } from "./vanta-data";
import { CountUpStat } from "@/hooks/count-up";
import { Mark } from "./mark";
import { copyToClipboard } from "./copy-utils";
import { toast } from "./toast";
import { cn } from "@/lib/utils";
import { useLanguage } from "@/lib/language-provider";

export function Hero({ onNavigate }: { onNavigate: (v: View) => void }) {
  const [copied, setCopied] = useState(false);
  const { t } = useLanguage();
  const tt = (key: string, fallback: string) => {
    const v = t(key);
    return v === key ? fallback : v;
  };

  const copyInstall = async () => {
    const ok = await copyToClipboard("pip install vantadb-py");
    if (ok) {
      setCopied(true);
      toast.copy("pip install vantadb-py");
      setTimeout(() => setCopied(false), 1600);
    }
  };

  return (
    <section
      aria-label={tt("hero.ariaLabel", "VantaDB — motor Rust embebido para memoria local durable y retrieval híbrido")}
      className="relative overflow-hidden border-b-4 border-black bg-[#FBF9F5]"
    >
      {/* Background layers: grid + halftone + speed lines */}
      <div className="pointer-events-none absolute inset-0 grid-tech" aria-hidden />
      <div
        className="pointer-events-none absolute -right-24 -top-24 h-[520px] w-[520px] halftone halftone-fade opacity-30"
        aria-hidden
      />
      <div
        className="pointer-events-none absolute bottom-0 left-0 h-40 w-full speed-lines opacity-[0.06]"
        aria-hidden
      />

      {/* Corner registration marks (technical drafting) */}
      <RegMark className="left-3 top-3" />
      <RegMark className="right-3 top-3" />
      <RegMark className="bottom-3 left-3" />
      <RegMark className="bottom-3 right-3" />

      <div className="relative mx-auto grid max-w-7xl grid-cols-1 items-center gap-8 px-4 py-12 sm:px-6 lg:grid-cols-12 lg:gap-6 lg:py-20">
        {/* LEFT — text hierarchy */}
        <div className="order-2 lg:order-1 lg:col-span-7">
          {/* Stamp badge */}
          <div className="mb-6 flex flex-wrap items-center gap-3">
            <span className="animate-stamp inline-flex rotate-[-6deg] items-center border-4 border-black bg-[#FF5500] px-3 py-1 font-display text-xs uppercase tracking-wider text-black shadow-[4px_4px_0_0_#000]">
              0.5.0 · MVP
            </span>
            <span className="inline-flex items-center gap-2 border-2 border-black bg-black px-3 py-1 font-tech text-[10px] font-bold uppercase tracking-[0.2em] text-[#FBF9F5]">
              <span className="h-2 w-2 animate-flicker bg-[#FF5500]" />
              Embedded · Local-First · Rust
            </span>
          </div>

          {/* H1 */}
          <h1 className="font-display text-stencil uppercase leading-[0.82] text-black ">
            <span className="glitch-hover block text-[15vw] sm:text-[12vw] lg:text-[9.5vw] xl:text-[140px]">
              Vanta
            </span>
            <span className="glitch-hover block text-[15vw] sm:text-[12vw] lg:text-[9.5vw] xl:text-[140px]">
              <span className="text-outline-neon glow-neon">DB</span>
            </span>
          </h1>

          {/* Subhead */}
          <p className="mt-5 max-w-xl border-l-4 border-[#FF5500] pl-4 font-tech text-sm leading-relaxed text-black sm:text-base">
            {tt("hero.tagline", VANTA.tagline)}
            <br />
            <span className="font-bold">
              {tt("hero.subheadStrong", "Persistent memory, crash-safe WAL recovery, and native hybrid search")}
            </span>{" "}
            {tt("hero.subheadRest", "— without external services, containers, or network dependencies.")}
          </p>

          {/* Install line */}
          <div className="mt-6 flex flex-wrap items-center gap-3">
            <button
              onClick={copyInstall}
              className="group inline-flex items-center gap-3 border-4 border-black bg-black px-4 py-3 font-tech text-sm font-bold text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500,6px_6px_0_2px_#000] transition-all hover:translate-x-[2px] hover:translate-y-[2px] hover:shadow-[4px_4px_0_0_#FF5500,4px_4px_0_2px_#000] active:translate-x-[6px] active:translate-y-[6px] active:shadow-none"
              title={tt("hero.installTitle", "Copy install command")}
            >
              <span className="text-[#FF5500]">$</span>
              <span>pip install vantadb-py</span>
              {copied ? (
                <Check className="h-4 w-4 text-[#FF5500]" />
              ) : (
                <Copy className="h-4 w-4 opacity-60 transition-opacity group-hover:opacity-100" />
              )}
            </button>
            <span className="font-tech text-[11px] uppercase tracking-wider text-black/50">
              Python {VANTA.pythonVersion} · Rust {VANTA.rustVersion}
            </span>
          </div>

          {/* CTA buttons */}
          <div className="mt-5 flex flex-wrap items-center gap-3">
            <button
              onClick={() => onNavigate("docs")}
              className="btn-neon-glow inline-flex items-center gap-2 border-4 border-black bg-[#FF5500] px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black"
            >
              <Terminal className="h-4 w-4" strokeWidth={2.5} />
              {tt("hero.ctaQuickstart", "5-Minute Quickstart")}
            </button>
            <button
              onClick={() => onNavigate("benchmarks")}
              className="btn-neon-glow inline-flex items-center gap-2 border-4 border-black bg-[#FBF9F5] px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black"
            >
              <Zap className="h-4 w-4" strokeWidth={2.5} />
              {tt("hero.ctaBenchmarks", "View Benchmarks")}
            </button>
            <a
              href={VANTA.repo}
              target="_blank"
              rel="noopener noreferrer"
              className="btn-neon-glow inline-flex items-center gap-2 border-4 border-black bg-[#FBF9F5] px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black"
            >
              <Github className="h-4 w-4" strokeWidth={2.5} />
              {tt("hero.ctaSource", "Source")}
            </a>
          </div>

          {/* Stats strip */}
          <div className="mt-8 grid grid-cols-2 gap-0 border-4 border-black shadow-[6px_6px_0_0_#000] sm:grid-cols-4">
            {HERO_STATS.map((s, i) => (
              <div
                key={s.label}
                className={cn(
                  "flex flex-col gap-0.5 bg-[#FBF9F5] p-3",
                  i < HERO_STATS.length - 1 && "border-r-2 border-b-2 border-black sm:border-b-0",
                  i % 2 === 0 && "sm:border-r-2",
                  i === 2 && "sm:border-l-0"
                )}
              >
                <span className="font-display text-3xl leading-none text-black  sm:text-4xl">
                  <CountUpStat value={s.value} />
                </span>
                <span className="font-tech text-[10px] font-bold uppercase tracking-wider text-black">
                  {tt(`hero.stats.${i}.label`, s.label)}
                </span>
                <span className="font-tech text-[9px] uppercase tracking-wider text-[#FF5500]">
                  {tt(`hero.stats.${i}.sub`, s.sub)}
                </span>
              </div>
            ))}
          </div>
        </div>

        {/* RIGHT — interactive mark (default) */}
        <div className="order-1 lg:order-2 lg:col-span-5">
          {/* Interactive mark */}
          <Mark variant="classic" />

          {/* Caption under mark */}
          <p className="mt-4 text-center font-tech text-[11px] uppercase tracking-[0.15em] text-black/60">
            {tt("hero.captionMark", "Interactive mark · move your mouse · BM25 + HNSW via RRF")}
          </p>
        </div>
      </div>

      {/* Scroll cue */}
      <div className="relative mx-auto flex max-w-7xl items-center justify-center pb-6">
        <button
          onClick={() => onNavigate("docs")}
          className="group inline-flex flex-col items-center gap-1 font-tech text-[10px] uppercase tracking-[0.3em] text-black/60 transition-colors hover:text-black"
        >
          <span className="flex items-center gap-1">
            {tt("hero.scrollCue", "Scroll · Core Capabilities")}
            <ChevronRight className="h-3 w-3 rotate-90" />
          </span>
          <ArrowDown className="h-4 w-4 animate-bounce" />
        </button>
      </div>
    </section>
  );
}

function RegMark({ className }: { className?: string }) {
  return (
    <div
      className={cn(
        "pointer-events-none absolute z-30 hidden h-6 w-6 items-center justify-center lg:flex",
        className
      )}
      aria-hidden
    >
      <div className="absolute h-px w-6 bg-black/40" />
      <div className="absolute h-6 w-px bg-black/40" />
      <div className="h-2 w-2 rounded-full border border-black/40" />
    </div>
  );
}
