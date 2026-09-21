// selfcheck-vs14.ts (VS-14): verificación mecánica de la lógica de diff del
// tab HISTORIAL. Corre con node 24 (type-stripping nativo, sin deps):
//   node scripts/selfcheck-vs14.ts    (desde desktop/)
//
// Fixture de 3 versiones en la MISMA forma que devuelve el bridge
// (`versions`/`getVersion` de vanta.ts — MemoryRecord plano):
//   v1: payload 3 líneas / meta {tema, estado:viejo} / vec [1,0,0]
//   v2: payload 4 líneas (1 editada) / meta {tema, estado:nuevo, extra} / vec [1,0,0] (igual)
//   v3: payload igual a v2 / meta sin `extra` / vec [1,1,0] (cambió)
import {
  diffMetadata,
  diffPayload,
  diffVersions,
  vecSummary,
  vectorChanged,
} from "../src/components/inspector/historial-diff.ts";
import type { MemoryRecord } from "../src/vanta.ts";

let failures = 0;
function assert(cond: boolean, msg: string): void {
  if (cond) {
    console.log(`  ok: ${msg}`);
  } else {
    failures += 1;
    console.error(`  FAIL: ${msg}`);
  }
}

function rec(partial: Partial<MemoryRecord> & { text: string; version: number }): MemoryRecord {
  return {
    id: "k1",
    namespace: "docs",
    metadata: {},
    vector: null,
    created_at_ms: 1000,
    updated_at_ms: 1000,
    node_id: null,
    sparse_vector: null,
    expires_at_ms: null,
    ...partial,
  };
}

const v1 = rec({
  version: 1,
  text: "linea uno\nlinea dos\nlinea tres",
  metadata: { tema: "a", estado: "viejo" },
  vector: [1, 0, 0],
  updated_at_ms: 1000,
});
const v2 = rec({
  version: 2,
  text: "linea uno\nlinea DOS\nlinea tres\nlinea cuatro",
  metadata: { tema: "a", estado: "nuevo", extra: 1 },
  vector: [1, 0, 0],
  updated_at_ms: 2000,
});
const v3 = rec({
  version: 3,
  text: "linea uno\nlinea DOS\nlinea tres\nlinea cuatro",
  metadata: { tema: "a", estado: "nuevo" },
  vector: [1, 1, 0],
  updated_at_ms: 3000,
});

console.log("self-check historial-diff (VS-14)");

// 1. Line-diff del payload (LCS).
{
  assert(diffPayload("", "").length === 0, "payload vacío → sin líneas");
  const same = diffPayload("a\nb", "a\nb");
  assert(same.length === 2 && same.every((l) => l.kind === "ctx"), "payloads idénticos → todo ctx");

  const d12 = diffVersions(v1, v2).payload;
  const del = d12.filter((l) => l.kind === "del").map((l) => l.text);
  const add = d12.filter((l) => l.kind === "add").map((l) => l.text);
  assert(del.includes("linea dos"), "v1→v2: linea dos marcada como quitada");
  assert(add.includes("linea DOS"), "v1→v2: linea DOS marcada como añadida");
  assert(add.includes("linea cuatro"), "v1→v2: linea cuatro marcada como añadida");
  assert(
    d12.filter((l) => l.kind === "ctx").map((l) => l.text).join(",") === "linea uno,linea tres",
    "v1→v2: contexto conserva linea uno y linea tres",
  );

  const empty = rec({ version: 4, text: "", updated_at_ms: 4000 });
  const dEmpty = diffVersions(empty, v3).payload;
  assert(
    dEmpty.filter((l) => l.kind === "add").length === 4 && dEmpty.filter((l) => l.kind === "del").length === 0,
    "vacío → v3: solo líneas añadidas",
  );
}

// 2. KV diff de metadata (añadido/quitado/cambiado).
{
  const m12 = diffVersions(v1, v2).metadata;
  assert(m12.added.length === 1 && m12.added[0] === "extra", "v1→v2: `extra` añadido");
  assert(m12.removed.length === 0, "v1→v2: nada quitado");
  assert(
    m12.changed.length === 1 && m12.changed[0].key === "estado",
    "v1→v2: `estado` cambiado (viejo → nuevo)",
  );

  const m23 = diffVersions(v2, v3).metadata;
  assert(m23.removed.length === 1 && m23.removed[0] === "extra", "v2→v3: `extra` quitado");
  assert(m23.added.length === 0 && m23.changed.length === 0, "v2→v3: sin añadidos ni cambios");

  const none = diffMetadata(null, undefined);
  assert(none.added.length === 0 && none.removed.length === 0 && none.changed.length === 0, "metadata ausente → sin diff");
}

// 3. Vector: norma/dim + "cambió" sí/no.
{
  const s1 = vecSummary([1, 0, 0]);
  assert(s1 !== null && s1.dim === 3 && Math.abs(s1.norm - 1) < 1e-9, "vec [1,0,0] → 3d, norma 1");
  const s3 = vecSummary([1, 1, 0]);
  assert(s3 !== null && Math.abs(s3.norm - Math.SQRT2) < 1e-9, "vec [1,1,0] → norma √2");
  assert(vecSummary(null) === null && vecSummary([]) === null, "vector ausente/vacío → null");

  assert(vectorChanged([1, 0, 0], [1, 0, 0]) === false, "vectores iguales → no cambió");
  assert(vectorChanged([1, 0, 0], [1, 1, 0]) === true, "valores distintos → cambió");
  assert(vectorChanged([1, 0, 0], [1, 0, 0, 0]) === true, "dimensión distinta → cambió");
  assert(vectorChanged([1, 0, 0], null) === true, "desaparece → cambió");
  assert(vectorChanged(null, null) === false, "ambos ausentes → no cambió");

  assert(diffVersions(v1, v2).vectorChanged === false, "v1→v2: vector igual → cambió: no");
  assert(diffVersions(v2, v3).vectorChanged === true, "v2→v3: vector distinto → cambió: sí");
  const d23 = diffVersions(v2, v3);
  assert(
    d23.vecA !== null &&
      d23.vecB !== null &&
      Math.abs(d23.vecA.norm - 1) < 1e-9 &&
      Math.abs(d23.vecB.norm - Math.SQRT2) < 1e-9,
    "v2→v3: normas de ambas versiones presentes y distintas (1 vs √2)",
  );
}

// 4. Idempotencia: misma versión contra sí misma → sin cambios netos.
{
  const same = diffVersions(v2, v2);
  assert(same.payload.every((l) => l.kind === "ctx"), "v2 vs v2 → payload todo ctx");
  assert(
    same.metadata.added.length === 0 && same.metadata.removed.length === 0 && same.metadata.changed.length === 0,
    "v2 vs v2 → metadata sin diff",
  );
  assert(same.vectorChanged === false, "v2 vs v2 → vector no cambió");
}

console.log(failures === 0 ? "PASS" : `FAILED (${failures})`);
process.exit(failures === 0 ? 0 : 1);