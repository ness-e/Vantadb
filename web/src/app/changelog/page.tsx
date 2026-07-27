"use client";

import { ChangelogSection } from "@/components/vanta/changelog-section";

/**
 * /changelog — Release timeline route.
 * <ChangelogSection /> renders its own §09 header with searchable, filterable timeline.
 * Hidden h1 for SEO/accessibility.
 */
export default function ChangelogPage() {
  return (
    <div className="animate-rise">
      <h1 className="sr-only">Changelog — VantaDB</h1>
      <ChangelogSection />
    </div>
  );
}
