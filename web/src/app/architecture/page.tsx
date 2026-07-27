"use client";

import { Architecture } from "@/components/vanta/architecture";
import { SearchSemantics } from "@/components/vanta/search-semantics";
import { useVantaNavigate } from "@/hooks/use-vanta-navigate";

/**
 * /architecture — Retrieval Pipeline + Search Semantics route.
 * <Architecture /> renders its own §07 header. SearchSemantics follows.
 * Hidden h1 for SEO/accessibility.
 */
export default function ArchitecturePage() {
  const navigate = useVantaNavigate();
  return (
    <div className="animate-rise">
      <h1 className="sr-only">Architecture — VantaDB</h1>
      <Architecture onNavigate={navigate} />
      <SearchSemantics />
    </div>
  );
}
