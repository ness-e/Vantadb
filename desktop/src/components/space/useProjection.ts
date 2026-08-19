import { useCallback, useEffect, useRef, useState } from "react";
import { listPage, type MemoryRecord } from "../../vanta";
import type { ProjectResponse } from "./projection.worker";

const PAGE_SIZE = 5000;
const MAX_POINTS = 100_000;
const SEED = 42;

export type ProjectionPhase = "idle" | "loading" | "done" | "error";

export interface ProjectionPoint {
  /** NDC x in [-1, 1]. */
  x: number;
  /** NDC y in [-1, 1]. */
  y: number;
  /** Namespace color key (index into the palette). */
  colorKey: number;
  /** Record this point represents. */
  record: MemoryRecord;
}

/** Projected record selection — indices into `points` (ready for ESPACIO-02). */
export interface ProjectionState {
  phase: ProjectionPhase;
  points: ProjectionPoint[];
  /** Namespaces present in the projection, in first-seen order. */
  namespaces: string[];
  /** Selected point indices (lasso output, ready for batch ops). */
  selected: Set<number>;
  error: string | null;
  /** True while a projection is in flight (namespace change cancels it). */
  cancelled: boolean;
}

const IDLE: ProjectionState = {
  phase: "idle",
  points: [],
  namespaces: [],
  selected: new Set(),
  error: null,
  cancelled: false,
};

/** Page `listPage` until the cursor runs out or `MAX_POINTS` is reached. */
async function fetchAllVectors(
  namespace: string | undefined,
  shouldCancel: () => boolean,
): Promise<{ records: MemoryRecord[]; namespaces: string[] }> {
  const records: MemoryRecord[] = [];
  const namespaces: string[] = [];
  let cursor: number | undefined;
  for (;;) {
    if (shouldCancel()) break;
    const page = await listPage({ namespace, limit: PAGE_SIZE, cursor });
    if (shouldCancel()) break;
    for (const r of page.records) {
      if (records.length >= MAX_POINTS) break;
      records.push(r);
      if (r.namespace && !namespaces.includes(r.namespace)) {
        namespaces.push(r.namespace);
      }
    }
    if (
      records.length >= MAX_POINTS ||
      page.next_cursor == null ||
      page.records.length === 0
    ) {
      break;
    }
    cursor = page.next_cursor;
  }
  return { records, namespaces };
}

/**
 * Projects embeddings into 2D via a UMAP-js web worker.
 *
 * - Pages `listPage` (limit 5000) until the cursor is exhausted or 100k records.
 * - Only records carrying a `vector` are projected; the rest are skipped.
 * - Changing namespace cancels any in-flight projection (worker is terminated).
 * - Fixed seed → reproducible embedding.
 */
export function useProjection() {
  const [state, setState] = useState<ProjectionState>(IDLE);
  const workerRef = useRef<Worker | null>(null);
  const cancelledRef = useRef(false);
  const tokenRef = useRef(0);

  const stopWorker = useCallback(() => {
    cancelledRef.current = true;
    if (workerRef.current) {
      workerRef.current.terminate();
      workerRef.current = null;
    }
  }, []);

  useEffect(() => stopWorker, [stopWorker]);

  const project = useCallback(
    async (namespace?: string) => {
      stopWorker();
      const token = ++tokenRef.current;
      cancelledRef.current = false;
      setState({ ...IDLE, phase: "loading", cancelled: false });

      try {
        const { records, namespaces } = await fetchAllVectors(namespace, () =>
          cancelledRef.current,
        );
        if (token !== tokenRef.current || cancelledRef.current) return;

        const vectors: number[][] = [];
        const points: ProjectionPoint[] = [];
        for (const r of records) {
          if (!r.vector || r.vector.length === 0) continue;
          const colorKey = namespaces.indexOf(r.namespace);
          vectors.push(r.vector);
          points.push({ x: 0, y: 0, colorKey, record: r });
        }
        if (vectors.length === 0) {
          setState({
            ...IDLE,
            phase: "error",
            error: "No hay registros con vector para proyectar",
          });
          return;
        }

        const worker = new Worker(
          new URL("./projection.worker.ts", import.meta.url),
          { type: "module" },
        );
        workerRef.current = worker;

        worker.onmessage = (e: MessageEvent<ProjectResponse>) => {
          if (token !== tokenRef.current) return;
          const msg = e.data;
          if (msg.type === "done") {
            msg.points.forEach(([x, y], i) => {
              points[i].x = x;
              points[i].y = y;
            });
            setState({
              phase: "done",
              points,
              namespaces,
              selected: new Set(),
              error: null,
              cancelled: false,
            });
          } else if (msg.type === "error") {
            setState({ ...IDLE, phase: "error", error: msg.message });
          }
        };
        worker.onerror = (e) => {
          if (token !== tokenRef.current) return;
          setState({ ...IDLE, phase: "error", error: e.message });
        };

        worker.postMessage({ type: "project", vectors, seed: SEED });
      } catch (err) {
        if (token !== tokenRef.current) return;
        setState({
          ...IDLE,
          phase: "error",
          error: err instanceof Error ? err.message : String(err),
        });
      }
    },
    [stopWorker],
  );

  /** Select / deselect point indices (ESPACIO-02: batch ops on `selected`). */
  const setSelected = useCallback((next: Set<number>) => {
    setState((s) => ({ ...s, selected: next }));
  }, []);

  return { state, project, setSelected };
}