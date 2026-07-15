import { tool } from "@opencode-ai/plugin"
import { readFileSync, writeFileSync, existsSync, readdirSync, statSync } from "node:fs"
import { resolve, join } from "node:path"
import { execSync } from "node:child_process"

// ---------- helpers ----------

function findPlanFile(worktree) {
  const planDir = join(worktree, "docs", "plans")
  if (!existsSync(planDir)) return null
  const files = readdirSync(planDir)
    .filter(f => f.endsWith(".md"))
    .map(f => ({ name: f, time: statSync(join(planDir, f)).mtimeMs }))
    .sort((a, b) => b.time - a.time)
  return files.length > 0 ? join(planDir, files[0].name) : null
}

function resolvePlan(planFile, worktree) {
  let planPath = null
  if (planFile) {
    planPath = resolve(worktree, planFile)
    if (!existsSync(planPath)) planPath = null
  }
  if (!planPath) planPath = findPlanFile(worktree)
  return planPath
}

function extractField(block, field) {
  const m = block.match(new RegExp(`- \\*\\*${field}:\\*\\*\\s*(.+)`))
  return m ? m[1].trim() : ""
}

function extractState(block) {
  const m = block.match(/- \*\*Estado:\*\*\s*(.+)/)
  if (!m) return "⬜ PENDING"
  const raw = m[1].trim()
  if (raw.includes("✅")) return "✅ COMPLETED"
  if (raw.includes("❌")) return "❌ FAILED"
  if (raw.includes("⏳")) return "⏳ IN PROGRESS"
  return "⬜ PENDING"
}

function parseTasks(content) {
  const tasks = []
  const blocks = content.split(/\n(?=### Task \d+)/)
  for (const block of blocks) {
    const headerMatch = block.match(/### Task (\d+):\s*(.+)/)
    if (!headerMatch) continue
    tasks.push({
      id: headerMatch[1],
      name: headerMatch[2].trim(),
      priority: extractField(block, "Prioridad"),
      effort: extractField(block, "Esfuerzo"),
      files: extractField(block, "Archivos clave"),
      contract: extractField(block, "Contrato"),
      state: extractState(block),
      source: extractField(block, "Fuente"),
      notes: extractField(block, "Notas"),
      block,
    })
  }
  return tasks
}

function parseRecitation(content) {
  const m = content.match(/=== RECITATION ===\n([\s\S]*?)=== END RECITATION ===/)
  if (!m) return null
  const block = m[1]
  const extract = (field) => {
    const r = block.match(new RegExp(`${field}:\\s*(.+?)(?:\\n|$)`))
    return r ? r[1].trim() : ""
  }
  return {
    activeGoal: extract("Objetivo activo"),
    status: extract("Estado"),
    lastAction: extract("Última acción"),
    result: extract("Resultado"),
    nextAction: extract("Próxima acción"),
    contract: extract("Contrato"),
    nextTask: extract("Próxima tarea si completa"),
  }
}

function countGateResults(content) {
  return {
    do: (content.match(/✅ DO/g) || []).length,
    defer: (content.match(/🟡 DEFER/g) || []).length,
    skip: (content.match(/❌ SKIP/g) || []).length,
    bloqueado: (content.match(/🔴 BLOQUEADO/g) || []).length,
  }
}

// ---------- tool definitions ----------

const campaign_get_task = tool({
  description: "Lee el plan de campaña y devuelve la próxima tarea pendiente con resumen de progreso",
  args: {
    planFile: tool.schema.string().optional().describe("Ruta al plan file. Si se omite, busca el más reciente en docs/plans/"),
  },
  async execute(args, context) {
    const worktree = context.worktree || context.directory
    if (!worktree) return { error: "No worktree/directory in context" }

    const planPath = resolvePlan(args.planFile, worktree)
    if (!planPath) return { error: "No plan file found in docs/plans/" }

    const content = readFileSync(planPath, "utf-8")
    const tasks = parseTasks(content)
    const pending = tasks.filter(t => t.state === "⬜ PENDING" || t.state === "⏳ IN PROGRESS")
    const completed = tasks.filter(t => t.state === "✅ COMPLETED").length
    const failed = tasks.filter(t => t.state === "❌ FAILED").length
    const gates = countGateResults(content)
    const recitation = parseRecitation(content)
    const nextTask = pending.length > 0 ? pending[0] : null

    return {
      planFile: planPath,
      hasTask: nextTask !== null,
      task: nextTask,
      summary: {
        completed,
        failed,
        pending: pending.length,
        total: tasks.length,
        doCount: gates.do,
        deferCount: gates.defer,
        skipCount: gates.skip,
        bloqueadoCount: gates.bloqueado,
      },
      recitation,
    }
  },
})

const STATE_MAP = {
  completed: "✅ COMPLETED",
  failed: "❌ FAILED",
  "in-progress": "⏳ EN PROGRESO",
  pending: "⬜ PENDING",
}

function findTaskById(content, taskId) {
  const pattern = new RegExp(
    `(### Task\\s*${taskId.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}[^\\n]*\\n[\\s\\S]*?)(?=\n### Task |\\n## |\\n---|\\n===|$)`
  )
  const m = content.match(pattern)
  if (!m) return null
  return { index: m.index, length: m[0].length, header: m[0] }
}

function updateState(content, taskId, newState) {
  const mapped = STATE_MAP[newState]
  if (!mapped) return content
  const taskInfo = findTaskById(content, taskId)
  if (!taskInfo) return content
  const taskBlock = content.slice(taskInfo.index, taskInfo.index + taskInfo.length)
  const updated = taskBlock.replace(/(- \*\*Estado:\*\*\s*).+/, `$1${mapped}`)
  return content.slice(0, taskInfo.index) + updated + content.slice(taskInfo.index + taskInfo.length)
}

function updateRecitation(content, data) {
  const hasRecitation = /=== RECITATION ===/.test(content)
  if (!hasRecitation) {
    const rec = [
      "=== RECITATION ===",
      `Objetivo activo: ${data.activeGoal || ""}`,
      `Estado: ${data.status || "in-progress"}`,
      `Última acción: ${data.lastAction || ""}`,
      `Resultado: ${data.result || ""}`,
      `Próxima acción: ${data.nextAction || ""}`,
      `Contrato: ${data.contract || ""}`,
      `Próxima tarea si completa: ${data.nextTask || ""}`,
      "=== END RECITATION ===",
    ].join("\n")
    return content.trimEnd() + "\n\n" + rec + "\n"
  }
  let updated = content
  const reps = [
    [/Objetivo activo:\s*.*/, `Objetivo activo: ${data.activeGoal || ""}`],
    [/Estado:\s*.*/, `Estado: ${data.status || "in-progress"}`],
    [/Última acción:\s*.*/, `Última acción: ${data.lastAction || ""}`],
    [/Resultado:\s*.*/, `Resultado: ${data.result || ""}`],
    [/Próxima acción:\s*.*/, `Próxima acción: ${data.nextAction || ""}`],
    [/Contrato:\s*.*/, `Contrato: ${data.contract || ""}`],
    [/Próxima tarea si completa:\s*.*/, `Próxima tarea si completa: ${data.nextTask || ""}`],
  ]
  for (const [pat, rep] of reps) updated = updated.replace(pat, rep)
  return updated
}

const campaign_update_task = tool({
  description: "Actualiza el estado de una tarea en el plan file y escribe la recitation estructurada",
  args: {
    taskId: tool.schema.string().describe("ID de la tarea a actualizar (ej: '14', 'DRV-068')"),
    newState: tool.schema.enum(["completed", "failed", "in-progress", "pending"]).describe("Nuevo estado de la tarea"),
    planFile: tool.schema.string().optional().describe("Ruta al plan file. Si se omite, busca el más reciente."),
    recitation: tool.schema.object({
      activeGoal: tool.schema.string().optional().describe("Objetivo activo actual"),
      lastAction: tool.schema.string().optional().describe("Qué se hizo en esta iteración"),
      result: tool.schema.string().optional().describe("Resultado (✅ o ❌)"),
      nextAction: tool.schema.string().optional().describe("Próxima acción a tomar"),
      contract: tool.schema.string().optional().describe("Contrato de validación cumplido"),
      nextTask: tool.schema.string().optional().describe("ID de la próxima tarea a ejecutar"),
    }).optional().describe("Datos estructurados de recitation"),
  },
  async execute(args, context) {
    const worktree = context.worktree || context.directory
    if (!worktree) return { error: "No worktree/directory in context" }

    const planPath = resolvePlan(args.planFile, worktree)
    if (!planPath) return { error: "No plan file found" }

    const original = readFileSync(planPath, "utf-8")
    let updated = updateState(original, args.taskId, args.newState)

    if (args.recitation) {
      updated = updateRecitation(updated, {
        activeGoal: args.recitation.activeGoal,
        status: args.newState,
        lastAction: args.recitation.lastAction,
        result: args.recitation.result,
        nextAction: args.recitation.nextAction,
        contract: args.recitation.contract,
        nextTask: args.recitation.nextTask,
      })
    }

    if (updated === original) {
      return { updated: false, warning: `Task ${args.taskId} not found or no changes needed` }
    }

    writeFileSync(planPath, updated, "utf-8")
    return { updated: true, taskId: args.taskId, newState: args.newState, planFile: planPath }
  },
})

const campaign_verify = tool({
  description: "Ejecuta un comando de validación y devuelve resultado estructurado. Contrato: cargo nextest, cargo check, cargo fmt, etc.",
  args: {
    command: tool.schema.string().describe("Comando a ejecutar (ej: 'cargo check -p vantadb-litellm', 'cargo nextest run --profile audit --workspace --build-jobs 2')"),
    expectedExitCode: tool.schema.number().optional().default(0).describe("Exit code esperado (default: 0)"),
    timeout: tool.schema.number().optional().default(300).describe("Timeout en segundos (default: 300)"),
    taskId: tool.schema.string().optional().describe("ID de tarea asociada para logging"),
  },
  async execute(args) {
    const startTime = Date.now()
    let stdout = ""
    let stderr = ""
    let exitCode = -1

    try {
      const out = execSync(args.command, {
        encoding: "utf-8",
        timeout: args.timeout * 1000,
        windowsHide: true,
        maxBuffer: 10 * 1024 * 1024,
        shell: process.platform === "win32" ? "pwsh" : true,
      })
      stdout = (out || "").trim()
      exitCode = 0
    } catch (e) {
      stdout = (e.stdout || "").trim()
      stderr = (e.stderr || "").trim()
      exitCode = e.status ?? -1
    }

    const elapsed = ((Date.now() - startTime) / 1000).toFixed(1)
    const passed = exitCode === args.expectedExitCode
    const nextestMatch = stdout.match(/(\d+)\s+passed.*?(\d+)\s+failed/s)
    const summary = nextestMatch
      ? { passed: parseInt(nextestMatch[1]), failed: parseInt(nextestMatch[2]) }
      : null

    return {
      passed,
      exitCode,
      expectedExitCode: args.expectedExitCode,
      elapsed: `${elapsed}s`,
      taskId: args.taskId || null,
      summary,
      stdout: stdout.length > 2000 ? stdout.slice(0, 2000) + `\n... [truncated, ${stdout.length} total chars]` : stdout,
      stderr: stderr.length > 1000 ? stderr.slice(0, 1000) + `\n... [truncated, ${stderr.length} total chars]` : stderr,
    }
  },
})

// ---------- plugin export ----------

export const CampaignToolsPlugin = async () => {
  return {
    tool: {
      campaign_get_next_task: campaign_get_task,
      campaign_update_task_state: campaign_update_task,
      campaign_verify_cmd: campaign_verify,
    },
  }
}
