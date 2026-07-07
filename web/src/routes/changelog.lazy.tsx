import { createLazyRoute } from "@tanstack/react-router";
import { useRef, useState, useMemo } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader, NbBlockAmber } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
import { PendingComponent } from "@/components/PendingComponent";
import "../styles/changelog.css";

export const Route = createLazyRoute("/changelog")({
  component: ChangelogPage,
  pendingComponent: PendingComponent,
});

const releases = [
  {
    version: "v0.1.5",
    date: "2026-06-22",
    tag: "Integrations + CLI Epic",
    changes: [
      {
        type: "feature",
        text: "10 integration crates: OpenAI, Ollama, Haystack, DSPy, LiteLLM, CrewAI, Mem0, Letta, MCP (experimental), WASM (experimental).",
      },
      {
        type: "feature",
        text: "Python SDK: 28 methods across query, graph, namespace, and admin operations.",
      },
      {
        type: "feature",
        text: "AsyncVantaDB wrapper with full async SDK and context manager support.",
      },
      {
        type: "feature",
        text: "CLI: 10 subcommands \u2014 put, get, delete, search, list, server, export, import, rebuild-index, stats, compact.",
      },
      {
        type: "feature",
        text: "MCP interface (vantadb-mcp) \u2014 expose VantaDB as an MCP tool server.",
      },
      {
        type: "feature",
        text: "WASM bindings (vantadb-wasm) \u2014 in-browser vector search via WebAssembly.",
      },
      { type: "feature", text: "SQ8 quantization \u2014 4x memory reduction for vector indices." },
      { type: "feature", text: "Zero-copy HNSW graph archive with rkyv serialization." },
      { type: "feature", text: "WAL compaction, TTL eviction, batch put with Rayon parallelism." },
      { type: "feature", text: "Python 3.13+ support, ARM64 Linux wheels, Homebrew formula." },
      { type: "fix", text: "16 risky .unwrap() calls replaced with ? + error handling." },
      { type: "fix", text: "Infinite recursion in text_index without advanced-tokenizer." },
      {
        type: "perf",
        text: "Cached inverse norms for cosine similarity; squared Euclidean distance.",
      },
      {
        type: "security",
        text: "PyO3 0.24 \u2192 0.29 upgrade, bincode 1.3 \u2192 2.0 migration.",
      },
    ],
  },
  {
    version: "v0.1.4",
    date: "2026-05-25",
    tag: "Hardening",
    changes: [
      { type: "feature", text: "Metrics hardening and heap memory drift instrumentation." },
      {
        type: "feature",
        text: "Expanded filter operators (Eq, Neq, Gt, Gte, Lt, Lte, In, Exists).",
      },
      { type: "feature", text: "delete_by_filter(), similar_to_key(), count() SDK methods." },
      { type: "fix", text: "All production unwrap() calls replaced with ? propagation." },
      { type: "fix", text: "File locking edge cases: antivirus, backup, stale lock recovery." },
    ],
  },
  {
    version: "v0.1.3",
    date: "2026-05-25",
    tag: "Security + ADRs",
    changes: [
      { type: "feature", text: "ADR-001: Unified config with readonly barrier." },
      { type: "feature", text: "ADR-002: WAL CRC32C auto-healing on corruption." },
      { type: "feature", text: "ADR-003: Sync/async decoupling with spawn_blocking." },
      { type: "security", text: "Security audit and advisory resolution." },
    ],
  },
  {
    version: "v0.1.2",
    date: "2026-05-25",
    tag: "Durability",
    changes: [
      { type: "feature", text: "Memory-mapped vector store with SIGBUS error handling." },
      { type: "feature", text: "Predictive mmap prefetching via madvise." },
      { type: "feature", text: "WAL CRC32C checksums and crash-injection recovery tests." },
      { type: "feature", text: "Durability guarantees documentation." },
    ],
  },
  {
    version: "v0.1.1",
    date: "2026-05-13",
    tag: "First Release",
    changes: [
      { type: "feature", text: "First public release post-pivot. Embedded-first architecture." },
      { type: "feature", text: "HNSW vector search + BM25 lexical search + RRF hybrid fusion." },
      { type: "feature", text: "3 storage backends: Fjall (default), RocksDB, InMemory." },
      { type: "feature", text: "Python bindings via PyO3 with full SDK." },
      { type: "feature", text: "WAL-backed durability with 3 sync modes." },
      {
        type: "feature",
        text: "CI/CD: rust_ci, python_wheels, heavy_certification, bench workflows.",
      },
    ],
  },
];

const TYPE_CONFIG: Record<string, { label: string }> = {
  feature: { label: "FEATURE" },
  perf: { label: "PERF" },
  fix: { label: "FIX" },
  breaking: { label: "BREAKING" },
  security: { label: "SECURITY" },
};

const ALL_TYPES = ["all", "feature", "perf", "fix", "security", "breaking"];

function ChangelogPage() {
  const [activeFilter, setActiveFilter] = useState("all");

  const filteredReleases = useMemo(
    () =>
      releases
        .map((r) => ({
          ...r,
          changes:
            activeFilter === "all" ? r.changes : r.changes.filter((c) => c.type === activeFilter),
        }))
        .filter((r) => r.changes.length > 0),
    [activeFilter],
  );

  const releasesRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nc-cl-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(releasesRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, releasesRef);

  return (
    <div>
      <NbSubpageHero
        pattern="p03"
        title={
          <span>
            What changed.
            <br />
            Why it matters.
          </span>
        }
        sub="Every release \u2014 features, performance gains, fixes, and migration notes. No marketing fluff."
      />

      <main>
        <NbSection variant="sm" ariaLabel="Filter releases">
          <div className="nc-cl-filter-bar">
            {ALL_TYPES.map((t) => {
              const cfg = TYPE_CONFIG[t];
              const isActive = activeFilter === t;
              return (
                <button
                  key={t}
                  onClick={() => setActiveFilter(t)}
                  className={`nc-cl-filter-btn ${isActive ? "nc-cl-filter-btn--active" : ""} ${t !== "all" && isActive ? `nc-cl-filter-btn--${t}` : ""}`}
                >
                  {t === "all" ? "ALL" : cfg?.label || t.toUpperCase()}
                </button>
              );
            })}
          </div>
        </NbSection>

        <NbSection ref={releasesRef} ariaLabel="Releases">
          <NbSectionHeader
            monoLabel="[RELEASES]"
            headline="Ship log."
            sub="Every version, every change. Filter by category to find what matters."
          />

          <div className="nc-cl-releases nc-cl-part">
            {filteredReleases.map((release, i) => (
              <div key={release.version} className="nc-cl-release">
                <div className="nc-cl-sidebar">
                  {i === 0 && <span className="nc-cl-badge">CURRENT</span>}
                  <span className="nc-cl-version">{release.version}</span>
                  <span className="nc-cl-date">{release.date}</span>
                  <span className="nc-cl-tag">{release.tag}</span>
                </div>

                <div className="nc-cl-changes">
                  {release.changes.map((change, ci) => {
                    const cfg = TYPE_CONFIG[change.type] || { label: change.type.toUpperCase() };
                    return (
                      <div key={ci} className="nc-cl-change">
                        <span className={`nc-cl-type nc-cl-type--${change.type}`}>{cfg.label}</span>
                        <span className="nc-cl-text">{change.text}</span>
                      </div>
                    );
                  })}
                </div>
              </div>
            ))}
          </div>

          <div className="nc-cl-semver nc-cl-part">
            <span className="nc-cl-semver-label">SEMVER</span>
            <p className="nc-cl-semver-text">
              VantaDB follows semantic versioning. Patch upgrades (0.1.x \u2192 0.1.y) require no
              code changes. Minor bumps (0.1 \u2192 0.2) may include breaking changes \u2014 check
              those tags before deploying to production.
            </p>
          </div>
        </NbSection>

        <NbSection variant="dark" className="nb-bg-dot" ariaLabel="Get started">
          <NbBlockAmber as="div">
            <div className="nc-cl-cta">
              <div>
                <h2 className="nc-cl-cta-title">Always improving. Ship with confidence.</h2>
                <p className="nc-cl-cta-sub">Install VantaDB in one command.</p>
              </div>
              <code className="nc-cl-cta-code">pip install vantadb-py</code>
            </div>
          </NbBlockAmber>
        </NbSection>
      </main>
    </div>
  );
}
