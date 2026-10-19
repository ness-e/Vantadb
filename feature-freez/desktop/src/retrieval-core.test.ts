// retrieval-core.test.ts (DESKTOP-35): el slider fija el modo de fusión
// server-side (MEM-01) — ya no hay re-rank client-side. Corre con vitest
// (`npm test`).
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  RRF_K,
  computeSegments,
  fusionModeFromSlider,
  rrfContribution,
} from "./components/lens/retrieval/retrieval-core.ts";

test("fusionModeFromSlider: stops discretos 0/50/100 → keyword/hybrid/vector", () => {
  assert.deepEqual(fusionModeFromSlider(0), { mode: "keyword" });
  assert.deepEqual(fusionModeFromSlider(50), { mode: "hybrid" });
  assert.deepEqual(fusionModeFromSlider(100), { mode: "vector" });
});

test("fusionModeFromSlider: fuera de rango se clampea al stop más cercano", () => {
  assert.equal(fusionModeFromSlider(-20).mode, "keyword");
  assert.equal(fusionModeFromSlider(24).mode, "keyword"); // redondeo al stop 0
  assert.equal(fusionModeFromSlider(74).mode, "hybrid"); // redondeo al stop 50
  assert.equal(fusionModeFromSlider(150).mode, "vector");
});

test("fusionModeFromSlider: NaN → hybrid (default del core)", () => {
  assert.equal(fusionModeFromSlider(NaN).mode, "hybrid");
});

test("computeSegments sigue intacto (desglose RRF del server sin pesos)", () => {
  const e = {
    score: rrfContribution(1) + rrfContribution(2),
    rrf_text_rank: 1,
    rrf_vector_rank: 2,
    bm25_terms: [],
  };
  const b = computeSegments(e, e.score);
  assert.equal(b.segments.length, 2);
  assert.ok(Math.abs(b.ramaSum - e.score) < 1e-9);
  // RRF_K sigue siendo la constante del core (60) para el desglose.
  assert.ok(Math.abs(rrfContribution(1) - 1 / (RRF_K + 1)) < 1e-12);
});

test("computeSegments: sin explanation → missing, sin crashear", () => {
  const b = computeSegments(null, 1);
  assert.equal(b.missing, true);
  assert.equal(b.segments.length, 0);
});
