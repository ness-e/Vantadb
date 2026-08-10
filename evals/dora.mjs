#!/usr/bin/env node
// P3-07 — DORA flow metrics harness.
// Sources:
//   - docs/plans/*.md  task blocks ("### Task N: ID — title", "- **Estado:** …") + plan
//     header ("**Inicio:**", "> **Estado:**") — dates are NOT structurally normalized yet,
//     so we derive best-effort: markers > budget epochs > file mtime, and document it.
//   - docs/plans/*.budget.json   startTime/lastActivity epoch ms per task
//   - .opencode/skills/campaign-executor/tasks/**/*.md  (flat + complete/ + closed/)
//     metadata: "**Estado:**", "**Fecha:**", "**Creado:**", "**Inicio:**", "**last-synced:**"
//   - .opencode/task-system/enforcement/verify-log.jsonl  (CFR; empty → 0 attempts)
// Emits docs/reports/dora.md. Never crashes on missing data — degrades to 0/empty with notes.
import { readFileSync, readdirSync, existsSync, statSync, mkdirSync, writeFileSync } from "node:fs"
import { resolve, join, dirname } from "node:path"
import { fileURLToPath } from "node:url"

const __dirname = dirname(fileURLToPath(import.meta.url))
const PROJECT_ROOT = resolve(__dirname, "..")
const PLANS_DIR = join(PROJECT_ROOT, "docs", "plans")
const TASKS_ROOT = join(PROJECT_ROOT, ".opencode", "skills", "campaign-executor", "tasks")
const LOG = join(PROJECT_ROOT, ".opencode", "task-system", "enforcement", "verify-log.jsonl")
const OUT = join(PROJECT_ROOT, "docs", "reports", "dora.md")
const DAY = 86400000

// ---------- helpers ----------
const datesOf = (s) => [...(s || "").matchAll(/\b(\d{4}-\d{2}-\d{2})\b/g)].map((m) => m[1])
const minDate = (arr) => (arr.length ? arr.reduce((a, b) => (a < b ? a : b)) : null)
const maxDate = (arr) => (arr.length ? arr.reduce((a, b) => (a > b ? a : b)) : null)
const dayKey = (d) => d.toISOString().slice(0, 10)
const dayFromMtime = (p) => {
  try { return dayKey(statSync(p).mtime) } catch { return null }
}
const diffDays = (a, b) => {
  if (!a || !b) return null
  const da = new Date(a), db = new Date(b)
  return isNaN(da) || isNaN(db) ? null : Math.round((db - da) / DAY)
}
const avg = (xs) => (xs.length ? xs.reduce((a, b) => a + b, 0) / xs.length : null)
const f1 = (n) => (n === null || n === undefined || !Number.isFinite(n) ? "—" : n.toFixed(1))
const deriveType = (taskId) => {
  const t = taskId || ""
  if (/^WEB-/.test(t)) return "frontend"
  if (/^PY-|^PYTHON/.test(t)) return "python"
  if (/^TS-|^NODE/.test(t)) return "typescript"
  if (/^AUD|^SEC|^ERR|^PERF|^CHAOS|^DRV/.test(t)) return "rust"
  if (/^DOC/.test(t)) return "docs"
  return "other"
}

// ---------- sources ----------
// Plan files: header (inicio, estado) + task blocks. mtimeFallback = true when a completed
// task got its completion date from the plan file's mtime instead of a written date.
function readPlans() {
  const plans = new Map()
  const planMeta = new Map()
  const numToId = new Map()
  if (!existsSync(PLANS_DIR)) return { plans, planMeta, numToId }
  for (const f of readdirSync(PLANS_DIR)) {
    if (!f.endsWith(".md")) continue
    const path = join(PLANS_DIR, f)
    let body = ""
    try { body = readFileSync(path, "utf-8") } catch { continue }
    const mtime = dayFromMtime(path)
    const inicio = (body.match(/\*\*Inicio:\*\*\s*(\d{4}-\d{2}-\d{2})/) || [])[1] || mtime
    const stLine = (body.match(/>\s*\*\*Estado:\*\*\s*([^\n]+)/) || [])[1] || ""
    const completedP = /COMPLETAD?O?|completed|✅/i.test(stLine)
    const stateP = completedP ? "completed" : /EN PROGRESO|in-progress|inprogress|⏳/i.test(stLine) ? "inprogress" : null
    const completedDateP = (stLine.match(/\((\d{4}-\d{2}-\d{2})\)/) || [])[1] || (completedP ? maxDate(datesOf(stLine)) : null) || (completedP ? mtime : null)
    planMeta.set(f, { inicio, state: stateP, completedDate: completedDateP })
    for (const block of body.split(/^### Task /m).slice(1)) {
      const m = block.split("\n")[0].match(/^(\d+):\s+([A-Za-z0-9-]+)/)
      if (!m) continue
      const num = m[1], id = m[2]
      numToId.set(num, id)
      const st = block.match(/-\s*\*\*Estado:\*\*\s*([^\n|]+)/)
      const emoji = st ? st[1].trim() : null
      const state = !emoji ? null
        : emoji.startsWith("✅") ? "completed"
        : emoji.startsWith("❌") ? "failed"
        : emoji.startsWith("⬜") ? "pending"
        : "inprogress"
      const bDates = datesOf(block)
      let completedDate = null, mtimeFallback = false
      if (state === "completed") {
        const written = maxDate(bDates)
        if (written) completedDate = written
        else if (mtime) { completedDate = mtime; mtimeFallback = true }
      }
      plans.set(id, { id, plan: f, state, inicio: inicio || mtime, completedDate, mtimeFallback })
    }
  }
  return { plans, planMeta, numToId }
}

// Task files (flat + complete/ + closed/): "**Estado:**", "**Fecha:**", "**Creado:**"/"**Inicio:**".
function readTaskFiles() {
  const tasks = new Map()
  const roots = [TASKS_ROOT, join(TASKS_ROOT, "complete"), join(TASKS_ROOT, "closed")]
  for (const root of roots) {
    if (!existsSync(root)) continue
    let files = []
    try { files = readdirSync(root).filter((f) => f.endsWith(".md")) } catch { continue }
    const bucket = root.includes("complete") ? "complete" : root.includes("closed") ? "closed" : "flat"
    for (const f of files) {
      const id = f.slice(0, -3)
      const path = join(root, f)
      let body = ""
      try { body = readFileSync(path, "utf-8") } catch { continue }
      const mtime = dayFromMtime(path)
      const dates = datesOf(body)
      const stLine = (body.match(/-\s*\*\*Estado:\*\*\s*([^\n]+)/) || [])[1] || ""
      const completed = /COMPLETED|COMPLETADO|✅/.test(stLine) && !/PENDING/.test(stLine)
      const state = completed ? "completed"
        : /FAILED|❌/.test(stLine) ? "failed"
        : /PENDING|⬜/.test(stLine) ? "pending"
        : /IN PROGRESS|⏳|inprogress/i.test(stLine) ? "inprogress"
        : null
      let completedDate = null, mtimeFallback = false
      if (completed) {
        const fa = (body.match(/\*\*Fecha:\*\*\s*(\d{4}-\d{2}-\d{2})/) || [])[1] || null
        const written = maxDate([...datesOf(stLine), fa, maxDate(dates)].filter(Boolean))
        if (written) { completedDate = written; mtimeFallback = false }
        else if (mtime) { completedDate = mtime; mtimeFallback = true }
      }
      const created = (body.match(/\*\*(?:Creado|Inicio):\*\*\s*(\d{4}-\d{2}-\d{2})/) || [])[1] || null
      const createdDate = created || minDate(dates) || mtime
      tasks.set(id, { id, state, createdDate, completedDate, mtimeFallback, src: `task:${bucket}` })
    }
  }
  return tasks
}

// Budget JSON: epoch-ms startTime/lastActivity → real day stamps.
function readBudgets(numToId) {
  const b = new Map()
  if (!existsSync(PLANS_DIR)) return b
  for (const f of readdirSync(PLANS_DIR).filter((f) => f.endsWith(".budget.json"))) {
    let j = null
    try { j = JSON.parse(readFileSync(join(PLANS_DIR, f), "utf-8")) } catch { continue }
    for (const [key, t] of Object.entries(j?.tasks || {})) {
      if (!t?.startTime && !t?.lastActivity) continue
      const id = /^\d+$/.test(key) ? numToId.get(key) || key : key
      const start = t.startTime ? dayKey(new Date(Number(t.startTime))) : null
      const last = t.lastActivity ? dayKey(new Date(Number(t.lastActivity))) : null
      b.set(id, { id, start, last, plan: f })
    }
  }
  return b
}

function readVerifyLog() {
  if (!existsSync(LOG)) return []
  try {
    return readFileSync(LOG, "utf-8").split("\n").filter(Boolean)
      .map((l) => { try { return JSON.parse(l) } catch { return null } })
      .filter(Boolean)
  } catch { return [] }
}

// ---------- aggregate ----------
const { plans, planMeta, numToId } = readPlans()
const taskFiles = readTaskFiles()
const budgets = readBudgets(numToId)
const entries = readVerifyLog()
const now = new Date()
const today = dayKey(now)

const allIds = new Set([...plans.keys(), ...taskFiles.keys(), ...budgets.keys()])
const tasks = [...allIds].map((id) => {
  const p = plans.get(id)
  const tf = taskFiles.get(id)
  const b = budgets.get(id)
  const state = p?.state || tf?.state || null
  const requestDate = p?.inicio || tf?.createdDate || b?.start || null
  const startOfWork = b?.start || tf?.createdDate || p?.inicio || null
  let completedDate = p?.completedDate || tf?.completedDate || null
  let mtimeFallback = (p?.mtimeFallback || tf?.mtimeFallback) ?? false
  if (!completedDate && state === "completed" && b?.last) { completedDate = b.last; mtimeFallback = false }
  return {
    id,
    type: deriveType(id),
    src: [p && "plan", tf && "task", b && "budget"].filter(Boolean).join("+"),
    plan: p?.plan || b?.plan || null,
    state,
    requestDate,
    startOfWork,
    completedDate,
    mtimeFallback,
  }
})

const completed = tasks.filter((t) => t.state === "completed")
const diffDaysSafe = (a, b) => diffDays(a, b)
const leadVals = (ts) => ts.map((t) => diffDaysSafe(t.requestDate, t.completedDate)).filter((d) => d !== null)
const cycleVals = (ts) => ts.map((t) => diffDaysSafe(t.startOfWork, t.completedDate)).filter((d) => d !== null)
const avgLead = (ts) => avg(leadVals(ts))
const avgCycle = (ts) => avg(cycleVals(ts))

// CFR
const attempts = entries.filter((e) => typeof e.passed === "boolean")
const failures = attempts.filter((e) => !e.passed)
const cfr = attempts.length ? (failures.length / attempts.length) * 100 : 0

// Throughput
const cutoff7 = dayKey(new Date(Date.now() - 7 * DAY))
const cutoff30 = dayKey(new Date(Date.now() - 30 * DAY))
const completedIn = (cutoff) => completed.filter((t) => t.completedDate && t.completedDate >= cutoff).length

// Flow buckets
const bucket = (ts) => ({
  total: ts.length,
  pending: ts.filter((t) => t.state === "pending").length,
  inprogress: ts.filter((t) => t.state === "inprogress").length,
  completed: ts.filter((t) => t.state === "completed").length,
  failed: ts.filter((t) => t.state === "failed").length,
  unknown: ts.filter((t) => !t.state).length,
})

const types = [...new Set(tasks.map((t) => t.type))].sort()
const plansF = [...new Set(tasks.map((t) => t.plan).filter(Boolean))].sort()

// ---------- markdown ----------
const code = (s) => "`" + s + "`"
let md = `# DORA Flow Metrics Report

> Generado por ${code("evals/dora.mjs")} (P3-07) — ${now.toISOString()}
> Fuentes: ${code("docs/plans/*.md")} (${plans.size} tareas en ${plansF.length} planes) + task files en ${code(".opencode/skills/campaign-executor/tasks/")} (${taskFiles.size}) + ${code("*.budget.json")} (${budgets.size} con timestamps) + ${code("verify-log.jsonl")} (${attempts.length} intentos de verify)
> ⚠️ **Fechas derivadas best-effort, NO normalizadas**. Prioridad: markers escritos (${code("**Inicio:**")}, ${code("**Estado:** COMPLETADO (fecha)")}, ${code("**Fecha:**")}, ${code("**Creado:**")}, fechas en bloque de tarea) -> budget epoch ms (${code("startTime")}/${code("lastActivity")}) -> **file mtime**. Donde se usó mtime se marca ${code("(mtime)")}. Esto es exactamente lo que P2-05 (traceId por tarea) va a resolver: con traceId real, cada task tendrá timestamps estructurados.
${attempts.length === 0 ? `> ⚠️ **verify-log.jsonl está vacío (${entries.length} líneas)** — CFR reportado en 0% como baseline; no hay intentos registrados todavía.\n` : ""}

## 1. Cycle / Lead time

| Métrica | Definición pragmática |
|---|---|
| **Lead time** | completado − request. Request = plan ${code("**Inicio:**")} del plan que la contiene, fallback ${code("**Creado:**")} del task file, fallback budget ${code("startTime")}, fallback mtime. |
| **Cycle time** | completado − startOfWork. Start = budget ${code("startTime")} (timestamps reales) → ${code("**Creado:**")}/${code("**Inicio:**")} del task file → plan inicio. Sin budget ni Creado, cycle == lead (mismo origen). |

### Por tipo de tarea

| Tipo | Tasks | Completed | Lead avg (días) | Cycle avg (días) |
|---|---|---|---|---|
${types.map((k) => {
  const ts = tasks.filter((t) => t.type === k)
  const comp = ts.filter((t) => t.state === "completed")
  return `| ${k} | ${ts.length} | ${comp.length} | ${f1(avgLead(comp))} | ${f1(avgCycle(comp))} |`
}).join("\n") || "| — | 0 | 0 | — | — |"}

### Por tarea (completadas con fechas derivables)

| Task | Tipo | Plan | Request | Start | Completed | Lead | Cycle |
|---|---|---|---|---|---|---|---|
${completed.filter((t) => t.requestDate && t.completedDate).sort((a, b) => a.id.localeCompare(b.id)).map((t) =>
  `| ${t.id} | ${t.type} | ${t.plan || t.src || "—"} | ${t.requestDate}${t.mtimeFallback ? " (mtime)" : ""} | ${t.startOfWork || "—"} | ${t.completedDate}${t.mtimeFallback ? " (mtime)" : ""} | ${f1(diffDaysSafe(t.requestDate, t.completedDate))} | ${f1(diffDaysSafe(t.startOfWork, t.completedDate))} |`).join("\n") || "| _(sin tareas completadas con fechas aún)_ | | | | | | | |"}

> Nota: ${completed.length - completed.filter((t) => t.requestDate && t.completedDate).length} tareas completadas sin fechas derivables quedan fuera de la tabla (estado ✅ sin fecha escrita ni mtime utilizable).

## 2. CFR (Change Failure Rate)

| Intentos de verify | Fallos | CFR |
|---|---|---|
| ${attempts.length} | ${failures.length} | **${cfr.toFixed(1)}%** |

${attempts.length === 0
  ? `> ⚠️ Sin intentos registrados en ${code("verify-log.jsonl")} — CFR 0% es **baseline sin datos**, no un resultado real. El log se alimenta desde ${code("campaign_verify_cmd")}.`
  : `> Detalle de fallos por tarea: ${failures.length ? failures.map((f) => `${f.taskId || "?"}(${f.exitCode ?? "?"})`).join(", ") : "ninguno."}`}

## 3. Throughput

| Periodo (días) | Tareas completadas |
|---|---|
| Últimos 7 | ${completedIn(cutoff7)} |
| Últimos 30 | ${completedIn(cutoff30)} |

${completed.length === 0 ? "> ⚠️ Sin tareas ✅ COMPLETED detectadas con fecha de completado — throughput 0." : ""}

## 4. Flow table

### Por tipo de tarea

| Tipo | Total | Pending | In-progress | Completed | Failed | Unknown |
|---|---|---|---|---|---|---|
${types.map((k) => {
  const b = bucket(tasks.filter((t) => t.type === k))
  return `| ${k} | ${b.total} | ${b.pending} | ${b.inprogress} | ${b.completed} | ${b.failed} | ${b.unknown} |`
}).join("\n") || "| — | 0 | 0 | 0 | 0 | 0 | 0 |"}

### Por plan file

| Plan | Total | Pending | In-progress | Completed | Failed | Lead avg (días) |
|---|---|---|---|---|---|---|
${plansF.map((p) => {
  const ts = tasks.filter((t) => t.plan === p)
  const b = bucket(ts)
  const comp = ts.filter((t) => t.state === "completed")
  const lead = f1(avgLead(comp))
  const init = planMeta.get(p)?.inicio || "—"
  return `| ${p} (inicio ${init}) | ${b.total} | ${b.pending} | ${b.inprogress} | ${b.completed} | ${b.failed} | ${lead} |`
}).join("\n") || "| _(sin plan files)_ | | | | | | |"}

### Sin plan file (task files sueltos)

| Bucket | Total | No completadas | Completed |
|---|---|---|---|
${["task:flat", "task:complete", "task:closed"].map((src) => {
  const ts = tasks.filter((t) => t.src === src)
  return `| ${src} | ${ts.length} | ${ts.filter((t) => t.state && t.state !== "completed").length} | ${ts.filter((t) => t.state === "completed").length} |`
}).join("\n") || "| — | 0 | 0 | 0 |"}

## 5. Limitaciones

- **Fechas no estructuradas**: los plan files no tienen un campo de timestamp por tarea normalizado; las fechas se extraen de markers ad-hoc (varían entre ${code("✅ COMPLETADO (2026-08-10)")}, ${code("**Estado: completed")}, ${code("**Fecha:**")}, ISO en task files, epoch ms en budget). Donde no hay marker se usó **file mtime** → esas filas son aproximaciones del día en que el archivo se tocó, no el día real del evento.
- **Cycle vs Lead**: sin budget ${code("startTime")} ni ${code("**Creado:**")}, startOfWork cae al plan ${code("**Inicio:**")} y cycle == lead. Los avgs por tipo mezclan ambas calidades.
- **CFR**: con ${code("verify-log.jsonl")} vacío (0 líneas) el 0% es baseline sin telemetría; no es evidencia de ausencia de fallos.
- **Throughput**: cuenta tareas ✅ COMPLETED con fecha de completado derivada; tareas completadas sin fecha quedan fuera.
- **Numeric budget keys**: claves numéricas de budget.json se resolvieron vía mapeo ${code("Task N → id")} del plan; si no hay match quedan como id numérico (tarea desconocida, cuenta como ${code("unknown")}).
- **P2-05 traceId**: con traceId por tarea, plan files deberán persistir ${code("createdAt")}/${code("completedAt")} estructurados; este reporte se recalculará sin fallback mtime.
- **Cobertura de task files**: los plan files P0/P1/P2/P3 no tienen todavía task files propios en ${code("tasks/")} (referencian ${code("skills/campaign-executor/tasks/P*-NN.md")} que aún no existen) — sus estados vienen del plan; los task files existentes (AUD/ERR/INV/COMP/DESKTOP/…) alimentan la vista de detalle.
`

mkdirSync(join(PROJECT_ROOT, "docs", "reports"), { recursive: true })
writeFileSync(OUT, md, "utf-8")
console.log(`Wrote ${OUT} (${allIds.size} tasks, ${completed.length} completed, lead avg ${f1(avgLead(completed))}d, CFR ${cfr.toFixed(1)}% over ${attempts.length} attempts)`)