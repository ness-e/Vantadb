// parsers.mjs — parsers puros del task-system (extraídos de campaign-server.mjs, TSYS-06-F1).
// Sin cambio de comportamiento: bodies verbatim desde el server; habilita tests de
// inyección de fallos (diseño docs/architecture/task-system-chaos-resilience.md §4, C1/C2/C11).
import { randomUUID } from "node:crypto"

export function extractField(block, field) {
  const m = block.match(new RegExp(`- \\*\\*${field}:\\*\\*\\s*(.+)`))
  return m ? m[1].trim() : ""
}

export function extractState(block) {
  const m = block.match(/- \*\*Estado:\*\*\s*(.+)/)
  if (!m) return "⬜ PENDING"
  const raw = m[1].trim()
  if (raw.includes("✅")) return "✅ COMPLETED"
  if (raw.includes("❌")) return "❌ FAILED"
  if (raw.includes("⏳")) return "⏳ IN PROGRESS"
  return "⬜ PENDING"
}

export function parseTasks(content) {
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

export function parseRecitation(content) {
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

export function extractCampaignId(content) {
  const m = content.match(/> \*\*Campaign ID:\*\*\s*(.+)/)
  if (!m) return null
  const id = m[1].trim()
  // Reject template placeholders so they never become trace filenames.
  return /^(\([^)]*\)|TODO|TBD|<[^>]+>)$|^[^0-9a-f-]{0,}$/.test(id) ? null : id
}

export function getOrCreateCampaignId(content) {
  const existing = extractCampaignId(content)
  if (existing) return { campaignId: existing, content }
  const id = randomUUID()
  const line = `> **Campaign ID:** ${id}\n`
  const updated = content.replace(/(^>\s\*\*Inicio:\*\*)/m, `${line}$1`)
  return { campaignId: id, content: updated }
}

export const STATE_MAP = {
  completed: "✅ COMPLETED",
  failed: "❌ FAILED",
  "in-progress": "⏳ EN PROGRESO",
  pending: "⬜ PENDING",
}

export function findTaskById(content, taskId) {
  const pattern = new RegExp(`(### Task\\s*${taskId.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}[^\\n]*\\n[\\s\\S]*?)(?=\n### Task |\\n## |\\n---|\\n===|$)`)
  const m = content.match(pattern)
  if (!m) return null
  return { index: m.index, length: m[0].length, header: m[0] }
}

export function updateState(content, taskId, newState) {
  const mapped = STATE_MAP[newState]
  if (!mapped) return content
  const taskInfo = findTaskById(content, taskId)
  if (!taskInfo) return content
  const taskBlock = content.slice(taskInfo.index, taskInfo.index + taskInfo.length)
  const updated = taskBlock.replace(/(- \*\*Estado:\*\*\s*).+/, `$1${mapped}`)
  return content.slice(0, taskInfo.index) + updated + content.slice(taskInfo.index + taskInfo.length)
}

export function updateRecitation(content, data) {
  const hasRecitation = /=== RECITATION ===/.test(content)
  if (!hasRecitation) {
    const rec = ["=== RECITATION ===", `Campaign ID: ${data.campaignId || ""}`, `Objetivo activo: ${data.activeGoal || ""}`, `Estado: ${data.status || "in-progress"}`, `Última acción: ${data.lastAction || ""}`, `Resultado: ${data.result || ""}`, `Próxima acción: ${data.nextAction || ""}`, `Contrato: ${data.contract || ""}`, `Próxima tarea si completa: ${data.nextTask || ""}`, "=== END RECITATION ==="].join("\n")
    return content.trimEnd() + "\n\n" + rec + "\n"
  }
  let updated = content
  // Anclados a inicio de línea (^...m): un `contract` que contenga literal
  // "Estado:" o "Contrato:" no debe corromper campos vecinos de la recitation.
  const reps = [
    [/^Campaign ID:\s*.*/m, `Campaign ID: ${data.campaignId || ""}`],
    [/^Objetivo activo:\s*.*/m, `Objetivo activo: ${data.activeGoal || ""}`],
    [/^Estado:\s*.*/m, `Estado: ${data.status || "in-progress"}`],
    [/^Última acción:\s*.*/m, `Última acción: ${data.lastAction || ""}`],
    [/^Resultado:\s*.*/m, `Resultado: ${data.result || ""}`],
    [/^Próxima acción:\s*.*/m, `Próxima acción: ${data.nextAction || ""}`],
    [/^Contrato:\s*.*/m, `Contrato: ${data.contract || ""}`],
    [/^Próxima tarea si completa:\s*.*/m, `Próxima tarea si completa: ${data.nextTask || ""}`],
  ]
  for (const [pat, rep] of reps) updated = updated.replace(pat, rep)
  return updated
}