"use client";

import { BenchmarksView } from "@/components/vanta/benchmarks-view";
import { BenchmarkRace } from "@/components/vanta/benchmark-race";
import { useVantaNavigate } from "@/hooks/use-vanta-navigate";

/**
 * Benchmarks route (/benchmarks) — BENCH-01 + SIFT1M + Latency Explorer.
 * <BenchmarkRace /> appends a live, animated head-to-head race.
 */
export default function BenchmarksPage() {
  const navigate = useVantaNavigate();
  return (
    <div className="animate-rise">
      <BenchmarksView onNavigate={navigate} />
      <BenchmarkRace />
    </div>
  );
}
