import { tool } from "@opencode-ai/plugin"
import * as fs from "fs"
import * as path from "path"

interface PlanTask {
  id: string
  name: string
  priority: string
  effort: string
  files: string
  contract: string
  state: string
  source: string
  notes: string
  block: string
}

interface Recitation {
  activeGoal: string
  status: string
  lastAction: string
  result: string
  nextAction: string
  contract: string
  nextTask: string
}

interface TaskResult {
  hasTask: boolean
  task: PlanTask | null
  summary: {
    completed: number
    failed: number
    pending: number
    total: number
    doCount: number
    deferCount: number
    skipCount: number
    bloqueadoCount: number
  }
  recitation: Recitation | null
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

function parseTasks(content: string): PlanTask[] {
  const tasks: PlanTask[] = []
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

function extractField(block: string, field: string): string {
  const m = block.match(new RegExp(`- \\*\\*${field}:\\*\\*\\s*(.+)`))
  return m ? m[1].trim() : ""
}

function extractState(block: string): string {
  const m = block.match(/- \*\*Estado:\*\*\s*(.+)/)
  if (!m) return "⬜ PENDING"
  const raw = m[1].trim()
  if (raw.includes("✅")) return "✅ COMPLETED"
  if (raw.includes("❌")) return "❌ FAILED"
  if (raw.includes("⏳")) return "⏳ IN PROGRESS"
  return "⬜ PENDING"
}

function parseRecitation(content: string): Recitation | null {
  const m = content.match(/=== RECITATION ===\n([\s\S]*?)=== END RECITATION ===/)
  if (!m) return null
  const block = m[1]
  return {
    activeGoal: extractRecLine(block, "Objetivo activo"),
    status: extractRecLine(block, "Estado"),
    lastAction: extractRecLine(block, "Última acción"),
    result: extractRecLine(block, "Resultado"),
    nextAction: extractRecLine(block, "Próxima acción"),
    contract: extractRecLine(block, "Contrato"),
    nextTask: extractRecLine(block, "Próxima tarea si completa"),
  }
}

function extractRecLine(block: string, field: string): string {
  const m = block.match(new RegExp(`${field}:\\s*(.+?)(?:\\n|$)`))
  return m ? m[1].trim() : ""
}

function countGateResults(content: string): { do: number; defer: number; skip: number; bloqueado: number } {
  return {
    do: (content.match(/✅ DO/g) || []).length,
    defer: (content.match(/🟡 DEFER/g) || []).length,
    skip: (content.match(/❌ SKIP/g) || []).length,
    bloqueado: (content.match(/🔴 BLOQUEADO/g) || []).length,
  }
}

export default tool({
  description: "Lee el plan de campaña y devuelve la próxima tarea pendiente con resumen de progreso",
  args: {
    planFile: tool.schema.string().optional().describe("Ruta al plan file. Si se omite, busca el más reciente en docs/plans/"),
  },
  async execute(args, context) {
    const worktree = context.worktree || context.directory
    if (!worktree) return { error: "No worktree/directory in context" }

    let planPath: string | null = null
    if (args.planFile) {
      planPath = path.resolve(worktree, args.planFile)
      if (!fs.existsSync(planPath)) planPath = null
    }
    if (!planPath) {
      planPath = findPlanFile(worktree)
    }
    if (!planPath) return { error: "No plan file found in docs/plans/" }

    const content = fs.readFileSync(planPath, "utf-8")
    const tasks = parseTasks(content)
    const pending = tasks.filter(t => t.state === "⬜ PENDING" || t.state === "⏳ IN PROGRESS")
    const completed = tasks.filter(t => t.state === "✅ COMPLETED").length
    const failed = tasks.filter(t => t.state === "❌ FAILED").length
    const gates = countGateResults(content)
    const recitation = parseRecitation(content)

    // Devolver solo task activa, no todo el archivo
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
    } satisfies TaskResult
  },
})
