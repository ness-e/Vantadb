// Data Explorer (ADMIN-07 → VS-05). Browses records of the active connection as
// a virtualized manga/linocut grid with cursor pagination (VS-CORE-01).
//
// The old "Load more" anti-pattern is gone: browsing lists pages via
// `listPage({ limit, cursor })` and TanStack Virtual renders only the visible
// rows; when the viewport reaches the end, the next cursor page is fetched
// (infinite scroll). Semantic search is a one-shot `search(top_k)` result set
// (no cursor exists for search), still virtualized. Sort/filter is client-side
// over the loaded rows, per column.
//
// VS-03 master-detail contract is preserved: `ExplorerRow` + `onSelectRow`
// (plus the enriched `record` for VS-06).
import { FormEvent, useEffect, useMemo, useRef, useState } from "react";
import {
  useTable,
  tableFeatures,
  createColumnHelper,
  columnFilteringFeature,
  rowSortingFeature,
  createFilteredRowModel,
  createSortedRowModel,
  flexRender,
  sortFn_alphanumeric,
  sortFn_text,
  type Row,
} from "@tanstack/react-table";
import { useVirtualizer } from "@tanstack/react-virtual";
import { listPage, search, vantaErrorMessage, type MemoryRecord } from "../vanta";
import { undoStore } from "../store/undo";
import { ExportButtons } from "./export/ExportButtons";
import { recordsToJsonl, downloadText } from "./export/export-jsonl";
// OP-02: selección múltiple del grid (batch ops) — ops puras testeadas.
import { rowKey, selectAll, toggleId } from "./batchSelection";
// VS-17: favorito (★) + copy-as por fila — slice aditivo en la celda de acciones.
import { favoritesStore } from "../store/favorites";
import { CopyButton } from "./copy/CopyButton";
import { recordToJson } from "./copy/copy-as";

export interface ExplorerRow {
  id: string;
  namespace: string;
  text: string;
  /** Relevance for search mode; null while browsing. */
  score: number | null;
  /** Full enriched record (VS-11) for the Inspector (VS-06). */
  record: MemoryRecord;
}

interface Props {
  active: boolean;
  busy: boolean;
  runError: (msg: string) => void;
  /** Master-detail (VS-03): clicking a row opens the record in the right Inspector. */
  onSelectRow?: (row: ExplorerRow) => void;
  /** OP-02: feedback de éxito de las batch ops (default noop). */
  onNotice?: (msg: string) => void;
  /** OP-02: remount del grid tras una mutación batch (patrón `key={gridKey}`
   * de WorkspaceShell, Task 9). Si no se provee, las filas borradas se quitan
   * localmente del estado. */
  onRefresh?: () => void;
}

const PAGE = 100;
const SEARCH_TOP_K = 200;
const GRID_H = 540;

// --- Formatting helpers (pure) ------------------------------------------------

function fmtDateTime(ms: number): string {
  const d = new Date(ms);
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`;
}

function relTime(ms: number, now: number): string {
  const diff = Math.max(0, now - ms);
  const m = Math.floor(diff / 60_000);
  if (m < 1) return "now";
  if (m < 60) return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ago`;
  return `${Math.floor(h / 24)}d ago`;
}

function fmtDuration(ms: number): string {
  const m = Math.max(0, Math.ceil(ms / 60_000));
  if (m < 60) return `${m}m`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ${m % 60}m`;
  return `${Math.floor(h / 24)}d ${h % 24}h`;
}

/** Value type tag for metadata chips (matches VantaValue kind). */
function valueTag(v: unknown): string {
  if (v === null || v === undefined) return "nil";
  if (typeof v === "string") return "str";
  if (typeof v === "boolean") return "bool";
  if (typeof v === "number") return Number.isInteger(v) ? "int" : "flt";
  if (Array.isArray(v)) return "lst";
  if (v instanceof Date) return "date";
  return "obj";
}

// --- Columns (module-level: stable identity for useTable) ----------------------

const features = tableFeatures({
  columnFilteringFeature,
  rowSortingFeature,
  filteredRowModel: createFilteredRowModel(),
  sortedRowModel: createSortedRowModel(),
  sortFns: { alphanumeric: sortFn_alphanumeric, text: sortFn_text },
});

type ExplorerRowRow = Row<typeof features, ExplorerRow>;

const helper = createColumnHelper<typeof features, ExplorerRow>();

/** Client-side case-insensitive filter over whatever a column shows. */
function textFilter(row: ExplorerRowRow, columnId: string, value: unknown): boolean {
  return String(row.getValue(columnId) ?? "")
    .toLowerCase()
    .includes(String(value ?? "").toLowerCase());
}

/** Numeric sort over the column's raw value (nulls sort last). */
function numSort(rowA: ExplorerRowRow, rowB: ExplorerRowRow, columnId: string): number {
  const av = rowA.getValue<number | null>(columnId);
  const bv = rowB.getValue<number | null>(columnId);
  return (av ?? -Infinity) - (bv ?? -Infinity);
}

function MetaChips({ meta }: { meta: Record<string, unknown> | null }) {
  const entries = Object.entries(meta ?? {}).slice(0, 3);
  if (entries.length === 0) return <span className="text-xs opacity-50">—</span>;
  return (
    <span className="flex flex-wrap gap-1">
      {entries.map(([k, v]) => (
        <span
          key={k}
          className="border-2 border-ink bg-paper px-1 font-tech text-[10px] leading-4 text-foreground"
          title={`${k} = ${typeof v === "string" ? v : JSON.stringify(v)}`}
        >
          {k}:{valueTag(v)}
        </span>
      ))}
      {Object.keys(meta ?? {}).length > 3 && (
        <span className="px-1 font-tech text-[10px] leading-4 opacity-50">
          +{Object.keys(meta ?? {}).length - 3}
        </span>
      )}
    </span>
  );
}

// P15 (VS-18): TTL nunca es solo-color. Estado = ícono + texto + patrón de
// barra: ● activo (fill foreground) · ⚠ expirando (fill rayas neon) · ✕ expirado
// (fill muted-foreground). El texto usa foreground (AA ≥4.5:1 en claro y dark;
// text-neon sobre paper = 3.01:1, falla).
function TtlCell({ record, now }: { record: MemoryRecord; now: number }) {
  const expires = record.expires_at_ms;
  if (!expires) return <span className="text-xs opacity-50">—</span>;
  const remain = expires - now;
  const total = expires - (record.updated_at_ms ?? record.created_at_ms ?? expires);
  const frac = total > 0 ? Math.min(1, Math.max(0, remain / total)) : 1;
  const expired = remain <= 0;
  const expiring = !expired && frac < 0.2;
  const barFill = expired ? "bg-muted-foreground" : expiring ? "stripes-neon" : "bg-foreground";
  return (
    <span className="block w-24" title={`expires ${fmtDateTime(expires)}`}>
      <span className="font-tech text-[10px] font-bold">
        {expired ? "✕ EXPIRED" : expiring ? `⚠ ${fmtDuration(remain)} left` : `● ${fmtDuration(remain)} left`}
      </span>
      <span className="mt-0.5 block h-2 w-full border-2 border-ink bg-paper">
        <span
          className={`block h-full ${barFill}`}
          style={{ width: `${expired ? 0 : Math.round(frac * 100)}%` }}
        />
      </span>
    </span>
  );
}

/** Botón 🗑 por fila (VS-08, Fix 4): soft-delete con confirmación inline de
 * dos pasos (P6). `undoStore.softDelete` hace remove en backend + tombstone de
 * sesión; Ctrl+Z lo restaura. El click no abre el inspector (stopPropagation). */
function DeleteButton({
  record,
  onDeleted,
  onError,
}: {
  record: MemoryRecord;
  onDeleted: () => void;
  onError: (msg: string) => void;
}) {
  const [confirming, setConfirming] = useState(false);
  const [busy, setBusy] = useState(false);

  async function handleDelete() {
    setBusy(true);
    try {
      await undoStore.softDelete(record);
      onDeleted();
    } catch (err) {
      onError(vantaErrorMessage(err));
    } finally {
      setBusy(false);
      setConfirming(false);
    }
  }

  return (
    <span className="flex items-center gap-1" onClick={(e) => e.stopPropagation()}>
      {confirming ? (
        <>
          <button
            type="button"
            onClick={handleDelete}
            disabled={busy}
            className="press border-2 border-foreground bg-neon px-2 py-1 text-[10px] font-bold text-background"
            title="Confirmar borrado"
          >
            {busy ? "…" : "BORRAR"}
          </button>
          <button
            type="button"
            onClick={() => setConfirming(false)}
            className="press flex h-6 w-6 items-center justify-center border-2 border-foreground text-[10px]"
            aria-label="Cancelar borrado"
          >
            ✕
          </button>
        </>
      ) : (
        <button
          type="button"
          onClick={() => setConfirming(true)}
          className="press flex h-6 w-6 items-center justify-center border-2 border-foreground text-[10px]"
          title="Mover a papelera (Ctrl+Z deshace)"
          aria-label={`Mover ${record.id} a papelera`}
        >
          🗑
        </button>
      )}
    </span>
  );
}

// Columnas base a nivel de módulo (identidad estable para useTable). VS-08
// agrega la columna de acciones DENTRO del componente (necesita refs latest:
// runError/setRows) — `columns` final = baseColumns + display col, en useMemo.
const baseColumns = helper.columns([
  helper.accessor((r) => r.id, {
    id: "key",
    header: "Key",
    cell: (info) => (
      <code className="break-all font-tech text-[12px] text-foreground">{info.getValue()}</code>
    ),
    sortFn: "alphanumeric",
    filterFn: textFilter,
  }),
  helper.accessor((r) => r.text, {
    id: "payload",
    header: "Payload",
    cell: (info) => (
      <span className="block max-w-[420px] overflow-hidden text-ellipsis whitespace-nowrap text-[13px]">
        {info.getValue()}
      </span>
    ),
    sortFn: "text",
    filterFn: textFilter,
  }),
  helper.accessor((r) => r.record.metadata ?? null, {
    id: "metadata",
    header: "Metadata",
    cell: (info) => <MetaChips meta={info.getValue()} />,
    sortFn: (rowA, rowB) => {
      const a = rowA.getValue<Record<string, unknown> | null>("metadata") ?? {};
      const b = rowB.getValue<Record<string, unknown> | null>("metadata") ?? {};
      return Object.keys(a).length - Object.keys(b).length;
    },
    filterFn: (row, columnId, value) =>
      JSON.stringify(row.getValue(columnId) ?? "")
        .toLowerCase()
        .includes(String(value ?? "").toLowerCase()),
  }),
  helper.accessor((r) => (r.record.vector ? r.record.vector.length : null), {
    id: "vector",
    header: "Vector",
    cell: (info) =>
      info.getValue() == null ? (
        // VS-18/P15: ausente = ◇ + "—" (no solo el guion opaco).
        <span className="text-xs opacity-60">◇ —</span>
      ) : (
        // VS-18/P15: presente = ◆ + dimensión; texto foreground (AA) con
        // borde neon como acento no-texto (3:1 ✅).
        <span className="border-2 border-neon px-1 font-tech text-[10px] text-foreground">
          ◆ {info.getValue()}d
        </span>
      ),
    sortFn: numSort,
    filterFn: textFilter,
  }),
  helper.accessor((r) => (r.record.version != null ? r.record.version : null), {
    id: "version",
    header: "Version",
    cell: (info) =>
      info.getValue() == null ? (
        <span className="text-xs opacity-50">—</span>
      ) : (
        <span className="border-2 border-ink bg-cream px-1 font-tech text-[10px]">
          v{info.getValue()}
        </span>
      ),
    sortFn: numSort,
    filterFn: textFilter,
  }),
  helper.accessor((r) => r.record.updated_at_ms ?? null, {
    id: "updated_at",
    header: "Updated",
    cell: (info) => {
      const ms = info.getValue();
      if (ms == null) return <span className="text-xs opacity-50">—</span>;
      return (
        <span className="block font-tech text-[11px] leading-4">
          <span>{fmtDateTime(ms)}</span>
          <span className="block text-[10px] opacity-50">{relTime(ms, Date.now())}</span>
        </span>
      );
    },
    sortFn: numSort,
    filterFn: (row, columnId, value) => {
      const ms = row.getValue<number | null>(columnId);
      if (ms == null) return false;
      return `${fmtDateTime(ms)} ${relTime(ms, Date.now())}`
        .toLowerCase()
        .includes(String(value ?? "").toLowerCase());
    },
  }),
  helper.accessor((r) => r.record, {
    id: "ttl",
    header: "TTL",
    cell: (info) => <TtlCell record={info.getValue()} now={Date.now()} />,
    sortFn: (rowA, rowB) => {
      const ra = rowA.getValue<MemoryRecord>("ttl").expires_at_ms ?? Infinity;
      const rb = rowB.getValue<MemoryRecord>("ttl").expires_at_ms ?? Infinity;
      return ra - rb;
    },
    filterFn: (row, columnId, value) => {
      const rec = row.getValue<MemoryRecord>(columnId);
      const q = String(value ?? "").toLowerCase();
      if (!rec.expires_at_ms) return "never".includes(q);
      const remain = rec.expires_at_ms - Date.now();
      return remain <= 0
        ? "expired".includes(q)
        : fmtDuration(remain).toLowerCase().includes(q);
    },
  }),
]);

export default function DataExplorer({ active, busy, runError, onSelectRow, onNotice, onRefresh }: Props) {
  const [query, setQuery] = useState("");
  const [rows, setRows] = useState<ExplorerRow[] | null>(null);
  const [mode, setMode] = useState<"list" | "search">("list");
  const [nextCursor, setNextCursor] = useState<number | null>(null);
  const [loading, setLoading] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const [, setNow] = useState(() => Date.now());

  // OP-02: selección múltiple (Set de `${namespace}:${id}` — mismo keying que
  // getRowId). select-all aplica a las filas cargadas (página actual), no al
  // namespace completo.
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [batchBusy, setBatchBusy] = useState<"export" | "delete" | null>(null);
  const [confirmBatchDelete, setConfirmBatchDelete] = useState(false);
  const selectAllRef = useRef<HTMLInputElement>(null);

  // Live TTL countdown: re-render periodically so bars/relative times move.
  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), 30_000);
    return () => clearInterval(t);
  }, []);

  // VS-17: re-render al cambiar favoritos (★ por fila) — el store notifica y
  // este estado fantasma fuerza el render para releer isFavorite.
  const [, setFavTick] = useState(0);
  useEffect(() => favoritesStore.subscribe(() => setFavTick((t) => t + 1)), []);

  // Guards for cursor pagination (avoid double-fetch from effect re-runs).
  const cursorRef = useRef<number | null>(null);
  const fetchingRef = useRef(false);
  const seqRef = useRef(0);

  useEffect(() => {
    cursorRef.current = nextCursor;
  }, [nextCursor]);

  async function fetchFirst(kind: "list" | "search", q: string) {
    const seq = ++seqRef.current;
    setLoading(true);
    setRows(null);
    setNextCursor(null);
    cursorRef.current = null;
    // OP-02: nueva query → la selección previa ya no aplica a las filas nuevas.
    setSelected(new Set());
    setConfirmBatchDelete(false);
    try {
      if (kind === "search") {
        const results = await search({ query: q, top_k: SEARCH_TOP_K });
        if (seqRef.current !== seq) return;
        setRows(
          results.map((r) => ({
            id: r.id,
            namespace: r.namespace,
            text: r.text,
            score: r.score,
            record: {
              id: r.id,
              namespace: r.namespace,
              text: r.text,
              metadata: r.metadata,
            } as MemoryRecord,
          })),
        );
      } else {
        const page = await listPage({ limit: PAGE, cursor: 0 });
        if (seqRef.current !== seq) return;
        setRows(
          page.records.map((rec) => ({
            id: rec.id,
            namespace: rec.namespace,
            text: rec.text,
            score: null,
            record: rec,
          })),
        );
        const c = page.next_cursor ?? null;
        setNextCursor(c);
        cursorRef.current = c;
      }
      setMode(kind);
    } catch (err) {
      runError(vantaErrorMessage(err));
    } finally {
      if (seqRef.current === seq) setLoading(false);
    }
  }

  async function fetchMore() {
    if (fetchingRef.current || cursorRef.current == null) return;
    fetchingRef.current = true;
    const seq = seqRef.current;
    const cursor = cursorRef.current;
    setLoadingMore(true);
    try {
      const page = await listPage({ limit: PAGE, cursor });
      if (seqRef.current !== seq) return;
      setRows((prev) => [
        ...(prev ?? []),
        ...page.records.map((rec) => ({
          id: rec.id,
          namespace: rec.namespace,
          text: rec.text,
          score: null,
          record: rec,
        })),
      ]);
      const c = page.next_cursor ?? null;
      setNextCursor(c);
      cursorRef.current = c;
    } catch (err) {
      runError(vantaErrorMessage(err));
    } finally {
      fetchingRef.current = false;
      setLoadingMore(false);
    }
  }

  // Browse on mount / when a connection appears.
  useEffect(() => {
    if (active) fetchFirst("list", "");
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [active]);

  function handleSubmit(e: FormEvent) {
    e.preventDefault();
    const q = query.trim();
    fetchFirst(q ? "search" : "list", q);
  }

  // VS-08: la columna de acciones necesita runError/setRows (per-render) pero
  // las columnas deben tener identidad estable para useTable → refs "latest"
  // (siempre apuntan al valor actual) + useMemo([]) que se crea UNA vez.
  const errorRef = useRef(runError);
  errorRef.current = runError;
  const removeRowRef = useRef<(r: ExplorerRow) => void>(() => {});
  removeRowRef.current = (r) => {
    setRows((prev) => prev?.filter((x) => !(x.id === r.id && x.namespace === r.namespace)) ?? prev);
  };

  // OP-02: derivados de selección. `selected`/`loadedKeys` cambian por render y
  // las columnas tienen identidad estable (useMemo []) → refs "latest" (mismo
  // patrón errorRef/removeRowRef de VS-08).
  const loadedKeys = useMemo(() => (rows ?? []).map((r) => rowKey(r.record)), [rows]);
  const selectedRecords = useMemo(
    () => (rows ?? []).filter((r) => selected.has(rowKey(r.record))).map((r) => r.record),
    [rows, selected],
  );
  const loadedKeysRef = useRef(loadedKeys);
  loadedKeysRef.current = loadedKeys;
  const selectedRef = useRef(selected);
  selectedRef.current = selected;

  const allLoadedSelected = loadedKeys.length > 0 && loadedKeys.every((k) => selected.has(k));
  const someLoadedSelected = loadedKeys.some((k) => selected.has(k));
  useEffect(() => {
    if (selectAllRef.current) {
      selectAllRef.current.indeterminate = someLoadedSelected && !allLoadedSelected;
    }
  }, [someLoadedSelected, allLoadedSelected]);

  // OP-02: batch ops sobre la selección. Export client-side JSONL (mismo path
  // que ExportButtons/ESPACIO-02); delete vía softDeleteBatch (UN entry de
  // undo — un solo Ctrl+Z restaura todo el lote).
  function batchStamp(): string {
    return new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
  }

  async function handleBatchExport() {
    setBatchBusy("export");
    try {
      const jsonl = recordsToJsonl(selectedRecords);
      downloadText(`vanta-selection-${batchStamp()}.jsonl`, jsonl);
      onNotice?.(`exportados ${selectedRecords.length} registros (JSONL)`);
    } catch (err) {
      runError(vantaErrorMessage(err));
    } finally {
      setBatchBusy(null);
    }
  }

  async function handleBatchDelete() {
    setBatchBusy("delete");
    try {
      await undoStore.softDeleteBatch(selectedRecords);
      setConfirmBatchDelete(false);
      setSelected(new Set());
      if (onRefresh) onRefresh();
      else {
        const gone = new Set(selected);
        setRows((prev) => prev?.filter((r) => !gone.has(rowKey(r.record))) ?? prev);
      }
      onNotice?.(`movidos ${selectedRecords.length} registros a papelera (Ctrl+Z deshace)`);
    } catch (err) {
      runError(vantaErrorMessage(err));
    } finally {
      setBatchBusy(null);
    }
  }

  const columns = useMemo(
    () => [
      // OP-02: columna de selección (checkbox + select-all de la página actual).
      // Native checkbox → Space/aria-label gratis (05 a11y). stopPropagation:
      // el click no abre el inspector (onSelectRow del <tr>).
      helper.display({
        id: "select",
        // 05 a11y: sin sort (el checkbox no va dentro de un <button>).
        enableSorting: false,
        header: () => {
          const keys = loadedKeysRef.current;
          const sel = selectedRef.current;
          const all = keys.length > 0 && keys.every((k) => sel.has(k));
          return (
            <input
              ref={selectAllRef}
              type="checkbox"
              checked={all}
              onChange={() => setSelected((s) => selectAll(s, keys))}
              aria-label={
                all
                  ? "Quitar selección de todas las filas cargadas"
                  : "Seleccionar todas las filas cargadas"
              }
              title="Seleccionar todas las filas cargadas (página actual)"
              className="h-4 w-4 cursor-pointer"
              onClick={(e) => e.stopPropagation()}
            />
          );
        },
        cell: ({ row }) => {
          const key = rowKey(row.original.record);
          const checked = selectedRef.current.has(key);
          return (
            <input
              type="checkbox"
              checked={checked}
              onChange={() => setSelected((s) => toggleId(s, key))}
              aria-label={`Seleccionar ${row.original.id}`}
              title="Seleccionar para operaciones por lote"
              className="h-4 w-4 cursor-pointer"
              onClick={(e) => e.stopPropagation()}
            />
          );
        },
      }),
      ...baseColumns,
      helper.display({
        id: "actions",
        header: "",
        cell: ({ row }) => {
          const rec = row.original.record;
          // VS-17: ★ favorito + copiar registro (JSON) — aditivo junto al 🗑.
          const fav = favoritesStore.isFavorite(rec.namespace, rec.id);
          return (
            <span className="flex items-center gap-1" onClick={(e) => e.stopPropagation()}>
              <button
                type="button"
                onClick={() => favoritesStore.toggle(rec.namespace, rec.id)}
                aria-pressed={fav}
                className={`press flex h-6 w-6 items-center justify-center border-2 border-foreground text-[10px] ${
                  fav ? "bg-neon text-background" : "bg-background"
                }`}
                title={fav ? `Quitar ${rec.id} de favoritos` : `Agregar ${rec.id} a favoritos`}
                aria-label={fav ? `Quitar ${rec.id} de favoritos` : `Agregar ${rec.id} a favoritos`}
              >
                ★
              </button>
              <CopyButton
                getText={() => recordToJson(rec)}
                label="⧉"
                title="Copiar registro completo (JSON)"
                onError={(m) => errorRef.current(m)}
                className="h-6 px-1.5"
              />
              <DeleteButton
                record={rec}
                onDeleted={() => removeRowRef.current(row.original)}
                onError={(m) => errorRef.current(m)}
              />
            </span>
          );
        },
      }),
    ],
    [],
  );

  // v9: self-managed state when no `state`/`atoms` options are passed.
  const table = useTable({
    features,
    columns,
    data: rows ?? [],
    getRowId: (r) => `${r.namespace}:${r.id}`,
    initialState: { sorting: [{ id: "updated_at", desc: true }] },
  });

  const scrollRef = useRef<HTMLDivElement>(null);
  const rowCount = table.getRowModel().rows.length;

  const virtualizer = useVirtualizer({
    count: rowCount,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => 46,
    overscan: 12,
  });
  const virtualRows = virtualizer.getVirtualItems();

  // Infinite scroll: fetch the next cursor page only when the user has really
  // scrolled to the bottom of an overflowing viewport (not when a column filter
  // leaves few rows — those have no overflow and must not trigger fetches).
  const lastVirtualIndex = virtualRows.length > 0 ? virtualRows[virtualRows.length - 1].index : -1;
  useEffect(() => {
    if (loadingMore || loading || mode !== "list") return;
    if (cursorRef.current == null) return;
    const sr = virtualizer.scrollRect;
    const offset = virtualizer.scrollOffset;
    const total = virtualizer.getTotalSize();
    if (!sr || offset == null) return;
    // Real bottom of an overflowing viewport: scrollOffset + viewport height
    // reaches the end of the virtual content.
    const atBottom = total > sr.height && offset + sr.height >= total - 80;
    if (atBottom && rowCount > 0 && lastVirtualIndex >= rowCount - 3) fetchMore();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [lastVirtualIndex, rowCount, mode, loading, loadingMore, virtualizer.getTotalSize()]);

  const visibleMeta = useMemo(() => {
    const meta = new Set<string>();
    for (const r of rows ?? []) for (const k of Object.keys(r.record.metadata ?? {})) meta.add(k);
    return Array.from(meta).slice(0, 8);
  }, [rows]);

  return (
    <section
      aria-label="Memorias"
      className="border-[3px] border-foreground bg-card p-4 shadow-ink"
    >
      <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
        <h2 className="m-0 font-tech text-xs uppercase tracking-widest">Memorias</h2>
        <span className="text-muted-foreground">
          {mode} · {rows ? `${rows.length} cargados` : "inactivo"}
          {mode === "list" && nextCursor != null && " · hay más"}
        </span>
        {rows && rows.length > 0 && (
          <ExportButtons
            viewRecords={table.getFilteredRowModel().rows.map((r) => r.original.record)}
            onError={runError}
          />
        )}
      </div>

      {/* OP-02: barra de batch ops — visible con selección; eliminar con
          confirmación que nombra la cantidad (05 anti-patrón 7). */}
      {selected.size > 0 && (
        <div
          role="toolbar"
          aria-label="Operaciones por lote"
          className="mt-2 flex flex-wrap items-center gap-2 border-2 border-neon bg-paper px-2 py-1.5 font-tech text-[10px] uppercase tracking-widest"
        >
          <span className="font-bold text-foreground">{selected.size} seleccionados</span>
          <button
            type="button"
            className="press border-2 border-foreground bg-background px-2 py-1"
            disabled={batchBusy !== null}
            onClick={handleBatchExport}
            title={`Exportar ${selectedRecords.length} registros seleccionados como JSONL`}
          >
            {batchBusy === "export" ? "…" : `Exportar (${selectedRecords.length})`}
          </button>
          {confirmBatchDelete ? (
            <>
              <button
                type="button"
                className="press border-2 border-foreground bg-neon px-2 py-1 font-bold text-background"
                disabled={batchBusy !== null}
                onClick={handleBatchDelete}
                title="Confirmar borrado (Ctrl+Z deshace)"
              >
                {batchBusy === "delete" ? "…" : `BORRAR ${selectedRecords.length}`}
              </button>
              <button
                type="button"
                className="press flex h-6 w-6 items-center justify-center border-2 border-foreground text-[10px]"
                onClick={() => setConfirmBatchDelete(false)}
                aria-label="Cancelar borrado por lote"
              >
                ✕
              </button>
            </>
          ) : (
            <button
              type="button"
              className="press border-2 border-foreground bg-background px-2 py-1"
              disabled={batchBusy !== null}
              onClick={() => setConfirmBatchDelete(true)}
              title="Mover a papelera (Ctrl+Z deshace)"
            >
              Eliminar ({selectedRecords.length})
            </button>
          )}
          <span className="flex-1" />
          <button
            type="button"
            className="press flex h-6 w-6 items-center justify-center border-2 border-foreground text-[10px]"
            onClick={() => {
              setSelected(new Set());
              setConfirmBatchDelete(false);
            }}
            aria-label="Limpiar selección"
            title="Limpiar selección"
          >
            ✕
          </button>
        </div>
      )}

      <form className="flex gap-2" onSubmit={handleSubmit}>
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Consulta semántica (vacío = listar registros)"
          aria-label="Consultar registros"
          className="min-w-0 flex-1 border-2 border-foreground bg-background px-2.5 py-1.5"
        />
        <button
          type="submit"
          disabled={busy || loading || !active}
          className="press cursor-pointer border-2 border-foreground bg-background px-2.5 py-1.5 text-sm disabled:cursor-default disabled:opacity-50"
        >
          {loading ? "Cargando…" : "Traer"}
        </button>
      </form>

      {!active ? (
        <p className="text-muted-foreground">Sin conexión activa — conectá una para explorar registros.</p>
      ) : rows === null ? (
        <p className="text-muted-foreground">Cargando registros…</p>
      ) : rows.length === 0 ? (
        <p className="text-muted-foreground">Sin registros{mode === "search" ? " que coincidan" : ""}.</p>
      ) : (
        <>
          {visibleMeta.length > 0 && (
            <p className="mt-2 font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
              campos de metadata en vista: {visibleMeta.join(" · ")}
            </p>
          )}
          <div
            ref={scrollRef}
            className="scroll-manga relative mt-2 overflow-auto border-4 border-ink bg-paper"
            style={{ height: GRID_H }}
          >
            <table className="w-full border-collapse text-left">
              <thead>
                {table.getHeaderGroups().map((hg) => (
                  <tr key={hg.id} className="sticky top-0 z-10 bg-paper">
                    {hg.headers.map((header) => {
                      const sorted = header.column.getIsSorted();
                      return (
                        <th
                          key={header.id}
                          className="border-2 border-ink bg-paper px-2 py-1 align-bottom font-tech text-[10px] uppercase tracking-widest"
                        >
                          {/* 05 a11y: el <button> de sort solo envuelve columnas sortables —
                              la columna select (checkbox) y actions se renderizan planas. */}
                          {header.column.getCanSort() ? (
                            <button
                              type="button"
                              className="flex items-center gap-1 hover:text-neon"
                              onClick={() => header.column.toggleSorting(sorted === "asc")}
                              title="Ordenar por columna"
                            >
                              {flexRender(header.column.columnDef.header, header.getContext())}
                              <span className="text-neon">
                                {sorted === "asc" ? "▲" : sorted === "desc" ? "▼" : ""}
                              </span>
                            </button>
                          ) : (
                            flexRender(header.column.columnDef.header, header.getContext())
                          )}
                          {header.column.getCanFilter() && (
                            <input
                              value={(header.column.getFilterValue() as string) ?? ""}
                              onChange={(e) => header.column.setFilterValue(e.target.value)}
                              placeholder="filtro"
                              aria-label={`Filtrar ${header.column.id}`}
                              className="mt-1 w-full border-2 border-ink bg-cream px-1 font-tech text-[10px]"
                            />
                          )}
                        </th>
                      );
                    })}
                  </tr>
                ))}
              </thead>
              <tbody style={{ position: "relative", height: `${virtualizer.getTotalSize()}px` }}>
                {virtualRows.map((vr) => {
                  const row = table.getRowModel().rows[vr.index];
                  if (!row) return null;
                  return (
                    <tr
                      key={row.id}
                      onClick={onSelectRow ? () => onSelectRow(row.original) : undefined}
                      className={onSelectRow ? "cursor-pointer" : undefined}
                      title={onSelectRow ? "Ver en inspector" : undefined}
                      style={{
                        position: "absolute",
                        top: 0,
                        left: 0,
                        width: "100%",
                        transform: `translateY(${vr.start}px)`,
                        height: `${vr.size}px`,
                      }}
                    >
                      {row.getAllCells().map((cell) => (
                        <td key={cell.id} className="border-2 border-ink px-2 py-1 align-middle">
                          {flexRender(cell.column.columnDef.cell, cell.getContext())}
                        </td>
                      ))}
                    </tr>
                  );
                })}
              </tbody>
            </table>
            {loadingMore && (
              <div className="sticky bottom-0 z-10 border-t-4 border-ink bg-paper p-2 font-tech text-[10px] uppercase tracking-widest text-neon">
                Cargando más…
              </div>
            )}
          </div>
          <p className="mt-2 text-[10px] text-muted-foreground">
            {rowCount} filas mostradas · scroll para cargar más (cursor) · ordenar/filtrar por
            columna sobre los datos cargados
          </p>
        </>
      )}
    </section>
  );
}
