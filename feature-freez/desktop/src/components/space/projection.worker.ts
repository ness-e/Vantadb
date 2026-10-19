/// <reference lib="webworker" />
import { UMAP } from "umap-js";

export type ProjectRequest = {
  type: "project";
  /** Raw vectors to embed (each row = one record's `vector` field). */
  vectors: number[][];
  /** Fixed seed so projections are reproducible across runs. */
  seed: number;
};

export type ProjectResponse =
  | { type: "done"; points: number[][] }
  | { type: "error"; message: string };

/** Deterministic PRNG (mulberry32) so UMAP yields the same embedding per seed. */
function mulberry32(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

let cancelled = false;

self.onmessage = (event: MessageEvent<ProjectRequest | { type: "cancel" }>) => {
  const msg = event.data;
  if (msg.type === "cancel") {
    cancelled = true;
    return;
  }
  if (msg.type !== "project") return;

  cancelled = false;
  const { vectors, seed } = msg;

  if (vectors.length === 0) {
    postMessage({ type: "error", message: "No hay vectores para proyectar" } satisfies ProjectResponse);
    return;
  }

  try {
    const umap = new UMAP({
      nComponents: 2,
      nNeighbors: Math.min(15, Math.max(2, vectors.length - 1)),
      minDist: 0.1,
      random: mulberry32(seed),
    });

    const embedding = umap.fitAsync(vectors, (epoch) => {
      if (cancelled) return false;
      if (epoch % 10 === 0) {
        postMessage({ type: "progress", epoch } as never);
      }
      return true;
    });

    void embedding
      .then((raw) => {
        if (cancelled) return;
        // Normalize to NDC [-1, 1] — regl-scatterplot expects that range.
        const xs = raw.map((p) => p[0]);
        const ys = raw.map((p) => p[1]);
        const minX = Math.min(...xs);
        const maxX = Math.max(...xs);
        const minY = Math.min(...ys);
        const maxY = Math.max(...ys);
        const rangeX = maxX - minX || 1;
        const rangeY = maxY - minY || 1;
        const points = raw.map((p) => [
          ((p[0] - minX) / rangeX) * 2 - 1,
          ((p[1] - minY) / rangeY) * 2 - 1,
        ]);
        postMessage({ type: "done", points } satisfies ProjectResponse);
      })
      .catch((err: unknown) => {
        if (cancelled) return;
        postMessage({
          type: "error",
          message: err instanceof Error ? err.message : String(err),
        } satisfies ProjectResponse);
      });
  } catch (err) {
    postMessage({
      type: "error",
      message: err instanceof Error ? err.message : String(err),
    } satisfies ProjectResponse);
  }
};