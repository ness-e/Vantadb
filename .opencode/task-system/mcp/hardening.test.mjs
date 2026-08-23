// hardening.test.mjs — H1/H2/H3 del plan 2026-08-23-task-system-hardening:
// detectType específico→genérico, recitation anclada, budget bajo lock, TTL WIP.
import { test } from "node:test"
import assert from "node:assert/strict"
import { mkdtempSync, mkdirSync, writeFileSync, utimesSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"
import {
  detectType,
  consumeBudget,
  findInProgressTasks,
  findPlanFile,
} from "./campaign-server.mjs"
import { updateRecitation, extractCampaignId, getOrCreateCampaignId, parseRecitation, extractAutonomous } from "./parsers.mjs"

function tmpWorktree() {
  const wt = mkdtempSync(join(tmpdir(), "vanta-hard-"))
  mkdirSync(join(wt, "docs", "plans"), { recursive: true })
  return wt
}

const PLAN = `# Plan de Ejecución: Test

> **Inicio:** 2026-08-23

### Task 1: T1 — demo
- **Estado:** ⬜ PENDING
`

test("H2: detectType — ruta python con /src/ no cae en rust ni multi", () => {
  assert.equal(detectType("vantadb-python/src/lib.rs").type, "python")
})

test("H2: detectType — web/src es frontend", () => {
  assert.equal(detectType("web/src/App.tsx").type, "frontend")
})

test("H2: detectType — src/engine.rs sigue siendo rust (fallback genérico)", () => {
  assert.equal(detectType("src/engine.rs, src/index/flat.rs:32").type, "rust")
})

test("H2: detectType — vantadb-server gana al fallback src/", () => {
  assert.equal(detectType("vantadb-server/src/main.rs").type, "server")
})

test("H1: updateRecitation — contract con 'Estado:' literal no corrompe campos", () => {
  const content = `=== RECITATION ===
Campaign ID: abc
Objetivo activo: T1
Estado: act
Última acción: editar
Resultado: OK
Próxima acción: verify
Contrato: revisar que Estado: verde y Contrato: cumple
Próxima tarea si completa: T2
=== END RECITATION ===
`
  const out = updateRecitation(content, {
    campaignId: "abc", activeGoal: "T1", status: "verify",
    lastAction: "editar más", result: "OK",
    nextAction: "cerrar", contract: "nuevo contrato con Estado: interno",
    nextTask: "T2",
  })
  assert.match(out, /^Estado: verify$/m)
  assert.match(out, /^Contrato: nuevo contrato con Estado: interno$/m)
  // El texto embebido dentro de Contrato NO fue reescrito por el campo Estado.
  assert.ok(!/^verde/m.test(out))
})

test("H1+: placeholder de Campaign ID no sombreada al ID real ni duplica líneas", () => {
  const withBoth = `> **Campaign ID:** (auto por MCP)\n> **Campaign ID:** 11111111-2222-3333-4444-555555555555\n> **Inicio:** hoy\n`
  assert.equal(extractCampaignId(withBoth), "11111111-2222-3333-4444-555555555555")
  const r = getOrCreateCampaignId(withBoth)
  // Dedup: queda UNA sola línea, con el ID válido.
  assert.equal((r.content.match(/Campaign ID/g) || []).length, 1)
  assert.equal(extractCampaignId(r.content), "11111111-2222-3333-4444-555555555555")
})

test("H1+: getOrCreateCampaignId reemplaza placeholder sin ID válido (no inserta otra línea)", () => {
  const onlyPlaceholder = `# Plan\n> **Campaign ID:** (auto por MCP)\n> **Inicio:** hoy\n`
  const r = getOrCreateCampaignId(onlyPlaceholder)
  assert.ok(r.campaignId)
  assert.equal((r.content.match(/Campaign ID/g) || []).length, 1)
  assert.equal(extractCampaignId(r.content), r.campaignId)
})

test("R1: recitation por-tarea — bloques de 2 tareas coexisten sin pisarse", () => {
  let content = "# Plan\n"
  content = updateRecitation(content, { taskId: "T8", campaignId: "c1", activeGoal: "T8", status: "verify", lastAction: "a8", result: "OK" })
  content = updateRecitation(content, { taskId: "T9", campaignId: "c1", activeGoal: "T9", status: "act", lastAction: "a9", result: "OK" })
  assert.match(content, /=== RECITATION T8 ===/)
  assert.match(content, /=== RECITATION T9 ===/)
  // Update de T9 NO toca el bloque de T8.
  content = updateRecitation(content, { taskId: "T9", campaignId: "c1", status: "verify", lastAction: "a9b" })
  const t9 = parseRecitation(content, "T9")
  const t8 = parseRecitation(content, "T8")
  assert.equal(t9.lastAction, "a9b")
  assert.equal(t8.lastAction, "a8")
  assert.equal(t8.status, "verify")
})

test("R1: bloque legacy sin ID sigue parseando y updateRecitation global no crea duplicados", () => {
  const legacy = "=== RECITATION ===\nEstado: act\n=== END RECITATION ==="
  const r = parseRecitation(legacy)
  assert.equal(r.status, "act")
  const out = updateRecitation(legacy + "\n", { status: "verify", lastAction: "x" })
  assert.equal((out.match(/=== RECITATION ===/g) || []).length, 1)
})

test("R7: extractAutonomous lee `> **Autonomous:** true`", () => {
  assert.equal(extractAutonomous("> **Autonomous:** true\n"), true)
  assert.equal(extractAutonomous("> **Autonomous:** false\n"), false)
  assert.equal(extractAutonomous("# plan sin flag\n"), null)
})

test("R5: >1 plan activo (<24h) → findPlanFile falla LOUD pidiendo ruta explícita", () => {
  const wt = tmpWorktree()
  writeFileSync(join(wt, "docs", "plans", "a.md"), PLAN)
  writeFileSync(join(wt, "docs", "plans", "b.md"), PLAN)
  assert.throws(() => findPlanFile(wt), /Ambiguous active plan/)
})

test("H3: consumeBudget persiste incrementos bajo lock", () => {
  const wt = tmpWorktree()
  writeFileSync(join(wt, "docs", "plans", "p.md"), PLAN)
  const r1 = consumeBudget("T1", wt)
  const r2 = consumeBudget("T1", wt)
  assert.equal(r1.toolCalls, 1)
  assert.equal(r2.toolCalls, 2)
  assert.ok(r2.withinBudget)
})

test("H3: findInProgressTasks — IN PROGRESS viejo (>24h) va a stale, no bloquea", () => {
  const wt = tmpWorktree()
  const tasksDir = join(wt, ".opencode", "skills", "campaign-executor", "tasks")
  mkdirSync(tasksDir, { recursive: true })
  const fp = join(tasksDir, "OLD-GHOST.md")
  writeFileSync(fp, "# OLD-GHOST\n- **Estado:** ⏳ IN PROGRESS\n")
  const old = new Date(Date.now() - 48 * 3600 * 1000)
  utimesSync(fp, old, old)

  const freshFp = join(tasksDir, "FRESH.md")
  writeFileSync(freshFp, "# FRESH\n- **Estado:** ⏳ IN PROGRESS\n")

  const active = findInProgressTasks(wt)
  assert.deepEqual(active.map(t => t.id), ["FRESH"])
  assert.deepEqual(active.stale.map(t => t.id), ["OLD-GHOST"])
})
