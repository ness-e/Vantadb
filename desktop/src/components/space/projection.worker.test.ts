// @vitest-environment node
// Smoke test ESPACIO-01: la proyección del worker responde. Stubea `self` /
// `postMessage` (el worker usa la API global de Web Workers) e importa el
// worker real para ejercitar el flujo UMAP-js completo con seed fijo.
// Nota: UMAP-js corre en JS puro; 100 pts ≈ 7.5s en node (en el browser va en
// un Web Worker, no bloquea la UI). Timeout de test ampliado por eso — esto
// valida correctness, no performance (perf → vanta-tuner, pipeline §1a).
import { describe, expect, it, vi } from "vitest";

// UMAP-js es lento en node (~15s/100 pts en máquina cargada); el default de
// vitest (5s) mata el test antes del "done". El waitForDone ya espera 20s.
vi.setConfig({ testTimeout: 60_000 });

function makeVectors(n: number, dim: number, cluster: number, noise: number): number[][] {
  const rng = (() => {
    let a = 1234;
    return () => {
      a |= 0;
      a = (a + 0x6d2b79f5) | 0;
      let t = Math.imul(a ^ (a >>> 15), 1 | a);
      t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
      return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
  })();
  return Array.from({ length: n }, () =>
    Array.from({ length: dim }, () => cluster + (rng() - 0.5) * noise),
  );
}

function startWorker(vectors: number[][], seed: number) {
  const posted: unknown[] = [];
  vi.stubGlobal("postMessage", (msg: unknown) => posted.push(msg));
  vi.stubGlobal("self", globalThis);
  return { posted, importWorker: () => import("./projection.worker") };
}

async function waitForDone(posted: unknown[], timeoutMs = 20_000): Promise<number[][]> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const done = posted.find((m) => (m as { type?: string })?.type === "done");
    if (done) return (done as { points: number[][] }).points;
    const err = posted.find((m) => (m as { type?: string })?.type === "error");
    if (err) throw new Error((err as { message: string }).message);
    await new Promise((r) => setTimeout(r, 50));
  }
  throw new Error("timeout: worker no respondió 'done'");
}

describe("projection worker (UMAP-js)", () => {
  it("proyecta 100 vectores → 2D normalizado a NDC [-1,1]", async () => {
    const { posted, importWorker } = startWorker(makeVectors(100, 8, 0.4, 0.2), 42);
    await importWorker();
    (globalThis as unknown as { onmessage: (e: MessageEvent) => void }).onmessage({
      data: { type: "project", vectors: makeVectors(100, 8, 0.4, 0.2), seed: 42 },
    } as MessageEvent);

    const points = await waitForDone(posted);
    expect(points).toHaveLength(100);
    for (const [x, y] of points) {
      expect(x).toBeGreaterThanOrEqual(-1);
      expect(x).toBeLessThanOrEqual(1);
      expect(y).toBeGreaterThanOrEqual(-1);
      expect(y).toBeLessThanOrEqual(1);
    }
  });

  it("misma seed → mismo embedding (reproducible)", async () => {
    const { posted, importWorker } = startWorker(makeVectors(100, 8, 0.4, 0.2), 42);
    await importWorker();
    const vectors = makeVectors(100, 8, 0.4, 0.2);
    (globalThis as unknown as { onmessage: (e: MessageEvent) => void }).onmessage({
      data: { type: "project", vectors, seed: 42 },
    } as MessageEvent);
    const first = await waitForDone(posted);

    const second = await new Promise<number[][]>((resolve, reject) => {
      const posted2: unknown[] = [];
      vi.stubGlobal("postMessage", (msg: unknown) => posted2.push(msg));
      (globalThis as unknown as { onmessage: (e: MessageEvent) => void }).onmessage({
        data: { type: "project", vectors, seed: 42 },
      } as MessageEvent);
      const deadline = Date.now() + 10_000;
      const poll = () => {
        const done = posted2.find((m) => (m as { type?: string })?.type === "done");
        if (done) return resolve((done as { points: number[][] }).points);
        if (Date.now() > deadline) return reject(new Error("timeout segunda corrida"));
        setTimeout(poll, 50);
      };
      poll();
    });

    expect(second).toEqual(first);
  });

  it("sin vectores → error descriptivo", async () => {
    const { posted, importWorker } = startWorker([], 42);
    await importWorker();
    (globalThis as unknown as { onmessage: (e: MessageEvent) => void }).onmessage({
      data: { type: "project", vectors: [], seed: 42 },
    } as MessageEvent);

    const deadline = Date.now() + 5_000;
    while (Date.now() < deadline) {
      const err = posted.find((m) => (m as { type?: string })?.type === "error");
      if (err) {
        expect((err as { message: string }).message).toContain("No hay vectores");
        return;
      }
      await new Promise((r) => setTimeout(r, 50));
    }
    throw new Error("timeout: no llegó el error esperado");
  });
});