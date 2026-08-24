"use client";

import { useEffect, useRef, useState } from "react";
import { Play, RotateCcw, Flag, Zap } from "lucide-react";
import { Reveal } from "./reveal";
import { useLanguage } from "@/lib/language-provider";

interface Bar {
  label: string;
  value: string;
  pct: number; // final width % (lower latency = wider for latency; higher recall = wider for recall)
  durationMs: number; // animation duration — slower pct means longer time
  vantadb?: boolean;
  rank: number;
}

interface Group {
  id: "latency" | "recall";
  titleKey: string;
  titleFallback: string;
  bars: Bar[];
}

export function BenchmarkRace() {
  const { tt } = useLanguage();

  const groups: Group[] = [
    {
      id: "latency",
      titleKey: "benchmarkRace.groups.0.title",
      titleFallback: "HNSW Vector Search · p50 Latency (ms)",
      bars: [
        { label: "VantaDB", value: "1.2ms", pct: 100, durationMs: 700, vantadb: true, rank: 1 },
        { label: "Qdrant", value: "—", pct: 80, durationMs: 1100, rank: 2 },
        { label: "Chroma", value: "—", pct: 65, durationMs: 1500, rank: 3 },
        { label: "Weaviate", value: "—", pct: 40, durationMs: 2200, rank: 4 },
        { label: "Pinecone", value: "—", pct: 20, durationMs: 3000, rank: 5 },
      ],
    },
    {
      id: "recall",
      titleKey: "benchmarkRace.groups.1.title",
      titleFallback: "Recall@10 (higher is better)",
      bars: [
        { label: "VantaDB", value: "0.998", pct: 99, durationMs: 800, vantadb: true, rank: 1 },
        { label: "Chroma", value: "—", pct: 94, durationMs: 1200, rank: 2 },
        { label: "Qdrant", value: "—", pct: 92, durationMs: 1600, rank: 3 },
        { label: "Weaviate", value: "—", pct: 89, durationMs: 2000, rank: 4 },
        { label: "Pinecone", value: "—", pct: 87, durationMs: 2500, rank: 5 },
      ],
    },
  ];

  const [running, setRunning] = useState(false);
  const [finished, setFinished] = useState(false);
  const sectionRef = useRef<HTMLElement>(null);
  const [inView, setInView] = useState(false);
  const timersRef = useRef<number[]>([]);

  useEffect(() => {
    const el = sectionRef.current;
    if (!el) return;
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setInView(true);
          observer.disconnect();
        }
      },
      { threshold: 0.15 }
    );
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  // Cleanup pending timers on unmount
  useEffect(() => {
    return () => {
      timersRef.current.forEach((id) => window.clearTimeout(id));
    };
  }, []);

  const start = () => {
    if (running) return;
    setRunning(true);
    setFinished(false);
    // Reset bars to 0 by toggling a key first
    const maxDuration = Math.max(...groups.flatMap((g) => g.bars.map((b) => b.durationMs)));
    const t = window.setTimeout(() => {
      setRunning(false);
      setFinished(true);
    }, maxDuration + 200);
    timersRef.current.push(t);
  };

  const reset = () => {
    timersRef.current.forEach((id) => window.clearTimeout(id));
    timersRef.current = [];
    setRunning(false);
    setFinished(false);
  };

  // Bars render at their target width only when `running` or `finished` is true.
  // Otherwise they sit at 0%.
  const showBars = running || finished;

  const podium = groups[0].bars.slice(0, 3); // top 3 from latency race

  return (
    <section
      ref={sectionRef}
      aria-label={tt("benchmarkRace.tagHeader", "Benchmarks · Live Race")}
      className="relative border-b-4 border-black bg-[#FBF9F5]  "
    >
      <div className="pointer-events-none absolute inset-0 grid-tech opacity-40" aria-hidden />
      <div className="relative mx-auto max-w-7xl px-4 py-14 sm:px-6">
        <Reveal direction="up">
          <div className="mb-8 flex flex-col gap-3 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:flex-row sm:items-end sm:justify-between">
            <div>
              <span className="inline-flex items-center gap-2 border-2 border-[#FF5500] bg-[#FF5500] px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-black">
                <span className="h-1.5 w-1.5 bg-black" />
                {tt("benchmarkRace.tagHeader", "Benchmarks · Live Race")}
              </span>
              <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-[0.9] sm:text-4xl lg:text-5xl">
                {tt("benchmarkRace.title", "VantaDB vs the field.")}
              </h2>
              <p className="mt-3 max-w-2xl font-tech text-xs text-[#FBF9F5]/70 sm:text-sm">
                {tt("benchmarkRace.subtitle", "Visualized p50 latency race. VantaDB finishes first — not by marketing, by architecture. Press Start and watch.")}
              </p>
            </div>
            <span className="flex items-center gap-2 font-tech text-[10px] uppercase tracking-wider text-[#FF5500]">
              <span className="h-2 w-2 animate-flicker bg-[#FF5500]" />
              {tt("benchmarkRace.tag", "Head-to-head · live race")}
            </span>
          </div>
        </Reveal>

        {/* Controls */}
        <Reveal direction="up" delay={60}>
          <div className="mb-6 flex flex-wrap items-center gap-3">
            <button
              type="button"
              onClick={start}
              disabled={running}
              className="press-lg inline-flex items-center gap-2 border-4 border-black bg-[#FF5500] px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black shadow-[4px_4px_0_0_#000] transition-opacity disabled:cursor-not-allowed disabled:opacity-50  "
            >
              <Play className="h-4 w-4" strokeWidth={2.5} />
              {tt("benchmarkRace.startBtn", "Start race")}
            </button>
            <button
              type="button"
              onClick={reset}
              disabled={running}
              className="press-lg inline-flex items-center gap-2 border-4 border-black bg-[#F2EDE2] px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black transition-opacity disabled:cursor-not-allowed disabled:opacity-50   "
            >
              <RotateCcw className="h-4 w-4" strokeWidth={2.5} />
              {tt("benchmarkRace.resetBtn", "Reset")}
            </button>
            {(inView && !running && !finished) && (
              <span className="font-tech text-[10px] uppercase tracking-wider text-black/50 ">
                ▸ Press Start
              </span>
            )}
            {running && (
              <span className="flex items-center gap-2 font-tech text-[10px] uppercase tracking-wider text-[#FF5500]">
                <span className="h-2 w-2 animate-flicker bg-[#FF5500]" />
                Racing…
              </span>
            )}
            {finished && (
              <span className="flex items-center gap-2 font-tech text-[10px] font-bold uppercase tracking-wider text-[#FF5500]">
                <Flag className="h-3 w-3" strokeWidth={3} />
                VantaDB wins.
              </span>
            )}
          </div>
        </Reveal>

        {/* Podium */}
        <Reveal direction="up" delay={100}>
          <div className="mb-6 border-4 border-black bg-[#F2EDE2] p-4 shadow-[6px_6px_0_0_#000]    sm:p-5">
            <div className="flex items-center justify-between gap-3">
              <span className="font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-black/60 ">
                {tt("benchmarkRace.podiumTitle", "Standings")}
              </span>
              <div className="flex items-end gap-3">
                {podium.map((p, i) => (
                  <div
                    key={p.label}
                    className={`flex flex-col items-center gap-1 border-2 px-3 py-2 ${
                      i === 0
                        ? "border-4 border-black bg-[#FF5500] text-black shadow-[3px_3px_0_0_#000]  "
                        : "border-black/30 bg-[#FBF9F5] text-black/70   "
                    }`}
                  >
                    <span className="font-display text-xl leading-none">{i + 1}</span>
                    <span className="font-tech text-[10px] font-bold uppercase tracking-wider">{p.label}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </Reveal>

        {/* Race charts */}
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          {groups.map((group, gi) => (
            <Reveal key={group.id} direction="up" delay={120 + gi * 80}>
              <div className="border-4 border-black bg-[#F2EDE2] p-5 shadow-[6px_6px_0_0_#000]   ">
                <div className="mb-4 flex items-center gap-2">
                  <span className="h-2 w-2 bg-[#FF5500]" />
                  <h3 className="font-tech text-xs font-bold uppercase tracking-wider text-black ">
                    {tt(group.titleKey, group.titleFallback)}
                  </h3>
                </div>
                <div className="flex flex-col gap-3">
                  {group.bars.map((bar) => {
                    const isVanta = !!bar.vantadb;
                    return (
                      <div key={bar.label} className="flex items-center gap-3">
                        <div className="w-20 shrink-0 font-tech text-[10px] font-bold uppercase tracking-wider text-black/70  sm:w-24">
                          {bar.label}
                        </div>
                        <div className="relative h-7 flex-1 border-2 border-black bg-[#FBF9F5]  ">
                          <div
                            className={`relative h-full ${isVanta ? "bg-[#FF5500]" : "bg-black/70 "}`}
                            style={{
                              width: showBars ? `${bar.pct}%` : "0%",
                              transition: `width ${bar.durationMs}ms var(--ease-default)`,
                              boxShadow: isVanta ? "3px 3px 0 0 #000" : "none",
                            }}
                          >
                            <span className="absolute right-2 top-1/2 -translate-y-1/2 font-tech text-[10px] font-bold uppercase text-[#FBF9F5] mix-blend-difference">
                              {bar.value}
                            </span>
                          </div>
                          {isVanta && finished && (
                            <span className="absolute -top-2 right-2 inline-flex items-center gap-1 border-2 border-black bg-[#FF5500] px-1.5 py-0.5 font-tech text-[9px] font-bold uppercase tracking-wider text-black ">
                              <Zap className="h-2.5 w-2.5" strokeWidth={3} />
                              {tt("benchmarkRace.fastest", "FASTEST")}
                            </span>
                          )}
                        </div>
                        <span className="w-6 shrink-0 text-right font-tech text-[10px] text-black/40 ">
                          #{bar.rank}
                        </span>
                      </div>
                    );
                  })}
                </div>
              </div>
            </Reveal>
          ))}
        </div>

        <Reveal direction="up" delay={200}>
          <p className="mt-6 border-l-4 border-[#FF5500] pl-4 font-tech text-xs leading-relaxed text-black/70 ">
            {tt("benchmarkRace.footer", "Real benchmarks. Reproducible. No marketing numbers. — src/benches")}
          </p>
        </Reveal>
      </div>
    </section>
  );
}
