// retrieval-core.test.ts (FEAT-01): lógica pura del slider de pesos híbridos —
// weighted RRF client-side (el core fusiona con RRF fijo; ver retrieval-core.ts).
// Corre con node:test: `node --test src/retrieval-core.test.ts`.
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  RRF_K,
  computeSegmentsWeighted,
  rerankByWeight,
  weightFromSlider,
  weightedScore,
} from "./components/lens/retrieval/retrieval-core.ts";

const expl = (rt: number | null, rv: number | null) => ({
  score: 0.1,
  rrf_text_rank: rt,
  rrf_vector_rank: rv,
  bm25_terms: [],
});

test("weightFromSlider: 0→0, 50→0.5, 100→1, fuera de rango clamp, NaN→default", () => {
  assert.equal(weightFromSlider(0), 0);
  assert.equal(weightFromSlider(50), 0.5);
  assert.equal(weightFromSlider(100), 1);
  assert.equal(weightFromSlider(-5), 0);
  assert.equal(weightFromSlider(150), 1);
  assert.equal(weightFromSlider(NaN), 0.5);
});

test("weightedScore: α=0 → solo texto, α=1 → solo vector, α=0.5 → media RRF", () => {
  const e = expl(1, 3);
  assert.ok(Math.abs(weightedScore(e, 0) - 1 / (RRF_K + 1)) < 1e-12);
  assert.ok(Math.abs(weightedScore(e, 1) - 1 / (RRF_K + 3)) < 1e-12);
  assert.ok(
    Math.abs(weightedScore(e, 0.5) - 0.5 * (1 / (RRF_K + 1) + 1 / (RRF_K + 3))) < 1e-12,
  );
});

test("weightedScore: rama ausente (rank null) contribuye 0; sin explanation → 0", () => {
  assert.ok(Math.abs(weightedScore(expl(2, null), 0.3) - 0.7 * (1 / (RRF_K + 2))) < 1e-12);
  assert.ok(Math.abs(weightedScore(expl(null, 4), 0.3) - 0.3 * (1 / (RRF_K + 4))) < 1e-12);
  assert.equal(weightedScore(null, 0.5), 0);
});

test("rerankByWeight: α=0 ordena por rank de texto, α=1 por rank de vector", () => {
  const hits = [
    { id: "a", score: 0, explanation: expl(1, 5) },
    { id: "b", score: 0, explanation: expl(3, 1) },
    { id: "c", score: 0, explanation: expl(2, 2) },
  ];
  assert.deepEqual(
    rerankByWeight(hits, 0).map((h) => h.id),
    ["a", "c", "b"],
    "α=0 → mejor rank de texto primero",
  );
  assert.deepEqual(
    rerankByWeight(hits, 1).map((h) => h.id),
    ["b", "c", "a"],
    "α=1 → mejor rank de vector primero",
  );
});

test("rerankByWeight: α=0.5 reproduce el orden del RRF del core (score = mitad)", () => {
  // RRF core = 1/(K+rt) + 1/(K+rv); weighted α=0.5 es exactamente la mitad →
  // mismo orden, score a escala 1/2 (honesto: es la contribución ponderada).
  const hits = [
    { id: "a", score: 0, explanation: expl(1, 4) },
    { id: "b", score: 0, explanation: expl(2, 1) },
  ];
  const rrfA = 1 / (RRF_K + 1) + 1 / (RRF_K + 4);
  const rrfB = 1 / (RRF_K + 2) + 1 / (RRF_K + 1);
  const best = rrfA > rrfB ? "a" : "b";
  const out = rerankByWeight(hits, 0.5);
  assert.equal(out[0].id, best);
  assert.ok(Math.abs(out[0].score - 0.5 * Math.max(rrfA, rrfB)) < 1e-12);
});

test("rerankByWeight: no muta el input y preserva el conjunto de candidatos", () => {
  const hits = [
    { id: "a", score: 0.9, explanation: expl(1, 2) },
    { id: "b", score: 0.8, explanation: expl(2, 1) },
  ];
  const idsBefore = hits.map((h) => h.id).join(",");
  const out = rerankByWeight(hits, 0.2);
  assert.equal(hits.map((h) => h.id).join(","), idsBefore, "input intacto");
  assert.deepEqual(
    out.map((h) => h.id).sort(),
    ["a", "b"],
    "el conjunto no cambia — solo el orden (gap documentado: candidatos fijados por el core)",
  );
});

test("computeSegmentsWeighted: α=0 → solo texto, α=1 → solo vector, α=0.5 → ambos a media contribución", () => {
  const e = expl(1, 2);
  const t = 1 / (RRF_K + 1);
  const v = 1 / (RRF_K + 2);

  const s0 = computeSegmentsWeighted(e, t, 0);
  assert.equal(s0.segments.length, 1);
  assert.equal(s0.segments[0].key, "text");
  assert.ok(Math.abs(s0.segments[0].value - t) < 1e-9);
  assert.equal(s0.score, t);

  const s1 = computeSegmentsWeighted(e, v, 1);
  assert.equal(s1.segments.length, 1);
  assert.equal(s1.segments[0].key, "vector");
  assert.ok(Math.abs(s1.segments[0].value - v) < 1e-9);

  const s5 = computeSegmentsWeighted(e, t, 0.5);
  assert.equal(s5.segments.length, 2);
  const textSeg = s5.segments.find((x) => x.key === "text");
  const vecSeg = s5.segments.find((x) => x.key === "vector");
  assert.ok(textSeg && Math.abs(textSeg.value - t / 2) < 1e-9);
  assert.ok(vecSeg && Math.abs(vecSeg.value - v / 2) < 1e-9);
  assert.equal(s5.ramaSum, s5.score, "ramaSum = score ponderado (sin residuo rrf)");
});

test("computeSegmentsWeighted: sin explanation → missing, sin crashear", () => {
  const b = computeSegmentsWeighted(null, 1, 0.5);
  assert.equal(b.missing, true);
  assert.equal(b.segments.length, 0);
  assert.equal(b.score, 0);
});