// parsers.test.mjs — unit tests de parsers.mjs (TSYS-06-F1).
// Cobertura: fixtures de corrupción C1/C2/C11 del diseño task-system-chaos-resilience.md §4.
// Node v24 built-in test runner — cero dependencias nuevas.
import { test } from "node:test"
import assert from "node:assert/strict"
import {
  extractField, extractState, parseTasks, parseRecitation, findTaskById,
  updateState, updateRecitation, extractCampaignId, getOrCreateCampaignId,
} from "./parsers.mjs"

const PLAN = `# Plan: TSYS-06 Test Fixture

> **Inicio:** 2026-08-17
> **Campaign ID:** 0a1b2c3d-4e5f-6789-abcd-ef0123456789

## Tasks

### Task 1: First task
- **Prioridad:** 🟢
- **Esfuerzo:** 🟢
- **Estado:** ⬜ PENDING
- **Archivos clave:** src/foo.rs
- **Contrato:** verify
- **Fuente:** test
- **Notas:** none

### Task 2: Second task
- **Prioridad:** 🟡
- **Esfuerzo:** 🟡
- **Estado:** ✅ COMPLETED
- **Archivos clave:** docs/bar.md
- **Contrato:** none
- **Fuente:** test
`

test("parseTasks: plan válido → tasks con campos correctos", () => {
  const tasks = parseTasks(PLAN)
  assert.strictEqual(tasks.length, 2)
  assert.strictEqual(tasks[0].id, "1")
  assert.strictEqual(tasks[0].name, "First task")
  assert.strictEqual(tasks[0].state, "⬜ PENDING")
  assert.strictEqual(tasks[0].files, "src/foo.rs")
  assert.strictEqual(tasks[0].contract, "verify")
  assert.strictEqual(tasks[1].state, "✅ COMPLETED")
})

test("extractState: emoji variants + estados rotos (C1)", () => {
  assert.strictEqual(extractState("- **Estado:** ✅ COMPLETED"), "✅ COMPLETED")
  assert.strictEqual(extractState("- **Estado:** ❌ FAILED"), "❌ FAILED")
  assert.strictEqual(extractState("- **Estado:** ⏳ IN PROGRESS"), "⏳ IN PROGRESS")
  // Campo ausente → PENDING (default).
  assert.strictEqual(extractState("- **Prioridad:** 🟢"), "⬜ PENDING")
  // Estado roto sin emoji → PENDING (degradación documentada, no crash).
  assert.strictEqual(extractState("- **Estado:** EN PROGRESO"), "⬜ PENDING")
})

test("C1: header truncado se salta sin crash (bloque no parseable)", () => {
  const corrupted = PLAN + `### Task 5
- **Estado:** ⬜ PENDING
- **Prioridad:** 🔴
`
  const tasks = parseTasks(corrupted)
  assert.strictEqual(tasks.length, 2) // Task 5 sin título no entra al DAG
})

test("C1: '### Task' suelto en una nota no crea tarea fantasma", () => {
  const withStray = PLAN.replace("## Tasks", "## Tasks\n\n### Task\n(nota suelta, no es un header válido)\n")
  const tasks = parseTasks(withStray)
  assert.strictEqual(tasks.length, 2)
})

test("C1: header roto (sin ':') no crashea y el bloque se degrada a PENDING", () => {
  const broken = PLAN.replace("### Task 2: Second task", "### Task 2 Second task")
  const tasks = parseTasks(broken)
  assert.strictEqual(tasks.length, 1) // el header roto no matchea /### Task (\d+):/
  assert.strictEqual(tasks[0].id, "1")
})

test("C2: bloque eliminado → findTaskById no encuentra fantasma", () => {
  const lines = PLAN.split("\n")
  const idx = lines.findIndex(l => l.startsWith("### Task 2"))
  const withoutTask2 = lines.slice(0, idx).join("\n")
  const info = findTaskById(withoutTask2, "2")
  assert.strictEqual(info, null)
  assert.strictEqual(parseTasks(withoutTask2).some(t => t.id === "2"), false)
})

test("findTaskById: encuentra bloque correcto y respeta boundaries", () => {
  const info = findTaskById(PLAN, "1")
  assert.ok(info)
  assert.ok(info.header.startsWith("### Task 1: First task"))
  assert.ok(!info.header.includes("### Task 2"))
  assert.strictEqual(findTaskById(PLAN, "999"), null)
})

test("updateState: cambia estado, preserva el resto; no-found/unknown son no-ops", () => {
  const updated = updateState(PLAN, "1", "in-progress")
  assert.ok(updated.includes("- **Estado:** ⏳ EN PROGRESO"))
  assert.ok(updated.includes("### Task 2: Second task")) // el resto intacto
  assert.strictEqual(updateState(PLAN, "999", "completed"), PLAN) // no encontrada
  assert.strictEqual(updateState(PLAN, "1", "bogus-state"), PLAN) // estado desconocido
})

test("C11: contenido vacío → parseTasks devuelve [] sin crash", () => {
  assert.deepStrictEqual(parseTasks(""), [])
  assert.deepStrictEqual(parseTasks("solo texto sin headers"), [])
})

test("getOrCreateCampaignId: idempotente con ID existente (C5)", () => {
  const { campaignId, content } = getOrCreateCampaignId(PLAN)
  assert.strictEqual(campaignId, "0a1b2c3d-4e5f-6789-abcd-ef0123456789")
  assert.strictEqual(content, PLAN) // sin cambios
})

test("getOrCreateCampaignId: crea UUID si falta, rechaza placeholders (C5)", () => {
  const noId = PLAN.replace(/> \*\*Campaign ID:\*\* .+\n/, "")
  const { campaignId, content } = getOrCreateCampaignId(noId)
  assert.match(campaignId, /^[0-9a-f-]{36}$/)
  assert.ok(content.includes(`> **Campaign ID:** ${campaignId}`))

  for (const placeholder of ["> **Campaign ID:** (pendiente)", "> **Campaign ID:** TODO", "> **Campaign ID:** <luego>"]) {
    const withPlaceholder = PLAN.replace(/> \*\*Campaign ID:\*\* .+/, placeholder)
    const fresh = getOrCreateCampaignId(withPlaceholder)
    assert.notStrictEqual(fresh.campaignId, placeholder.replace("> **Campaign ID:** ", ""))
    assert.match(fresh.campaignId, /^[0-9a-f-]{36}$/)
  }
})

test("extractCampaignId: null cuando no existe o es placeholder", () => {
  assert.strictEqual(extractCampaignId(PLAN), "0a1b2c3d-4e5f-6789-abcd-ef0123456789")
  assert.strictEqual(extractCampaignId("sin id"), null)
  assert.strictEqual(extractCampaignId("> **Campaign ID:** TODO"), null)
})

const RECITATION = `=== RECITATION ===
Campaign ID: abc
Objetivo activo: goal
Estado: in-progress
Última acción: action
Resultado: OK
Próxima acción: next
Contrato: contract
Próxima tarea si completa: T2
=== END RECITATION ===`

test("parseRecitation: parsea sección; null si no existe", () => {
  const rec = parseRecitation(`${PLAN}\n\n${RECITATION}\n`)
  assert.ok(rec)
  assert.strictEqual(rec.activeGoal, "goal")
  assert.strictEqual(rec.status, "in-progress")
  assert.strictEqual(rec.nextTask, "T2")
  assert.strictEqual(parseRecitation(PLAN), null)
})

test("updateRecitation: crea sección si falta, actualiza campos si existe", () => {
  const created = updateRecitation(PLAN, { campaignId: "cid", activeGoal: "g", status: "in-progress" })
  assert.ok(/=== RECITATION ===/.test(created))
  assert.ok(created.includes("Objetivo activo: g"))

  const updated = updateRecitation(`${PLAN}\n\n${RECITATION}\n`, { activeGoal: "nuevo", nextTask: "T9" })
  assert.ok(updated.includes("Objetivo activo: nuevo"))
  assert.ok(updated.includes("Próxima tarea si completa: T9"))
  // Contrato verbatim: los 8 campos se sobrescriben; los no provistos se vacían.
  assert.ok(updated.includes("Última acción: "))
})