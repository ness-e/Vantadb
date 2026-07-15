import type { Plugin } from "@opencode-ai/plugin"
import * as fs from "fs"
import * as path from "path"

interface Recitation {
  activeGoal: string
  status: string
  lastAction: string
  result: string
  nextAction: string
  contract: string
  nextTask: string
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

function extractRecitation(content: string): Recitation | null {
  const m = content.match(/=== RECITATION ===\n([\s\S]*?)=== END RECITATION ===/)
  if (!m) return null
  const block = m[1]
  const extract = (field: string) => {
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

export const CampaignCompactionPlugin: Plugin = async (ctx) => {
  return {
    "experimental.session.compacting": async (_input, output) => {
      const worktree = ctx.worktree || ctx.directory
      if (!worktree) return

      const planPath = findPlanFile(worktree)
      if (!planPath) return

      try {
        const content = fs.readFileSync(planPath, "utf-8")
        const recitation = extractRecitation(content)
        if (!recitation) return

        output.context.push(`## Campaign State (persisted across compaction)

Current campaign: ${path.basename(planPath)}
Active goal: ${recitation.activeGoal}
Last action: ${recitation.lastAction}
Result: ${recitation.result}
Next action: ${recitation.nextAction}
Next task: ${recitation.nextTask}
Contract to verify: ${recitation.contract}

You are in the middle of a backlog-execution campaign. Use the campaign_get_task and campaign_update_task tools to continue. Do NOT restart from scratch — pick up where the recitation left off.`)
      } catch {
        // Silently skip if file can't be read
      }
    },
  }
}
