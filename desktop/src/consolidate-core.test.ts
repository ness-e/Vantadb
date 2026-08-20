// FEAT-03a CONSOLIDAR pure logic tests (node --test, no React).
// Detección de pares candidatos por similitud (search kNN) + diff + superseded_by.
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  buildCandidatePairs,
  countSuperseded,
  fmtSim,
  mergeSuperseded,
  pairKey,
  SUPERSEDED_BY_KEY,
  supersededBy,
} from "./components/consolidate/consolidate-core.ts";

const rec = (id: string, text = `texto ${id}`) => ({ id, namespace: "mem", text });
const hit = (id: string, score: number) => ({
  id,
  namespace: "mem",
  text: `texto ${id}`,
  score,
});

test("pairKey: orden canónico independiente del orden de argumentos", () => {
  assert.equal(pairKey("a", "b"), pairKey("b", "a"));
  assert.equal(pairKey("b", "a"), "a\u0000b");
  assert.equal(pairKey("x", "x"), "x\u0000x");
});

test("buildCandidatePairs: dedupe dirección inversa, se queda con el mejor score", () => {
  const a = rec("a");
  const b = rec("b");
  // a consulta y encuentra b (score 0.02); b consulta y encuentra a (score 0.05).
  const hits = new Map<string, ReturnType<typeof hit>[]>([
    ["a", [hit("b", 0.02)]],
    ["b", [hit("a", 0.05)]],
  ]);
  const pairs = buildCandidatePairs([a, b], hits, { minScore: 0.01 });
  assert.equal(pairs.length, 1);
  assert.equal(pairs[0].score, 0.05);
});

test("buildCandidatePairs: excluye self-hits, filtro por minScore, orden desc", () => {
  const a = rec("a");
  const b = rec("b");
  const c = rec("c");
  const hits = new Map([
    ["a", [hit("a", 0.9), hit("c", 0.04), hit("b", 0.02)]],
  ]);
  const pairs = buildCandidatePairs([a, b, c], hits, { minScore: 0.03 });
  assert.deepEqual(pairs.map((p) => p.b.id), ["c"]); // self a→a excluido, b bajo umbral
  assert.deepEqual(pairs.map((p) => p.score), [0.04]);
});

test("buildCandidatePairs: ignora hits de registros no cargados (otro namespace)", () => {
  const a = rec("a");
  const hits = new Map([
    ["a", [hit("ghost", 0.9)]],
  ]);
  assert.deepEqual(buildCandidatePairs([a], hits), []);
});

test("buildCandidatePairs: respeta maxPairs", () => {
  const records = [rec("a"), rec("b"), rec("c"), rec("d")];
  const hits = new Map([
    ["a", [hit("b", 0.1), hit("c", 0.09), hit("d", 0.08)]],
  ]);
  const pairs = buildCandidatePairs(records, hits, { minScore: 0.01, maxPairs: 2 });
  assert.equal(pairs.length, 2);
  assert.equal(pairs[0].b.id, "b");
  assert.equal(pairs[1].b.id, "c");
});

test("mergeSuperseded: preserva metadata existente y setea superseded_by", () => {
  const merged = mergeSuperseded({ source: "notion", tags: ["a"] }, "rec-9");
  assert.equal(merged.source, "notion");
  assert.deepEqual(merged.tags, ["a"]);
  assert.equal(merged[SUPERSEDED_BY_KEY], "rec-9");
  // metadata ausente → solo la clave nueva.
  assert.deepEqual(mergeSuperseded(undefined, "rec-9"), { superseded_by: "rec-9" });
});

test("supersededBy: null para ausente/no-string/vacío; id para marcado", () => {
  assert.equal(supersededBy(undefined), null);
  assert.equal(supersededBy({}), null);
  assert.equal(supersededBy({ superseded_by: 42 }), null);
  assert.equal(supersededBy({ superseded_by: "" }), null);
  assert.equal(supersededBy({ superseded_by: "rec-9" }), "rec-9");
});

test("fmtSim: pct relativo al max score del run, clamp 0..100, max 0 safe", () => {
  assert.equal(fmtSim(0.08, 0.16).pct, 50);
  assert.equal(fmtSim(0.16, 0.16).pct, 100);
  assert.equal(fmtSim(0.02, 0.02).label, "100%");
  assert.equal(fmtSim(0.01, 0).pct, 0);
});

test("countSuperseded: cuenta solo records con superseded_by válido", () => {
  const records = [
    rec("a"),
    { ...rec("b"), metadata: { superseded_by: "a" } },
    { ...rec("c"), metadata: { superseded_by: "" } },
  ];
  assert.equal(countSuperseded(records), 1);
});