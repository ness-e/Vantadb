"use client";

import { BenchmarksView } from "@/components/vanta/benchmarks-view";
import { BenchmarkRace } from "@/components/vanta/benchmark-race";
import { CompetitiveTable } from "@/components/vanta/competitive-table";
import { useVantaNavigate } from "@/hooks/use-vanta-navigate";

/**
 * Benchmarks route (/benchmarks) — BENCH-01 + SIFT1M + Latency Explorer.
 * <BenchmarkRace /> appends a live, animated head-to-head race;
 * <CompetitiveTable /> appends the JSON-contract comparison table (INV-007-B).
 */
export default function BenchmarksPage() {
  const navigate = useVantaNavigate();
  return (
    <div className="animate-rise">
      <BenchmarksView onNavigate={navigate} />
      <BenchmarkRace />
      <CompetitiveTable />
    </div>
  );
}
