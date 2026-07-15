import { tool } from "@opencode-ai/plugin"
import * as fs from "fs"
import * as path from "path"

const STATE_MAP: Record<string, string> = {
  completed: "✅ COMPLETED",
  failed: "❌ FAILED",
  "in-progress": "⏳ EN PROGRESO",
  pending: "⬜ PENDING",
}

function findPlanFile(worktree: string): string | null {
  const planDir = path.join(worktree, "docs", "plans")
  if (!fs.existsSync(planDir)) return null
  const files = fs.readdirSync(planDir)
    .filter(f => f.endsWith(".md"))
    .map(f => ({ name: f, time: fs.statSync(path.join(planDir, f)).mtimeMs }))
    .sort((a, b) => b.time - a.time)
  return files.length > 0 ? path.join(planDir, files[0].name) : null
}

function findTaskById(content: string, taskId: string): { index: number; length: number; header: string } | null {
  // Match any task header — both numeric IDs and alphanumeric (DRV-068, etc.)
  const pattern = new RegExp(
    `(### Task\\s*${taskId.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}[^\\n]*\\n[\\s\\S]*?)(?=\n### Task |\\n## |\\n---|\\n===|$)`
  )
  const m = content.match(pattern)
  if (!m) return null
  const index = m.index!
  return { index, length: m[0].length, header: m[0] }
}

function extractField(block: string, field: string): string | null {
  const m = block.match(new RegExp(`(- \\*\\*${field}:\\*\\*\\s*).+`))
  return m ? m[1] : null
}

function updateState(content: string, taskId: string, newState: string): string {
  const mapped = STATE_MAP[newState]
  if (!mapped) return content

  const taskInfo = findTaskById(content, taskId)
  if (!taskInfo) return content

  const taskBlock = content.slice(taskInfo.index, taskInfo.index + taskInfo.length)
  const updated = taskBlock.replace(
    /(- \*\*Estado:\*\*\s*).+/,
    `$1${mapped}`
  )
  return content.slice(0, taskInfo.index) + updated + content.slice(taskInfo.index + taskInfo.length)
}

function updateRecitation(
  content: string,
  data: {
    activeGoal?: string
    status?: string
    lastAction?: string
    result?: string
    nextAction?: string
    contract?: string
    nextTask?: string
  }
): string {
  const hasRecitation = /=== RECITATION ===/.test(content)

  if (!hasRecitation) {
    // Create new recitation block
    const recitation = `=== RECITATION ===
Objetivo activo: ${data.activeGoal || ""}
Estado: ${data.status || "in-progress"}
Última acción: ${data.lastAction || ""}
Resultado: ${data.result || ""}
Próxima acción: ${data.nextAction || ""}
Contrato: ${data.contract || ""}
Próxima tarea si completa: ${data.nextTask || ""}
=== END RECITATION ===`
    return content.trimEnd() + "\n\n" + recitation + "\n"
  }

  let updated = content
  const replacements: [RegExp, string][] = [
    [/Objetivo activo:\s*.*/, `Objetivo activo: ${data.activeGoal || ""}`],
    [/Estado:\s*.*/, `Estado: ${data.status || "in-progress"}`],
    [/Última acción:\s*.*/, `Última acción: ${data.lastAction || ""}`],
    [/Resultado:\s*.*/, `Resultado: ${data.result || ""}`],
    [/Próxima acción:\s*.*/, `Próxima acción: ${data.nextAction || ""}`],
    [/Contrato:\s*.*/, `Contrato: ${data.contract || ""}`],
    [/Próxima tarea si completa:\s*.*/, `Próxima tarea si completa: ${data.nextTask || ""}`],
  ]
  for (const [pattern, replacement] of replacements) {
    updated = updated.replace(pattern, replacement)
  }
  return updated
}

export default tool({
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

    let planPath: string | null = null
    if (args.planFile) {
      planPath = path.resolve(worktree, args.planFile)
      if (!fs.existsSync(planPath)) planPath = null
    }
    if (!planPath) planPath = findPlanFile(worktree)
    if (!planPath) return { error: "No plan file found" }

    const original = fs.readFileSync(planPath, "utf-8")
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

    fs.writeFileSync(planPath, updated, "utf-8")
    return { updated: true, taskId: args.taskId, newState: args.newState, planFile: planPath }
  },
})
