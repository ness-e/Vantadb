// Workspace shell (VS-03): Sidebar + Topbar + central surface + right Inspector.
// Layout/estética replican el prototipo VS-00 (desktop/prototype/index.html) con
// tokens/utilities de VS-01 (--color-neon, .press, .scroll-manga, font-display,
// font-tech, dark overrides). Los paneles legacy (MetricsGrid/KpiCards/SopPanel/
// ProcessPanel/ConnectionPanel/IngestForm/DataExplorer/ExportPanel) se reubican
// como superficies/lentes — funcionalidad intacta, nada se borra.
//
// Superficies: RESUMEN (ops: connections/metrics/KPIs/SOP/export) · MEMORIAS
// (ingest + grid master) · ACTIVITY (ProcessPanel) · ÍNDICES/IQL (lentes
// pendientes Fase 1/2). Búsqueda global en Topbar (reemplaza SearchBar: misma
// llamada search() + ResultsList). Inspector derecho = master-detail del
// registro seleccionado (placeholder fino; VS-06 lo completa con tabs).
import { FormEvent, useEffect, useRef, useState } from "react";
import { list, search, SearchResult, vantaErrorMessage } from "../../vanta";
import { ConnectionActions, VantaState } from "../../hooks/useConnectionState";
import ConnectionPanel from "../ConnectionPanel";
import IngestForm from "../IngestForm";
import KpiCards from "../KpiCards";
import MetricsGrid from "../MetricsGrid";
import DataExplorer, { ExplorerRow } from "../DataExplorer";
import SopPanel from "../SopPanel";
import ProcessPanel from "../ProcessPanel";
import ExportPanel from "../ExportPanel";
import ResultsList from "../ResultsList";
import { MarkStudio } from "../mark/mark-studio";

type Surface = "resumen" | "memorias" | "actividad" | "indices" | "iql";

interface NamespaceCount {
  name: string;
  count: number;
}

/** Shape normalizada para el Inspector (master-detail). VS-06 lo enriquece. */
export interface InspectableRecord {
  id: string;
  namespace: string;
  text: string;
  score: number | null;
  metadata?: Record<string, unknown>;
  created_at_ms?: number | null;
}

interface WorkspaceShellProps {
  state: VantaState;
  actions: ConnectionActions;
  notice: string | null;
  onNotice: (msg: string) => void;
  onDismissNotice: () => void;
  onError: (msg: string) => void;
  dark: boolean;
  onToggleTheme: () => void;
}

/** Conteos por namespace, derivados del list del bridge activo. */
// ponytail: conteos client-side desde list(limit 500). Swap a namespace_stats
// (VS-CORE-02) cuando el bridge desktop lo exponga (lo hace VS-04).
function useNamespaceCounts(active: boolean): NamespaceCount[] {
  const [namespaces, setNamespaces] = useState<NamespaceCount[]>([]);

  useEffect(() => {
    let alive = true;
    if (!active) {
      setNamespaces([]);
      return;
    }
    list({ limit: 500 })
      .then((records) => {
        if (!alive) return;
        const counts = new Map<string, number>();
        for (const r of records) {
          counts.set(r.namespace, (counts.get(r.namespace) ?? 0) + 1);
        }
        setNamespaces(
          [...counts.entries()]
            .map(([name, count]) => ({ name, count }))
            .sort((a, b) => b.count - a.count),
        );
      })
      .catch(() => {
        if (alive) setNamespaces([]);
      });
    return () => {
      alive = false;
    };
  }, [active]);

  return namespaces;
}

function SideButton({
  icon,
  label,
  hint,
  active,
  onClick,
}: {
  icon: string;
  label: string;
  hint?: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-current={active ? "page" : undefined}
      className={`press flex w-full items-center gap-3 border-2 border-foreground px-3 py-2 text-left text-sm font-semibold ${
        active ? "bg-foreground text-background" : "bg-background"
      }`}
    >
      <span className="text-neon">{icon}</span>
      {label}
      {hint && <span className="ml-auto font-tech text-[9px] text-neon">{hint}</span>}
    </button>
  );
}

function DetailRow({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex gap-2 border-b-2 border-foreground py-2">
      <span className="w-24 shrink-0 font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
        {label}
      </span>
      <div className="min-w-0 flex-1 break-words">{children}</div>
    </div>
  );
}

function LensPlaceholder({ title, phase }: { title: string; phase: string }) {
  return (
    <section className="press-lg mx-auto mt-6 max-w-2xl border-4 border-foreground bg-card p-8 text-center">
      <div className="font-display text-3xl text-stencil">{title}</div>
      <p className="mt-2 font-tech text-[11px] uppercase tracking-widest text-muted-foreground">
        lente pendiente · {phase}
      </p>
    </section>
  );
}

export default function WorkspaceShell({
  state,
  actions,
  notice,
  onNotice,
  onDismissNotice,
  onError,
  dark,
  onToggleTheme,
}: WorkspaceShellProps) {
  const [surface, setSurface] = useState<Surface>("resumen");
  const [selected, setSelected] = useState<InspectableRecord | null>(null);

  // Búsqueda global (Topbar) — hereda la funcionalidad de SearchBar.
  const searchRef = useRef<HTMLInputElement>(null);
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[] | null>(null);
  const [searching, setSearching] = useState(false);

  const namespaces = useNamespaceCounts(!!state.active);

  // Ctrl+K / ⌘K → foco en la búsqueda global (VS-09 implementa la palette real).
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        searchRef.current?.focus();
        searchRef.current?.select();
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  async function handleSearch(e: FormEvent) {
    e.preventDefault();
    const q = query.trim();
    if (!q) return;
    setSearching(true);
    try {
      setResults(await search({ query: q, top_k: 8 }));
    } catch (err) {
      onError(vantaErrorMessage(err));
    } finally {
      setSearching(false);
    }
  }

  function openRecord(r: InspectableRecord) {
    setSelected(r);
    setResults(null);
  }

  const closeInspector = () => setSelected(null);

  return (
    <div className="fixed inset-0 flex overflow-hidden bg-background text-foreground">
      {/* ========== SIDEBAR ========== */}
      <aside
        className="flex w-60 shrink-0 flex-col border-r-4 border-foreground bg-background"
        aria-label="Panel lateral"
      >
        {/* Brand */}
        <div className="border-b-4 border-foreground p-4">
          <div className="flex items-center gap-2">
            <div className="flex h-8 w-8 items-center justify-center border-2 border-foreground bg-neon font-display text-lg leading-none text-background shadow-[2px_2px_0_0_#000]">
              ◆
            </div>
            <div className="leading-none">
              <div className="font-display text-xl tracking-wide">Vanta Studio</div>
              <div className="font-tech text-[9px] uppercase tracking-widest text-muted-foreground">
                memory workspace
              </div>
            </div>
          </div>
        </div>

        {/* Nav */}
        <nav className="flex-1 overflow-y-auto scroll-manga p-3" aria-label="Navegación principal">
          <div className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
            Workspace
          </div>
          <div className="mt-2 space-y-2">
            <SideButton icon="◫" label="RESUMEN" active={surface === "resumen"} onClick={() => setSurface("resumen")} />
            <SideButton icon="▦" label="MEMORIAS" active={surface === "memorias"} onClick={() => setSurface("memorias")} />
            <SideButton icon="◷" label="ACTIVITY" hint="F1" active={surface === "actividad"} onClick={() => setSurface("actividad")} />
            <SideButton icon="⠿" label="ÍNDICES" hint="F1" active={surface === "indices"} onClick={() => setSurface("indices")} />
            <SideButton icon="⌘" label="IQL" hint="F2" active={surface === "iql"} onClick={() => setSurface("iql")} />
          </div>

          <div className="mt-6 font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
            Namespaces
          </div>
          <div className="mt-2 space-y-2">
            {namespaces.length === 0 ? (
              <p className="font-tech text-[10px] text-muted-foreground">sin registros</p>
            ) : (
              namespaces.map((n) => (
                <button
                  key={n.name}
                  type="button"
                  onClick={() => setSurface("memorias")}
                  className="press flex w-full items-center justify-between gap-2 border-2 border-foreground bg-background px-3 py-2 text-left text-sm"
                  title={`Ver ${n.name} en MEMORIAS`}
                >
                  <span className="truncate">{n.name}</span>
                  <span className="font-display text-base leading-none">{n.count}</span>
                </button>
              ))
            )}
          </div>
        </nav>

        {/* Footer */}
        <div className="border-t-4 border-foreground p-3">
          <div className="flex items-center justify-between gap-2">
            <div className="font-tech text-[9px] uppercase tracking-wider text-muted-foreground">
              v0.1.0 · <span className="text-neon">embedded</span>
            </div>
            <button
              type="button"
              onClick={onToggleTheme}
              className="press flex h-7 w-7 items-center justify-center border-2 border-foreground text-sm"
              title="Cambiar tema"
              aria-label="Cambiar tema claro/oscuro"
            >
              {dark ? "☀" : "☾"}
            </button>
          </div>
        </div>
      </aside>

      {/* ========== MAIN ========== */}
      <div className="flex flex-1 flex-col overflow-hidden">
        {/* ========== TOPBAR ========== */}
        <header className="flex items-center gap-3 border-b-4 border-foreground px-4 py-3">
          <button
            type="button"
            onClick={() => setSurface("resumen")}
            className="press flex items-center gap-2 border-2 border-foreground bg-background px-3 py-1.5 text-xs font-semibold"
            title="Namespace activo — volver a RESUMEN"
          >
            <span className="text-neon">◆</span>
            <span className="max-w-[140px] truncate">
              {state.active ? state.active.name : "sin backend"}
            </span>
          </button>

          <form className="relative flex-1" onSubmit={handleSearch} role="search">
            <input
              ref={searchRef}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              type="search"
              placeholder="Buscar memoria… (Ctrl+K)"
              aria-label="Búsqueda global"
              className="w-full border-2 border-foreground bg-background px-3 py-1.5 pl-9 text-sm placeholder:text-muted-foreground"
            />
            <span className="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-neon">
              🔎
            </span>
            {searching && (
              <span className="absolute right-2 top-1/2 -translate-y-1/2 font-tech text-[10px] text-muted-foreground">
                …
              </span>
            )}
          </form>

          <div className="flex items-center gap-2">
            <div className="hidden items-center gap-1 border-2 border-foreground bg-muted px-2 py-1 font-tech text-[10px] uppercase md:flex">
              <span
                className={`h-1.5 w-1.5 rounded-full ${
                  state.healthStatus === "ok" ? "bg-neon animate-pulse-ring" : "bg-muted-foreground"
                }`}
              />
              {state.healthStatus === "ok" ? "BM25 · HNSW · RRF" : "OFFLINE"}
            </div>
            <kbd className="hidden border-2 border-foreground bg-background px-2 py-1 font-tech text-[10px] uppercase md:inline">
              Ctrl+K
            </kbd>
            <button
              type="button"
              onClick={() => setSurface("memorias")}
              className="btn-neon-glow border-2 border-foreground bg-neon px-3 py-1.5 text-xs font-bold text-background"
            >
              + INGEST
            </button>
          </div>
        </header>

        {notice && (
          <div
            role="alert"
            onClick={onDismissNotice}
            className="cursor-pointer border-b-4 border-neon bg-card px-4 py-2 text-sm"
          >
            {notice}
          </div>
        )}

        {/* ========== CENTRAL SURFACE ========== */}
        <main className="flex-1 overflow-y-auto scroll-manga">
          {results !== null && (
            <section className="border-b-4 border-foreground bg-card">
              <div className="mx-auto max-w-6xl p-4">
                <div className="flex items-center justify-between">
                  <span className="font-tech text-[10px] uppercase tracking-widest text-neon">
                    Resultados de búsqueda
                  </span>
                  <button
                    type="button"
                    onClick={() => setResults(null)}
                    className="press border-2 border-foreground bg-background px-2 py-1 text-xs"
                  >
                    ✕ cerrar
                  </button>
                </div>
                <ResultsList results={results} onSelect={(r) => openRecord(r)} />
              </div>
            </section>
          )}

          {surface === "resumen" && (
            <div className="mx-auto max-w-5xl space-y-5 p-6">
              {!state.active && (
                <section className="press-lg border-4 border-foreground bg-card p-6">
                  <div className="relative mx-auto aspect-square w-full max-w-[200px]">
                    <MarkStudio status="error" />
                  </div>
                  <p className="mt-2 text-center font-tech text-[11px] uppercase tracking-widest text-muted-foreground">
                    sin backend activo — conectá uno para operar
                  </p>
                </section>
              )}
              <ConnectionPanel
                connections={state.connections}
                activeId={state.activeId}
                health={state.health}
                healthStatus={state.healthStatus}
                busy={state.busy}
                onConnectNative={actions.connectNativePath}
                onDisconnect={actions.disconnectId}
                onActivate={actions.activate}
                onProbeHealth={actions.probeHealth}
              />
              <MetricsGrid
                health={state.health}
                healthStatus={state.healthStatus}
                activeName={state.active ? state.active.name : null}
              />
              <KpiCards />
              <SopPanel />
              <ExportPanel />
            </div>
          )}

          {surface === "memorias" && (
            <div className="mx-auto max-w-6xl space-y-5 p-6">
              <IngestForm onDone={(ids) => onNotice(`Stored ${ids.length} record(s).`)} runError={onError} />
              <DataExplorer
                active={!!state.active}
                busy={state.busy}
                runError={onError}
                onSelectRow={(row: ExplorerRow) =>
                  openRecord({ id: row.id, namespace: row.namespace, text: row.text, score: row.score })
                }
              />
            </div>
          )}

          {surface === "actividad" && (
            <div className="mx-auto max-w-5xl p-6">
              <ProcessPanel
                connections={state.connections}
                activeId={state.activeId}
                onShutdown={actions.disconnectId}
                onActivate={actions.activate}
              />
            </div>
          )}

          {surface === "indices" && <LensPlaceholder title="ÍNDICES" phase="Fase 1" />}
          {surface === "iql" && <LensPlaceholder title="IQL" phase="Fase 2" />}
        </main>
      </div>

      {/* ========== INSPECTOR (master-detail, derecha) ========== */}
      {selected && (
        <aside
          className="flex w-[400px] shrink-0 flex-col overflow-hidden border-l-4 border-foreground bg-card"
          aria-label="Inspector de registro"
        >
          <div className="flex items-center justify-between border-b-4 border-foreground px-4 py-3">
            <span className="font-tech text-[10px] uppercase tracking-widest text-neon">Inspector</span>
            <button
              type="button"
              onClick={closeInspector}
              className="press flex h-6 w-6 items-center justify-center border-2 border-foreground text-xs"
              aria-label="Cerrar inspector"
            >
              ✕
            </button>
          </div>
          <div className="flex-1 overflow-y-auto scroll-manga p-4">
            <DetailRow label="key">
              <code className="text-sm">{selected.id}</code>
            </DetailRow>
            <DetailRow label="namespace">
              <span className="border-2 border-foreground bg-background px-2 py-0.5 font-tech text-[11px]">
                {selected.namespace}
              </span>
            </DetailRow>
            {selected.score != null && (
              <DetailRow label="score">
                <span className="text-neon">{selected.score.toFixed(3)}</span>
              </DetailRow>
            )}
            <DetailRow label="text">
              <p className="text-sm">{selected.text}</p>
            </DetailRow>
            {selected.metadata && Object.keys(selected.metadata).length > 0 && (
              <DetailRow label="metadata">
                <pre className="whitespace-pre-wrap font-mono text-[11px]">
                  {JSON.stringify(selected.metadata, null, 2)}
                </pre>
              </DetailRow>
            )}
            {selected.created_at_ms != null && (
              <DetailRow label="created">
                <span className="text-sm">{new Date(selected.created_at_ms).toLocaleString()}</span>
              </DetailRow>
            )}
            <p className="mt-4 font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
              inspector base — edición completa en VS-06
            </p>
          </div>
        </aside>
      )}
    </div>
  );
}