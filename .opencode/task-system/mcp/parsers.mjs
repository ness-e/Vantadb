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

// Recitations por-tarea (`=== RECITATION <ID> ===`): con waves paralelas cada
// sub-agente escribe SU bloque sin pisar el de otros. El bloque legacy sin ID
// se mantiene por compat y se trata como recitation global.
function recitationBlocks(content) {
  const out = []
  const re = /=== RECITATION(?:\s+([^\s=]+))?\s*===\n([\s\S]*?)=== END RECITATION ===/g
  for (const m of content.matchAll(re)) out.push({ id: m[1] || null, body: m[2], index: m.index, length: m[0].length })
  return out
}

export function parseRecitation(content, taskId = null) {
  const blocks = recitationBlocks(content)
  if (blocks.length === 0) return null
  let block = null
  if (taskId) {
    // Específica por tarea si existe; si no, cae a la última disponible.
    block = blocks.find(b => b.id === String(taskId)) || blocks[blocks.length - 1]
  } else {
    block = blocks[blocks.length - 1] // la más reciente gana
  }
  const b = block.body
  const extract = (field) => {
    const r = b.match(new RegExp(`${field}:\\s*(.+?)(?:\\n|$)`))
    return r ? r[1].trim() : ""
  }
  return {
    taskId: block.id,
    activeGoal: extract("Objetivo activo"),
    status: extract("Estado"),
    lastAction: extract("Última acción"),
    result: extract("Resultado"),
    nextAction: extract("Próxima acción"),
    contract: extract("Contrato"),
    nextTask: extract("Próxima tarea si completa"),
  }
}

function isPlaceholderId(id) {
  return /^(\([^)]*\)|TODO|TBD|<[^>]+>)$/.test(id)
}

// Escanea TODAS las líneas Campaign ID y devuelve el primer ID válido.
// Antes tomaba solo el primer match: un placeholder tipo "(auto por MCP)"
// sombreaba al ID real para siempre y cada write insertaba otra línea.
export function extractCampaignId(content) {
  for (const m of content.matchAll(/> \*\*Campaign ID:\*\*\s*(.+)/g)) {
    const id = m[1].trim()
    if (id && !isPlaceholderId(id)) return id
  }
  return null
}

export function getOrCreateCampaignId(content) {
  const existing = extractCampaignId(content)
  if (existing) {
    // Dedup: colapsa a UNA sola línea con el ID válido (borra placeholders/duplicados).
    let kept = false
    const cleaned = content.split("\n").filter(l => {
      if (!/^> \*\*Campaign ID:\*\*/.test(l)) return true
      if (kept || !l.includes(existing)) return false
      kept = true
      return true
    })
    return { campaignId: existing, content: cleaned.join("\n") }
  }
  const id = randomUUID()
  const line = `> **Campaign ID:** ${id}`
  // Si hay línea placeholder, reemplázala; si no, inserta antes de Inicio.
  if (/> \*\*Campaign ID:\*\*/.test(content)) {
    return { campaignId: id, content: content.replace(/^> \*\*Campaign ID:\*\*.*/m, line) }
  }
  const updated = content.replace(/(^>\s\*\*Inicio:\*\*)/m, `${line}\n$1`)
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
  const tag = data.taskId ? ` ${data.taskId}` : ""
  const open = `=== RECITATION${tag} ===`
  if (!content.includes(open)) {
    const rec = [open, `Campaign ID: ${data.campaignId || ""}`, `Objetivo activo: ${data.activeGoal || ""}`, `Estado: ${data.status || "in-progress"}`, `Última acción: ${data.lastAction || ""}`, `Resultado: ${data.result || ""}`, `Próxima acción: ${data.nextAction || ""}`, `Contrato: ${data.contract || ""}`, `Próxima tarea si completa: ${data.nextTask || ""}`, "=== END RECITATION ==="].join("\n")
    return content.trimEnd() + "\n\n" + rec + "\n"
  }
  // Reemplaza SOLO el bloque de esta tarea (anclado entre sus marcadores) —
  // las recitations de otras tareas en waves paralelas quedan intactas.
  const openRe = open.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")
  const re = new RegExp(`${openRe}\\n[\\s\\S]*?=== END RECITATION ===`)
  const body = [`Campaign ID: ${data.campaignId || ""}`, `Objetivo activo: ${data.activeGoal || ""}`, `Estado: ${data.status || "in-progress"}`, `Última acción: ${data.lastAction || ""}`, `Resultado: ${data.result || ""}`, `Próxima acción: ${data.nextAction || ""}`, `Contrato: ${data.contract || ""}`, `Próxima tarea si completa: ${data.nextTask || ""}`, "=== END RECITATION ==="].join("\n")
  return content.replace(re, `${open}\n${body}`)
}

// Modo autónomo del plan: `> **Autonomous:** true` suprime Gates P/D/C
// (Gate V y seguridad siempre operan — ver prompts/question-gates.md §Anti-abuso).
export function extractAutonomous(content) {
  const m = content.match(/^>\s*\*\*Autonomous:\*\*\s*(true|false)\s*$/im)
  return m ? m[1].toLowerCase() === "true" : null
}