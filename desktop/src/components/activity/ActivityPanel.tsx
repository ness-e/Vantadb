// ActivityPanel (VS-15): superficie ACTIVITY del audit log configurado por
// VS-12 (`vanta_audit_events`). Compone la Timeline agrupada por hora/día +
// una tabla filtrable (namespace/op/outcome) paginada con el cursor del bridge.
// Empty state honesto: si el audit no está configurado (VS-12 rechaza con
// `Unsupported("audit log no configurado")`) muestra un banner con el hint de
// dónde se configura. Hover de fila → peek bar con key/record; clic → Inspector
// vía `onInspect` (el shell hace `get` + `openRecord`).
import { useCallback, useEffect, useMemo, useState } from "react";
import { auditEvents, type AuditEvent, vantaErrorMessage } from "../../vanta";
import { eventClock, type BucketGranularity } from "./logic";
import { OpChip, OutcomeBadge } from "./EventChip";
import Timeline from "./Timeline";

interface Props {
  onNotice: (msg: string) => void;
  /** Abrir un registro en el Inspector (el shell resuelve key → record). */
  onInspect: (namespace: string, key: string) => void;
}

/** Fragmento del mensaje de error de VS-12 cuando no hay audit configurado. */
const UNCONFIGURED_MARKER = "no configurado";
const PAGE_SIZE = 100;

interface Filters {
  namespace: string;
  op: string;
  outcome: string;
}

const NO_FILTERS: Filters = { namespace: "", op: "", outcome: "" };

export default function ActivityPanel({ onNotice, onInspect }: Props) {
  const [events, setEvents] = useState<AuditEvent[]>([]);
  const [cursor, setCursor] = useState<number | null>(null);
  const [filters, setFilters] = useState<Filters>(NO_FILTERS);
  const [granularity, setGranularity] = useState<BucketGranularity>("hour");
  const [loading, setLoading] = useState(false);
  const [unconfigured, setUnconfigured] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [peek, setPeek] = useState<AuditEvent | null>(null);

  // Los errores de carga se muestran inline (evita depender de la identidad de
  // `onError` del shell en un effect — el fetch vive en useEffect).
  const fetchPage = useCallback(
    async (cursorArg: number | null, replace: boolean) => {
      setLoading(true);
      try {
        const page = await auditEvents({
          namespace: filters.namespace || undefined,
          op: filters.op || undefined,
          outcome: filters.outcome || undefined,
          limit: PAGE_SIZE,
          cursor: cursorArg ?? undefined,
        });
        setUnconfigured(false);
        setLoadError(null);
        setEvents((prev) => (replace ? page.events : [...prev, ...page.events]));
        setCursor(page.next_cursor ?? null);
      } catch (err) {
        const msg = vantaErrorMessage(err);
        if (msg.includes(UNCONFIGURED_MARKER)) {
          setUnconfigured(true);
          setEvents([]);
          setCursor(null);
          setLoadError(null);
        } else {
          setLoadError(msg);
        }
      } finally {
        setLoading(false);
      }
    },
    [filters.namespace, filters.op, filters.outcome],
  );

  // Primera página al montar y cada vez que cambia un filtro (reset de cursor).
  useEffect(() => {
    void fetchPage(null, true);
  }, [fetchPage]);

  const namespaces = useMemo(
    () => [...new Set(events.map((e) => e.namespace))].sort(),
    [events],
  );
  const ops = useMemo(() => [...new Set(events.map((e) => e.op))].sort(), [events]);

  function setFilter(key: keyof Filters, value: string) {
    setFilters((f) => ({ ...f, [key]: value }));
  }

  function handleInspect(e: AuditEvent) {
    if (e.namespace === "N/A" || e.key === "N/A") {
      onNotice(`${e.op}: operación sin registro asociado (${e.namespace}:${e.key})`);
      return;
    }
    onInspect(e.namespace, e.key);
  }

  const filterActive = filters.namespace !== "" || filters.op !== "" || filters.outcome !== "";

  const selectCls =
    "border-2 border-foreground bg-background px-2 py-1 font-tech text-[11px] uppercase tracking-wider";

  return (
    <section className="press-lg border-4 border-foreground bg-card" aria-label="Actividad (audit log)">
      {/* Header */}
      <div className="border-b-4 border-foreground p-4">
        <div className="flex flex-wrap items-baseline justify-between gap-2">
          <h2 className="font-display text-3xl text-stencil">ACTIVITY</h2>
          <div className="flex items-center gap-2">
            <div className="flex border-2 border-foreground bg-background font-tech text-[10px]" role="group" aria-label="Agrupación temporal">
              {(["hour", "day"] as const).map((g) => (
                <button
                  key={g}
                  type="button"
                  onClick={() => setGranularity(g)}
                  aria-pressed={granularity === g}
                  className={`px-2 py-1 uppercase tracking-widest ${
                    granularity === g ? "bg-neon text-background" : "text-muted-foreground"
                  }`}
                >
                  {g === "hour" ? "por hora" : "por día"}
                </button>
              ))}
            </div>
            <button
              type="button"
              onClick={() => void fetchPage(null, true)}
              disabled={loading}
              className="press border-2 border-foreground bg-background px-2 py-1 font-tech text-[10px] uppercase tracking-widest"
              title="Recargar desde el tail del log"
            >
              ⟳
            </button>
          </div>
        </div>
        <p className="mt-1 font-tech text-[11px] text-muted-foreground">
          audit log del backend activo — escrituras, borrados, export/import · newest-first
        </p>
      </div>

      {unconfigured ? (
        /* Empty state honesto (contrato e): audit no configurado */
        <div role="alert" className="p-6">
          <div className="border-2 border-dashed border-foreground bg-background p-4">
            <div className="font-tech text-[11px] font-bold uppercase tracking-widest text-neon">
              ⚠ audit log no habilitado
            </div>
            <p className="mt-2 text-sm">
              La conexión activa no tiene audit log configurado (VS-12 rechaza con{" "}
              <code className="font-tech text-[11px]">Unsupported("audit log no configurado")</code>).
            </p>
            <p className="mt-1 font-tech text-[11px] text-muted-foreground">
              Se configura automáticamente al conectar un backend nativo:{" "}
              <code className="font-tech text-[11px]">NativeConnection::open</code> escribe en{" "}
              <code className="font-tech text-[11px]">&lt;storage_path&gt;/audit.jsonl</code>{" "}
              (VantaConfig.audit_log_path). Conectá un backend en RESUMEN para habilitarlo.
            </p>
          </div>
        </div>
      ) : (
        <>
          {/* Filtros (namespace/op/outcome) — el filtrado real corre en Rust */}
          <div className="flex flex-wrap items-center gap-2 border-b-4 border-foreground bg-background p-3">
            <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
              filtrar
            </span>
            <label className="sr-only" htmlFor="act-namespace">Filtrar por namespace</label>
            <select
              id="act-namespace"
              className={selectCls}
              value={filters.namespace}
              onChange={(e) => setFilter("namespace", e.target.value)}
            >
              <option value="">todos los namespaces</option>
              {namespaces.map((n) => (
                <option key={n} value={n}>{n}</option>
              ))}
            </select>
            <label className="sr-only" htmlFor="act-op">Filtrar por operación</label>
            <select
              id="act-op"
              className={selectCls}
              value={filters.op}
              onChange={(e) => setFilter("op", e.target.value)}
            >
              <option value="">todas las ops</option>
              {ops.map((o) => (
                <option key={o} value={o}>{o}</option>
              ))}
            </select>
            <label className="sr-only" htmlFor="act-outcome">Filtrar por outcome</label>
            <select
              id="act-outcome"
              className={selectCls}
              value={filters.outcome}
              onChange={(e) => setFilter("outcome", e.target.value)}
            >
              <option value="">ok + err</option>
              <option value="ok">✓ ok</option>
              <option value="err">✕ err</option>
            </select>
            {filterActive && (
              <button
                type="button"
                onClick={() => setFilters(NO_FILTERS)}
                className="press border-2 border-foreground bg-background px-2 py-1 font-tech text-[10px] uppercase tracking-widest"
              >
                ✕ limpiar
              </button>
            )}
            {loading && (
              <span className="ml-auto font-tech text-[10px] uppercase tracking-widest text-neon" role="status">
                cargando…
              </span>
            )}
          </div>

          {loadError && (
            <div role="alert" className="border-b-4 border-foreground bg-card px-4 py-2 font-tech text-[11px] text-neon">
              error al leer el audit log: {loadError}
            </div>
          )}

          {events.length === 0 && !loadError ? (
            <p className="p-8 text-center font-tech text-[11px] uppercase tracking-widest text-muted-foreground">
              {filterActive
                ? "ningún evento coincide con los filtros"
                : "sin eventos de auditoría todavía — hacé un put/delete para generar actividad"}
            </p>
          ) : (
            <>
              <div className="border-b-4 border-foreground p-4">
                <div className="mb-3 flex items-baseline gap-2">
                  <span className="font-tech text-[10px] uppercase tracking-widest text-neon">
                    timeline · {granularity === "hour" ? "por hora" : "por día"}
                  </span>
                  <span className="font-tech text-[10px] text-muted-foreground">
                    {events.length} evento{events.length === 1 ? "" : "s"} cargados
                  </span>
                </div>
                <Timeline events={events} granularity={granularity} onInspect={handleInspect} onPeek={setPeek} />
              </div>

              {/* Tabla filtrable (contrato c) */}
              <div className="overflow-x-auto scroll-manga">
                <table className="w-full border-collapse text-left">
                  <caption className="sr-only">Eventos de auditoría</caption>
                  <thead>
                    <tr className="border-b-2 border-foreground font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
                      <th scope="col" className="px-2 py-1.5">op</th>
                      <th scope="col" className="px-2 py-1.5">namespace</th>
                      <th scope="col" className="px-2 py-1.5">key</th>
                      <th scope="col" className="px-2 py-1.5">outcome</th>
                      <th scope="col" className="px-2 py-1.5">timestamp</th>
                      <th scope="col" className="px-2 py-1.5">reason</th>
                    </tr>
                  </thead>
                  <tbody>
                    {events.map((e, i) => (
                      <tr
                        key={`${e.timestamp}-${i}`}
                        onClick={() => handleInspect(e)}
                        onMouseEnter={() => setPeek(e)}
                        onMouseLeave={() => setPeek(null)}
                        className="cursor-pointer border-b border-foreground hover:bg-muted"
                      >
                        <td className="px-2 py-1.5"><OpChip op={e.op} /></td>
                        <td className="px-2 py-1.5">
                          <span className="border-2 border-foreground bg-background px-1.5 py-0.5 font-tech text-[10px]">
                            {e.namespace}
                          </span>
                        </td>
                        <td className="px-2 py-1.5">
                          <button
                            type="button"
                            onClick={(ev) => {
                              ev.stopPropagation();
                              handleInspect(e);
                            }}
                            className="font-tech text-[12px] underline decoration-neon underline-offset-2 hover:text-neon"
                            title={
                              e.namespace === "N/A" || e.key === "N/A"
                                ? "operación sin registro asociado"
                                : `abrir ${e.namespace}:${e.key} en Inspector`
                            }
                          >
                            {e.key}
                          </button>
                        </td>
                        <td className="px-2 py-1.5"><OutcomeBadge outcome={e.outcome} /></td>
                        <td className="px-2 py-1.5 font-tech text-[11px] text-muted-foreground">
                          {eventClock(e.timestamp)}
                        </td>
                        <td className="max-w-[240px] truncate px-2 py-1.5 font-tech text-[10px] text-muted-foreground">
                          {e.reason ?? "—"}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>

              {/* Peek bar: hover → key/record (sin saltar al Inspector) */}
              <div
                className="min-h-[28px] border-t-4 border-foreground bg-background px-4 py-1.5 font-tech text-[10px] uppercase tracking-widest"
                aria-live="polite"
              >
                {peek ? (
                  <span className="truncate">
                    <span className="text-neon">hover</span> · {peek.namespace}:{peek.key} — {peek.op}{" "}
                    {peek.outcome === "err" ? "✕" : "✓"} · clic para abrir en Inspector
                  </span>
                ) : (
                  <span className="text-muted-foreground">hover sobre una fila para ver el registro</span>
                )}
              </div>

              {/* Paginación con cursor de VS-12 */}
              <div className="flex items-center justify-between gap-2 border-t-4 border-foreground p-3">
                <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
                  {cursor != null ? "hay más eventos viejos" : events.length > 0 ? "fin del log" : ""}
                </span>
                {cursor != null && (
                  <button
                    type="button"
                    onClick={() => void fetchPage(cursor, false)}
                    disabled={loading}
                    className="press border-2 border-foreground bg-background px-3 py-1.5 text-xs font-semibold"
                  >
                    {loading ? "…" : "← cargar más viejos"}
                  </button>
                )}
              </div>
            </>
          )}
        </>
      )}
    </section>
  );
}