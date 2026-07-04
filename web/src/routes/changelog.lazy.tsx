import { createLazyRoute } from "@tanstack/react-router";
import { useState, useMemo } from "react";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";
import { PendingComponent } from "@/components/PendingComponent";

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
        text: "Python SDK: `put`, `put_batch`, `get`, `get_memory`, `delete`, `delete_memory`, `search`, `search_memory`, `query`, `flush`, `compact_wal`, `rebuild_index`, `add_edge`, `graph_bfs`, `graph_dfs`, `graph_topological_sort`, `graph_is_dag`, `export_namespace`, `export_all`, `import_file`, `audit_text_index`, `repair_text_index`, `purge_expired`, `capabilities`, `hardware_profile`, `operational_metrics`, `list_namespaces`, `generate_snippet`, `explain_memory_search`, `close`.",
      },
      {
        type: "feature",
        text: "AsyncVantaDB wrapper with full async SDK and context manager support.",
      },
      {
        type: "feature",
        text: "CLI: 10 subcommands — `put`, `get`, `delete`, `search`, `list`, `server`, `export`, `import`, `rebuild-index`, `stats`, `compact`.",
      },
      {
        type: "feature",
        text: "MCP interface (`vantadb-mcp`) — expose VantaDB as an MCP tool server.",
      },
      {
        type: "feature",
        text: "WASM bindings (`vantadb-wasm`) — in-browser vector search via WebAssembly.",
      },
      {
        type: "feature",
        text: "SQ8 quantization — 4x memory reduction for vector indices.",
      },
      {
        type: "feature",
        text: "Zero-copy HNSW graph archive with rkyv serialization.",
      },
      {
        type: "feature",
        text: "WAL compaction, TTL eviction, batch put with Rayon parallelism.",
      },
      {
        type: "feature",
        text: "Python 3.13+ support, ARM64 Linux wheels, Homebrew formula.",
      },
      {
        type: "fix",
        text: "16 risky `.unwrap()` calls replaced with `?` + error handling.",
      },
      {
        type: "fix",
        text: "Infinite recursion in text_index without advanced-tokenizer.",
      },
      {
        type: "perf",
        text: "Cached inverse norms for cosine similarity; squared Euclidean distance.",
      },
      {
        type: "security",
        text: "PyO3 0.24 → 0.29 upgrade, bincode 1.3 → 2.0 migration.",
      },
    ],
  },
  {
    version: "v0.1.4",
    date: "2026-05-25",
    tag: "Hardening",
    changes: [
      {
        type: "feature",
        text: "Metrics hardening and heap memory drift instrumentation.",
      },
      {
        type: "feature",
        text: "Expanded filter operators (Eq, Neq, Gt, Gte, Lt, Lte, In, Exists).",
      },
      {
        type: "feature",
        text: "`delete_by_filter()`, `similar_to_key()`, `count()` SDK methods.",
      },
      {
        type: "fix",
        text: "All production `unwrap()` calls replaced with `?` propagation.",
      },
      {
        type: "fix",
        text: "File locking edge cases: antivirus, backup, stale lock recovery.",
      },
    ],
  },
  {
    version: "v0.1.3",
    date: "2026-05-25",
    tag: "Security + ADRs",
    changes: [
      {
        type: "feature",
        text: "ADR-001: Unified config with readonly barrier.",
      },
      {
        type: "feature",
        text: "ADR-002: WAL CRC32C auto-healing on corruption.",
      },
      {
        type: "feature",
        text: "ADR-003: Sync/async decoupling with spawn_blocking.",
      },
      {
        type: "security",
        text: "Security audit and advisory resolution.",
      },
    ],
  },
  {
    version: "v0.1.2",
    date: "2026-05-25",
    tag: "Durability",
    changes: [
      {
        type: "feature",
        text: "Memory-mapped vector store with SIGBUS error handling.",
      },
      {
        type: "feature",
        text: "Predictive mmap prefetching via madvise.",
      },
      {
        type: "feature",
        text: "WAL CRC32C checksums and crash-injection recovery tests.",
      },
      {
        type: "feature",
        text: "Durability guarantees documentation.",
      },
    ],
  },
  {
    version: "v0.1.1",
    date: "2026-05-13",
    tag: "First Release",
    changes: [
      {
        type: "feature",
        text: "First public release post-pivot. Embedded-first architecture.",
      },
      {
        type: "feature",
        text: "HNSW vector search + BM25 lexical search + RRF hybrid fusion.",
      },
      {
        type: "feature",
        text: "3 storage backends: Fjall (default), RocksDB, InMemory.",
      },
      {
        type: "feature",
        text: "Python bindings via PyO3 with full SDK.",
      },
      {
        type: "feature",
        text: "WAL-backed durability with 3 sync modes.",
      },
      {
        type: "feature",
        text: "CI/CD: rust_ci, python_wheels, heavy_certification, bench workflows.",
      },
    ],
  },
];

const TYPE_CONFIG: Record<string, { label: string; color: string }> = {
  feature: { label: "FEATURE", color: "var(--foreground)" },
  perf: { label: "PERF", color: "var(--amber)" },
  fix: { label: "FIX", color: "var(--steel)" },
  breaking: { label: "BREAKING", color: "var(--steel)" },
  security: { label: "SECURITY", color: "var(--foreground)" },
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

  return (
    <div className="engine-page">
      <SwissSubpageHero
        num="12"
        eyebrow="Changelog"
        title={
          <span>
            What changed.
            <br />
            Why it matters.
          </span>
        }
        sub="Every release — features, performance gains, fixes, and migration notes. No marketing fluff."
      />

      <main className="engine-main">
        <div
          style={{
            display: "flex",
            gap: "1px",
            background: "var(--border)",
            border: "1px solid var(--border)",
            marginBottom: "4rem",
            width: "fit-content",
          }}
        >
          {ALL_TYPES.map((t) => {
            const cfg = TYPE_CONFIG[t];
            const isActive = activeFilter === t;
            return (
              <button
                key={t}
                onClick={() => setActiveFilter(t)}
                style={{
                  background: isActive ? "var(--surface-raised)" : "var(--background)",
                  border: "none",
                  borderBottom: isActive
                    ? `2px solid ${t === "all" ? "var(--foreground)" : cfg?.color || "var(--foreground)"}`
                    : "2px solid transparent",
                  padding: "0.75rem 1.5rem",
                  fontFamily: "var(--font-mono)",
                  fontSize: "0.6rem",
                  fontWeight: 700,
                  textTransform: "uppercase",
                  letterSpacing: "0.1em",
                  color: isActive
                    ? t === "all"
                      ? "var(--foreground)"
                      : cfg?.color || "var(--foreground)"
                    : "var(--steel)",
                  cursor: "pointer",
                  transition: "all 150ms var(--ease-cut)",
                  whiteSpace: "nowrap",
                }}
              >
                {t === "all" ? "ALL" : cfg?.label || t.toUpperCase()}
              </button>
            );
          })}
        </div>

        <div
          style={{
            display: "grid",
            gridTemplateColumns: "1fr",
            gap: "1px",
            background: "var(--border)",
            border: "1px solid var(--border)",
          }}
        >
          {filteredReleases.map((release, i) => (
            <div
              key={release.version}
              style={{
                display: "grid",
                gridTemplateColumns: "220px 1fr",
                background: "var(--background)",
              }}
            >
              <div
                style={{
                  padding: "2.5rem 2rem",
                  borderRight: "1px solid var(--border)",
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.5rem",
                  position: "relative",
                }}
              >
                {i === 0 && (
                  <span
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.55rem",
                      fontWeight: 700,
                      textTransform: "uppercase",
                      letterSpacing: "0.1em",
                      color: "var(--amber)",
                      background: "rgba(255, 85, 0, 0.1)",
                      padding: "0.2rem 0.5rem",
                      marginBottom: "0.5rem",
                      width: "fit-content",
                    }}
                  >
                    CURRENT
                  </span>
                )}
                <span
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "1.5rem",
                    fontWeight: 800,
                    letterSpacing: "-0.04em",
                    color: "var(--foreground)",
                    lineHeight: 1,
                  }}
                >
                  {release.version}
                </span>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.6rem",
                    color: "var(--steel)",
                    textTransform: "uppercase",
                    letterSpacing: "0.06em",
                  }}
                >
                  {release.date}
                </span>
                <span
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "0.72rem",
                    color: "var(--muted)",
                    marginTop: "0.25rem",
                  }}
                >
                  {release.tag}
                </span>
              </div>

              <div
                style={{
                  padding: "2.5rem 2.5rem",
                  display: "flex",
                  flexDirection: "column",
                  gap: "0",
                }}
              >
                {release.changes.map((change, ci) => {
                  const cfg = TYPE_CONFIG[change.type] || {
                    label: change.type.toUpperCase(),
                    color: "var(--muted)",
                  };
                  return (
                    <div
                      key={ci}
                      style={{
                        display: "grid",
                        gridTemplateColumns: "80px 1fr",
                        gap: "1rem",
                        alignItems: "baseline",
                        padding: "0.85rem 0",
                        borderBottom:
                          ci < release.changes.length - 1 ? "1px solid var(--border)" : "none",
                      }}
                    >
                      <span
                        style={{
                          fontFamily: "var(--font-mono)",
                          fontSize: "0.55rem",
                          fontWeight: 700,
                          textTransform: "uppercase",
                          letterSpacing: "0.1em",
                          color: cfg.color,
                          paddingTop: "2px",
                        }}
                      >
                        {cfg.label}
                      </span>
                      <span
                        style={{
                          fontFamily: "var(--font-sans)",
                          fontSize: "0.82rem",
                          color: "var(--muted)",
                          lineHeight: 1.6,
                        }}
                      >
                        {change.text}
                      </span>
                    </div>
                  );
                })}
              </div>
            </div>
          ))}
        </div>

        <section
          className="engine-section"
          style={{
            borderTop: "1px solid var(--border)",
            marginTop: "1px",
            background: "var(--surface)",
          }}
        >
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "220px 1fr",
              gap: "3rem",
              alignItems: "start",
            }}
          >
            <div>
              <span
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "0.6rem",
                  color: "var(--amber)",
                  textTransform: "uppercase",
                  letterSpacing: "0.08em",
                }}
              >
                SEMVER
              </span>
            </div>
            <p
              style={{
                fontFamily: "var(--font-sans)",
                fontSize: "0.82rem",
                color: "var(--muted)",
                lineHeight: 1.7,
                margin: 0,
                maxWidth: "600px",
              }}
            >
              VantaDB follows semantic versioning. Patch upgrades (0.1.x → 0.1.y) require no code
              changes. Minor bumps (0.1 → 0.2) may include breaking changes — check those tags
              before deploying to production.
            </p>
          </div>
        </section>
      </main>

      <style>{`
        @media (max-width: 768px) {
          [style*="grid-template-columns: 220px 1fr"] { grid-template-columns: 1fr !important; }
          .engine-section [style*="grid-template-columns: 220px 1fr"] { grid-template-columns: 1fr !important; gap: 1rem !important; }
        }
        @media (max-width: 640px) {
          [style*="grid-template-columns: 80px 1fr"] { grid-template-columns: 1fr !important; gap: 0.25rem !important; }
        }
      `}</style>
    </div>
  );
}


