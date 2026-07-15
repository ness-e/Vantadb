import { createLazyRoute } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection } from "@/components/nb";
import { PendingComponent } from "@/components/PendingComponent";

export const Route = createLazyRoute("/demo")({
  component: DemoPage,
  pendingComponent: PendingComponent,
});

function DemoPage() {
  return (
    <main className="demo-page">
      <NbSubpageHero
        title={
          <span>
            Try VantaDB
            <br />
            in Your Browser
          </span>
        }
        sub="AI-powered vector memory running entirely client-side via WebAssembly. No server, no install."
      />
      <NbSection>
        <div style={{ textAlign: "center", padding: "4rem 1rem" }}>
          <h2>WASM Demo Coming Soon</h2>
          <p style={{ marginTop: "1rem", color: "var(--color-muted)" }}>
            The browser-based WASM demo requires the <code>vantadb-wasm</code> npm package, which is
            pending publication. Check back after the next release.
          </p>
        </div>
      </NbSection>
    </main>
  );
}
