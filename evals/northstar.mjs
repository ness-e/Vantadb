#!/usr/bin/env node
// P1-06 — North Star metrics harness.
// Reads .opencode/task-system/enforcement/verify-log.jsonl (written by campaign_verify_cmd,
// line format: {ts, taskId, command, passed, exitCode, expectedExitCode, elapsed, summary, plan})
// plus docs/plans/*.md task blocks and docs/plans/*.budget.json task maps, and produces
// docs/reports/northstar.md against the RULES.md North Star:
//   - tasa completado primer intento >90%
//   - falsos positivos = 0
//   - regresión silenciosa = 0
// Degrada a 0 + nota cuando los datos de telemetría faltan — nunca crashea.
import { readFileSync, readdirSync, existsSync, mkdirSync, writeFileSync } from "node:fs"
import { resolve, join, dirname } from "node:path"
import { fileURLToPath } from "node:url"

const __dirname = dirname(fileURLToPath(import.meta.url))
const PROJECT_ROOT = resolve(__dirname, "..")
const TASK_SYSTEM = resolve(PROJECT_ROOT, ".opencode", "task-system")
const LOG = join(TASK_SYSTEM, "enforcement", "verify-log.jsonl")
const PLANS_DIR = join(PROJECT_ROOT, "docs", "plans")
const OUT = join(PROJECT_ROOT, "docs", "reports", "northstar.md")

function readLog() {
  if (!existsSync(LOG)) return []
  try {
    return readFileSync(LOG, "utf-8").split("\n").filter(Boolean).map(l => { try { return JSON.parse(l) } catch { return null } }).filter(Boolean)
  } catch { return [] }
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

// Parse docs/plans/*.md task blocks ("### Task N: ID — title ... - **Estado:** …")
// and docs/plans/*.budget.json task maps ({tasks: {id: {consecutiveFails, ...}}}).
function parsePlans(dir) {
  const plans = new Map()   // id → { id, plan, state }
  const budgets = new Map() // id → { id, plan, consecutiveFails }
  const numToId = new Map() // plan-file "Task N" number → task ID (budget keys may be numeric)
  if (!existsSync(dir)) return { plans, budgets }
  const files = readdirSync(dir)
  // Pass 1: parse *.md task blocks first so numeric budget keys can resolve to task IDs.
  for (const f of files.filter(f => f.endsWith(".md"))) {
    let body = ""
    try { body = readFileSync(join(dir, f), "utf-8") } catch { continue }
    for (const block of body.split(/^### Task /m).slice(1)) {
      const head = block.split("\n")[0]
      const m = head.match(/^(\d+):\s+([A-Za-z0-9-]+)/)
      if (!m) continue
      const num = m[1], id = m[2]
      const st = block.match(/-\s*\*\*Estado:\*\*\s*([^\n|]+)/)
      const emoji = st ? st[1].trim() : null
      const state = !emoji ? null
        : emoji.startsWith("✅") ? "completed"
        : emoji.startsWith("❌") ? "failed"
        : emoji.startsWith("⬜") ? "pending"
        : "inprogress"
      numToId.set(num, id)
      plans.set(id, { id, plan: f, state })
    }
  }
  // Pass 2: parse *.budget.json (now numToId is fully populated).
  for (const f of files.filter(f => f.endsWith(".budget.json"))) {
    let j = null
    try { j = JSON.parse(readFileSync(join(dir, f), "utf-8")) } catch { j = null }
    for (const [key, t] of Object.entries(j?.tasks || {})) {
      const id = /^\d+$/.test(key) ? (numToId.get(key) || key) : key
      budgets.set(id, { id, plan: f, consecutiveFails: t?.consecutiveFails ?? 0 })
    }
  }
  return { plans, budgets }
}

// Per-task aggregation over verify-log.
function aggregateLog(entries) {
  const byTask = new Map()
  for (const e of entries) {
    if (!e.taskId) continue
    if (!byTask.has(e.taskId)) byTask.set(e.taskId, { id: e.taskId, invocations: 0, passes: 0, fails: 0, firstPassed: null, passedThenFailed: false })
    const t = byTask.get(e.taskId)
    t.invocations++
    if (e.passed) {
      if (t.firstPassed === null) t.firstPassed = true
      t.passes++
    } else {
      if (t.firstPassed === null) t.firstPassed = false
      if (t.passes > 0) t.passedThenFailed = true
      t.fails++
    }
  }
  return byTask
}

const entries = readLog()
const { plans, budgets } = parsePlans(PLANS_DIR)
const logByTask = aggregateLog(entries)

// Union task view across all three sources.
const allIds = new Set([...plans.keys(), ...budgets.keys(), ...logByTask.keys()])
const tasks = [...allIds].map(id => {
  const l = logByTask.get(id)
  return {
    id,
    plan: (plans.get(id) || budgets.get(id))?.plan || null,
    state: plans.get(id)?.state || null,
    budgetFails: budgets.get(id)?.consecutiveFails || 0,
    invocations: l?.invocations || 0,
    passes: l?.passes || 0,
    fails: l?.fails || 0,
    firstPassed: l?.firstPassed ?? null,
    passedThenFailed: l?.passedThenFailed || false,
  }
})

// ---- Metric 1: tasa completado primer intento ----
const completed = tasks.filter(t => t.state === "completed")
const totalCompleted = completed.length
// Primer intento = COMPLETED sin evidencia de fallo antes de completarse:
// verify fallido (passed:false), budgetFails>0, o primer verify fallido.
const firstTryOk = completed.filter(t => t.fails === 0 && t.budgetFails === 0 && t.firstPassed !== false).length
const firstTryRate = totalCompleted === 0 ? 0 : (firstTryOk / totalCompleted * 100)

// ---- Metric 2: falsos positivos (union de reglas, sin dedupe en componentes) ----
const fpCompletedWithVerifyFail = completed.filter(t => t.fails > 0).length   // COMPLETED pero verify falló
const fpRerun = tasks.filter(t => t.passes > 0 && t.invocations > 1).length   // verified-then-rerun
const fpBudget = tasks.filter(t => t.budgetFails > 0).length                  // fails registrados en budget
const fpIds = new Set()
completed.filter(t => t.fails > 0).forEach(t => fpIds.add(t.id))
tasks.filter(t => t.passes > 0 && t.invocations > 1).forEach(t => fpIds.add(t.id))
tasks.filter(t => t.budgetFails > 0).forEach(t => fpIds.add(t.id))
const falsePositives = fpIds.size

// ---- Metric 3: regresión (pasó verify y luego falló para la misma tarea) ----
const regressions = tasks.filter(t => t.passedThenFailed).length

// ---- Thresholds (RULES.md) ----
const verifyData = entries.length > 0
const budgetData = budgets.size > 0
const telemetry = verifyData || budgetData
const status = (ok, evaluable) => !evaluable ? "⚠️" : (ok ? "✅" : "🚩")

const typeStats = tasks.reduce((acc, t) => { const k = deriveType(t.id); acc[k] = (acc[k] || 0) + 1; return acc }, {})

let md = `# North Star Report

> Generado por \`evals/northstar.mjs\` (P1-06) — ${new Date().toISOString()}
> Datos: \`.opencode/task-system/enforcement/verify-log.jsonl\` (${entries.length} invocaciones de verify) + \`docs/plans/*.md\` (${plans.size} tareas) + \`docs/plans/*.budget.json\` (${budgets.size} tareas trackeadas)
${!verifyData ? "> ⚠️ **verify-log.jsonl está vacío** — sin telemetría de verificación, las métricas se reportan en 0 y los thresholds no pueden evaluarse aún.\n" : ""}

## Definiciones (documentadas en este header)

| Métrica | Definición pragmática |
|---|---|
| **Primer intento** | Tarea con estado ✅ COMPLETED sin evidencia de fallo antes de completarse: ninguna invocación verify fallida (\`passed:false\`), ni \`consecutiveFails>0\` en budget, ni primer verify fallido. Sin datos de verify ni budget se asume primer intento (best-effort). |
| **Falso positivo** | Registro donde una verificación no fue confiable: (a) **verified-then-rerun** = la tarea se verificó (\`passed:true\`) y luego se volvió a invocar verify (>1 invocación); (b) **verified-then-failed** = pasó una verify y falló en otra posterior, o quedó ✅ COMPLETED con verify fallido; (c) **budget fail** = \`consecutiveFails>0\` en budget para una tarea. Headline = unión de tareas en (a)+(b)+(c). |
| **Regresión** | Tarea que pasó verify al menos una vez y luego falló en una verify posterior (mismo \`taskId\`: patrón passed→failed). Equivale a la "regresión silenciosa" de RULES.md (romper tests que antes pasaban). |

## 1. Tasa completado primer intento

| Métrica | Valor |
|---|---|
| Tareas ✅ COMPLETED | ${totalCompleted} |
| Completadas en primer intento (sin fallos registrados) | ${firstTryOk} |
| **Tasa primer intento** | **${firstTryRate.toFixed(1)}%** |

## 2. Falsos positivos

| Componente | Count |
|---|---|
| COMPLETED con verify fallido | ${fpCompletedWithVerifyFail} |
| Verified-then-rerun (>1 invocación verify) | ${fpRerun} |
| Budget fails (consecutiveFails > 0) | ${fpBudget} |
| **Falsos positivos (unión)** | **${falsePositives}** |

## 3. Regresión

| Métrica | Valor |
|---|---|
| Tareas con patrón passed→failed (verify) | ${regressions} |

## 4. Comparación contra North Star (RULES.md)

| Métrica | Threshold | Actual | Status |
|---|---|---|---|
| Tasa completado primer intento | >90% | ${firstTryRate.toFixed(1)}% | ${status(firstTryRate >= 90, verifyData)} |
| Falsos positivos | 0 | ${falsePositives} | ${status(falsePositives === 0, telemetry)} |
| Regresión silenciosa | 0 | ${regressions} | ${status(regressions === 0, verifyData)} |

${!verifyData ? "> ⚠️ Sin telemetría de verify — los thresholds de primer intento y regresión **no pueden evaluarse aún** (baseline pendiente); con budget solo, falsos positivos es parcialmente evaluable.\n" : ""}

## Por tipo de tarea

| Tipo | Tareas |
|---|---|
${Object.entries(typeStats).sort((a, b) => b[1] - a[1]).map(([k, v]) => `| ${k} | ${v} |`).join("\n") || "| — | 0 |"}

## Detalle por tarea

| Task | Plan | Estado | Verify ok | Verify fail | Primer intento | Presencia regresión | Budget fails |
|---|---|---|---|---|---|---|---|
${tasks.sort((a, b) => a.id.localeCompare(b.id)).map(t =>
  `| ${t.id} | ${t.plan || "—"} | ${t.state || "sin-plan"} | ${t.passes} | ${t.fails} | ${(t.fails === 0 && t.budgetFails === 0 && t.firstPassed !== false) ? "✅" : "❌"} | ${t.passedThenFailed ? "✅" : "—"} | ${t.budgetFails} |`).join("\n") || "| _(sin datos de tareas aún)_ | | | | | | | |"}

## Notas
- "Primer intento" se infiere de plan + budget + verify-log; sin telemetría de verify la tasa es best-effort (asume primer intento cuando no hay evidencia de fallo).
- Falsos positivos y regresión se solapan por diseño: una tarea COMPLETED con patrón passed→failed cuenta en ambas — el headline de FP es unión de tareas, la regresión es el patrón de verify.
- El log se alimenta automáticamente desde \`campaign_verify_cmd\` (campaign-server.mjs); los budget.json se alimentan desde \`consumeBudget\`. Este reporte es la referencia del threshold de RULES.md.
- Fuente de planes: \`docs/plans/*.md\` raíz (el subdirectorio \`archive/\` no se incluye).
`

mkdirSync(join(PROJECT_ROOT, "docs", "reports"), { recursive: true })
writeFileSync(OUT, md, "utf-8")
console.log(`Wrote ${OUT} (${totalCompleted} completed / ${firstTryOk} first-try, ${falsePositives} FP, ${regressions} regressions, ${entries.length} verify calls)`)