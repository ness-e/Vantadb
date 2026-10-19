// selfcheck-retrieval.ts (VS-13): verificación mecánica de la lógica de
// segmentos de la lente RETRIEVAL. Corre con node 24 (type-stripping nativo,
// sin deps): `node scripts/selfcheck-retrieval.ts`.
//
// Casos: (1) explanation ausente → segmentos 0, missing=true, NO crashea;
// (2) ranks presentes → contribuciones RRF exactas 1/(60+rank);
// (3) score híbrido 2-ramas → residuo rrf = 0; (4) score con excedente → residuo;
// (5) anchors: widthPct normalizado contra maxScore.
import { computeSegments, rrfContribution, RRF_K } from "../src/components/lens/retrieval/retrieval-core.ts";

let failures = 0;
function assert(cond: boolean, msg: string): void {
  if (cond) {
    console.log(`  ok: ${msg}`);
  } else {
    failures += 1;
    console.error(`  FAIL: ${msg}`);
  }
}

console.log("self-check retrieval-core");

// (1) Sin explanation → sin segmentos, missing, sin crashear.
{
  const b = computeSegments(null, 1);
  assert(b.missing === true, "explanation null → missing");
  assert(b.segments.length === 0, "explanation null → 0 segmentos");
  assert(b.score === 0 && b.ramaSum === 0, "explanation null → scores 0");
}

// (2) ranks 1-based → contribución exacta 1/(RRF_K + rank).
{
  assert(Math.abs(rrfContribution(1) - 1 / (RRF_K + 1)) < 1e-9, "rank 1 → 1/(60+1)");
  assert(Math.abs(rrfContribution(3) - 1 / (RRF_K + 3)) < 1e-9, "rank 3 → 1/(60+3)");
  assert(rrfContribution(null) === 0 && rrfContribution(0) === 0, "rank ausente/0 → 0");
}

// (3) Híbrido 2-ramas: text rank 1 + vector rank 2 → suma = score, sin residuo.
{
  const score = 1 / (RRF_K + 1) + 1 / (RRF_K + 2);
  const b = computeSegments(
    { score, rrf_text_rank: 1, rrf_vector_rank: 2, bm25_terms: [] },
    1,
  );
  assert(b.missing === false, "explicación presente → missing false");
  assert(b.segments.length === 2, "híbrido 2-ramas → 2 segmentos (text, vector)");
  const text = b.segments.find((s) => s.key === "text");
  const vector = b.segments.find((s) => s.key === "vector");
  assert(!!text && Math.abs(text.value - 1 / (RRF_K + 1)) < 1e-9, "segmento text = 1/(60+1)");
  assert(!!vector && Math.abs(vector.value - 1 / (RRF_K + 2)) < 1e-9, "segmento vector = 1/(60+2)");
  assert(Math.abs(b.ramaSum - score) < 1e-9, "ramaSum = score (sin residuo)");
  assert(Math.abs(text!.widthPct - (1 / (RRF_K + 1)) * 100) < 1e-6, "widthPct normalizado vs maxScore=1");
}

// (4) Excedente de score sobre ramas → segmento rrf residual.
{
  const b = computeSegments({ score: 0.5, rrf_text_rank: 1, rrf_vector_rank: 1, bm25_terms: [] }, 1);
  const rrf = b.segments.find((s) => s.key === "rrf");
  assert(!!rrf && Math.abs(rrf.value - (0.5 - 2 / (RRF_K + 1))) < 1e-9, "residuo rrf = score − ramas");
}

// (5) Solo vector (sin texto) → un solo segmento vector.
{
  const b = computeSegments({ score: 1 / (RRF_K + 5), rrf_text_rank: null, rrf_vector_rank: 5, bm25_terms: [] }, 1);
  assert(b.segments.length === 1 && b.segments[0].key === "vector", "vector-only → 1 segmento vector");
}

// (6) maxScore 0/negativo no crashea y no produce anchos inválidos.
{
  const b = computeSegments({ score: 0.01, rrf_text_rank: 1, rrf_vector_rank: null, bm25_terms: [] }, 0);
  assert(b.segments.every((s) => s.widthPct >= 0 && s.widthPct <= 100), "maxScore 0 → anchos acotados");
}

console.log(failures === 0 ? "PASS" : `FAILED (${failures})`);
process.exit(failures === 0 ? 0 : 1);