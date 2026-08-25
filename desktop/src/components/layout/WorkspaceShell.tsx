// Workspace shell (VS-03): Sidebar + Topbar + central surface + right Inspector.
// Layout/estética replican el prototipo VS-00 (desktop/prototype/index.html) con
// tokens/utilities de VS-01 (--color-neon, .press, .scroll-manga, font-display,
// font-tech, dark overrides). Los paneles legacy (MetricsGrid/KpiCards/SopPanel/
// ConnectionPanel/IngestForm/DataExplorer/ExportPanel) se reubican
// como superficies/lentes — funcionalidad intacta, nada se borra.
//
// Superficies: RESUMEN (ops: connections/metrics/KPIs/SOP/export) · MEMORIAS
// (ingest + grid master) · ACTIVITY (audit log: ActivityPanel + Timeline, VS-15)
// · ÍNDICES/IQL (lentes pendientes Fase 1/2). Búsqueda global en Topbar (reemplaza
// SearchBar: misma llamada search() + ResultsList). Inspector derecho = master-detail del
// registro seleccionado (VS-06: tabs General/Metadata/Vector/Payload con
// commit explícito — grid pasa el record completo, búsqueda lo completa vía get).
import { FormEvent, lazy, ReactNode, Suspense, useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { RuleGroupType } from "react-querybuilder";
// FIX-D3a: glifos con presentación-EMOJI en Windows (♻ ⚙ ☀ ☾ 🔎 🗑 ✳) → Lucide.
// Los glifos geométricos monocromos (◆ ▫ ▦ ◷ ⛁ ⠿ ⇄ ⌘ ◉ ⇋ ★ ✕ ⧩ ⤒ ⤓ ─ □)
// son identidad linocut y se quedan. DAUD-06: ✎ (renombrar) → Pencil Lucide.
import { Asterisk, Moon, Pencil, Search, Settings as SettingsIcon, Sun, Trash2 } from "lucide-react";
import { HelpPanel } from "./HelpPanel";
import { createNamespace, get, list, namespaceStats, search, SearchResult, vantaErrorMessage, type MemoryRecord, type NamespaceStatsMap, type VantaDeepLink } from "../../vanta";
import { useDeepLink } from "../../hooks/useDeepLink";
import { ConnectionActions, VantaState } from "../../hooks/useConnectionState";
import { EMPTY_QUERY, evaluateQuery, inferMetaFields, toVantaMemoryFilter } from "../search/filters-core";
import ConnectionPanel from "../ConnectionPanel";
import IngestForm from "../IngestForm";
import KpiCards from "../KpiCards";
import MetricsGrid from "../MetricsGrid";
import DataExplorer, { ExplorerRow } from "../DataExplorer";
import SopPanel from "../SopPanel";
import ExportPanel from "../ExportPanel";
import ActivityPanel from "../activity/ActivityPanel";
import ResultsList from "../ResultsList";
import { MarkStudio } from "../mark/mark-studio";
import HomeOverview from "../home/HomeOverview";
import TrashLens from "../trash/TrashLens";
// VS-13: lente RETRIEVAL — liviana (FiltersBuilder ya es lazy dentro), se
// importa estática para que la surface responda al instante.
import RetrievalLens from "../lens/retrieval/RetrievalLens";
// FEAT-02: lente ÍNDICES real (reemplaza el placeholder VS-03) — liviana,
// sin dependencias pesadas, import estática como RETRIEVAL.
import IndicesLens from "../indices/IndicesLens";
// FEAT-03a: lente CONSOLIDAR (D16 (a) UI-only) — liviana, import estática.
import ConsolidateLens from "../consolidate/ConsolidateLens";
import { undoStore } from "../../store/undo";
// DESKTOP-32: modal de CRUD de namespaces (crear/renombrar/borrar con
// confirmación en 2 pasos).
import NamespaceDialog, { type NsDialog } from "./NamespaceDialog";
// VS-17: favoritos + historial de búsqueda (localStorage, slice aditivo).
import { favoritesStore, type Favorite } from "../../store/favorites";
import { searchHistory } from "../../store/search-history";
// DESKTOP-23: preferencias de workspace persistidas (surface + filtros).
import { workspacePrefs } from "../../store/preferences";
// DESKTOP-31: perfiles de conexión + defaults de búsqueda persistidos.
import { connectionPrefs, type ConnectionProfile } from "../../store/connections";
// DESKTOP-37: lente MEMORIA — liviana (solo listas read-only), import estática
// como RETRIEVAL/ÍNDICES/CONSOLIDAR.
import MemoryLens from "../memory/MemoryLens";
// DESKTOP-38: lente PROXY — dashboard REST del proxy local; liviana (fetch +
// tablas), import estática. `proxyUrl`/PROXY_URL_EVENT condicionan el botón.
import ProxyDashboard, { proxyUrl, PROXY_URL_EVENT } from "../proxy/ProxyDashboard";
import Settings from "../../pages/Settings";
// CodeMirror/react-markdown pesan (~600 kB) y solo los usa el Inspector → chunk
// lazy: el shell inicial no paga ese coste (Tauri local, carga on-demand).
const Inspector = lazy(() => import("../inspector/Inspector"));
// react-querybuilder (~200 kB) solo lo abre el panel de filtros → lazy igual.
const FiltersBuilder = lazy(() => import("../search/FiltersBuilder"));
// CommandPalette (VS-09) + cmdk (~12 kB gzip): chunk lazy, se monta al abrir.
const CommandPalette = lazy(() => import("../palette/CommandPalette"));
// ImportPaste (OP-01): modal de import CSV/JSON pegado — chunk lazy, solo el
// botón IMPORT lo monta.
const ImportPaste = lazy(() => import("../ingest/ImportPaste"));
// ImportDrop (WASM-04): modal de import por ARCHIVO (drag&drop .vdbdump/.jsonl/
// .csv) — mismo chunk lazy, lo monta el botón IMPORT ARCHIVO.
const ImportDrop = lazy(() => import("../ingest/ImportDrop"));
// GraphLens (GRAFO-02): three + drei + r3f pesan ~600 kB → chunk lazy, solo
// la surface IQL los paga (mitigación "Riesgos" del plan; mismo patrón).
const GraphLens = lazy(() => import("../graph/GraphLens"));
// SpaceLens (ESPACIO-01): regl-scatterplot + regl pesan ~200 kB → chunk lazy,
// solo la surface ESPACIO los paga (mismo patrón que GraphLens/Inspector).
const SpaceLens = lazy(() => import("../space/SpaceLens"));

export type Surface = "resumen" | "memorias" | "papelera" | "actividad" | "retrieval" | "indices" | "consolidar" | "iql" | "espacio" | "memoria" | "proxy" | "ajustes";

interface NamespaceCount {
  name: string;
  count: number;
}

/** Selección del Inspector (master-detail): record completo (VS-11) + score
 * opcional (resultados de búsqueda global). */
interface InspectorSelection {
  record: MemoryRecord;
  score: number | null;
}

interface WorkspaceShellProps {
  /** WEB-05: true outside Tauri (web build) — hides Tauri-only connection UI. */
  embedded?: boolean;
  state: VantaState;
  actions: ConnectionActions;
  notice: string | null;
  onNotice: (msg: string) => void;
  onDismissNotice: () => void;
  onError: (msg: string) => void;
  dark: boolean;
  onToggleTheme: () => void;
}

/** Conteos por namespace: stats reales del bridge (VS-CORE-02) con fallback
 * client-side desde list(limit 500) solo cuando el backend no las expone.
 * DESKTOP-32: `refresh` fuerza re-fetch tras crear/renombrar/borrar. */
function useNamespaceCounts(active: boolean, refresh: number): NamespaceCount[] {
  const [namespaces, setNamespaces] = useState<NamespaceCount[]>([]);

  useEffect(() => {
    let alive = true;
    if (!active) {
      setNamespaces([]);
      return;
    }
    const fromStats = (stats: NamespaceStatsMap): NamespaceCount[] =>
      Object.entries(stats)
        .map(([name, s]) => ({ name, count: s.count }))
        .sort((a, b) => b.count - a.count);
    const fromList = (records: MemoryRecord[]): NamespaceCount[] => {
      const counts = new Map<string, number>();
      for (const r of records) {
        counts.set(r.namespace, (counts.get(r.namespace) ?? 0) + 1);
      }
      return [...counts.entries()]
        .map(([name, count]) => ({ name, count }))
        .sort((a, b) => b.count - a.count);
    };
    namespaceStats()
      .then((stats) => {
        if (alive) setNamespaces(fromStats(stats));
      })
      .catch(() => {
        // Fallback: transporte sin stats (o error transitorio) → conteo local.
        if (!alive) return;
        list({ limit: 500 })
          .then((records) => {
            if (alive) setNamespaces(fromList(records));
          })
          .catch(() => {
            if (alive) setNamespaces([]);
          });
      });
    return () => {
      alive = false;
    };
  }, [active, refresh]);

  return namespaces;
}

function SideButton({
  icon,
  label,
  hint,
  title,
  active,
  onClick,
}: {
  icon: ReactNode;
  label: string;
  hint?: string;
  /** DESKTOP-34: tooltip nativo (title) + accesible (aria-label). */
  title?: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-current={active ? "page" : undefined}
      aria-label={title ?? label}
      title={title ?? label}
      className={`press flex w-full items-center gap-3 border-2 border-foreground px-3 py-2 text-left text-sm font-semibold ${
        active ? "bg-foreground text-background" : "bg-background"
      }`}
    >
      <span className="text-neon">{icon}</span>
      {label}
      {hint && <span className="ml-auto font-tech text-[11px] text-accent-text">{hint}</span>}
    </button>
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
  embedded = false,
  state,
  actions,
  notice,
  onNotice,
  onDismissNotice,
  onError,
  dark,
  onToggleTheme,
}: WorkspaceShellProps) {
  // DESKTOP-23: hidratación única de preferencias (surface/filtros) al montar.
  const prefs = workspacePrefs.get();
  const [surface, setSurface] = useState<Surface>(
    (prefs.surface as Surface | undefined) ?? "resumen",
  );
  const [selected, setSelected] = useState<InspectorSelection | null>(null);

  // OP-01: modal de import CSV/JSON + remount del grid tras importar.
  const [importOpen, setImportOpen] = useState(false);
  const [importFileOpen, setImportFileOpen] = useState(false);
  const [gridKey, setGridKey] = useState(0);

  // Filtros compuestos (VS-07): query builder AND/OR sobre metadata tipada.
  // El estado vive en el shell → sobrevive a cerrar el panel y alimenta la
  // búsqueda global; los campos se infieren de los resultados actuales.
  // DESKTOP-23: estado inicial hidratado de preferencias persistidas.
  const [ruleGroup, setRuleGroup] = useState<RuleGroupType>(prefs.ruleGroup ?? EMPTY_QUERY);
  const [showFilters, setShowFilters] = useState<boolean>(prefs.showFilters ?? false);

  // DESKTOP-23: write-through de layout/filtros — reiniciar la app los conserva.
  useEffect(() => {
    workspacePrefs.set({ surface, showFilters, ruleGroup });
  }, [surface, showFilters, ruleGroup]);

  // Búsqueda global (Topbar) — hereda la funcionalidad de SearchBar.
  const searchRef = useRef<HTMLInputElement>(null);
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[] | null>(null);
  const [searching, setSearching] = useState(false);

  // DESKTOP-32: CRUD de namespaces — modal + re-fetch de conteos tras cada op.
  const [nsRefresh, setNsRefresh] = useState(0);
  const [nsDialog, setNsDialog] = useState<NsDialog | null>(null);
  const namespaces = useNamespaceCounts(!!state.active, nsRefresh);

  async function handleCreateNs(name: string) {
    try {
      await createNamespace(name);
      setNsRefresh((k) => k + 1);
      onNotice(`Namespace "${name}" creado.`);
    } catch (err) {
      onError(vantaErrorMessage(err));
    }
  }

  async function handleRenameNs(from: string, to: string) {
    try {
      const n = await undoStore.renameNamespace(from, to);
      setNsRefresh((k) => k + 1);
      onNotice(`"${from}" renombrado a "${to}" (${n} registros) — Ctrl+Z para deshacer.`);
    } catch (err) {
      onError(vantaErrorMessage(err));
    }
  }

  async function handleDeleteNs(name: string) {
    try {
      const n = await undoStore.deleteNamespace(name);
      setNsRefresh((k) => k + 1);
      onNotice(`"${name}" movido a papelera (${n} registros) — Ctrl+Z para deshacer.`);
    } catch (err) {
      onError(vantaErrorMessage(err));
    }
  }

  // VS-17: favoritos (sidebar + palette) e historial (palette) reactivos a los
  // stores de localStorage — mismo patrón de suscripción que undo/VS-08.
  const [favorites, setFavorites] = useState<Favorite[]>(favoritesStore.getFavorites());
  useEffect(() => favoritesStore.subscribe(() => setFavorites(favoritesStore.getFavorites())), []);
  const [history, setHistory] = useState<string[]>(searchHistory.get());
  useEffect(() => searchHistory.subscribe(() => setHistory(searchHistory.get())), []);

  // DESKTOP-38: botón PROXY solo con proxy configurado; el dashboard dispara
  // PROXY_URL_EVENT al guardar/borrar la URL para refrescar el sidebar.
  const [proxyConfigured, setProxyConfigured] = useState(!!proxyUrl());
  useEffect(() => {
    const sync = () => setProxyConfigured(!!proxyUrl());
    window.addEventListener(PROXY_URL_EVENT, sync);
    return () => window.removeEventListener(PROXY_URL_EVENT, sync);
  }, []);

  const filterActive = ruleGroup.rules.length > 0;
  const filterFields = useMemo(() => (results ? inferMetaFields(results) : []), [results]);
  // Filtros se aplican client-side sobre los hits de la búsqueda híbrida global
  // (el wire del bridge solo admite el map plano Eq — VS-07 no toca src-tauri).
  const visibleResults = useMemo(() => {
    if (!results || !filterActive) return results;
    return results.filter((r) => evaluateQuery(ruleGroup, r.metadata ?? {}));
  }, [results, ruleGroup, filterActive]);

  // VS-08 (Fix 4): acciones reales del store de undo/papelera. VS-09 los llama
  // desde la palette con esta misma firma (los stubs previos se reemplazan).
  async function handleUndo() {
    try {
      const label = await undoStore.undo();
      onNotice(label);
    } catch (err) {
      onError(vantaErrorMessage(err));
    }
  }

  async function handleDelete() {
    const sel = selected;
    if (!sel) {
      onNotice("Seleccioná un registro para borrarlo (grid → inspector)");
      return;
    }
    try {
      await undoStore.softDelete(sel.record);
      setSelected(null);
      onNotice(`movido a papelera ${sel.record.id}`);
    } catch (err) {
      onError(vantaErrorMessage(err));
    }
  }

  // Ctrl+K / ⌘K global → abre/cierra la command palette (VS-09). El kbd del
  // Topbar anuncia el atajo; la búsqueda global sigue disponible en su input.
  // Ctrl+Z / ⌘Z → undo de sesión (VS-08): deshace la última mutación destructiva
  // (delete/restore/purge). Se saltea inputs/CodeMirror para no pisar el undo
  // nativo de texto del navegador.
  const [paletteOpen, setPaletteOpen] = useState(false);
  // FIND-25: in-app usage guide, toggled with "?".
  const [helpOpen, setHelpOpen] = useState(false);

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      const ctrl = e.ctrlKey || e.metaKey;
      if (ctrl && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setPaletteOpen((o) => !o);
        return;
      }
      if (ctrl && e.key.toLowerCase() === "z") {
        const t = e.target as HTMLElement | null;
        if (t && (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.isContentEditable)) return;
        e.preventDefault();
        void handleUndo();
        return;
      }
      // FIND-25: "?" opens the usage guide (skip inputs/CodeMirror).
      if (e.key === "?") {
        const t = e.target as HTMLElement | null;
        if (t && (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.isContentEditable)) return;
        e.preventDefault();
        setHelpOpen((o) => !o);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  async function runSearch(q: string) {
    const qTrim = q.trim();
    if (!qTrim) return;
    // VS-17: grabar en el historial (funnel único de topbar + palette).
    searchHistory.add(qTrim);
    setSearching(true);
    try {
      // DESKTOP-31: default top_k del store de conexiones (Settings → topbar).
      const { topK } = connectionPrefs.get();
      // Con filtro activo pedimos más hits: el filtrado es client-side, así el
      // subconjunto resultante no se vacía con top_k=8.
      setResults(await search({ query: qTrim, top_k: filterActive ? 50 : (topK ?? 8) }));
    } catch (err) {
      onError(vantaErrorMessage(err));
    } finally {
      setSearching(false);
    }
  }

  function handleSearch(e: FormEvent) {
    e.preventDefault();
    runSearch(query);
  }

  /** Palette "buscar key": query vacío → foco en la búsqueda global; con texto
   * → MEMORIAS + búsqueda global (resultados encima de la superficie). */
  function handlePaletteSearch(q: string) {
    setSurface("memorias");
    if (q.trim()) {
      setQuery(q);
      runSearch(q);
    } else {
      requestAnimationFrame(() => searchRef.current?.focus());
    }
  }

  function openRecord(record: MemoryRecord, score: number | null) {
    setSelected({ record, score });
    setResults(null);
  }

  /** DESKTOP-31: conectar vía perfil guardado (nativo o server + Bearer).
   * Cierra la conexión activa implícitamente: el backend abre la nueva y la
   * activa; el perfil queda marcado como activo en el store persistido. */
  async function useProfile(p: ConnectionProfile) {
    const id =
      p.kind === "native"
        ? await actions.connectNativePath(p.path ?? "")
        : await actions.connectServerCfg(p.url ?? "", p.port ?? 8080, p.token ?? "");
    if (id) {
      connectionPrefs.set({ activeProfileId: p.id });
      onNotice(`Conectado vía perfil "${p.name}".`);
    }
  }

  /** Búsqueda global: el SearchResult no trae version/vector/node_id — se
   * completa con `get()`; si falla, se abre un record mínimo con lo disponible. */
  async function openSearchResult(r: SearchResult) {
    try {
      const full = await get(r.id, r.namespace);
      openRecord(full, r.score);
    } catch (err) {
      onError(vantaErrorMessage(err));
      openRecord(
        { id: r.id, namespace: r.namespace, text: r.text, metadata: r.metadata } as MemoryRecord,
        r.score,
      );
    }
  }

  const closeInspector = () => setSelected(null);

  /** VS-15: evento de la tabla/timeline de ACTIVITY → registro completo en el
   * Inspector. Los eventos export/import usan `key: "N/A"` (sin record) — el
   * panel ya los intercepta con un notice antes de llegar acá. */
  async function handleInspectAudit(namespace: string, key: string) {
    try {
      const full = await get(key, namespace);
      openRecord(full, null);
    } catch (err) {
      onError(vantaErrorMessage(err));
    }
  }

  /** VS-17: favorito de palette/sidebar. key → abrir registro en Inspector;
   * namespace (key null) → superficie MEMORIAS (como los botones del sidebar). */
  function handleOpenFavorite(fav: Favorite) {
    if (fav.key) {
      void (async () => {
        try {
          openRecord(await get(fav.key!, fav.namespace), null);
        } catch (err) {
          onError(vantaErrorMessage(err));
        }
      })();
    } else {
      setSurface("memorias");
    }
  }

  // VS-16: deep links `vanta://`. Refs "latest" (mismo patrón VS-08) para que
  // el callback pasado a useDeepLink sea estable — el listener no se
  // re-suscribe en cada render.
  const dlRefs = useRef({
    setSurface,
    runSearch,
    openRecord,
    get,
    onError,
    onNotice,
  });
  dlRefs.current = { setSurface, runSearch, openRecord, get, onError, onNotice };

  const handleDeepLink = useCallback((link: VantaDeepLink) => {
    const { setSurface, runSearch, openRecord, get, onError, onNotice } = dlRefs.current;
    setSurface("memorias");

    // vanta://ns/key → abrir el registro en el Inspector.
    if (link.namespace && link.key) {
      void (async () => {
        try {
          const full = await get(link.key!, link.namespace!);
          openRecord(full, null);
        } catch (err) {
          onError(vantaErrorMessage(err));
        }
      })();
      return;
    }

    // vanta://?query=x o vanta://ns?query=x → búsqueda global sobre MEMORIAS.
    if (link.query) {
      runSearch(link.query);
      return;
    }

    // vanta://ns → superficie MEMORIAS (el grid lista sin filtro de namespace;
    // los conteos de la sidebar ya muestran qué contiene).
    onNotice(`vanta://${link.namespace ?? ""} — abriendo MEMORIAS`);
  }, []);

  useDeepLink(handleDeepLink);

  return (
    // FIND-19: flex-1 (not fixed inset-0) — the shell lives inside App's
    // column layout, below the custom TitleBar. fixed inset-0 covered it.
    <div className="relative flex min-h-0 flex-1 overflow-hidden bg-background text-foreground">
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
              <div className="font-tech text-[11px] uppercase tracking-widest text-muted-foreground">
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
            <SideButton icon="◫" label="RESUMEN" title="Ir a RESUMEN — vista general de operaciones" active={surface === "resumen"} onClick={() => setSurface("resumen")} />
            <SideButton icon="▦" label="MEMORIAS" title="Ir a MEMORIAS — ingestar y explorar registros" active={surface === "memorias"} onClick={() => setSurface("memorias")} />
            <SideButton icon={<Trash2 className="h-4 w-4" strokeWidth={2.5} />} label="PAPELERA" hint="Ctrl+Z" title="Ir a PAPELERA — registros borrados (restaurar o purgar)" active={surface === "papelera"} onClick={() => setSurface("papelera")} />
            <SideButton icon="◷" label="ACTIVIDAD" title="Ir a ACTIVIDAD — audit log de la base" active={surface === "actividad"} onClick={() => setSurface("actividad")} />
            {/* VS-13: lente contextual — hereda el registro seleccionado como seed (P4). */}
            <SideButton icon="⛁" label="BÚSQUEDA" title="Ir a BÚSQUEDA — lente retrieval híbrida (BM25 + vector)" active={surface === "retrieval"} onClick={() => setSurface("retrieval")} />
            <SideButton icon="⠿" label="ÍNDICES" title="Ir a ÍNDICES — estado de HNSW, BM25 y WAL" active={surface === "indices"} onClick={() => setSurface("indices")} />
            <SideButton icon="⇄" label="CONSOLIDAR" title="Ir a CONSOLIDAR — detectar y fusionar duplicados" active={surface === "consolidar"} onClick={() => setSurface("consolidar")} />
            <SideButton icon="⌘" label="IQL" title="Ir a IQL — consola de queries sobre grafo" active={surface === "iql"} onClick={() => setSurface("iql")} />
            <SideButton icon={<Asterisk className="h-4 w-4" strokeWidth={2.5} />} label="ESPACIO" title="Ir a ESPACIO — proyección 2D de embeddings" active={surface === "espacio"} onClick={() => setSurface("espacio")} />
            {/* DESKTOP-37: sexta lente — memoria contextual de vanta-memory. */}
            <SideButton icon="◉" label="MEMORIA" title="Ir a MEMORIA — escenas con heat, persona, skills versionadas y generation log (L1/L2/L3)" active={surface === "memoria"} onClick={() => setSurface("memoria")} />
            {/* DESKTOP-38: lente PROXY — solo si el proxy está configurado. */}
            {proxyConfigured && (
              <SideButton icon="⇋" label="PROXY" title="Ir a PROXY — TurnReports, sesiones activas, cola write-back y rate-limit del proxy local" active={surface === "proxy"} onClick={() => setSurface("proxy")} />
            )}
            {/* DESKTOP-31: ajustes — perfiles de conexión, defaults de búsqueda, idioma. */}
            <SideButton icon={<SettingsIcon className="h-4 w-4" strokeWidth={2.5} />} label="AJUSTES" title="Ir a AJUSTES — perfiles de conexión (server + Bearer), defaults de búsqueda e idioma" active={surface === "ajustes"} onClick={() => setSurface("ajustes")} />
          </div>

          {/* VS-17: favoritos persistidos (ns o ns/key) — slice aditivo. */}
          <div className="mt-6 font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
            Favoritos
          </div>
          <div className="mt-2 space-y-2">
            {favorites.length === 0 ? (
              <p className="font-tech text-[10px] text-muted-foreground">sin favoritos — usá ★</p>
            ) : (
              favorites.map((f) => {
                const label = f.key ? `${f.namespace}/${f.key}` : f.namespace;
                return (
                  <div key={`${f.namespace}:${f.key ?? ""}`} className="flex items-stretch gap-1">
                    <button
                      type="button"
                      onClick={() => handleOpenFavorite(f)}
                      className="press flex flex-1 items-center gap-2 border-2 border-foreground bg-background px-3 py-2 text-left text-sm"
                      title={`Abrir ${label}`}
                    >
                      <span className="text-neon">★</span>
                      <span className="truncate">{label}</span>
                    </button>
                    <button
                      type="button"
                      onClick={() => favoritesStore.toggle(f.namespace, f.key)}
                      className="press flex w-9 items-center justify-center border-2 border-foreground text-[10px]"
                      title={`Quitar ${label} de favoritos`}
                      aria-label={`Quitar ${label} de favoritos`}
                    >
                      ✕
                    </button>
                  </div>
                );
              })
            )}
          </div>

          <div className="mt-6 flex items-center justify-between">
            <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
              Namespaces
            </span>
            <button
              type="button"
              onClick={() => setNsDialog({ mode: "create" })}
              disabled={!state.active}
              className="press flex h-8 w-8 items-center justify-center border-2 border-foreground text-sm leading-none"
              title={state.active ? "Crear namespace vacío" : "Conectá un backend primero"}
              aria-label="Crear namespace"
            >
              +
            </button>
          </div>
          <div className="mt-2 space-y-2">
            {namespaces.length === 0 ? (
              <p className="font-tech text-[10px] text-muted-foreground">sin registros</p>
            ) : (
              namespaces.map((n) => {
                const fav = favoritesStore.isFavorite(n.name, null);
                return (
                  <div key={n.name} className="group flex items-stretch gap-1">
                    <button
                      type="button"
                      onClick={() => setSurface("memorias")}
                      className="press flex min-w-0 flex-1 items-center justify-between gap-2 border-2 border-foreground bg-background px-3 py-2 text-left text-sm"
                      title={`Ver ${n.name} en MEMORIAS`}
                    >
                      <span className="truncate">{n.name}</span>
                      <span className="font-display text-base leading-none">{n.count}</span>
                    </button>
                    {/* DESKTOP-32: renombrar / borrar con confirmación + undo.
                        Acciones reveladas al hover/focus: el nombre respira y
                        la fila no compite con los iconos (visual-critique). */}
                    <button
                      type="button"
                      onClick={() => setNsDialog({ mode: "rename", name: n.name })}
                      className="press flex w-9 items-center justify-center border-2 border-foreground bg-background text-xs opacity-0 transition-opacity focus-visible:opacity-100 group-hover:opacity-100"
                      title={`Renombrar ${n.name}`}
                      aria-label={`Renombrar ${n.name}`}
                    >
                      <Pencil className="h-3.5 w-3.5" strokeWidth={2.5} />
                    </button>
                    <button
                      type="button"
                      onClick={() => setNsDialog({ mode: "delete", name: n.name })}
                      className="press flex w-9 items-center justify-center border-2 border-foreground bg-background text-xs opacity-0 transition-opacity focus-visible:opacity-100 group-hover:opacity-100"
                      title={`Borrar ${n.name} (va a la papelera)`}
                      aria-label={`Borrar ${n.name}`}
                    >
                      <Trash2 className="h-3.5 w-3.5" strokeWidth={2.5} />
                    </button>
                    <button
                      type="button"
                      onClick={() => favoritesStore.toggle(n.name, null)}
                      aria-pressed={fav}
                      className={`press flex w-9 items-center justify-center border-2 border-foreground text-sm transition-opacity focus-visible:opacity-100 group-hover:opacity-100 ${
                        fav ? "bg-neon text-background" : "bg-background opacity-0"
                      }`}
                      title={fav ? `Quitar ${n.name} de favoritos` : `Agregar ${n.name} a favoritos`}
                      aria-label={fav ? `Quitar ${n.name} de favoritos` : `Agregar ${n.name} a favoritos`}
                    >
                      ★
                    </button>
                  </div>
                );
              })
            )}
          </div>
        </nav>

        {/* Footer */}
        <div className="border-t-4 border-foreground p-3">
          <div className="flex items-center justify-between gap-2">
            <div className="font-tech text-[11px] uppercase tracking-wider text-muted-foreground">
              v0.1.0 · <span className="text-accent-text">embedded</span>
            </div>
            <button
              type="button"
              onClick={onToggleTheme}
              className="press flex h-9 w-9 items-center justify-center border-2 border-foreground"
              title="Cambiar tema"
              aria-label="Cambiar tema claro/oscuro"
            >
              {dark ? <Sun className="h-4 w-4" strokeWidth={2.5} /> : <Moon className="h-4 w-4" strokeWidth={2.5} />}
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
              placeholder="Buscar memoria…"
              aria-label="Búsqueda global"
              className="w-full border-2 border-foreground bg-background py-1.5 pr-3 pl-10 text-sm placeholder:text-muted-foreground"
            />
            <span className="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-neon">
              <Search className="h-4 w-4" strokeWidth={2.5} />
            </span>
            {searching && (
              <span className="absolute right-2 top-1/2 -translate-y-1/2 font-tech text-[10px] text-muted-foreground">
                …
              </span>
            )}
          </form>

          {/* FIX-D4 (Von Restorff): INGEST es el ÚNICO fill neón de la topbar.
              FILTROS usa el lenguaje de active-state del sistema
              (bg-foreground text-background, igual que SideButton) cuando el
              panel está abierto — coincide con aria-pressed. El contador de
              reglas activas lleva un mini-badge text-neon. */}
          <button
            type="button"
            onClick={() => setShowFilters((v) => !v)}
            aria-pressed={showFilters}
            title="Filtros compuestos por metadata (AND/OR, sin JSON)"
            className={`press border-2 border-foreground px-2.5 py-1.5 text-xs font-semibold ${
              showFilters ? "bg-foreground text-background" : "bg-background"
            }`}
          >
            ⧩ FILTROS
            {filterActive && (
              <span className="ml-1 font-tech text-accent-text">({toVantaMemoryFilter(ruleGroup).length})</span>
            )}
          </button>

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
          /* UX-15: botón ✕ enfocable — antes el bar entero era un div clickeable
             sin control de teclado para cerrar. */
          <div
            role="alert"
            onClick={onDismissNotice}
            className="flex cursor-pointer items-center gap-2 border-b-4 border-neon bg-card px-4 py-2 text-sm"
          >
            <span className="min-w-0 flex-1">{notice}</span>
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                onDismissNotice();
              }}
              className="press flex h-6 w-6 shrink-0 items-center justify-center border-2 border-foreground text-[10px]"
              aria-label="Cerrar aviso"
              title="Cerrar aviso"
            >
              ✕
            </button>
          </div>
        )}

        {/* ========== FILTROS COMPUESTOS (VS-07) ========== */}
        {showFilters && (
          <section className="border-b-4 border-foreground bg-card" aria-label="Filtros compuestos por metadata">
            <div className="mx-auto max-w-6xl p-4">
              <div className="mb-2 flex flex-wrap items-center justify-between gap-2">
                <span className="font-tech text-[10px] uppercase tracking-widest text-accent-text">
                  filtros compuestos · metadata tipada
                  {filterActive && (
                    <span className="text-muted-foreground">
                      {" "}· {toVantaMemoryFilter(ruleGroup).length} reglas → VantaMemoryFilter
                    </span>
                  )}
                </span>
                <div className="flex items-center gap-2">
                  {filterActive && (
                    <button
                      type="button"
                      onClick={() => setRuleGroup(EMPTY_QUERY)}
                      className="press border-2 border-foreground bg-background px-2 py-0.5 text-[10px] font-semibold"
                    >
                      ✕ limpiar
                    </button>
                  )}
                  <button
                    type="button"
                    onClick={() => setShowFilters(false)}
                    className="press border-2 border-foreground bg-background px-2 py-0.5 text-[10px] font-semibold"
                  >
                    ocultar
                  </button>
                </div>
              </div>
              {filterFields.length === 0 ? (
                <p className="font-tech text-[11px] text-muted-foreground">
                  Sin campos de metadata en los resultados — ejecutá una búsqueda global para
                  inferir tipos (string/int/float/bool/datetime).
                </p>
              ) : (
                <Suspense fallback={<p className="font-tech text-[11px] text-muted-foreground">Cargando builder…</p>}>
                  <FiltersBuilder fields={filterFields} query={ruleGroup} onChange={setRuleGroup} />
                </Suspense>
              )}
            </div>
          </section>
        )}

        {/* ========== CENTRAL SURFACE ========== */}
        <main className="flex-1 overflow-y-auto scroll-manga">
          {results !== null && (
            <section className="border-b-4 border-foreground bg-card">
              <div className="mx-auto max-w-6xl p-4">
                <div className="flex items-center justify-between">
                  <span className="font-tech text-[10px] uppercase tracking-widest text-accent-text">
                    Resultados de búsqueda
                    {filterActive && results && visibleResults && results.length !== visibleResults.length && (
                      <span className="text-muted-foreground">
                        {" "}· {visibleResults.length}/{results.length} tras filtro
                      </span>
                    )}
                  </span>
                  <button
                    type="button"
                    onClick={() => setResults(null)}
                    className="press border-2 border-foreground bg-background px-2 py-1 text-xs"
                  >
                    ✕ cerrar
                  </button>
                </div>
                <ResultsList
                  results={visibleResults}
                  onSelect={(r) => openSearchResult(r)}
                  onClearSearch={() => {
                    setQuery("");
                    setResults(null);
                  }}
                />
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
              {state.active && <HomeOverview active />}
              {/* WEB-05: connection manager is Tauri-only (multi-connection);
                  embedded mode = single implicit HTTP connection. */}
              {!embedded && (
                <ConnectionPanel
                  connections={state.connections}
                  activeId={state.activeId}
                  health={state.health}
                  healthStatus={state.healthStatus}
                  busy={state.busy}
                  onConnectNative={actions.connectNativePath}
                  onUseProfile={useProfile}
                  onDisconnect={actions.disconnectId}
                  onActivate={actions.activate}
                  onProbeHealth={actions.probeHealth}
                />
              )}
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
              <div className="flex items-center justify-end gap-2">
                <button
                  className="press border-2 border-foreground bg-background px-2 py-1 font-tech text-[10px] uppercase tracking-widest"
                  onClick={() => setImportOpen(true)}
                  title="Importar CSV o JSON pegado (hasta 1000 registros)"
                >
                  ⤒ IMPORT CSV/JSON
                </button>
                <button
                  className="press border-2 border-foreground bg-background px-2 py-1 font-tech text-[10px] uppercase tracking-widest"
                  onClick={() => setImportFileOpen(true)}
                  title="Importar archivo .csv/.json/.jsonl/.vdbdump por drag&drop"
                >
                  ⤓ IMPORT ARCHIVO
                </button>
              </div>
              {/* UX-15: microcopy ES (antes "Stored N record(s)."). UX-17: el
                  ingest manual refresca el grid vía remount (mismo gridKey que
                  batch delete/imports). */}
              <IngestForm
                onDone={(ids) => onNotice(`Guardados ${ids.length} registro(s).`)}
                onRefresh={() => setGridKey((k) => k + 1)}
                runError={onError}
              />
              <DataExplorer
                key={gridKey}
                active={!!state.active}
                busy={state.busy}
                runError={onError}
                onSelectRow={(row: ExplorerRow) => openRecord(row.record, row.score)}
                // OP-02: batch delete refresca el grid via remount (patrón Task 9).
                onRefresh={() => setGridKey((k) => k + 1)}
                // UX-11: empty state → scroll al formulario de ingest (mismo surface).
                onGoToIngest={() =>
                  document.getElementById("ingest-form")?.scrollIntoView({ behavior: "smooth", block: "start" })
                }
              />
            </div>
          )}

          {surface === "papelera" && (
            <div className="mx-auto max-w-6xl p-6">
              <TrashLens onNotice={onNotice} onError={onError} />
            </div>
          )}

          {surface === "actividad" && (
            <div className="mx-auto max-w-5xl p-6">
              <ActivityPanel onNotice={onNotice} onInspect={handleInspectAudit} />
            </div>
          )}

          {/* VS-13: Lente RETRIEVAL — slice aditivo; seed = registro seleccionado
              del Inspector (lente contextual, no destino aparte — P4). */}
          {surface === "retrieval" && (
            <div className="mx-auto max-w-6xl p-6">
              <RetrievalLens
                seed={selected?.record ?? null}
                onNotice={onNotice}
                onError={onError}
                onOpenRecord={(record, score) => openRecord(record, score)}
              />
            </div>
          )}

          {/* FEAT-02: lente ÍNDICES real (counts por namespace, HNSW, BM25,
              WAL, salud) — reemplaza el placeholder VS-03. */}
          {surface === "indices" && (
            <div className="mx-auto max-w-6xl p-6">
              <IndicesLens
                health={state.health}
                healthStatus={state.healthStatus}
                activeName={state.active ? state.active.name : null}
              />
            </div>
          )}
          {/* FEAT-03a: lente CONSOLIDAR (D16 (a)) — pares por similitud textual
              + diff visible + metadata.superseded_by (vanta_put). */}
          {surface === "consolidar" && (
            <div className="mx-auto max-w-6xl p-6">
              <ConsolidateLens
                active={!!state.active}
                activeName={state.active ? state.active.name : null}
                onNotice={onNotice}
                onError={onError}
              />
            </div>
          )}
          {/* GRAFO-02: lente GRAFO montada en la surface IQL (F2). */}
          {surface === "iql" && (
            <Suspense fallback={<LensPlaceholder title="IQL" phase="cargando visor…" />}>
              <GraphLens onNotice={onNotice} onError={onError} dark={dark} />
            </Suspense>
          )}
          {/* ESPACIO-01: scatterplot WebGL de embeddings (worker UMAP-js). */}
          {surface === "espacio" && (
            <Suspense fallback={<LensPlaceholder title="ESPACIO" phase="cargando visor…" />}>
              <SpaceLens
                onNotice={onNotice}
                onError={onError}
                dark={dark}
                onOpenRecord={(record, score) => openRecord(record, score)}
              />
            </Suspense>
          )}
          {/* DESKTOP-37: lente MEMORIA — read-only sobre vanta-memory; genlog
              con anchor_id abre el record real en el Inspector. */}
          {surface === "memoria" && (
            <div className="mx-auto max-w-6xl p-6">
              <MemoryLens
                active={!!state.active}
                sessionKey="user-1"
                onNotice={onNotice}
                onError={onError}
                onOpenRecord={(record, score) => openRecord(record, score)}
              />
            </div>
          )}
          {/* DESKTOP-38: lente PROXY — dashboard REST del proxy local (proceso
              aparte; sin URL configurada muestra el formulario y no polla). */}
          {surface === "proxy" && (
            <div className="mx-auto max-w-6xl p-6">
              <ProxyDashboard />
            </div>
          )}
          {/* DESKTOP-31: superficie AJUSTES. */}
          {surface === "ajustes" && (
            <Settings
              embedded={embedded}
              busy={state.busy}
              onConnectNative={actions.connectNativePath}
              onConnectServer={actions.connectServerCfg}
              onNotice={onNotice}
            />
          )}
        </main>
      </div>

      {/* ========== INSPECTOR (master-detail, derecha) ========== */}
      {selected && (
        <Suspense
          fallback={<aside className="w-[400px] shrink-0 border-l-4 border-foreground bg-card" aria-hidden="true" />}
        >
          <Inspector
            key={`${selected.record.namespace}:${selected.record.id}`}
            record={selected.record}
            score={selected.score}
            dark={dark}
            onClose={closeInspector}
            onSaved={(updated) => setSelected((cur) => (cur ? { ...cur, record: updated } : cur))}
            onError={onError}
          />
        </Suspense>
      )}

      {/* ========== COMMAND PALETTE (VS-09, Ctrl+K global) ========== */}
      <Suspense fallback={null}>
        <CommandPalette
          open={paletteOpen}
          onOpenChange={setPaletteOpen}
          namespaces={namespaces}
          dark={dark}
          activeConnection={state.active ? state.active.name : null}
          onNavigate={(s) => setSurface(s)}
          onSearch={handlePaletteSearch}
          onToggleTheme={onToggleTheme}
          onUndo={handleUndo}
          onDelete={handleDelete}
          onError={onError}
          favorites={favorites}
          onOpenFavorite={handleOpenFavorite}
          history={history}
          onClearHistory={() => searchHistory.clear()}
        />
      </Suspense>

      {/* ========== HELP PANEL (FIND-25, "?" global) ========== */}
      {helpOpen && <HelpPanel onClose={() => setHelpOpen(false)} />}

      {/* ========== NAMESPACE CRUD (DESKTOP-32, acciones de la sidebar) ========== */}
      {nsDialog && (
        <NamespaceDialog
          dialog={nsDialog}
          existing={namespaces.map((n) => n.name)}
          onClose={() => setNsDialog(null)}
          onCreate={handleCreateNs}
          onRename={handleRenameNs}
          onDelete={handleDeleteNs}
        />
      )}

      {/* ========== IMPORT PASTE (OP-01, botón en MEMORIAS) ========== */}
      <Suspense fallback={null}>
        <ImportPaste
          open={importOpen}
          onClose={() => setImportOpen(false)}
          defaultNamespace={state.active?.name ?? "default"}
          onImported={(count) => {
            setGridKey((k) => k + 1);
            setSurface("memorias");
            onNotice(`Importados ${count} registros.`);
          }}
          onError={onError}
        />
      </Suspense>

      {/* ========== IMPORT ARCHIVO (WASM-04, drag&drop .vdbdump/.jsonl/.csv) ========== */}
      <Suspense fallback={null}>
        <ImportDrop
          open={importFileOpen}
          onClose={() => setImportFileOpen(false)}
          defaultNamespace={state.active?.name ?? "default"}
          onImported={(count) => {
            setGridKey((k) => k + 1);
            setSurface("memorias");
            onNotice(`Importados ${count} registros.`);
          }}
          onError={onError}
        />
      </Suspense>
    </div>
  );
}