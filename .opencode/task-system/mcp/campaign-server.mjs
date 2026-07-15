import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js"
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js"
import { z } from "zod"
import { readFileSync, writeFileSync, existsSync, readdirSync, statSync, rmSync, appendFileSync } from "node:fs"
import { fileURLToPath } from "url"
import { resolve, join, dirname } from "node:path"
import { execSync } from "node:child_process"
import { randomUUID } from "node:crypto"
import { emit as traceEmit, getHealth } from "../traces/tracer.mjs"
import { getTraits, listModels, escalateTier, tierForModel, TIERS } from "../config/model-traits.mjs"
import { STATE_TOOLS, getAllowedTools, validateAction } from "../config/state-tools.mjs"

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)
const PROJECT_ROOT = resolve(__dirname, "..", "..", "..")
const TASK_SYSTEM = resolve(__dirname, "..")

const server = new McpServer({ name: "campaign-tools", version: "1.0.0" })

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

function extractCampaignId(content) {
  const m = content.match(/> \*\*Campaign ID:\*\*\s*(.+)/)
  return m ? m[1].trim() : null
}

function getOrCreateCampaignId(content) {
  const existing = extractCampaignId(content)
  if (existing) return { campaignId: existing, content }
  const id = randomUUID()
  const line = `> **Campaign ID:** ${id}\n`
  const updated = content.replace(/(^>\s\*\*Inicio:\*\*)/m, `${line}$1`)
  return { campaignId: id, content: updated }
}

// ---------- Output Validation (LLM05) ----------

const DANGEROUS_CMD = [
  /\brm\s+-rf?\b/i, /\brm\s+-fr?\b/i, /\bformat\s+\w:?\b/i,
  /\brd\s+\/s\s+\/q\b/i, /\bmkfs\.\w+\b/, /\bdd\s+if=/,
  /\bchmod\s+777\s+\//, /\bchown\s+.*\s+\//,
  /:\(\)\{/, />\s*\/dev\/(sda|sdb|sdc|null)/,
]
const PIPED_SHELL = [/\|\s*(bash|sh|zsh|pwsh|powershell|cmd)\b/, /`[^`]*`/, /\$\(/]
const SYS_DIRS = ["/etc", "/bin", "/sbin", "/usr", "/boot", "/dev", "/proc", "/sys",
  "C:\\Windows", "C:\\System32", "C:\\Program Files"]
const DANGEROUS_PY = ["import os", "import subprocess", "import sys", "eval(", "exec(", "__import__("]
const DDL_SQL = /\b(drop|truncate|alter|create|grant|revoke)\s+/i

function validateShellCommand(cmd) {
  const errors = [], warnings = [], checks = []
  if (!cmd || !cmd.trim()) return { valid: false, riskLevel: "dangerous", errors: ["Empty command"], warnings: [], checksPassed: [] }
  for (const pat of DANGEROUS_CMD) { if (pat.test(cmd)) errors.push(`Dangerous pattern: ${pat}`) }
  for (const pat of PIPED_SHELL) { if (pat.test(cmd)) warnings.push(`Piped to shell interpreter: ${pat}`) }
  checks.push("Shell command checked")
  return { valid: errors.length === 0, riskLevel: errors.length ? "dangerous" : warnings.length ? "moderate" : "safe", errors, warnings, checksPassed: checks }
}

function validateFilePath(fp, workspace) {
  const errors = [], warnings = [], checks = []
  if (!fp || !fp.trim()) return { valid: false, riskLevel: "dangerous", errors: ["Empty path"], warnings: [], checksPassed: [] }
  if (fp.includes("..")) errors.push("Path traversal detected")
  const resolved = resolve(fp)
  if (workspace) {
    try { if (!resolved.startsWith(resolve(workspace))) errors.push("Path escapes workspace") }
    catch { errors.push("Cannot resolve path") }
  }
  for (const d of SYS_DIRS) { if (resolved.toLowerCase().startsWith(d.toLowerCase())) errors.push(`Writes to system directory: ${d}`) }
  checks.push("File path checked")
  return { valid: errors.length === 0, riskLevel: errors.length ? "dangerous" : "safe", errors, warnings, checksPassed: checks }
}

function validatePythonCode(code) {
  const errors = [], warnings = [], checks = []
  if (!code || !code.trim()) return { valid: false, riskLevel: "dangerous", errors: ["Empty code"], warnings: [], checksPassed: [] }
  for (const d of DANGEROUS_PY) { if (code.includes(d)) warnings.push(`Contains: ${d}`) }
  checks.push("Python code checked (dangerous imports)")
  return { valid: true, riskLevel: warnings.length ? "moderate" : "safe", errors, warnings, checksPassed: checks }
}

function validateSql(sql) {
  const errors = [], warnings = [], checks = []
  if (!sql || !sql.trim()) return { valid: false, riskLevel: "dangerous", errors: ["Empty SQL"], warnings: [], checksPassed: [] }
  if (DDL_SQL.test(sql)) warnings.push("SQL contains DDL/DCL keyword")
  checks.push("SQL checked")
  return { valid: true, riskLevel: warnings.length ? "moderate" : "safe", errors, warnings, checksPassed: checks }
}

function validateHtml(html) {
  const warnings = []
  if (html && html.includes("<script")) warnings.push("HTML contains <script> — XSS risk")
  return { valid: true, riskLevel: warnings.length ? "moderate" : "safe", errors: [], warnings, checksPassed: ["HTML checked"] }
}

function validateOutput(content, type = "text", workspace = null) {
  switch (type) {
    case "shell": return validateShellCommand(content)
    case "file_path": return validateFilePath(content, workspace)
    case "python": case "code": return validatePythonCode(content)
    case "sql": return validateSql(content)
    case "html": return validateHtml(content)
    default: return { valid: true, riskLevel: "safe", errors: [], warnings: [], checksPassed: ["Text validated"] }
  }
}

// ---------- Budget Tracking (#3) ----------

const BUDGET_LIMITS = {
  maxIterations: 10,
  maxToolCalls: 15,
  maxSubAgents: 40,
  maxConsecutiveFails: 5,
  maxDurationMinutes: 120,
}

function budgetPath(worktree) { const p = findPlanFile(worktree); return p ? p.replace(/\.md$/, ".budget.json") : null }

function readBudget(planPath) {
  const bp = planPath.replace(/\.md$/, ".budget.json")
  try { return JSON.parse(readFileSync(bp, "utf-8")) }
  catch { return { tasks: {} } }
}

function writeBudget(planPath, state) {
  writeFileSync(planPath.replace(/\.md$/, ".budget.json"), JSON.stringify(state, null, 2), "utf-8")
}

function initTaskBudget(planPath, taskId) {
  const state = readBudget(planPath)
  if (!state.tasks[taskId]) {
    state.tasks[taskId] = { taskId, toolCalls: 0, subAgentCalls: 0, consecutiveFails: 0, startTime: Date.now(), lastActivity: Date.now() }
  }
  state.tasks[taskId].lastActivity = Date.now()
  writeBudget(planPath, state)
  return state.tasks[taskId]
}

function consumeBudget(taskId, worktree) {
  const planPath = findPlanFile(worktree)
  if (!planPath) return null
  const state = readBudget(planPath)
  if (!state.tasks[taskId]) initTaskBudget(planPath, taskId)
  const t = state.tasks[taskId]
  t.toolCalls++
  t.lastActivity = Date.now()
  const elapsed = (t.lastActivity - t.startTime) / 60000
  const withinBudget = t.toolCalls <= BUDGET_LIMITS.maxToolCalls && elapsed <= BUDGET_LIMITS.maxDurationMinutes && t.consecutiveFails <= BUDGET_LIMITS.maxConsecutiveFails
  writeBudget(planPath, state)
  return { withinBudget, toolCalls: t.toolCalls, consecutiveFails: t.consecutiveFails, elapsedMinutes: Math.round(elapsed), limits: BUDGET_LIMITS }
}

function budgetStatus(taskId, worktree) {
  const planPath = findPlanFile(worktree)
  if (!planPath) return null
  const state = readBudget(planPath)
  const t = state.tasks[taskId]
  if (!t) return { exists: false }
  const elapsed = (Date.now() - t.startTime) / 60000
  return {
    exists: true, taskId, toolCalls: t.toolCalls, subAgentCalls: t.subAgentCalls,
    consecutiveFails: t.consecutiveFails, elapsedMinutes: Math.round(elapsed),
    withinBudget: t.toolCalls <= BUDGET_LIMITS.maxToolCalls && elapsed <= BUDGET_LIMITS.maxDurationMinutes && t.consecutiveFails <= BUDGET_LIMITS.maxConsecutiveFails,
    limits: BUDGET_LIMITS,
  }
}

function budgetReset(taskId, worktree) {
  const planPath = findPlanFile(worktree)
  if (!planPath) return null
  const state = readBudget(planPath)
  delete state.tasks[taskId]
  writeBudget(planPath, state)
  return { reset: true }
}

// ---------- Tool: campaign_validate_output ----------

server.tool(
  "campaign_validate_output",
  {
    content: z.string().describe("Content to validate"),
    type: z.enum(["shell", "file_path", "python", "code", "sql", "html", "text"]).optional().default("text").describe("Content type"),
    workspace: z.string().optional().describe("Workspace root (required for file_path validation)"),
  },
  async ({ content, type, workspace }) => {
    const result = validateOutput(content, type, workspace)
    return { content: [{ type: "text", text: JSON.stringify(result, null, 2) }] }
  },
)

// ---------- Tool 1: campaign_get_next_task ----------

server.tool(
  "campaign_get_next_task",
  { planFile: z.string().optional().describe("Ruta al plan file. Si se omite, busca el más reciente en docs/plans/") },
  async ({ planFile }) => {
    const worktree = PROJECT_ROOT
    const planPath = resolvePlan(planFile, worktree)
    if (!planPath) return { content: [{ type: "text", text: JSON.stringify({ error: "No plan file found in docs/plans/" }) }] }

    let content = readFileSync(planPath, "utf-8")
    const { campaignId, content: updatedContent } = getOrCreateCampaignId(content)
    if (updatedContent !== content) {
      writeFileSync(planPath, updatedContent, "utf-8")
      content = updatedContent
    }
    const tasks = parseTasks(content)
    const pending = tasks.filter(t => t.state === "⬜ PENDING" || t.state === "⏳ IN PROGRESS")
    const completed = tasks.filter(t => t.state === "✅ COMPLETED").length
    const failed = tasks.filter(t => t.state === "❌ FAILED").length
    const gates = countGateResults(content)
    const recitation = parseRecitation(content)
    const nextTask = pending.length > 0 ? pending[0] : null

    if (nextTask) {
      initTaskBudget(planPath, nextTask.id)
      traceEmit(campaignId, "task.started", { taskId: nextTask.id, taskName: nextTask.name, taskState: nextTask.state, taskType: nextTask.type || "unknown" }, worktree)
    } else {
      traceEmit(campaignId, "campaign.idle", { pending: pending.length, total: tasks.length }, worktree)
    }
    const budget = nextTask ? budgetStatus(nextTask.id, worktree) : null

    return {
      content: [{ type: "text", text: JSON.stringify({
        planFile: planPath,
        campaignId,
        hasTask: nextTask !== null,
        task: nextTask,
        summary: { completed, failed, pending: pending.length, total: tasks.length, doCount: gates.do, deferCount: gates.defer, skipCount: gates.skip, bloqueadoCount: gates.bloqueado },
        recitation,
        budget,
      }) }],
    }
  },
)

// ---------- Budget MCP tools ----------

server.tool(
  "campaign_budget_status",
  {
    taskId: z.string().describe("ID de tarea"),
    planFile: z.string().optional().describe("Ruta al plan file"),
  },
  async ({ taskId, planFile }) => {
    const worktree = PROJECT_ROOT
    const planPath = resolvePlan(planFile, worktree)
    if (!planPath) return { content: [{ type: "text", text: JSON.stringify({ error: "No plan file found" }) }] }
    const status = budgetStatus(taskId, worktree)
    return { content: [{ type: "text", text: JSON.stringify(status) }] }
  },
)

server.tool(
  "campaign_budget_consume",
  {
    taskId: z.string().describe("ID de tarea"),
    resource: z.enum(["tool_call", "sub_agent", "fail"]).describe("Recurso a consumir"),
    planFile: z.string().optional().describe("Ruta al plan file"),
  },
  async ({ taskId, resource, planFile }) => {
    const worktree = PROJECT_ROOT
    const planPath = resolvePlan(planFile, worktree)
    if (!planPath) return { content: [{ type: "text", text: JSON.stringify({ error: "No plan file found" }) }] }
    initTaskBudget(planPath, taskId)
    const state = readBudget(planPath)
    const t = state.tasks[taskId]
    if (resource === "tool_call") t.toolCalls++
    else if (resource === "sub_agent") t.subAgentCalls++
    else if (resource === "fail") t.consecutiveFails++
    t.lastActivity = Date.now()
    writeBudget(planPath, state)
    const elapsed = (t.lastActivity - t.startTime) / 60000
    const withinBudget = t.toolCalls <= BUDGET_LIMITS.maxToolCalls && elapsed <= BUDGET_LIMITS.maxDurationMinutes && t.consecutiveFails <= BUDGET_LIMITS.maxConsecutiveFails
    return { content: [{ type: "text", text: JSON.stringify({ consumed: resource, taskId, toolCalls: t.toolCalls, consecutiveFails: t.consecutiveFails, withinBudget, limits: BUDGET_LIMITS }) }] }
  },
)

server.tool(
  "campaign_budget_reset",
  {
    taskId: z.string().describe("ID de tarea a resetear"),
    planFile: z.string().optional().describe("Ruta al plan file"),
  },
  async ({ taskId, planFile }) => {
    const worktree = PROJECT_ROOT
    const planPath = resolvePlan(planFile, worktree)
    if (!planPath) return { content: [{ type: "text", text: JSON.stringify({ error: "No plan file found" }) }] }
    budgetReset(taskId, worktree)
    return { content: [{ type: "text", text: JSON.stringify({ reset: true, taskId }) }] }
  },
)

// ---------- Sandbox MCP tool (#4) ----------

server.tool(
  "campaign_run_sandboxed",
  {
    command: z.string().describe("Shell command to execute in sandbox"),
    stageFiles: z.array(z.string()).optional().describe("Paths to copy into sandbox before execution"),
    workDir: z.string().optional().describe("Relative working dir inside sandbox"),
    timeout: z.number().optional().default(60).describe("Max execution seconds"),
    blockNetwork: z.boolean().optional().default(true).describe("Block HTTP_PROXY/HTTPS_PROXY inside sandbox"),
  },
  async ({ command, stageFiles, workDir, timeout, blockNetwork }) => {
    const sandboxScript = join(TASK_SYSTEM, "sandbox", "run-sandboxed.ps1")
    if (!existsSync(sandboxScript)) return { content: [{ type: "text", text: JSON.stringify({ error: `Sandbox script not found at ${sandboxScript}` }) }] }

    const filesArg = stageFiles?.length ? `-StageFiles @(${stageFiles.map(f => `'${f.replace(/'/g, "''")}'`).join(", ")})` : ""
    const workArg = workDir ? `-WorkDir '${workDir}'` : ""
    const netArg = blockNetwork ? "-BlockNetwork" : ""
    const psCmd = `& '${sandboxScript}' -Command '${command.replace(/'/g, "''")}' ${filesArg} ${workArg} -TimeoutSeconds ${timeout} ${netArg} -NoCleanup`

    try {
      const out = execSync(psCmd, { encoding: "utf-8", timeout: (timeout + 30) * 1000, shell: "pwsh" })
      const result = JSON.parse(out.trim())
      const clean = { ...result }
      if (!result.error && result.exitCode === 0) {
        if (result.sandboxDir) { try { rmSync(result.sandboxDir, { recursive: true, force: true }) } catch {} }
        clean.sandboxDir = null
      }
      return { content: [{ type: "text", text: JSON.stringify(clean, null, 2) }] }
    } catch (e) {
      return { content: [{ type: "text", text: JSON.stringify({ valid: false, error: `Sandbox execution failed: ${e.message}`, elapsed: "0s" }) }] }
    }
  },
)

// ---------- Trace event tool (#5) ----------

server.tool(
  "campaign_emit_event",
  {
    event: z.string().describe("Event name (e.g. 'task.started', 'campaign.completed')"),
    campaignId: z.string().describe("Campaign ID"),
    data: z.record(z.any()).optional().default({}).describe("Arbitrary event payload"),
  },
  async ({ event, campaignId, data }) => {
    const entry = traceEmit(campaignId, event, data, PROJECT_ROOT)
    return { content: [{ type: "text", text: JSON.stringify({ emitted: true, event, campaignId }) }] }
  },
)

// ---------- Memory tools (#6) ----------

const MEMORY_DIR = join(TASK_SYSTEM, "memory")

server.tool(
  "campaign_memory_read",
  {
    file: z.enum(["lessons", "decisions"]).describe("Memory file to read"),
    limit: z.number().optional().default(20).describe("Max entries (lines) to return from end"),
  },
  async ({ file, limit }) => {
    const fp = join(MEMORY_DIR, `${file}.md`)
    if (!existsSync(fp)) return { content: [{ type: "text", text: JSON.stringify({ error: `Memory file ${file}.md not found` }) }] }
    const content = readFileSync(fp, "utf-8")
    const lines = content.split("\n")
    const tail = lines.slice(Math.max(0, lines.length - limit)).join("\n")
    return { content: [{ type: "text", text: tail }] }
  },
)

server.tool(
  "campaign_memory_write",
  {
    file: z.enum(["lessons", "decisions"]).describe("Memory file to append to"),
    entry: z.string().describe("Markdown entry line (one line preferred)"),
    campaignId: z.string().optional().describe("Campaign ID for trace event"),
  },
  async ({ file, entry, campaignId }) => {
    const fp = join(MEMORY_DIR, `${file}.md`)
    const date = new Date().toISOString().slice(0, 10)
    const line = `- ${date} | ${entry}\n`
    appendFileSync(fp, line, "utf-8")
    if (campaignId) traceEmit(campaignId, `memory.${file}.written`, { entry, date })
    return { content: [{ type: "text", text: JSON.stringify({ written: true, file, date, line: line.trim() }) }] }
  },
)

// ---------- Model traits tools (#9) ----------

server.tool(
  "campaign_model_traits",
  {
    model: z.string().optional().default("default").describe("Model name (e.g. deepseek-v4-flash-free, sonnet, haiku)"),
  },
  async ({ model }) => {
    const traits = getTraits(model)
    return { content: [{ type: "text", text: JSON.stringify({ model, traits }, null, 2) }] }
  },
)

server.tool(
  "campaign_model_list",
  {},
  async () => {
    const models = listModels()
    return { content: [{ type: "text", text: JSON.stringify({ models }, null, 2) }] }
  },
)

server.tool(
  "campaign_mom_escalate",
  {
    currentModel: z.string().optional().default("haiku").describe("Current model name"),
    retryCount: z.number().optional().default(0).describe("How many retries so far (0-based)"),
  },
  async ({ currentModel, retryCount }) => {
    const currentTier = retryCount > 0 ? Math.min(retryCount, 3) : tierForModel(currentModel)
    const next = escalateTier(currentTier)
    return {
      content: [{ type: "text", text: JSON.stringify({
        currentTier, currentModel,
        nextTier: next.tier, nextLabel: next.label,
        nextModels: next.models, nextCost: next.cost,
        tierConfig: TIERS,
      }, null, 2) }],
    }
  },
)

server.tool(
  "campaign_get_state_allowed_tools",
  { state: z.string().describe("C0 state name (PLAN, ACT, VERIFY, etc.)") },
  async ({ state }) => {
    const tools = getAllowedTools(state.toUpperCase())
    return { content: [{ type: "text", text: JSON.stringify(tools, null, 2) }] }
  },
)

server.tool(
  "campaign_validate_action",
  {
    state: z.string().describe("Current C0 state"),
    toolName: z.string().describe("Tool name to validate"),
  },
  async ({ state, toolName }) => {
    const result = validateAction(state.toUpperCase(), toolName)
    return { content: [{ type: "text", text: JSON.stringify(result, null, 2) }] }
  },
)

// ---------- Tool 2: campaign_update_task_state ----------

const STATE_MAP = {
  completed: "✅ COMPLETED",
  failed: "❌ FAILED",
  "in-progress": "⏳ EN PROGRESO",
  pending: "⬜ PENDING",
}

function findTaskById(content, taskId) {
  const pattern = new RegExp(`(### Task\\s*${taskId.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}[^\\n]*\\n[\\s\\S]*?)(?=\n### Task |\\n## |\\n---|\\n===|$)`)
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
    const rec = ["=== RECITATION ===", `Campaign ID: ${data.campaignId || ""}`, `Objetivo activo: ${data.activeGoal || ""}`, `Estado: ${data.status || "in-progress"}`, `Última acción: ${data.lastAction || ""}`, `Resultado: ${data.result || ""}`, `Próxima acción: ${data.nextAction || ""}`, `Contrato: ${data.contract || ""}`, `Próxima tarea si completa: ${data.nextTask || ""}`, "=== END RECITATION ==="].join("\n")
    return content.trimEnd() + "\n\n" + rec + "\n"
  }
  let updated = content
  const reps = [
    [/Campaign ID:\s*.*/, `Campaign ID: ${data.campaignId || ""}`],
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

server.tool(
  "campaign_update_task_state",
  {
    taskId: z.string().describe("ID de la tarea a actualizar (ej: '14', 'DRV-068')"),
    newState: z.enum(["completed", "failed", "in-progress", "pending"]).describe("Nuevo estado de la tarea"),
    planFile: z.string().optional().describe("Ruta al plan file. Si se omite, busca el más reciente."),
    recitation: z.object({
      activeGoal: z.string().optional().describe("Objetivo activo actual"),
      lastAction: z.string().optional().describe("Qué se hizo en esta iteración"),
      result: z.string().optional().describe("Resultado (✅ o ❌)"),
      nextAction: z.string().optional().describe("Próxima acción a tomar"),
      contract: z.string().optional().describe("Contrato de validación cumplido"),
      nextTask: z.string().optional().describe("ID de la próxima tarea a ejecutar"),
    }).optional().describe("Datos estructurados de recitation"),
  },
  async ({ taskId, newState, planFile, recitation }) => {
    const worktree = PROJECT_ROOT
    const planPath = resolvePlan(planFile, worktree)
    if (!planPath) return { content: [{ type: "text", text: JSON.stringify({ error: "No plan file found" }) }] }

    const original = readFileSync(planPath, "utf-8")
    const { campaignId } = getOrCreateCampaignId(original)
    let updated = updateState(original, taskId, newState)

    if (recitation) {
      updated = updateRecitation(updated, {
        campaignId,
        activeGoal: recitation.activeGoal,
        status: newState,
        lastAction: recitation.lastAction,
        result: recitation.result,
        nextAction: recitation.nextAction,
        contract: recitation.contract,
        nextTask: recitation.nextTask,
      })
    }

    if (updated === original) {
      return { content: [{ type: "text", text: JSON.stringify({ updated: false, warning: `Task ${taskId} not found or no changes needed`, campaignId }) }] }
    }

    writeFileSync(planPath, updated, "utf-8")
    traceEmit(campaignId, `task.${newState}`, { taskId, newState, taskType: "unknown" }, worktree)
    if (newState === "completed" || newState === "failed") {
      const tasks = parseTasks(updated)
      const task = tasks.find(t => t.id === taskId)
      const note = task ? `Task ${taskId} (${task.name}) → ${newState} | Contract: ${task.contract || "none"}` : `Task ${taskId} → ${newState}`
      const memFile = join(MEMORY_DIR, "lessons.md")
      const date = new Date().toISOString().slice(0, 10)
      try { appendFileSync(memFile, `- ${date} | ${taskId} | ${note}\n`, "utf-8") } catch {}
    }
    return { content: [{ type: "text", text: JSON.stringify({ updated: true, taskId, newState, campaignId, planFile: planPath }) }] }
  },
)

// ---------- Tool 3: campaign_verify_cmd (with output validation & budget) ----------

server.tool(
  "campaign_verify_cmd",
  {
    command: z.string().describe("Comando a ejecutar"),
    expectedExitCode: z.number().optional().default(0).describe("Exit code esperado (default: 0)"),
    timeout: z.number().optional().default(300).describe("Timeout en segundos (default: 300)"),
    taskId: z.string().optional().describe("ID de tarea asociada para budget tracking"),
  },
  async ({ command, expectedExitCode, timeout, taskId }) => {
    const validation = validateShellCommand(command)
    if (!validation.valid) {
      return { content: [{ type: "text", text: JSON.stringify({ error: "Command rejected by output validation", validation, executed: false }) }] }
    }
    const budgetCheck = taskId ? consumeBudget(taskId, PROJECT_ROOT) : null
    if (budgetCheck && !budgetCheck.withinBudget) {
      return { content: [{ type: "text", text: JSON.stringify({ error: "Budget exceeded", budget: budgetCheck, executed: false }) }] }
    }

    const startTime = Date.now()
    let stdout = "", stderr = "", exitCode = -1

    try {
      const out = execSync(command, { encoding: "utf-8", timeout: (timeout || 300) * 1000, windowsHide: true, maxBuffer: 10 * 1024 * 1024, shell: process.platform === "win32" ? "pwsh" : true })
      stdout = (out || "").trim()
      exitCode = 0
    } catch (e) {
      stdout = (e.stdout || "").trim()
      stderr = (e.stderr || "").trim()
      exitCode = e.status ?? -1
    }

    const elapsed = ((Date.now() - startTime) / 1000).toFixed(1)
    const passed = exitCode === expectedExitCode
    const nextestMatch = stdout.match(/(\d+)\s+passed.*?(\d+)\s+failed/s)
    const summary = nextestMatch ? { passed: parseInt(nextestMatch[1]), failed: parseInt(nextestMatch[2]) } : null

    return {
      content: [{ type: "text", text: JSON.stringify({
        passed, exitCode, expectedExitCode, elapsed: `${elapsed}s`, taskId: taskId || null, summary, budget: budgetCheck,
        stdout: stdout.length > 2000 ? stdout.slice(0, 2000) + `\n... [truncated, ${stdout.length} total chars]` : stdout,
        stderr: stderr.length > 1000 ? stderr.slice(0, 1000) + `\n... [truncated, ${stderr.length} total chars]` : stderr,
      }) }],
    }
  },
)

// ---------- Tool 4: campaign_detect_task_type ----------

const TYPE_PATTERNS = [
  { pattern: /src\//, type: "rust", label: "Rust core", skills: ["source-driven-development", "doubt-driven-development", "ponytail (full)"], checks: ["cargo check -p vantadb", "cargo fmt --check", "cargo clippy --workspace --all-targets --all-features -- -D warnings", "cargo nextest run --profile audit --workspace --build-jobs 2"] },
  { pattern: /vantadb-python\//, type: "python", label: "Python SDK", skills: ["source-driven-development"], checks: ["python -m pytest vantadb-python/tests/ -v"] },
  { pattern: /vantadb-ts\//, type: "typescript", label: "TypeScript SDK", skills: ["source-driven-development"], checks: ["npx tsc --noEmit", "npm test"] },
  { pattern: /web\/src\//, type: "frontend", label: "Web frontend", skills: ["frontend-ui-engineering", "design-taste-frontend"], checks: ["npx tsc --noEmit", "npm run lint"] },
  { pattern: /docs\//, type: "docs", label: "Documentation", skills: ["writing-guidelines", "writing-plans"], checks: ["scripts/validate-docs-coverage.ps1"] },
  { pattern: /\.github\//, type: "devops", label: "CI/CD / DevOps", skills: ["ci-cd-and-automation", "doubt-driven-development"], checks: ["yamllint .github/"] },
  { pattern: /vantadb-server\//, type: "server", label: "HTTP server", skills: ["source-driven-development", "security-and-hardening"], checks: ["cargo check -p vantadb-server"] },
]

const ESTIMATE_MAP = { "🟢": { turns: "5-10", label: "Bajo" }, "🟡": { turns: "15-30", label: "Medio" }, "🔴": { turns: "30-60", label: "Alto" } }

function detectType(archivosClave) {
  if (!archivosClave || archivosClave.trim() === "") return { type: "unknown", label: "No detectable", skills: [], checks: [], estimate: null }

  const matched = TYPE_PATTERNS.filter(tp => tp.pattern.test(archivosClave))
  if (matched.length === 0) return { type: "unknown", label: "No detectable", skills: ["campaign-executor"], checks: ["cargo check -p vantadb"], estimate: null }

  const tiposUnicos = [...new Set(matched.map(m => m.type))]
  if (tiposUnicos.length > 1) {
    return {
      type: "multi", label: `Múltiple (${tiposUnicos.join(", ")})`, typeList: tiposUnicos,
      skills: [...new Set(matched.flatMap(m => m.skills))],
      checks: matched.flatMap(m => m.checks),
      estimate: { turns: "15-45", label: "Medio-Alto" },
    }
  }

  const m = matched[0]
  const effortMatch = archivosClave.match(/[🟢🟡🔴]/)
  const estimate = effortMatch ? ESTIMATE_MAP[effortMatch[0]] : null

  return { type: m.type, label: m.label, skills: m.skills, checks: m.checks, estimate }
}

server.tool(
  "campaign_detect_task_type",
  {
    archivosClave: z.string().describe("Campo 'Archivos clave' del plan file (ej: 'src/index/flat.rs:32, src/engine.rs')"),
    effort: z.string().optional().describe("Indicador de esfuerzo opcional: 🟢 🟡 🔴"),
  },
  async ({ archivosClave, effort }) => {
    const result = detectType(archivosClave)
    if (effort && ESTIMATE_MAP[effort]) result.estimate = ESTIMATE_MAP[effort]
    return { content: [{ type: "text", text: JSON.stringify(result, null, 2) }] }
  },
)

// ---------- Tool 5: campaign_analyze_task ----------

server.tool(
  "campaign_analyze_task",
  {
    taskId: z.string().describe("ID de la tarea a analizar (ej: '14', 'DRV-068')"),
    planFile: z.string().optional().describe("Ruta al plan file. Si se omite, busca el más reciente."),
  },
  async ({ taskId, planFile }) => {
    const worktree = PROJECT_ROOT
    const planPath = resolvePlan(planFile, worktree)
    if (!planPath) return { content: [{ type: "text", text: JSON.stringify({ error: "No plan file found" }) }] }

    const content = readFileSync(planPath, "utf-8")
    const tasks = parseTasks(content)
    const task = tasks.find(t => t.id === taskId)
    if (!task) return { content: [{ type: "text", text: JSON.stringify({ error: `Task ${taskId} not found in plan` }) }] }

    const typeInfo = detectType(task.files)
    return {
      content: [{ type: "text", text: JSON.stringify({
        taskId: task.id, name: task.name, state: task.state, contract: task.contract, files: task.files,
        type: typeInfo, priority: task.priority, effort: task.effort, source: task.source, notes: task.notes,
      }, null, 2) }],
    }
  },
)

// ---------- Tool 6: campaign_load_skills ----------

server.tool(
  "campaign_load_skills",
  {
    archivosClave: z.string().describe("Campo 'Archivos clave' del plan file"),
    extraSkills: z.array(z.string()).optional().describe("Skills adicionales a incluir (ej: ['systematic-debugging', 'test-driven-development'])"),
  },
  async ({ archivosClave, extraSkills }) => {
    const typeInfo = detectType(archivosClave)
    const skills = [...new Set([...(typeInfo.skills || []), "campaign-executor", "progreso", "ponytail (full)", ...(extraSkills || [])])]
    const sortOrder = ["campaign-executor", "progreso", "ponytail (full)"]
    const sorted = [...sortOrder.filter(s => skills.includes(s)), ...skills.filter(s => !sortOrder.includes(s))]
    const commands = sorted.map(s => `skill ${s}`)

    return {
      content: [{ type: "text", text: JSON.stringify({
        type: typeInfo.type, label: typeInfo.label,
        skills: sorted, commands,
        checks: typeInfo.checks || [],
        estimate: typeInfo.estimate,
      }, null, 2) }],
    }
  },
)

// ---------- Tool 7: campaign_get_task_detail ----------

server.tool(
  "campaign_get_task_detail",
  {
    taskId: z.string().describe("ID de la tarea (ej: '14', 'DRV-068')"),
    planFile: z.string().optional().describe("Ruta al plan file. Si se omite, busca el más reciente."),
  },
  async ({ taskId, planFile }) => {
    const worktree = PROJECT_ROOT
    const planPath = resolvePlan(planFile, worktree)
    if (!planPath) return { content: [{ type: "text", text: JSON.stringify({ error: "No plan file found" }) }] }

    const content = readFileSync(planPath, "utf-8")
    const pattern = new RegExp(`(### Task\\s*${taskId.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}[^\\n]*\\n[\\s\\S]*?)(?=\n### Task |\\n## |\\n---|\\n===|$)`)
    const m = content.match(pattern)
    if (!m) return { content: [{ type: "text", text: JSON.stringify({ error: `Task ${taskId} block not found` }) }] }

    return { content: [{ type: "text", text: m[0].trim() }] }
  },
)

// ---------- Tool 8: campaign_stalled_tasks ----------

server.tool(
  "campaign_stalled_tasks",
  {
    planFile: z.string().optional().describe("Ruta al plan file. Si se omite, busca el más reciente."),
    threshold: z.number().optional().default(30).describe("Minutos sin actividad para considerar stalled (default: 30)"),
  },
  async ({ planFile, threshold }) => {
    const worktree = PROJECT_ROOT
    const planPath = resolvePlan(planFile, worktree)
    if (!planPath) return { content: [{ type: "text", text: JSON.stringify({ error: "No plan file found" }) }] }

    const content = readFileSync(planPath, "utf-8")
    const tasks = parseTasks(content)
    const recitation = parseRecitation(content)
    const budget = readBudget(planPath)

    const now = Date.now()
    const stalled = tasks.filter(t => {
      if (t.state !== "⏳ IN PROGRESS") return false
      const entry = budget.tasks[t.id]
      if (!entry) return true
      return (now - entry.lastActivity) / 60000 > threshold
    })
    const pendingCount = tasks.filter(t => t.state === "⬜ PENDING").length
    const recitationStalled = recitation && recitation.status === "stalled"

    return {
      content: [{ type: "text", text: JSON.stringify({
        stalledCount: stalled.length,
        pendingCount,
        inProgressCount: tasks.filter(t => t.state === "⏳ IN PROGRESS").length,
        stalledTasks: stalled.map(t => {
          const entry = budget.tasks[t.id]
          const idleMinutes = entry ? Math.round((now - entry.lastActivity) / 60000) : null
          return { id: t.id, name: t.name, files: t.files, idleMinutes }
        }),
        recitationStalled,
        recitationState: recitation ? recitation.status : null,
        recitationAction: recitation ? recitation.nextAction : null,
      }, null, 2) }],
    }
  },
)

// ---------- Tool 9: campaign_health_status ----------

server.tool(
  "campaign_health_status",
  {},
  async () => {
    const health = getHealth(PROJECT_ROOT)
    return { content: [{ type: "text", text: JSON.stringify(health, null, 2) }] }
  },
)

const ACTIVE_MODEL_FILE = join(PROJECT_ROOT, "traces", "active-model.json")

function readActiveModel() {
  try { return JSON.parse(readFileSync(ACTIVE_MODEL_FILE, "utf-8")) } catch { return { model: "default" } }
}

function writeActiveModel(data) {
  const dir = join(PROJECT_ROOT, "traces")
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true })
  writeFileSync(ACTIVE_MODEL_FILE, JSON.stringify(data, null, 2), "utf-8")
}

const WORKFLOWS_DIR = join(TASK_SYSTEM, "workflows")

const WORKFLOW_KEYWORDS = {
  "bug-fix": ["bug", "fix", "error", "crash", "panic", "incorrect", "wrong", "fails", "broken"],
  "feature-add": ["feature", "add", "implement", "new", "create", "support", "integrate"],
  "refactor": ["refactor", "clean", "simplify", "rename", "extract", "inline", "split", "restructure"],
  "research": ["research", "investigate", "explore", "how does", "find", "search", "learn"],
}

function classifyWorkflow(taskName, taskDescription) {
  const text = `${taskName || ""} ${taskDescription || ""}`.toLowerCase()
  let best = null
  let bestScore = 0
  for (const [wf, keywords] of Object.entries(WORKFLOW_KEYWORDS)) {
    const score = keywords.filter(k => text.includes(k)).length
    if (score > bestScore) { bestScore = score; best = wf }
  }
  return best
}

function loadWorkflow(wfName) {
  const path = join(WORKFLOWS_DIR, `${wfName}.json`)
  try { return JSON.parse(readFileSync(path, "utf-8")) } catch { return null }
}

// ---------- Tool 10: campaign_set_model ----------

server.tool(
  "campaign_set_model",
  {
    model: z.string().describe("Model name to switch to (deepseek-v4-flash-free, sonnet, haiku, gpt-4o, deepseek-v4)"),
  },
  async ({ model }) => {
    const validModels = listModels()
    if (!validModels.includes(model)) {
      return { content: [{ type: "text", text: JSON.stringify({
        switched: false, error: `Unknown model '${model}'. Valid models: ${validModels.join(", ")}`,
      }, null, 2) }] }
    }
    const traits = getTraits(model)
    writeActiveModel({ model, traits, switchedAt: new Date().toISOString() })
    const envHint = model.includes("deepseek") ? "ANTHROPIC_BASE_URL, ANTHROPIC_AUTH_TOKEN" : model === "sonnet" ? "ANTHROPIC_AUTH_TOKEN (default)" : "custom provider vars"
    return {
      content: [{ type: "text", text: JSON.stringify({
        switched: true,
        activeModel: model,
        traits,
        envVars: envHint,
        note: `Model switched to ${model}. If using OpenCode, set model via /model ${model}. If using deepclaude, set ${envHint}.`,
      }, null, 2) }],
    }
  },
)

// ---------- Tool 11: campaign_get_active_model ----------

server.tool(
  "campaign_get_active_model",
  {},
  async () => {
    const state = readActiveModel()
    const traits = getTraits(state.model)
    return { content: [{ type: "text", text: JSON.stringify({ ...state, traits }, null, 2) }] }
  },
)

// ---------- Tool 12: campaign_classify_workflow ----------

server.tool(
  "campaign_classify_workflow",
  {
    taskName: z.string().describe("Task name from plan file"),
    taskDescription: z.string().optional().default("").describe("Optional description for better classification"),
  },
  async ({ taskName, taskDescription }) => {
    const wfName = classifyWorkflow(taskName, taskDescription)
    const workflow = loadWorkflow(wfName)
    const available = Object.keys(WORKFLOW_KEYWORDS).filter(w => loadWorkflow(w))
    return {
      content: [{ type: "text", text: JSON.stringify({
        workflow: wfName,
        states: workflow ? Object.keys(workflow.definition.states) : [],
        initial: workflow ? workflow.definition.initial : null,
        availableTemplates: available,
        hasCustomWorkflow: workflow !== null,
        fallback: !workflow ? "Use generic C0 state machine from iter.md" : undefined,
      }, null, 2) }],
    }
  },
)

// ---------- start ----------

const transport = new StdioServerTransport()
await server.connect(transport)
