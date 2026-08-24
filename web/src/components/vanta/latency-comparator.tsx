"use client";

import { useState, useMemo, useRef, useEffect } from "react";
import { motion } from "framer-motion";
import { RotateCcw, Zap, Activity, Play, Square, Cpu, Timer, Copy } from "lucide-react";
import { copyToClipboard } from "./copy-utils";
import { toast } from "./toast";
import { cn } from "@/lib/utils";

type Engine = {
  name: string;
  p50: number; // ms
  p99: number; // ms
  throughput: number; // qps
  color: string;
  barColor: string;
};

// Baseline engines — sourced from docs/operations/BENCHMARKS.md §2 (SDK, 10K records, 128d, Cosine)
// BM25 excluded: p50 0.0035 ms is a degenerate outlier from a single-document query and not representative.
const ENGINES: Engine[] = [
  {
    name: "VantaDB · Hybrid",
    p50: 3.114,
    p99: 5.507,
    throughput: 321,
    color: "bg-[#FF5500]",
    barColor: "#FF5500",
  },
  {
    name: "VantaDB · HNSW",
    p50: 2.024,
    p99: 4.403,
    throughput: 494,
    color: "bg-black ",
    barColor: "#000000",
  },
  {
    name: "Network DB (typical)",
    p50: 12.5,
    p99: 28.0,
    throughput: 80,
    color: "bg-black/30 ",
    barColor: "rgba(0,0,0,0.4)",
  },
];

// Preset workloads — typical configurations for common use cases
const PRESETS = [
  { id: "rag", label: "RAG", desc: "top_k=10 · 768d · 50K", topK: 10, dim: 768, dataset: "50K" as const },
  { id: "edge", label: "Edge", desc: "top_k=5 · 128d · 10K", topK: 5, dim: 128, dataset: "10K" as const },
  { id: "agent", label: "Agent", desc: "top_k=20 · 384d · 100K", topK: 20, dim: 384, dataset: "100K" as const },
  { id: "custom", label: "Custom", desc: "your params", topK: 5, dim: 128, dataset: "10K" as const },
];

export function LatencyComparator() {
  const [topK, setTopK] = useState(5);
  const [dim, setDim] = useState(128);
  const [dataset, setDataset] = useState<"10K" | "50K" | "100K">("10K");
  const [activePreset, setActivePreset] = useState("edge");
  const [benchmarking, setBenchmarking] = useState(false);
  const [benchProgress, setBenchProgress] = useState(0);
  const [benchResult, setBenchResult] = useState<{ ops: number; latency: number; duration: number } | null>(null);
  const benchTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Clear benchmark interval on unmount
  useEffect(() => {
    return () => {
      if (benchTimerRef.current) clearInterval(benchTimerRef.current);
    };
  }, []);

  // Scale factors based on slider inputs (illustrative model)
  const factors = useMemo(() => {
    const topKFactor = 0.6 + (topK / 50) * 0.8; // 0.6 → 1.4
    const dimFactor = 0.7 + (dim / 1024) * 0.6; // 0.7 → 1.3
    const dsFactor = dataset === "10K" ? 1 : dataset === "50K" ? 1.25 : 1.6;
    return { topKFactor, dimFactor, dsFactor };
  }, [topK, dim, dataset]);

  const scaled = useMemo(
    () =>
      ENGINES.map((e) => ({
        ...e,
        sP50: +(e.p50 * factors.topKFactor * factors.dimFactor * factors.dsFactor).toFixed(2),
        sP99: +(e.p99 * factors.topKFactor * factors.dimFactor * factors.dsFactor).toFixed(2),
        sThroughput: Math.round(e.throughput / (factors.topKFactor * factors.dimFactor * factors.dsFactor)),
      })),
    [factors]
  );

  const maxLatency = Math.max(...scaled.map((e) => e.sP99));
  const maxThroughput = Math.max(...scaled.map((e) => e.sThroughput));

  const applyPreset = (presetId: string) => {
    const preset = PRESETS.find((p) => p.id === presetId);
    if (!preset) return;
    setActivePreset(presetId);
    setTopK(preset.topK);
    setDim(preset.dim);
    setDataset(preset.dataset);
    setBenchResult(null);
  };

  // Mark as custom when user manually adjusts sliders
  const onTopKChange = (v: number) => { setTopK(v); setActivePreset("custom"); setBenchResult(null); };
  const onDimChange = (v: number) => { setDim(v); setActivePreset("custom"); setBenchResult(null); };
  const onDatasetChange = (v: "10K" | "50K" | "100K") => { setDataset(v); setActivePreset("custom"); setBenchResult(null); };

  const reset = () => {
    setTopK(5);
    setDim(128);
    setDataset("10K");
    setActivePreset("edge");
    setBenchResult(null);
    stopBenchmark();
  };

  const stopBenchmark = () => {
    if (benchTimerRef.current) {
      clearInterval(benchTimerRef.current);
      benchTimerRef.current = null;
    }
    setBenchmarking(false);
  };

  const runBenchmark = () => {
    if (benchmarking) {
      stopBenchmark();
      return;
    }
    setBenchResult(null);
    setBenchmarking(true);
    setBenchProgress(0);

    const vantaEngine = scaled.find((e) => e.name === "VantaDB · Hybrid")!;
    const targetOps = Math.min(vantaEngine.sThroughput * 5, 5000);
    const targetLatency = vantaEngine.sP50;
    const duration = 3000; // 3 second simulated benchmark
    const startTime = Date.now();

    if (benchTimerRef.current) clearInterval(benchTimerRef.current);
    benchTimerRef.current = setInterval(() => {
      const elapsed = Date.now() - startTime;
      const pct = Math.min((elapsed / duration) * 100, 100);
      setBenchProgress(pct);

      if (elapsed >= duration) {
        stopBenchmark();
        // Add small jitter to feel "real"
        const jitter = 1 + (Math.random() - 0.5) * 0.08;
        setBenchResult({
          ops: Math.round(targetOps * jitter),
          latency: +(targetLatency * jitter).toFixed(2),
          duration: elapsed,
        });
      }
    }, 50);
  };

  const exportJson = async () => {
    if (!benchResult) return;
    const json = JSON.stringify({
      benchmark: "VantaDB Latency Explorer (simulated)",
      workload: { top_k: topK, dimensions: dim, dataset, preset: activePreset },
      results: {
        ops_per_sec: benchResult.ops,
        avg_latency_ms: benchResult.latency,
        duration_ms: benchResult.duration,
      },
      comparison: scaled.map((e) => ({
        engine: e.name,
        p50_ms: e.sP50,
        p99_ms: e.sP99,
        throughput_qps: e.sThroughput,
      })),
      baseline: "BENCH-01 §2 (10K records, 128d, cosine, Python SDK)",
      note: "Illustrative model with ±4% jitter. Not a real benchmark.",
      timestamp: new Date().toISOString(),
    }, null, 2);
    const ok = await copyToClipboard(json);
    if (ok) toast.copy("JSON del benchmark copiado");
  };

  return (
    <section className="relative border-b-4 border-black bg-[#FBF9F5]  ">
      <div className="pointer-events-none absolute inset-0 grid-tech opacity-50" aria-hidden />
      <div className="relative mx-auto max-w-7xl px-4 py-14 sm:px-6">
        <div className="flex flex-col gap-3 border-4 border-black bg-[#FBF9F5] p-6 shadow-[6px_6px_0_0_#000]    sm:flex-row sm:items-end sm:justify-between">
          <div>
            <h2 className="mt-3 font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
              Latency Explorer
            </h2>
            <p className="mt-2 max-w-xl font-tech text-xs text-black/70 ">
              Adjust the workload parameters and watch how VantaDB compares against a
              typical network-bound database. Numbers scale from the BENCH-01 baseline
              using an illustrative cost model.
            </p>
          </div>
          <button
            onClick={reset}
            className="press inline-flex items-center gap-2 border-4 border-black bg-[#FBF9F5] px-3 py-2 font-tech text-xs font-bold uppercase tracking-wider text-black   "
          >
            <RotateCcw className="h-3.5 w-3.5" strokeWidth={2.5} />
            Reset
          </button>
        </div>

        {/* Preset workload buttons */}
        <div className="mt-4 flex flex-wrap items-center gap-2">
          <span className="font-tech text-[10px] font-bold uppercase tracking-[0.2em] text-black/70 ">
            Presets:
          </span>
          {PRESETS.map((p) => (
            <button
              key={p.id}
              onClick={() => applyPreset(p.id)}
              className={cn(
                "group flex flex-col items-start border-2 border-black px-3 py-1.5 transition-all ",
                activePreset === p.id
                  ? "bg-[#FF5500] text-black"
                  : "bg-[#FBF9F5] text-black/70 hover:bg-[#F2EDE2]   "
              )}
              aria-pressed={activePreset === p.id}
              title={p.desc}
            >
              <span className="font-tech text-xs font-bold uppercase tracking-wider">{p.label}</span>
              <span className={cn("font-tech text-[9px] uppercase tracking-wider", activePreset === p.id ? "text-black/70" : "text-black/70 ")}>
                {p.desc}
              </span>
            </button>
          ))}
        </div>

        {/* Controls */}
        <div className="mt-6 grid grid-cols-1 gap-4 md:grid-cols-3">
          <SliderCard
            label="top_k"
            value={topK}
            min={1}
            max={50}
            step={1}
            unit=""
            onChange={onTopKChange}
            desc="Results returned per query"
          />
          <SliderCard
            label="dimensions"
            value={dim}
            min={64}
            max={1024}
            step={64}
            unit="d"
            onChange={onDimChange}
            desc="Embedding vector width"
          />
          <div className="press-lg border-4 border-black bg-[#FBF9F5] p-4  ">
            <div className="mb-2 flex items-center justify-between">
              <label className="font-tech text-[10px] font-bold uppercase tracking-[0.2em] text-black ">
                dataset
              </label>
              <span className="font-mono text-xs font-bold text-[#FF5500]">{dataset}</span>
            </div>
            <div className="grid grid-cols-3 gap-1">
              {(["10K", "50K", "100K"] as const).map((d) => (
                <button
                  key={d}
                  onClick={() => onDatasetChange(d)}
                  className={cn(
                    "border-2 border-black py-2 font-tech text-[11px] font-bold uppercase tracking-wider transition-all ",
                    dataset === d
                      ? "bg-[#FF5500] text-black"
                      : "bg-[#FBF9F5] text-black/70 hover:bg-[#F2EDE2]   "
                  )}
                >
                  {d}
                </button>
              ))}
            </div>
            <p className="mt-2 font-tech text-[10px] text-black/70 ">
              Indexed vector count
            </p>
          </div>
        </div>

        {/* Comparison chart */}
        <div className="mt-6 border-4 border-black bg-[#FBF9F5] p-5 shadow-[8px_8px_0_0_#000]   ">
          <div className="mb-4 flex items-center justify-between">
            <h3 className="flex items-center gap-2 font-display text-xl uppercase text-black ">
              <Activity className="h-5 w-5 text-[#FF5500]" strokeWidth={2.5} />
              p99 latency · lower is faster
            </h3>
            <span className="font-tech text-[10px] uppercase tracking-wider text-black/70 ">
              scale 0–{maxLatency.toFixed(1)}ms
            </span>
          </div>
          <div className="space-y-3">
            {scaled.map((e) => {
              const pct = (e.sP99 / maxLatency) * 100;
              const isVanta = e.name.startsWith("VantaDB");
              return (
                <div key={e.name} className="group/bar relative">
                  <div className="mb-1 flex items-center justify-between font-tech text-[11px]">
                    <span
                      className={cn(
                        "font-bold uppercase tracking-wider",
                        isVanta ? "text-black " : "text-black/60 "
                      )}
                    >
                      {e.name}
                    </span>
                    <span className={cn("font-mono", isVanta && "text-[#FF5500]")}>
                      {e.sP99}ms
                    </span>
                  </div>
                  <div className="relative h-6 border-2 border-black bg-[#F2EDE2]  ">
                    <motion.div
                      className="absolute left-0 top-0 h-full"
                      style={{ backgroundColor: e.barColor }}
                      initial={{ width: 0 }}
                      animate={{ width: `${pct}%` }}
                      transition={{ duration: 0.4, ease: [0.2, 0.8, 0.2, 1] }}
                    />
                    {isVanta && (
                      <span className="absolute right-2 top-1/2 -translate-y-1/2 font-tech text-[9px] font-bold uppercase tracking-wider text-black mix-blend-difference">
                        ◆
                      </span>
                    )}
                  </div>
                  {/* Tooltip on hover */}
                  <div className="pointer-events-none absolute -top-1 right-0 z-20 translate-y-[-100%] border-2 border-black bg-black px-2 py-1 font-tech text-[10px] text-[#FBF9F5] opacity-0 shadow-[3px_3px_0_0_#FF5500] transition-opacity duration-150 group-hover/bar:opacity-100 ">
                    <div className="flex gap-3">
                      <span>p50: <span className="text-[#FF5500]">{e.sP50}ms</span></span>
                      <span>p99: <span className="text-[#FF5500]">{e.sP99}ms</span></span>
                    </div>
                    <div className="text-[#FBF9F5]/60">qps: {e.sThroughput}</div>
                  </div>
                </div>
              );
            })}
          </div>

          {/* Throughput row */}
          <div className="mt-6 border-t-2 border-black/20 pt-4 ">
            <h3 className="mb-3 flex items-center gap-2 font-display text-lg uppercase text-black ">
              <Zap className="h-4 w-4 text-[#FF5500]" strokeWidth={2.5} />
              throughput · queries/sec
            </h3>
            <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
              {scaled.map((e) => (
                <div
                  key={e.name}
                  className="border-2 border-black bg-[#F2EDE2] p-3  "
                >
                  <p className="font-tech text-[9px] font-bold uppercase tracking-wider text-black/60 ">
                    {e.name.replace("VantaDB · ", "")}
                  </p>
                  <p className="mt-1 font-display text-2xl text-black ">
                    {e.sThroughput}
                  </p>
                  <div className="mt-1 h-1 w-full bg-black/10 ">
                    <motion.div
                      className="h-full bg-[#FF5500]"
                      initial={{ width: 0 }}
                      animate={{ width: `${(e.sThroughput / maxThroughput) * 100}%` }}
                      transition={{ duration: 0.5, ease: [0.2, 0.8, 0.2, 1] }}
                    />
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Benchmark your machine panel */}
        <div className="mt-6 border-4 border-black bg-black p-5 shadow-[6px_6px_0_0_#FF5500] ">
          <div className="mb-3 flex items-center justify-between">
            <h3 className="flex items-center gap-2 font-display text-lg uppercase text-[#FBF9F5]">
              <Cpu className="h-4 w-4 text-[#FF5500]" strokeWidth={2.5} />
              Simulate benchmark
            </h3>
            <span className="font-tech text-[9px] uppercase tracking-wider text-[#FBF9F5]/40">
              {activePreset} workload · {dataset}
            </span>
          </div>
          <p className="mb-3 font-tech text-[11px] text-[#FBF9F5]/60">
            Run a 3-second simulated benchmark of the current workload configuration.
            Results are extrapolated from the BENCH-01 baseline with ±4% jitter.
          </p>

          {/* Run button + progress */}
          <div className="flex items-center gap-3">
            <button
              onClick={runBenchmark}
              className={cn(
                "press inline-flex items-center gap-2 border-4 px-4 py-2 font-tech text-xs font-bold uppercase tracking-wider transition-all",
                benchmarking
                  ? "border-[#FF5500] bg-[#1A1A1A] text-[#FF5500]"
                  : "border-[#FBF9F5] bg-[#FF5500] text-black"
              )}
              aria-label={benchmarking ? "Stop benchmark" : "Run benchmark"}
            >
              {benchmarking ? (
                <>
                  <Square className="h-3.5 w-3.5 fill-current" strokeWidth={2.5} />
                  Stop
                </>
              ) : (
                <>
                  <Play className="h-3.5 w-3.5 fill-current" strokeWidth={2.5} />
                  Run Benchmark
                </>
              )}
            </button>

            {/* Progress bar */}
            {(benchmarking || benchProgress > 0) && (
              <div className="flex-1">
                <div className="relative h-3 border-2 border-[#FBF9F5]/30 bg-[#1A1A1A]">
                  <div
                    className="absolute left-0 top-0 h-full bg-[#FF5500] transition-[width] duration-75"
                    style={{ width: `${benchProgress}%` }}
                  />
                  {benchmarking && (
                    <span className="absolute right-2 top-1/2 -translate-y-1/2 font-mono text-[9px] font-bold text-[#FBF9F5]">
                      {Math.round(benchProgress)}%
                    </span>
                  )}
                </div>
              </div>
            )}
          </div>

          {/* Results */}
          {benchResult && !benchmarking && (
            <>
            <div className="mt-4 grid grid-cols-3 gap-2">
              <div className="border-2 border-[#FF5500] bg-[#1A1A1A] p-3">
                <div className="flex items-center gap-1 font-tech text-[9px] font-bold uppercase tracking-wider text-[#FBF9F5]/50">
                  <Zap className="h-3 w-3 text-[#FF5500]" strokeWidth={2.5} />
                  ops/sec
                </div>
                <BenchResultValue value={benchResult.ops} format="int" className="mt-1 font-display text-2xl text-[#FF5500]" />
              </div>
              <div className="border-2 border-[#FBF9F5]/30 bg-[#1A1A1A] p-3">
                <div className="flex items-center gap-1 font-tech text-[9px] font-bold uppercase tracking-wider text-[#FBF9F5]/50">
                  <Timer className="h-3 w-3 text-[#FF5500]" strokeWidth={2.5} />
                  avg latency
                </div>
                <BenchResultValue value={benchResult.latency} format="decimal" suffix="ms" className="mt-1 font-display text-2xl text-[#FBF9F5]" />
              </div>
              <div className="border-2 border-[#FBF9F5]/30 bg-[#1A1A1A] p-3">
                <div className="font-tech text-[9px] font-bold uppercase tracking-wider text-[#FBF9F5]/50">
                  duration
                </div>
                <BenchResultValue value={benchResult.duration / 1000} format="decimal1" suffix="s" className="mt-1 font-display text-2xl text-[#FBF9F5]" />
              </div>
            </div>

            {/* Export results */}
            <div className="mt-3 flex items-center gap-2">
              <button
                onClick={exportJson}
                className="inline-flex items-center gap-1.5 border-2 border-[#FBF9F5]/30 bg-[#1A1A1A] px-2.5 py-1.5 font-tech text-[10px] font-bold uppercase tracking-wider text-[#FBF9F5] transition-colors hover:border-[#FF5500] hover:text-[#FF5500]"
                aria-label="Exportar resultados como JSON"
              >
                <Copy className="h-3 w-3" strokeWidth={2.5} />
                Copy JSON
              </button>
              <span className="font-tech text-[9px] uppercase tracking-wider text-[#FBF9F5]/30">
                shareable result
              </span>
            </div>
            </>
          )}
        </div>

        <p className="mt-4 border-l-4 border-[#FF5500] bg-[#FBF9F5] px-4 py-2 font-tech text-[11px] italic text-black/70  ">
          <span className="font-bold not-italic uppercase tracking-wider">Note:</span>{" "}
          Baseline from BENCH-01 (10K vectors, 128d, cosine). The cost model is
          illustrative — actual figures depend on hardware, AVX2, and ef_search
          configuration. Run{" "}
          <code className="font-mono">python benchmarks/vantadb_local_bench.py</code> for
          real numbers on your machine.
        </p>
      </div>
    </section>
  );
}

function SliderCard({
  label,
  value,
  min,
  max,
  step,
  unit,
  onChange,
  desc,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  unit: string;
  onChange: (v: number) => void;
  desc: string;
}) {
  const pct = ((value - min) / (max - min)) * 100;
  return (
    <div className="press-lg border-4 border-black bg-[#FBF9F5] p-4  ">
      <div className="mb-2 flex items-center justify-between">
        <label className="font-tech text-[10px] font-bold uppercase tracking-[0.2em] text-black ">
          {label}
        </label>
        <span className="font-mono text-xs font-bold text-[#FF5500]">
          {value}
          {unit}
        </span>
      </div>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        className="vanta-slider w-full"
        style={{
          background: `linear-gradient(to right, #FF5500 0%, #FF5500 ${pct}%, #000 ${pct}%, #000 100%)`,
        }}
        aria-label={label}
      />
      <p className="mt-2 font-tech text-[10px] text-black/70 ">{desc}</p>
    </div>
  );
}

/**
 * BenchResultValue — count-up animated value for benchmark results.
 * Animates from 0 to target on mount (results are always visible when rendered).
 */
function BenchResultValue({
  value,
  format,
  suffix,
  className,
}: {
  value: number;
  format: "int" | "decimal" | "decimal1";
  suffix?: string;
  className?: string;
}) {
  const [current, setCurrent] = useState(0);

  useEffect(() => {
    const prefersReduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (prefersReduced) {
      Promise.resolve().then(() => setCurrent(value));
      return;
    }
    const duration = 800;
    const startTime = performance.now();
    let raf = 0;
    const tick = (now: number) => {
      const t = Math.min((now - startTime) / duration, 1);
      const eased = 1 - Math.pow(1 - t, 3);
      setCurrent(value * eased);
      if (t < 1) raf = requestAnimationFrame(tick);
      else setCurrent(value);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [value]);

  const display =
    format === "int"
      ? Math.round(current).toLocaleString("en-US")
      : format === "decimal1"
        ? current.toFixed(1)
        : current.toFixed(2);

  return (
    <div className={className}>
      {display}
      {suffix}
    </div>
  );
}
