// HomeOverview (VS-04): HOME/overview de la superficie RESUMEN — "overview
// first" (Shneiderman P3). Cards de resumen ESTÁTICAS (nada abre: la navegación
// profunda vive en el grid/inspector) con encoding redundante (color + ícono +
// texto). Estética manga/linocut de VS-01/VS-00 (tokens --color-neon, .press-lg,
// .ink-divider, .stagger-children, dark overrides).
//
// Fuente de datos (VS-CORE-02): namespace_stats reales del bridge para
// total/namespaces/expiring/expired (incluyen expirados no purgados que list()
// oculta), con fallback client-side desde list() solo si el backend no las
// expone. Los detalles por record (types, actividad, TTL próximos) siguen
// viniendo de list().
import { useCallback, useEffect, useState } from "react";
import { list, namespaceStats, MemoryRecord, NamespaceStatsMap } from "../../vanta";
import { Hourglass } from "lucide-react";

// 24h — espeja DEFAULT_EXPIRING_SOON_WINDOW_MS del core (src/sdk/types.rs:243).
const EXPIRING_WINDOW_MS = 24 * 60 * 60 * 1000;
// 7d — proxy local de "tendencia" (namespace_stats no expone actualizaciones).
const TREND_WINDOW_MS = 7 * 24 * 60 * 60 * 1000;
// Mismo límite que la sidebar (VS-03); los totales exactos vienen de las stats.
const LIST_LIMIT = 500;

interface NamespaceRow {
  name: string;
  count: number;
}

interface TypeRow {
  type: string;
  count: number;
}

interface TtlRow {
  key: string;
  namespace: string;
  expires_at_ms: number;
}

interface ActivityRow {
  key: string;
  namespace: string;
  updated_at_ms: number;
}

interface HomeData {
  total: number;
  updated7d: number;
  namespaces: NamespaceRow[];
  types: TypeRow[];
  expiringSoon: TtlRow[];
  expiringCount: number;
  expiredCount: number;
  withVector: number;
  activity: ActivityRow[];
}

/** Clasificación de un valor de metadata por tipo (VantaValue-lite). */
function valueType(v: unknown): string {
  if (v === null) return "null";
  if (Array.isArray(v)) return "list";
  switch (typeof v) {
    case "string":
      return "string";
    case "boolean":
      return "bool";
    case "number":
      return Number.isInteger(v) ? "int" : "float";
    default:
      return "obj";
  }
}

/** Deriva todas las cards desde una sola pasada sobre `list()` (fallback local). */
function deriveHomeData(records: MemoryRecord[], now: number): HomeData {
  const nsCounts = new Map<string, number>();
  const typeCounts = new Map<string, number>();
  const expiring: TtlRow[] = [];
  const activity: ActivityRow[] = [];
  let expiredCount = 0;
  let withVector = 0;
  let updated7d = 0;

  for (const r of records) {
    nsCounts.set(r.namespace, (nsCounts.get(r.namespace) ?? 0) + 1);

    const meta = r.metadata;
    if (meta) {
      for (const v of Object.values(meta)) {
        const t = valueType(v);
        typeCounts.set(t, (typeCounts.get(t) ?? 0) + 1);
      }
    }

    if (Array.isArray(r.vector) && r.vector.length > 0) withVector += 1;

    const u = r.updated_at_ms;
    if (u != null && u <= now) {
      if (now - u <= TREND_WINDOW_MS) updated7d += 1;
      activity.push({ key: r.id, namespace: r.namespace, updated_at_ms: u });
    }

    const e = r.expires_at_ms;
    if (e != null) {
      if (e > now && e - now <= EXPIRING_WINDOW_MS) {
        expiring.push({ key: r.id, namespace: r.namespace, expires_at_ms: e });
      } else if (e <= now) {
        // list() del core excluye expirados → 0 con el bridge actual (Notas).
        expiredCount += 1;
      }
    }
  }

  const byCount = (a: { count: number }, b: { count: number }) => b.count - a.count;

  return {
    total: records.length,
    updated7d,
    namespaces: [...nsCounts.entries()]
      .map(([name, count]) => ({ name, count }))
      .sort(byCount)
      .slice(0, 5),
    types: [...typeCounts.entries()]
      .map(([type, count]) => ({ type, count }))
      .sort(byCount)
      .slice(0, 6),
    expiringSoon: expiring.sort((a, b) => a.expires_at_ms - b.expires_at_ms).slice(0, 3),
    expiringCount: expiring.length,
    expiredCount,
    withVector,
    activity: activity.sort((a, b) => b.updated_at_ms - a.updated_at_ms).slice(0, 5),
  };
}

/** Reemplaza los conteos derivados de list() con las stats reales del bridge
 * (VS-CORE-02): total/namespaces/expiring/expired exactos (incluyen expirados
 * no purgados, que list() oculta). Los detalles por record (types, actividad,
 * próximos a expirar) siguen viniendo de list(). */
function mergeStats(base: HomeData, stats: NamespaceStatsMap): HomeData {
  const entries = Object.entries(stats);
  return {
    ...base,
    total: entries.reduce((sum, [, s]) => sum + s.count, 0),
    namespaces: entries
      .map(([name, s]) => ({ name, count: s.count }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 5),
    expiringCount: entries.reduce((sum, [, s]) => sum + s.expiring_soon, 0),
    expiredCount: entries.reduce((sum, [, s]) => sum + s.expired, 0),
  };
}

/** Cuenta regresiva compacta ("02:13:44", "1d 3h"). */
function fmtCountdown(ms: number): string {
  const s = Math.max(0, Math.floor(ms / 1000));
  const d = Math.floor(s / 86400);
  const h = Math.floor((s % 86400) / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  if (d > 0) return `${d}d ${h}h`;
  if (h > 0) return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
  return `${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
}

/** Edad relativa compacta ("ahora", "5s", "3m", "2h", "1d"). */
function relTime(ageMs: number): string {
  const s = Math.max(0, Math.floor(ageMs / 1000));
  if (s < 5) return "ahora";
  if (s < 60) return `${s}s`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h`;
  return `${Math.floor(h / 24)}d`;
}

function Card({ icon, title, children }: { icon: React.ReactNode; title: string; children: React.ReactNode }) {
  return (
    <article className="press-lg border-4 border-foreground bg-card p-5">
      <div className="flex items-center justify-between">
        <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">{title}</span>
        <span className="text-neon" aria-hidden="true">
          {icon}
        </span>
      </div>
      {children}
    </article>
  );
}

/** UX-12: header ÚNICO de RESUMEN — el mismo h1 en loading y loaded, para que
 * "VISTA GENERAL" no salte entre dos layouts (antes: card centrada text-3xl en
 * carga vs h1 text-4xl a la izquierda cargado). El h1 vive siempre acá. */
function OverviewHeader({
  refresh,
  children,
}: {
  refresh?: () => void;
  children?: React.ReactNode;
}) {
  return (
    <div className="flex flex-wrap items-end justify-between gap-2">
      <div>
        <div className="font-tech text-[11px] uppercase tracking-widest text-accent-text">Overview</div>
        <h1 className="font-display text-4xl text-stencil">VISTA GENERAL</h1>
      </div>
      <div className="flex items-center gap-2">
        {children}
        {refresh && (
          <button
            type="button"
            onClick={refresh}
            className="press flex h-7 w-7 items-center justify-center border-2 border-foreground bg-background text-sm"
            title="Actualizar resumen"
            aria-label="Actualizar resumen"
          >
            ⟳
          </button>
        )}
      </div>
    </div>
  );
}

// Ciclo de colores por fila del histograma (encoding redundante: color + label + %).
const TYPE_COLORS = ["bg-foreground", "bg-neon", "bg-chart-3", "bg-chart-5", "bg-chart-4", "bg-muted-foreground"];

export default function HomeOverview({ active }: { active: boolean }) {
  const [data, setData] = useState<HomeData | null>(null);
  const [failed, setFailed] = useState(false);
  const [tick, setTick] = useState(0);
  const refresh = useCallback(() => setTick((t) => t + 1), []);

  useEffect(() => {
    let alive = true;
    if (!active) {
      setData(null);
      setFailed(false);
      return;
    }
    setFailed(false);
    list({ limit: LIST_LIMIT })
      .then((records) => {
        if (!alive) return;
        const base = deriveHomeData(records, Date.now());
        namespaceStats()
          .then((stats) => {
            // Stats reales (VS-CORE-02): override de total/ns/expiring/expired.
            if (alive) setData(mergeStats(base, stats));
          })
          .catch(() => {
            // Fallback local: backend sin stats (o error transitorio).
            if (alive) setData(base);
          });
      })
      .catch(() => {
        if (alive) setFailed(true);
      });
    return () => {
      alive = false;
    };
  }, [active, tick]);

  if (!active || data === null) {
    // UX-12: mismo header que el estado cargado — solo cambia el cuerpo.
    return (
      <section aria-label="Resumen de la memoria">
        <OverviewHeader />
        <div className="ink-divider mt-4" aria-hidden="true" />
        <div className="mt-6 border-2 border-dashed border-foreground bg-card p-8 text-center">
          <p className="font-tech text-[11px] uppercase tracking-widest text-muted-foreground">
            {failed ? "no se pudo leer el backend" : "cargando…"}
          </p>
        </div>
      </section>
    );
  }

  const now = Date.now();
  const typeTotal = data.types.reduce((s, x) => s + x.count, 0);
  const vectorPct = data.total ? Math.round((data.withVector / data.total) * 100) : 0;

  return (
    <section aria-label="Resumen de la memoria">
      {/* Header + sync (UX-12: OverviewHeader compartido con el estado de carga) */}
      <OverviewHeader refresh={refresh}>
        <span className="font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
          <span className="text-accent-text">▲ {data.updated7d}</span> actualizados / 7d
        </span>
      </OverviewHeader>

      <div className="ink-divider mt-4" aria-hidden="true" />

      {/* Cards overview (estáticas — la navegación profunda es del grid/inspector) */}
      <div className="stagger-children mt-6 grid grid-cols-1 gap-5 sm:grid-cols-2 xl:grid-cols-3">
        {/* Total + tendencia (proxy 7d) */}
        <Card icon="▦" title="Total de registros">
          <div className="mt-2 font-display text-5xl">{data.total.toLocaleString()}</div>
          <div className="mt-2 flex flex-wrap items-center gap-2">
            <span className="inline-flex items-center gap-1 border-2 border-foreground bg-muted px-2 py-0.5 font-tech text-[10px] uppercase">
              <span className="text-accent-text">▲</span> {data.updated7d} / 7d
            </span>
            <span className="inline-flex items-center gap-1 border-2 border-foreground bg-background px-2 py-0.5 font-tech text-[10px] uppercase">
              <span className="text-accent-text">▤</span> {data.namespaces.length} ns
            </span>
          </div>
        </Card>

        {/* Por namespace */}
        <Card icon="▤" title="Por namespace">
          <div className="mt-3 space-y-2">
            {data.namespaces.length === 0 ? (
              <p className="font-tech text-[11px] text-muted-foreground">sin registros</p>
            ) : (
              data.namespaces.map((n) => {
                const pct = data.total ? Math.round((n.count / data.total) * 100) : 0;
                return (
                  <div key={n.name} className="border-2 border-foreground bg-background px-2 py-1.5">
                    <div className="flex items-center justify-between gap-2">
                      <span className="truncate font-tech text-[11px]">{n.name}</span>
                      <span className="font-display text-lg leading-none">{n.count}</span>
                    </div>
                    <div className="mt-1 h-1.5 border-2 border-foreground bg-background" aria-hidden="true">
                      <div className="h-full bg-neon" style={{ width: `${pct}%` }} />
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </Card>

        {/* Distribución de tipos metadata (mini-histograma) */}
        <Card icon="▦" title="Tipos de metadata">
          <div className="mt-3 space-y-2">
            {data.types.length === 0 ? (
              <p className="font-tech text-[11px] text-muted-foreground">sin metadata tipada</p>
            ) : (
              data.types.map((t, i) => {
                const pct = typeTotal ? Math.round((t.count / typeTotal) * 100) : 0;
                return (
                  <div key={t.type} className="flex items-center gap-2">
                    <span className="w-14 font-tech text-[10px] uppercase">{t.type}</span>
                    <div className="h-3 flex-1 border-2 border-foreground bg-background" aria-hidden="true">
                      <div className={`h-full ${TYPE_COLORS[i % TYPE_COLORS.length]}`} style={{ width: `${pct}%` }} />
                    </div>
                    <span className="w-10 text-right font-tech text-[10px]">{pct}%</span>
                  </div>
                );
              })
            )}
          </div>
        </Card>

        {/* Próximos a expirar (TTL) */}
        <Card
          icon={<Hourglass className="h-4 w-4" strokeWidth={2.5} />}
          title="Próximos a expirar"
        >
          <div className="mt-2 font-display text-5xl">{data.expiringCount}</div>
          <div className="mt-1 font-tech text-[11px] text-muted-foreground">en las próximas 24h</div>
          <div className="mt-3 space-y-2">
            {data.expiringSoon.length === 0 ? (
              <p className="font-tech text-[11px] text-muted-foreground">ninguno</p>
            ) : (
              data.expiringSoon.map((r) => {
                const remaining = Math.max(0, r.expires_at_ms - now);
                const pct = Math.max(2, Math.min(100, Math.round((remaining / EXPIRING_WINDOW_MS) * 100)));
                return (
                  <div key={r.key}>
                    <div className="flex items-center justify-between gap-2">
                      <span className="truncate font-tech text-[11px]">
                        {r.namespace}::{r.key}
                      </span>
                      <span className="shrink-0 font-tech text-[10px] text-accent-text">{fmtCountdown(remaining)}</span>
                    </div>
                    <div className="mt-0.5 h-2 border-2 border-foreground bg-background" aria-hidden="true">
                      <div className="h-full bg-neon" style={{ width: `${pct}%` }} />
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </Card>

        {/* Expirados recientes */}
        <Card icon="∅" title="Expirados recientes">
          <div className="mt-2 font-display text-5xl">{data.expiredCount}</div>
          <div className="mt-1 font-tech text-[11px] text-muted-foreground">expirados no purgados · auto-limpiados por WAL</div>
          <p className="mt-3 font-tech text-[10px] uppercase tracking-widest text-muted-foreground">
            incluye expirados que list() no trae — via namespace_stats (VS-CORE-02)
          </p>
        </Card>

        {/* Actividad reciente (updated_at desc — decisión usuario; audit log en Fase 1) */}
        <Card icon="◷" title="Actividad reciente">
          <div className="mt-3 space-y-2.5">
            {data.activity.length === 0 ? (
              <p className="font-tech text-[11px] text-muted-foreground">sin actividad reciente</p>
            ) : (
              data.activity.map((a) => (
                <div key={a.key} className="flex items-center gap-2">
                  <span className="h-2 w-2 shrink-0 bg-neon" aria-hidden="true" />
                  <span className="truncate font-tech text-[11px]">
                    {a.namespace}::{a.key}
                  </span>
                  <span className="ml-auto shrink-0 font-tech text-[10px] text-muted-foreground">
                    {relTime(now - a.updated_at_ms)}
                  </span>
                </div>
              ))
            )}
          </div>
        </Card>

        {/* Con vector */}
        <Card icon="⠿" title="Registros con vector">
          <div className="mt-2 font-display text-5xl">{data.withVector}</div>
          <div className="mt-1 font-tech text-[11px] text-muted-foreground">
            {vectorPct}% del total · embedding denso
          </div>
        </Card>
      </div>
    </section>
  );
}
