"use client";

import { CoreEngine } from "@/components/vanta/core-engine";
import { WalSimulator } from "@/components/vanta/wal-simulator";

/**
 * /engine — Core Engine route.
 * <CoreEngine /> renders its own §05 header with the full pipeline + durability cards.
 * <WalSimulator /> adds the interactive WAL crash/recovery demo below.
 * Hidden h1 for SEO/accessibility (visual header is h2 in CoreEngine).
 */
export default function EnginePage() {
  return (
    <div className="animate-rise">
      <h1 className="sr-only">Core Engine — VantaDB</h1>
      <CoreEngine />
      <WalSimulator />
    </div>
  );
}
