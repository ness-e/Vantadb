// Self-check VS-07 (filtros compuestos): valida la lógica pura de
// `filters-core.ts` (inferMetaFields / toVantaMemoryFilter / evaluateQuery).
// Sin framework: `node scripts/vs07-filters-check.ts` (Node ≥ 23, type stripping).
import {
  EMPTY_QUERY,
  evaluateQuery,
  inferMetaFields,
  toVantaMemoryFilter,
} from "../src/components/search/filters-core.ts";

let failures = 0;
function check(name: string, got: boolean | number, want: boolean | number) {
  if (got !== want) {
    failures++;
    console.error(`FAIL ${name}: got ${got}, want ${want}`);
  } else {
    console.log(`ok   ${name}`);
  }
}

// --- inferMetaFields ---
const records = [
  { metadata: { kind: "note", priority: 3, ok: true, ts: "2026-08-18T10:00:00Z" } },
  { metadata: { kind: "task", priority: 2.5, ok: false, ts: "2026-08-19T11:30:00Z" } },
  { metadata: { note: "no meta" } },
];
const fields = inferMetaFields(records);
check("fields count", fields.length, 5);
check("kind string", fields.find((f) => f.name === "kind")?.type === "string", true);
check("priority int (first seen)", fields.find((f) => f.name === "priority")?.type === "int", true);
check("ok bool", fields.find((f) => f.name === "ok")?.type === "bool", true);
check("ts datetime", fields.find((f) => f.name === "ts")?.type === "datetime", true);

// --- toVantaMemoryFilter (AND flattened, ops 1:1) ---
const andQuery = {
  combinator: "and",
  rules: [
    { field: "kind", operator: "=", value: "task" },
    { field: "priority", operator: ">=", value: 3 },
  ],
};
const serialized = toVantaMemoryFilter(andQuery);
check("serialize count", serialized.length, 2);
check("op = → Eq", serialized[0].op === "Eq", true);
check("op >= → Gte", serialized[1].op === "Gte", true);
check(
  "serialize skips empty value",
  toVantaMemoryFilter({ combinator: "and", rules: [{ field: "kind", operator: "=", value: "" }] })
    .length,
  0,
);

// --- evaluateQuery: AND/OR, tipos, listas, campo ausente ---
const note = {
  metadata: {
    kind: "note",
    priority: 3,
    ok: true,
    ts: "2026-08-18T10:00:00Z",
    tags: ["a", "b"],
  },
};
check("AND both match", evaluateQuery(andQuery, note.metadata), false); // kind=note vs task
const kindNote = {
  ...andQuery,
  rules: [
    { field: "kind", operator: "=", value: "note" },
    { field: "priority", operator: ">=", value: 3 },
  ],
};
check("AND match", evaluateQuery(kindNote, note.metadata), true);
const orQuery = {
  combinator: "or",
  rules: [
    { field: "kind", operator: "=", value: "task" },
    { field: "priority", operator: ">=", value: 3 },
  ],
};
check("OR match (priority)", evaluateQuery(orQuery, note.metadata), true);
check(
  "Neq",
  evaluateQuery({ combinator: "and", rules: [{ field: "kind", operator: "!=", value: "task" }] }, note.metadata),
  true,
);
check(
  "Gt string lexicographic",
  evaluateQuery({ combinator: "and", rules: [{ field: "kind", operator: ">", value: "a" }] }, note.metadata),
  true,
);
check(
  "datetime Lt",
  evaluateQuery({ combinator: "and", rules: [{ field: "ts", operator: "<", value: "2026-12-31T00:00:00Z" }] }, note.metadata),
  true,
);
check(
  "missing field no match",
  evaluateQuery({ combinator: "and", rules: [{ field: "nope", operator: "=", value: 1 }] }, note.metadata),
  false,
);
check(
  "list any-match",
  evaluateQuery({ combinator: "and", rules: [{ field: "tags", operator: "=", value: "b" }] }, note.metadata),
  true,
);
check("empty group true", evaluateQuery(EMPTY_QUERY, note.metadata), true);
check(
  "nested OR-in-AND",
  evaluateQuery(
    {
      combinator: "and",
      rules: [
        { field: "ok", operator: "=", value: true },
        {
          combinator: "or",
          rules: [
            { field: "kind", operator: "=", value: "x" },
            { field: "priority", operator: ">", value: 2 },
          ],
        },
      ],
    },
    note.metadata,
  ),
  true,
);

if (failures > 0) {
  console.error(`\n${failures} FAILURES`);
  process.exit(1);
}
console.log("\nall checks passed");