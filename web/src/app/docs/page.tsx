"use client";

import { DocsView } from "@/components/vanta/docs-view";
import { useVantaNavigate } from "@/hooks/use-vanta-navigate";

/**
 * Docs route (/docs) — Installation + Quickstart + CLI + Server + Playground.
 */
export default function DocsPage() {
  const navigate = useVantaNavigate();
  return <DocsView onNavigate={navigate} />;
}
