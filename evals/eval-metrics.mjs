#!/usr/bin/env node
// EVAL-01 — pipeline eval harness.
// Reads .opencode/task-system/enforcement/verify-log.jsonl (written by campaign_verify_cmd)
// plus plan files, and produces docs/reports/pipeline-evals.md against the RULES.md North Star:
//   - tasa completado >90% primer intento
//   - falsos positivos = 0  (verify FAILED pero task pasó a COMPLETED)
//   - regresión silenciosa = 0 (verify pasó y luego falló para la misma tarea)
import { readFileSync, readdirSync, existsSync, mkdirSync, writeFileSync } from "node:fs"
import { resolve, join, dirname } from "node:path"
import { fileURLToPath } from "node:url"
import { execSync } from "node:child_process"

const __dirname = dirname(fileURLToPath(import.meta.url))
const PROJECT_ROOT = resolve(__dirname, "..")
const TASK_SYSTEM = resolve(PROJECT_ROOT, ".opencode", "task-system")
const LOG = join(TASK_SYSTEM, "enforcement", "verify-log.jsonl")
const OUT = join(PROJECT_ROOT, "docs", "reports", "pipeline-evals.md")

function readLog() {
  if (!existsSync(LOG)) return []
  return readFileSync(LOG, "utf-8").split("\n").filter(Boolean).map(l => { try { return JSON.parse(l) } catch { return null } }).filter(Boolean)
}

function deriveType(taskId) {
  const t = taskId || ""
  if (/^WEB-/.test(t)) return "frontend"
  if (/^PY-|^PYTHON/.test(t)) return "python"
  if (/^TS-|^NODE/.test(t)) return "typescript"
  if (/^AUD|^SEC|^ERR|^PERF|^CHAOS|^DRV/.test(t)) return "rust"
  if (/^DOC/.test(t)) return "docs"
  return "other"
}

function planState(taskId, plansDir) {
  if (!existsSync(plansDir)) return null
  for (const f of readdirSync(plansDir).filter(f => f.endsWith(".md"))) {
    const body = readFileSync(join(plansDir, f), "utf-8")
    const re = new RegExp(`### Task [^\\n]+: ${taskId}[\\s\\S]*?- \\*\\*Estado:\\*\\*\\s*(✅ COMPLETED|❌ FAILED|⬜ PENDING|⏳ IN PROGRESS)`)
    const m = body.match(re)
    if (m) return { plan: f, state: m[1] }
  }
  return null
}

const entries = readLog()
const plansDir = join(PROJECT_ROOT, "docs", "plans")

// per-task aggregation
const byTask = new Map()
for (const e of entries) {
  if (!e.taskId) continue
  if (!byTask.has(e.taskId)) byTask.set(e.taskId, { id: e.taskId, fails: 0, passes: 0, firstOk: null, regressions: 0, plan: null })
  const t = byTask.get(e.taskId)
  if (e.passed) {
    if (t.passes === 0 && t.fails === 0) t.firstOk = true  // first attempt passed
    else if (t.fails > 0) t.regressions++                   // passed after fails within same task = reworked attempt
    t.passes++
    t.fails = 0
  } else {
    if (t.passes > 0) t.regressions++                       // passed before, now failing = silent regression
    t.fails++
  }
}

const tasks = [...byTask.values()]
for (const t of tasks) { const s = planState(t.id, plansDir); t.plan = s ? s.plan : null; t.finalState = s ? s.state : null }

// North Star metrics
const totalTasks = tasks.length || 1
const firstTryOk = tasks.filter(t => t.firstOk).length
const firstTryRate = (firstTryOk / totalTasks * 100).toFixed(1)
const falsePositives = tasks.filter(t => t.finalState === "✅ COMPLETED" && t.fails > 0).length
const silentRegressions = tasks.filter(t => t.regressions > 0).length
const typeStats = tasks.reduce((acc, t) => { const k = deriveType(t.id); acc[k] = (acc[k] || 0) + 1; return acc }, {})

let md = `# Pipeline Evaluation Report

> Generado por \`evals/eval-metrics.mjs\` (EVAL-01) — ${new Date().toISOString()}
> Datos: \`.opencode/task-system/enforcement/verify-log.jsonl\` (${entries.length} invocaciones de verify) + \`docs/plans/*.md\`

## North Star (RULES.md)

| Métrica | Threshold | Actual | Status |
|---|---|---|---|
| Tasa completado primer intento | >90% | ${firstTryRate}% | ${firstTryRate >= 90 ? "✅" : "🚩"} |
| Falsos positivos (COMPLETED con verify fallido) | 0 | ${falsePositives} | ${falsePositives === 0 ? "✅" : "🚩"} |
| Regresión silenciosa (verify falla tras pasar) | 0 | ${silentRegressions} | ${silentRegressions === 0 ? "✅" : "🚩"} |

## Por tipo de tarea

| Tipo | Tareas |
|---|---|
${Object.entries(typeStats).sort((a, b) => b[1] - a[1]).map(([k, v]) => `| ${k} | ${v} |`).join("\n")}

## Detalle por tarea

| Task | Plan | Verify ok | Verify fail | Primer intento | Regresiones | Estado final |
|---|---|---|---|---|---|---|
${tasks.sort((a, b) => a.id.localeCompare(b.id)).map(t =>
  `| ${t.id} | ${t.plan || "—"} | ${t.passes} | ${t.fails} | ${t.firstOk ? "✅" : "❌" } | ${t.regressions} | ${t.finalState || "—"} |`).join("\n") || "| _(sin datos de verify aún)_ | | | | | | |"}

## Notas
- Un "Primer intento" = la primera invocación de verify de la tarea pasó.
- "Regresiones" cuenta verifies que fallaron después de haber pasado para la misma tarea.
- El log se alimenta automáticamente desde \`campaign_verify_cmd\`; este reporte es la referencia del threshold de RULES.md.
`

mkdirSync(join(PROJECT_ROOT, "docs", "reports"), { recursive: true })
writeFileSync(OUT, md, "utf-8")
console.log(`Wrote ${OUT} (${tasks.length} tasks from ${entries.length} verify calls)`)