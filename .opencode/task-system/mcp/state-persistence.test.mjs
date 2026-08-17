// state-persistence.test.mjs — tests de los 3 behavior changes TSYS-06 §6
// (diseño docs/architecture/task-system-chaos-resilience.md):
//   (a) C3/C7  corrupción visible de budget (readBudget → budgetCorrupted; writeBudget strip)
//   (b) C4/C6  writes con checksum + atómicos (temp+rename, detectConflict)
//   (c) C12    WIP check-and-set atómico bajo withPlanLock
// Importa campaign-server.mjs: el isMain guard evita conectar el transporte stdio.
// Node v24 built-in test runner — cero dependencias nuevas.
import { test, beforeEach, afterEach } from "node:test"
import assert from "node:assert/strict"
import { mkdtempSync, writeFileSync, readFileSync, existsSync, rmSync, mkdirSync, readdirSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"
import {
  readBudget, writeBudget, withPlanLock, findInProgressTasks,
  detectConflict, sha1, updateTaskStateCore, budgetStatus,
} from "./campaign-server.mjs"

const PLAN = `# Plan: TSYS-06 Persistence Fixture

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

### Task 2: Second task
- **Prioridad:** 🟡
- **Esfuerzo:** 🟡
- **Estado:** ⬜ PENDING
- **Archivos clave:** docs/bar.md
- **Contrato:** none
- **Fuente:** test
`

let tmp
let planPath
let worktree

function setupWorktree(content = PLAN) {
  worktree = mkdtempSync(join(tmpdir(), "tsys06-"))
  const planDir = join(worktree, "docs", "plans")
  mkdirSync(planDir, { recursive: true })
  planPath = join(planDir, "plan.md")
  writeFileSync(planPath, content, "utf-8")
  return planPath
}

beforeEach(() => {
  setupWorktree()
})

afterEach(() => {
  if (worktree) { try { rmSync(worktree, { recursive: true, force: true }) } catch {} }
})

// ---------- (a) Corrupción visible de budget (C3/C7) ----------

test("C3: readBudget con JSON truncado → budgetCorrupted:true, nunca reset silencioso", () => {
  writeFileSync(planPath.replace(/\.md$/, ".budget.json"), `{ "tasks": {`, "utf-8")
  const state = readBudget(planPath)
  assert.deepStrictEqual(state.tasks, {})
  assert.strictEqual(state.budgetCorrupted, true)
})

test("C3: readBudget con JSON inválido → budgetCorrupted:true", () => {
  writeFileSync(planPath.replace(/\.md$/, ".budget.json"), `{ "tasks": }`, "utf-8")
  const state = readBudget(planPath)
  assert.strictEqual(state.budgetCorrupted, true)
})

test("C7: writeBudget strip del flag — la corrupción no se persiste como dato", () => {
  writeBudget(planPath, { tasks: {}, budgetCorrupted: true })
  const persisted = readFileSync(planPath.replace(/\.md$/, ".budget.json"), "utf-8")
  assert.ok(!persisted.includes("budgetCorrupted"))
  const state = readBudget(planPath)
  assert.strictEqual(state.budgetCorrupted, undefined)
})

test("C3: budgetStatus surfaca budgetCorrupted en la respuesta", () => {
  writeFileSync(planPath.replace(/\.md$/, ".budget.json"), `{ "tasks": {`, "utf-8")
  const status = budgetStatus("1", worktree)
  assert.strictEqual(status.exists, false)
  assert.strictEqual(status.budgetCorrupted, true)
})

test("ENOENT: plan sin budget (primer uso) NO es corrupción — flag ausente", () => {
  const state = readBudget(planPath)
  assert.deepStrictEqual(state.tasks, {})
  assert.strictEqual(state.budgetCorrupted, undefined)
})

// ---------- (b) Writes con checksum + atómicos (C4/C6) ----------

test("C6: updateTaskStateCore escribe atómico — archivo válido, sin .tmp residual", () => {
  const res = updateTaskStateCore(planPath, "1", "completed", null, worktree)
  assert.strictEqual(res.updated, true)
  assert.strictEqual(res.newState, "completed")
  // Sin archivo temporal residual.
  assert.ok(!existsSync(`${planPath}.tmp`))
  // Contenido íntegro y parseable con el nuevo estado.
  const after = readFileSync(planPath, "utf-8")
  const task1Block = after.slice(after.indexOf("### Task 1"), after.indexOf("### Task 2"))
  assert.ok(task1Block.includes("- **Estado:** ✅ COMPLETED"))
  assert.ok(task1Block.includes("### Task 1: First task")) // bloque completo, no truncado
  assert.ok(after.includes("### Task 2: Second task")) // el resto del plan intacto
})

test("C6: write atómico conserva el plan ante fallo — update fallido no deja archivo tocado", () => {
  // Task inexistente → updated:false, el archivo queda exactamente como estaba.
  const before = readFileSync(planPath, "utf-8")
  const res = updateTaskStateCore(planPath, "999", "completed", null, worktree)
  assert.strictEqual(res.updated, false)
  assert.ok(res.warning)
  assert.strictEqual(readFileSync(planPath, "utf-8"), before)
  assert.ok(!existsSync(`${planPath}.tmp`))
})

test("C4 mechanism: detectConflict detecta modificación concurrente del plan", () => {
  const original = readFileSync(planPath, "utf-8")
  assert.strictEqual(detectConflict(original, sha1(original)), false)
  const modified = original.replace("⬜ PENDING", "✅ COMPLETED")
  assert.strictEqual(detectConflict(modified, sha1(original)), true)
})

test("C4 mechanism: modificación concurrente dentro de withPlanLock se detecta", () => {
  // Simula la ventana read→check del core, con un "otro writer" modificando el
  // archivo mientras sostenemos el lock (carrera C4 comprimida en el tiempo).
  const detected = withPlanLock(planPath, () => {
    const original = readFileSync(planPath, "utf-8")
    const checksum = sha1(original)
    writeFileSync(planPath, original.replace("- **Estado:** ⬜ PENDING", "- **Estado:** ✅ COMPLETED"), "utf-8")
    return detectConflict(readFileSync(planPath, "utf-8"), checksum)
  })
  assert.strictEqual(detected, true)
})

test("withPlanLock: crea lock file durante la sección y lo libera al salir", () => {
  let sawLock = false
  const res = withPlanLock(planPath, () => {
    sawLock = existsSync(`${planPath}.lock`)
    return 42
  })
  assert.strictEqual(sawLock, true)
  assert.strictEqual(res, 42)
  assert.ok(!existsSync(`${planPath}.lock`))
})

// ---------- (c) WIP check-and-set atómico (C12) ----------

test("C12: doble claim in-progress → solo el primero gana, el segundo wipBlocked", () => {
  const r1 = updateTaskStateCore(planPath, "1", "in-progress", null, worktree)
  assert.strictEqual(r1.updated, true)

  // El segundo claim ve la tarea 1 en progreso (check-and-set dentro del lock).
  const r2 = updateTaskStateCore(planPath, "2", "in-progress", null, worktree)
  assert.strictEqual(r2.updated, false)
  assert.strictEqual(r2.wipBlocked, true)
  assert.ok(r2.error.includes("one-task-at-a-time"))

  // El estado de la tarea 2 no se tocó.
  const after = readFileSync(planPath, "utf-8")
  const task2Block = after.slice(after.indexOf("### Task 2"))
  assert.ok(task2Block.includes("- **Estado:** ⬜ PENDING"))
})

test("C12: liberar el WIP (completar) permite el siguiente claim", () => {
  updateTaskStateCore(planPath, "1", "in-progress", null, worktree)
  updateTaskStateCore(planPath, "1", "completed", null, worktree)
  const r3 = updateTaskStateCore(planPath, "2", "in-progress", null, worktree)
  assert.strictEqual(r3.updated, true)
})

test("C12: findInProgressTasks escanea worktree y no cuenta la propia tarea", () => {
  updateTaskStateCore(planPath, "1", "in-progress", null, worktree)
  const active = findInProgressTasks(worktree)
  assert.strictEqual(active.length, 1)
  assert.strictEqual(active[0].id, "1")
  assert.strictEqual(active[0].source, "docs/plans/plan.md")
})

test("C2 regression: task removida del plan mientras in-progress → updated:false + warning", () => {
  updateTaskStateCore(planPath, "1", "in-progress", null, worktree)
  // "Otro proceso" elimina la tarea 1 del plan (simula edición manual concurrente).
  const lines = readFileSync(planPath, "utf-8").split("\n")
  const idx = lines.findIndex(l => l.startsWith("### Task 1"))
  writeFileSync(planPath, lines.slice(0, idx).join("\n"), "utf-8")

  const res = updateTaskStateCore(planPath, "1", "completed", null, worktree)
  assert.strictEqual(res.updated, false)
  assert.ok(res.warning)
  // Sin bloque fantasma: findInProgressTasks ya no la cuenta.
  assert.deepStrictEqual(findInProgressTasks(worktree).filter(t => t.id === "1"), [])
})

test("lock: timeout con lock ajeno sostenido → error claro, no hang", () => {
  // Sostenemos el lock manualmente (simula un writer concurrente vivo).
  const lockPath = `${planPath}.lock`
  writeFileSync(lockPath, JSON.stringify({ pid: -1, ts: Date.now() }), "utf-8")
  const started = Date.now()
  assert.throws(() => updateTaskStateCore(planPath, "1", "in-progress", null, worktree), /Timed out waiting for plan lock/)
  assert.ok(Date.now() - started < 8000) // timeout 5s + margen
  rmSync(lockPath, { force: true })
})