// SOP operational panels (ADMIN-06). Three runbook panels: WAL Replay,
// Reindex, and Health Check. Health runs `vanta_health` live. WAL Replay and
// Reindex are READ-ONLY: the core exposes no replay/reindex trigger yet
// (DESKTOP-28 / ADMIN-06), so they only surface the last recorded metrics
// values — no fake action buttons.
import { useCallback, useEffect, useState } from "react";
import {
  health,
  HealthReport,
  metrics,
  OperationalMetrics,
  vantaErrorMessage,
} from "../vanta";

type Status = "ok" | "err";
interface Result {
  status: Status;
  detail: string;
}

const PANEL =
  "flex flex-col gap-2 border-2 border-foreground bg-card p-3 shadow-ink-sm";

/** ok detail when a metrics snapshot exists, err detail when polling failed.
 * El texto crudo del error (TypeError/HTTP) queda en consola; en la UI solo
 * el estado amigable (visual-critique: no filtrar stack a la cara del usuario). */
function metricsResult(err: string | null, okDetail: string | null): Result | null {
  if (err) {
    console.warn("[vanta] metrics poll:", err);
    return { status: "err", detail: "sin snapshot de métricas" };
  }
  return okDetail ? { status: "ok", detail: okDetail } : null;
}

function SopResult({ result }: { result: Result | null }) {
  return (
    <span
      className={`max-w-[60%] text-right font-tech text-xs [overflow-wrap:anywhere] ${
        result ? (result.status === "ok" ? "text-foreground" : "text-destructive") : "text-muted-foreground"
      }`}
    >
      {result ? result.detail : "Sin datos aún"}
    </span>
  );
}

export default function SopPanel() {
  const [m, setM] = useState<OperationalMetrics | null>(null);
  const [metricsErr, setMetricsErr] = useState<string | null>(null);
  const [healthResult, setHealthResult] = useState<Result | null>(null);
  const [healthBusy, setHealthBusy] = useState(false);

  const pollMetrics = useCallback(async () => {
    try {
      setM(await metrics());
      setMetricsErr(null);
    } catch (e) {
      setMetricsErr(vantaErrorMessage(e));
    }
  }, []);

  const runHealth = useCallback(async () => {
    setHealthBusy(true);
    try {
      const h: HealthReport = await health();
      setHealthResult({
        status: "ok",
        detail: `${h.status} · ${h.backend} · ${h.latency_ms}ms${h.message ? ` — ${h.message}` : ""}`,
      });
    } catch (e) {
      setHealthResult({ status: "err", detail: vantaErrorMessage(e) });
    } finally {
      setHealthBusy(false);
    }
  }, []);

  useEffect(() => {
    pollMetrics();
    runHealth();
  }, [pollMetrics, runHealth]);

  return (
    <section
      aria-label="Operaciones SOP"
      className="border-[3px] border-foreground bg-card p-4 shadow-ink"
    >
      <div className="mb-3 flex items-center justify-between gap-2">
        <h2 className="m-0 font-tech text-xs uppercase tracking-widest">Operaciones SOP</h2>
        <span className="text-muted-foreground">runbook manual</span>
      </div>
      <div className="grid gap-4 md:grid-cols-3">
        {/* ADMIN-06/DESKTOP-28: sin trigger en core → solo lectura, sin botón falso. */}
        <article className={PANEL}>
          <div className="flex items-center justify-between gap-2">
            <h3 className="m-0 font-tech text-xs uppercase tracking-widest text-muted-foreground">WAL Replay</h3>
            <span
              title="Solo lectura — el core aún no expone trigger de replay (ADMIN-06)"
              className="border border-foreground px-1 font-tech text-[10px] uppercase tracking-widest text-muted-foreground"
            >
              solo lectura
            </span>
          </div>
          <p className="m-0 flex-1 text-xs text-muted-foreground">
            Último replay registrado por el motor (no hay comando de disparo aún).
          </p>
          <SopResult
            result={metricsResult(
              metricsErr,
              m ? `replay de ${m.wal_records_replayed.toLocaleString()} registros en ${m.wal_replay_ms}ms` : null,
            )}
          />
        </article>
        <article className={PANEL}>
          <div className="flex items-center justify-between gap-2">
            <h3 className="m-0 font-tech text-xs uppercase tracking-widest text-muted-foreground">Reindex</h3>
            <span
              title="Solo lectura — el core aún no expone trigger de reindex (ADMIN-06)"
              className="border border-foreground px-1 font-tech text-[10px] uppercase tracking-widest text-muted-foreground"
            >
              solo lectura
            </span>
          </div>
          <p className="m-0 flex-1 text-xs text-muted-foreground">
            Últimos rebuilds registrados (no hay comando de disparo aún).
          </p>
          <SopResult
            result={metricsResult(
              metricsErr,
              m
                ? `ANN ${m.ann_rebuild_ms}ms · derived ${m.derived_rebuild_ms}ms · text ${m.text_index_rebuild_ms}ms`
                : null,
            )}
          />
        </article>
        <article className={PANEL}>
          <h3 className="m-0 font-tech text-xs uppercase tracking-widest text-muted-foreground">Health Check</h3>
          <p className="m-0 flex-1 text-xs text-muted-foreground">
            Probe del motor embebido vía vanta_health.
          </p>
          <div className="flex items-center justify-between gap-2">
            <button
              type="button"
              onClick={runHealth}
              disabled={healthBusy}
              className="press cursor-pointer border-2 border-foreground bg-background px-2.5 py-1.5 text-sm disabled:cursor-default disabled:opacity-50"
            >
              {healthBusy ? "Verificando…" : "Ejecutar chequeo"}
            </button>
            <SopResult result={healthResult} />
          </div>
        </article>
      </div>
    </section>
  );
}
