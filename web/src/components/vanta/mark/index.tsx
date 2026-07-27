"use client";

import { useState } from "react";
import { MarkClassic } from "./mark-classic";
import type { MarkVariantName } from "./types";

/**
 * Mark — entry point for the VantaDB interactive mark.
 * Renders the active variant. Variant switching is managed here so the hero
 * can offer multiple mark styles without touching variant internals.
 *
 * Currently available: "classic"
 * Future variants can be added to MARK_VARIANTS and rendered below.
 */
export function Mark({ variant = "classic" }: { variant?: MarkVariantName }) {
  // Variant state is internal for now (classic is default). When more variants
  // are added, the hero toggle will pass variant prop.
  const [activeVariant] = useState<MarkVariantName>(variant);

  switch (activeVariant) {
    case "classic":
    default:
      return <MarkClassic />;
    // case "neo": return <MarkNeo />;  // future
    // case "mini": return <MarkMini />; // future
  }
}

export { MarkClassic } from "./mark-classic";
export { MARK_VARIANTS } from "./types";
export type { MarkVariantName } from "./types";
