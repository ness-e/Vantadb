// Self-check VS-15 (ACTIVITY + Timeline) — el desktop no tiene test runner, así
// que este script compila el módulo puro `activity/logic.ts` con tsc (React-free,
// type-only imports se erasan) y valida contra un fixture JSONL con la MISMA
// forma que escribe el core (`src/audit.rs`):
//   {"timestamp":"2026-08-18T14:03:00Z","op":"put","namespace":"docs","key":"k1","outcome":"ok","reason":null}
//
// Uso: node scripts/selfcheck-vs15.mjs   (desde desktop/)
import { execSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const desktop = resolve(fileURLToPath(new URL("..", import.meta.url)));
const logicTs = join(desktop, "src", "components", "activity", "logic.ts");
const tscBin =
  process.platform === "win32"
    ? join(desktop, "node_modules", ".bin", "tsc.cmd")
    : join(desktop, "node_modules", ".bin", "tsc");

const tmp = mkdtempSync(join(tmpdir(), "vs15-selfcheck-"));
try {
  // 1. Compilar el módulo puro a CommonJS (sin React, sin imports runtime).
  // execSync (no execFileSync): Node >=20 no spawnea .cmd directamente en win32.
  execSync(
    `"${tscBin}" "${logicTs}" --module commonjs --target es2020 --outDir "${tmp}" --skipLibCheck`,
    { cwd: desktop, stdio: "pipe" },
  );
  // tsc preserva la estructura relativa al source (outDir/components/activity/).
  const logic = await import(pathToFileURL(join(tmp, "components", "activity", "logic.js")).href);

  // 2. Fixture JSONL — 7 eventos (las 7 ops del core) en orden newest-first
  //    (como los devuelve vanta_audit_events), repartidos en 2 días y 4 horas
  //    + 1 línea malformada que el parse debe saltar (como VS-12/core).
  const rows = [
    { timestamp: "2026-08-18T16:00:00Z", op: "export_namespace", namespace: "docs", key: "N/A", outcome: "ok", reason: null },
    { timestamp: "2026-08-18T15:01:00Z", op: "delete_by_filter", namespace: "docs", key: "N/A", outcome: "err", reason: "invalid filter" },
    { timestamp: "2026-08-18T15:00:00Z", op: "delete", namespace: "docs", key: "k1", outcome: "ok", reason: "user purge" },
    { timestamp: "2026-08-18T14:05:30Z", op: "put_batch", namespace: "mem", key: "N/A", outcome: "ok", reason: null },
    { timestamp: "2026-08-18T14:03:00Z", op: "put", namespace: "docs", key: "k1", outcome: "ok", reason: null },
    { timestamp: "2026-08-17T09:30:00Z", op: "put", namespace: "mem", key: "k2", outcome: "err", reason: "index rebuild" },
    { timestamp: "2026-08-17T09:00:00Z", op: "import_file", namespace: "N/A", key: "N/A", outcome: "ok", reason: null },
    "not json",
  ];
  const jsonl = rows.map((r) => (typeof r === "string" ? r : JSON.stringify(r))).join("\n") + "\n";

  const events = jsonl
    .trim()
    .split("\n")
    .map((line) => logic.parseAuditLine(line))
    .filter((e) => e !== null);

  assert(events.length === 7, `parse: esperaba 7 eventos, hay ${events.length}`);
  assert(events[0].op === "export_namespace" && events[0].namespace === "docs", "parse: shape del primer evento (newest-first)");

  // 3. Encoding redundante por op (label + ícono + tone) — contrato (d).
  assert(logic.opMeta("put").label === "PUT" && logic.opMeta("put").icon === "✎" && logic.opMeta("put").tone === "neutral", "opMeta put neutral");
  assert(logic.opMeta("put_batch").tone === "batch", "opMeta put_batch batch");
  assert(logic.opMeta("delete").tone === "danger" && logic.opMeta("delete_by_filter").tone === "danger", "opMeta deletes danger");
  assert(logic.opMeta("export_namespace").tone === "transfer" && logic.opMeta("export_all").tone === "transfer" && logic.opMeta("import_file").tone === "transfer", "opMeta transfers");
  const unknown = logic.opMeta("expire");
  assert(unknown.tone === "unknown" && unknown.label === "EXPIRE", "opMeta fallback para ops futuras");

  // 4. Outcomes (ícono + texto, err flag).
  assert(logic.outcomeMeta("ok").icon === "✓" && logic.outcomeMeta("ok").err === false, "outcome ok");
  assert(logic.outcomeMeta("err").icon === "✕" && logic.outcomeMeta("err").err === true, "outcome err");

  // 5. Timeline por día — 2 buckets (17 y 18 ago), newest-first.
  const byDay = logic.groupByBucket(events, "day");
  assert(byDay.length === 2, `day: esperaba 2 buckets, hay ${byDay.length}`);
  assert(byDay[0].events.length === 5 && byDay[1].events.length === 2, "day: conteos por bucket");

  // 6. Timeline por hora — 4 buckets (16h, 15h, 14h del 18; 9h del 17),
  //    con conteos [1, 2, 2, 2] y orden newest-first dentro de cada bucket.
  const byHour = logic.groupByBucket(events, "hour");
  assert(byHour.length === 4, `hour: esperaba 4 buckets, hay ${byHour.length}`);
  assert(
    byHour[0].events.length === 1 && byHour[1].events.length === 2 && byHour[2].events.length === 2 && byHour[3].events.length === 2,
    `hour: conteos por bucket [1,2,2,2], hay [${byHour.map((b) => b.events.length).join(",")}]`,
  );
  assert(byHour[1].events[0].op === "delete_by_filter", "hour: dentro del bucket 15h el más nuevo va primero");

  console.log("✅ self-check VS-15: fixture JSONL (7 eventos, 7 ops, 2 días) — parse/encoding/grouping OK");
} finally {
  rmSync(tmp, { recursive: true, force: true });
}

function assert(cond, msg) {
  if (!cond) throw new Error("❌ self-check VS-15: " + msg);
}