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
import { TriangleAlert } from "lucide-react";
import { tp, tt, type DesktopLang } from "../../i18n";
import { connectionPrefs } from "../../store/connections";

interface Props {
  onNotice: (msg: string) => void;
  /** Abrir un registro en el Inspector (el shell resuelve key → record). */
  onInspect: (namespace: string, key: string) => void;
  lang?: DesktopLang;
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

export default function ActivityPanel({ onNotice, onInspect, lang = connectionPrefs.get().lang ?? "es" }: Props) {
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
      onNotice(tp(lang, "activity.noRecordNotice", "{op}: operación sin registro asociado ({ns}:{key})", { op: e.op, ns: e.namespace, key: e.key }));
      return;
    }
    onInspect(e.namespace, e.key);
  }

  const filterActive = filters.namespace !== "" || filters.op !== "" || filters.outcome !== "";

  const selectCls =
    "border-2 border-foreground bg-background px-2 py-1 font-tech text-[11px] uppercase tracking-wider";

  return (
    <section className="press-lg border-4 border-foreground bg-card" aria-label={tt(lang, "activity.aria", "Actividad (audit log)")}>
      {/* Header */}
      <div className="border-b-4 border-foreground p-4">
        <div className="flex flex-wrap items-baseline justify-between gap-2">
          <h2 className="font-display text-3xl text-stencil">ACTIVITY</h2>
          <div className="flex items-center gap-2">
            <div className="flex border-2 border-foreground bg-background font-tech text-[10px]" role="group" aria-label={tt(lang, "activity.groupAria", "Agrupación temporal")}>
              {(["hour", "day"] as const).map((g) => (
                <button
                  key={g}
                  type="button"
                  onClick={() => setGranularity(g)}
                  aria-pressed={granularity === g}
                  // UX-15: estos botones no tenían `.press` (único grupo del
                  // shell sin el efecto físico del design system).
                  className={`press px-2 py-1 uppercase tracking-widest ${
                    granularity === g ? "bg-neon text-background" : "text-muted-foreground"
                  }`}
                >
                  {g === "hour" ? tt(lang, "activity.hourBtn", "por hora") : tt(lang, "activity.dayBtn", "por día")}
                </button>
              ))}
            </div>
            <button
              type="button"
              onClick={() => void fetchPage(null, true)}
              disabled={loading}
              className="press border-2 border-foreground bg-background px-2 py-1 font-tech text-[10px] uppercase tracking-widest"
              title={tt(lang, "activity.reloadTitle", "Recargar desde el tail del log")}
              aria-label={tt(lang, "activity.reload", "⟳ recargar")}
            >
              ⟳
            </button>
          </div>
        </div>
        <p className="mt-1 font-tech text-[11px] text-muted-foreground">
          {tt(lang, "activity.hint", "audit log del backend activo — escrituras, borrados, export/import · newest-first")}
        </p>
      </div>

      {unconfigured ? (
        /* Empty state honesto (contrato e): audit no configurado. UX-13: el
           texto al usuario no filtra internos (Unsupported/ NativeConnection::
           open); el detalle técnico queda en <details>. */
        <div role="alert" className="p-6">
          <div className="border-2 border-dashed border-foreground bg-background p-4">
            <div className="font-tech text-[11px] font-bold uppercase tracking-widest text-neon">
              <TriangleAlert className="mr-1 inline h-3.5 w-3.5 align-[-2px]" strokeWidth={2.5} aria-hidden="true" />
              {tt(lang, "activity.unconfiguredTitle", "audit log no habilitado")}
            </div>
            <p className="mt-2 text-sm">
              {tt(lang, "activity.unconfiguredBody", "La conexión activa no tiene el audit log configurado, así que no hay actividad para mostrar. Se habilita al conectar un backend nativo — conectá uno desde RESUMEN y volvé a abrir ACTIVITY.")}
            </p>
            <details className="mt-2">
              <summary className="cursor-pointer font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
                {tt(lang, "activity.techDetail", "detalle técnico")}
              </summary>
              <p className="mt-1 font-tech text-[10px] text-muted-foreground">
                El backend rechaza la consulta con <code>Unsupported("audit log no configurado")</code>.
                Al conectar un backend nativo, <code>NativeConnection::open</code> escribe el
                log en <code>&lt;storage_path&gt;/audit.jsonl</code> (VantaConfig.audit_log_path).
              </p>
            </details>
          </div>
        </div>
      ) : (
        <>
          {/* Filtros (namespace/op/outcome) — el filtrado real corre en Rust */}
          <div className="flex flex-wrap items-center gap-2 border-b-4 border-foreground bg-background p-3">
            <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
              {tt(lang, "activity.filterLabel", "filtrar")}
            </span>
            <label className="sr-only" htmlFor="act-namespace">{tt(lang, "activity.nsFilterAria", "Filtrar por namespace")}</label>
            <select
              id="act-namespace"
              className={selectCls}
              value={filters.namespace}
              onChange={(e) => setFilter("namespace", e.target.value)}
            >
              <option value="">{tt(lang, "activity.nsAll", "todos los namespaces")}</option>
              {namespaces.map((n) => (
                <option key={n} value={n}>{n}</option>
              ))}
            </select>
            <label className="sr-only" htmlFor="act-op">{tt(lang, "activity.opFilterAria", "Filtrar por operación")}</label>
            <select
              id="act-op"
              className={selectCls}
              value={filters.op}
              onChange={(e) => setFilter("op", e.target.value)}
            >
              <option value="">{tt(lang, "activity.opAll", "todas las ops")}</option>
              {ops.map((o) => (
                <option key={o} value={o}>{o}</option>
              ))}
            </select>
            <label className="sr-only" htmlFor="act-outcome">{tt(lang, "activity.outcomeFilterAria", "Filtrar por outcome")}</label>
            <select
              id="act-outcome"
              className={selectCls}
              value={filters.outcome}
              onChange={(e) => setFilter("outcome", e.target.value)}
            >
              <option value="">{tt(lang, "activity.outcomeAll", "ok + err")}</option>
              <option value="ok">✓ ok</option>
              <option value="err">✕ err</option>
            </select>
            {filterActive && (
              <button
                type="button"
                onClick={() => setFilters(NO_FILTERS)}
                className="press border-2 border-foreground bg-background px-2 py-1 font-tech text-[10px] uppercase tracking-widest"
              >
                {tt(lang, "activity.clearFilters", "✕ limpiar")}
              </button>
            )}
            {loading && (
              <span className="ml-auto font-tech text-[10px] uppercase tracking-widest text-neon" role="status">
                {tt(lang, "activity.loading", "cargando…")}
              </span>
            )}
          </div>

          {loadError && (
            <div role="alert" className="border-b-4 border-foreground bg-card px-4 py-2 font-tech text-[11px] text-neon">
              {tp(lang, "activity.loadError", "error al leer el audit log: {e}", { e: loadError })}
            </div>
          )}

          {events.length === 0 && !loadError ? (
            <p className="p-8 text-center font-tech text-[11px] uppercase tracking-widest text-muted-foreground">
              {filterActive
                ? tt(lang, "activity.emptyFiltered", "ningún evento coincide con los filtros")
                : tt(lang, "activity.emptyNone", "sin eventos de auditoría todavía — hacé un put/delete para generar actividad")}
            </p>
          ) : (
            <>
              <div className="border-b-4 border-foreground p-4">
                <div className="mb-3 flex items-baseline gap-2">
                  <span className="font-tech text-[10px] uppercase tracking-widest text-neon">
                    {tp(lang, "activity.timelineTitle", "timeline · {g}", { g: granularity === "hour" ? tt(lang, "activity.hourBtn", "por hora") : tt(lang, "activity.dayBtn", "por día") })}
                  </span>
                  <span className="font-tech text-[10px] text-muted-foreground">
                    {events.length === 1
                      ? tp(lang, "activity.eventsLoadedOne", "{n} evento cargado", { n: String(events.length) })
                      : tp(lang, "activity.eventsLoadedMany", "{n} eventos cargados", { n: String(events.length) })}
                  </span>
                </div>
                <Timeline events={events} granularity={granularity} lang={lang} onInspect={handleInspect} onPeek={setPeek} />
              </div>

              {/* Tabla filtrable (contrato c) */}
              <div className="overflow-x-auto scroll-manga">
                <table className="w-full border-collapse text-left">
                  <caption className="sr-only">{tt(lang, "activity.tableCaption", "Eventos de auditoría")}</caption>
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
                        <td className="px-2 py-1.5"><OpChip op={e.op} lang={lang} /></td>
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
                                ? tt(lang, "activity.rowNoRecord", "operación sin registro asociado")
                                : tp(lang, "activity.openInInspector", "abrir {ns}:{key} en Inspector", { ns: e.namespace, key: e.key })
                            }
                          >
                            {e.key}
                          </button>
                        </td>
                        <td className="px-2 py-1.5"><OutcomeBadge outcome={e.outcome} lang={lang} /></td>
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
                    <span className="text-neon">hover</span> · {tp(lang, "activity.peekRest", "{ns}:{key} — {op} {ok} · clic para abrir en Inspector", { ns: peek.namespace, key: peek.key, op: peek.op, ok: peek.outcome === "err" ? "✕" : "✓" })}
                  </span>
                ) : (
                  <span className="text-muted-foreground">{tt(lang, "activity.peekEmpty", "hover sobre una fila para ver el registro")}</span>
                )}
              </div>

              {/* Paginación con cursor de VS-12 */}
              <div className="flex items-center justify-between gap-2 border-t-4 border-foreground p-3">
                <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
                  {cursor != null ? tt(lang, "activity.moreOld", "hay más eventos viejos") : events.length > 0 ? tt(lang, "activity.logEnd", "fin del log") : ""}
                </span>
                {cursor != null && (
                  <button
                    type="button"
                    onClick={() => void fetchPage(cursor, false)}
                    disabled={loading}
                    className="press border-2 border-foreground bg-background px-3 py-1.5 text-xs font-semibold"
                  >
                    {loading ? tt(lang, "activity.loadingMore", "…") : tt(lang, "activity.loadMore", "← cargar más viejos")}
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