// Shared poller for `vanta_metrics` (DESKTOP-29). One module-level store +
// one setInterval (4s) regardless of how many components consume the hook;
// the interval starts with the first subscriber and stops with the last.
// Consumers read `history` (newest last, capped), `error` and `polledAt`.
import { useSyncExternalStore } from "react";
import { metrics, OperationalMetrics, vantaErrorMessage } from "../vanta";

const POLL_MS = 4000;
const WINDOW = 12;

export interface MetricsSnapshot {
  /** Last snapshots, newest last (max WINDOW). */
  history: OperationalMetrics[];
  error: string | null;
  polledAt: number | null;
}

let snap: MetricsSnapshot = { history: [], error: null, polledAt: null };
let timer: number | undefined;
let inFlight = false;
const listeners = new Set<() => void>();

function emit(): void {
  for (const l of listeners) l();
}

async function tick(): Promise<void> {
  if (inFlight) return; // slow backend: skip tick instead of piling up calls
  inFlight = true;
  try {
    const m = await metrics();
    snap = { history: [...snap.history.slice(-(WINDOW - 1)), m], error: null, polledAt: Date.now() };
  } catch (e) {
    snap = { ...snap, error: vantaErrorMessage(e) };
  } finally {
    inFlight = false;
  }
  emit();
}

function subscribe(listener: () => void): () => void {
  if (listeners.size === 0) {
    void tick();
    timer = window.setInterval(tick, POLL_MS);
  }
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
    if (listeners.size === 0 && timer !== undefined) {
      window.clearInterval(timer);
      timer = undefined;
    }
  };
}

function getSnapshot(): MetricsSnapshot {
  return snap;
}

/** Single shared subscription to `vanta_metrics` — use instead of per-component polls. */
export function useMetricsPoll(): MetricsSnapshot {
  return useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
}
