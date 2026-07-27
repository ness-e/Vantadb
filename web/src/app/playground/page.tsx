"use client";

import { CodePlayground } from "@/components/vanta/code-playground";

/**
 * /playground — Interactive Code Playground route.
 * <CodePlayground /> renders its own §INTERACTIVE header with editor + simulator.
 * Hidden h1 for SEO/accessibility.
 */
export default function PlaygroundPage() {
  return (
    <div className="animate-rise">
      <h1 className="sr-only">Code Playground — VantaDB</h1>
      <CodePlayground />
    </div>
  );
}
