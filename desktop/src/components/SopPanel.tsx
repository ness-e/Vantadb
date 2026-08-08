// SOP operational panels (ADMIN-06). Three actionable runbook panels: WAL
// Replay, Reindex, and Health Check. Health runs `vanta_health` live; Replay
// and Reindex have no core trigger command yet, so their action re-polls
// `vanta_metrics` and surfaces the last recorded value as status. When the
// core exposes replay/reindex triggers, swap the Refresh actions for real ones.
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

interface SopPanelProps {
  title: string;
  description: string;
  actionLabel: string;
  busy: boolean;
  result: Result | null;
  onAction: () => void;
}

function SopCard({ title, description, actionLabel, busy, result, onAction }: SopPanelProps) {
  return (
    <article className="sop-panel">
      <h3>{title}</h3>
      <p className="sop-desc">{description}</p>
      <div className="sop-foot">
        <button onClick={onAction} disabled={busy}>
          {busy ? "Working…" : actionLabel}
        </button>
        <span className="sop-result" data-status={result ? result.status : "idle"}>
          {result ? result.detail : "No run yet"}
        </span>
      </div>
    </article>
  );
}

/** ok detail when a metrics snapshot exists, err detail when polling failed. */
function metricsResult(err: string | null, okDetail: string | null): Result | null {
  if (err) return { status: "err", detail: err };
  return okDetail ? { status: "ok", detail: okDetail } : null;
}

export default function SopPanel() {
  const [m, setM] = useState<OperationalMetrics | null>(null);
  const [metricsErr, setMetricsErr] = useState<string | null>(null);
  const [metricsBusy, setMetricsBusy] = useState(false);
  const [healthResult, setHealthResult] = useState<Result | null>(null);
  const [healthBusy, setHealthBusy] = useState(false);

  const pollMetrics = useCallback(async () => {
    setMetricsBusy(true);
    try {
      setM(await metrics());
      setMetricsErr(null);
    } catch (e) {
      setMetricsErr(vantaErrorMessage(e));
    } finally {
      setMetricsBusy(false);
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
    <section className="panel sop" aria-label="SOP operations">
      <div className="panel-head">
        <h2>SOP Operations</h2>
        <span className="muted">manual runbook</span>
      </div>
      <div className="sop-grid">
        <SopCard
          title="WAL Replay"
          description="Replay the write-ahead log. No trigger command yet — shows the last recorded replay from metrics."
          actionLabel="Refresh"
          busy={metricsBusy}
          result={metricsResult(
            metricsErr,
            m ? `replayed ${m.wal_records_replayed.toLocaleString()} records in ${m.wal_replay_ms}ms` : null,
          )}
          onAction={pollMetrics}
        />
        <SopCard
          title="Reindex"
          description="Rebuild derived and text indexes. No trigger command yet — shows the last rebuild timings from metrics."
          actionLabel="Refresh"
          busy={metricsBusy}
          result={metricsResult(
            metricsErr,
            m
              ? `ANN ${m.ann_rebuild_ms}ms · derived ${m.derived_rebuild_ms}ms · text ${m.text_index_rebuild_ms}ms`
              : null,
          )}
          onAction={pollMetrics}
        />
        <SopCard
          title="Health Check"
          description="Probe the native embedded engine via vanta_health."
          actionLabel="Run check"
          busy={healthBusy}
          result={healthResult}
          onAction={runHealth}
        />
      </div>
    </section>
  );
}
