import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js"
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js"
import { z } from "zod"
import { readFileSync, writeFileSync, existsSync, readdirSync, statSync, rmSync, appendFileSync, mkdirSync, openSync, writeSync, closeSync, renameSync } from "node:fs"
import { fileURLToPath } from "url"
import { resolve, join, dirname, basename } from "node:path"
import { execSync } from "node:child_process"
import { randomUUID, createHash } from "node:crypto"
import { emit as traceEmit, getHealth } from "../traces/tracer.mjs"
import { getTraits, listModels, escalateTier, tierForModel, TIERS } from "../config/model-traits.mjs"
import { STATE_TOOLS, getAllowedTools, validateAction } from "../config/state-tools.mjs"
import { parseTasks, parseRecitation, getOrCreateCampaignId, extractCampaignId, extractAutonomous, updateState, updateRecitation } from "./parsers.mjs"

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
    .map(f => ({ name: join(planDir, f), time: statSync(join(planDir, f)).mtimeMs }))
    .sort((a, b) => b.time - a.time)
  if (files.length === 0) return null
  // Guard multi-plan (MEM-51): con >1 plan modificado <24h, el fallback por mtime
  // cruza budget/recitations entre planes. Fallar LOUD pidiendo ruta explícita.
  const ACTIVE_MS = 24 * 60 * 60 * 1000
  const recent = files.filter(f => Date.now() - f.time < ACTIVE_MS)
  if (recent.length > 1) {
    const names = recent.map(f => basename(f.name)).join(", ")
    throw new Error(`Ambiguous active plan: ${recent.length} planes modificados en las últimas 24h (${names}). Pasá planFile explícito.`)
  }
  return files[0].name
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

function countGateResults(content) {
  return {
    do: (content.match(/✅ DO/g) || []).length,
    defer: (content.match(/🟡 DEFER/g) || []).length,
    skip: (content.match(/❌ SKIP/g) || []).length,
    bloqueado: (content.match(/🔴 BLOQUEADO/g) || []).length,
  }
}

// ---------- P2-04: WIP hard-limit ----------

// Escanea tareas activas (in-progress): plan files en docs/plans/ (task blocks
// con `- **Estado:**` = IN PROGRESS/in-progress) + task files en tasks/.
// TTL: un IN PROGRESS sin actividad > staleMinutes (budget.lastActivity para
// tareas de plan; mtime para task files) NO bloquea claims nuevos — se devuelve
// como propiedad `stale` del array retornado (crash residual no deadlocka el pipeline).
function findInProgressTasks(worktree, staleMinutes = 1440) {
  const active = []
  const stale = []
  const opencodeRoot = resolve(worktree, ".opencode")
  const isStale = lastActivityMs => !lastActivityMs || (Date.now() - lastActivityMs) / 60000 > staleMinutes

  // 1) Plan files en docs/plans/.
  const planDir = join(worktree, "docs", "plans")
  if (existsSync(planDir)) {
    for (const f of readdirSync(planDir).filter(f => f.endsWith(".md"))) {
      try {
        const fp = join(planDir, f)
        const content = readFileSync(fp, "utf-8")
        const bp = fp.replace(/\.md$/, ".budget.json")
        let budgetTasks = null
        try { budgetTasks = JSON.parse(readFileSync(bp, "utf-8")).tasks } catch {}
        for (const t of parseTasks(content)) {
          if (t.state !== "⏳ IN PROGRESS") continue
          const entry = { id: t.id, name: t.name, state: "in-progress", source: `docs/plans/${f}` }
          if (isStale(budgetTasks?.[t.id]?.lastActivity ?? statSync(fp).mtimeMs)) stale.push(entry)
          else active.push(entry)
        }
      } catch {}
    }
  }

  // 2) Task files (raíz + complete/ + closed/).
  const tasksDir = join(opencodeRoot, "skills", "campaign-executor", "tasks")
  for (const sub of ["", "complete", "closed"]) {
    const dir = sub ? join(tasksDir, sub) : tasksDir
    if (!existsSync(dir)) continue
    for (const f of readdirSync(dir).filter(f => f.endsWith(".md"))) {
      try {
        const fp = join(dir, f)
        const content = readFileSync(fp, "utf-8")
        if (/-\s*\*\*Estado:\*\*\s*(IN PROGRESS|in-progress|⏳)/i.test(content)) {
          const entry = { id: f.replace(/\.md$/, ""), name: f, state: "in-progress", source: sub ? `tasks/${sub}/${f}` : `tasks/${f}` }
          if (isStale(statSync(fp).mtimeMs)) stale.push(entry)
          else active.push(entry)
        }
      } catch {}
    }
  }

  active.stale = stale
  return active
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

// TSYS-06 §6.1 (C3/C7): corrupción visible — ante JSON inválido/truncado, NUNCA
// reset silencioso: se devuelve estado vacío con `budgetCorrupted: true` para que
// las tools lo surfacen. writeBudget strip del flag: la corrupción es transitoria,
// nunca se persiste como dato.
function readBudget(planPath) {
  const bp = planPath.replace(/\.md$/, ".budget.json")
  try { return JSON.parse(readFileSync(bp, "utf-8")) }
  catch (e) {
    // ENOENT = plan sin budget todavía (primer uso) → vacío limpio, NO es corrupción.
    if (e.code === "ENOENT") return { tasks: {} }
    // Cualquier otro error (JSON truncado/inválido) = corrupción visible, nunca silenciosa.
    return { tasks: {}, budgetCorrupted: true }
  }
}

function writeBudget(planPath, state) {
  const clean = { ...state }
  delete clean.budgetCorrupted
  writeFileSync(planPath.replace(/\.md$/, ".budget.json"), JSON.stringify(clean, null, 2), "utf-8")
}

function initTaskBudgetUnlocked(planPath, taskId) {
  const state = readBudget(planPath)
  if (!state.tasks[taskId]) {
    state.tasks[taskId] = { taskId, toolCalls: 0, subAgentCalls: 0, consecutiveFails: 0, startTime: Date.now(), lastActivity: Date.now() }
  }
  state.tasks[taskId].lastActivity = Date.now()
  writeBudget(planPath, state)
  return state.tasks[taskId]
}

// RMW del budget SIEMPRE bajo plan lock: sin esto, dos instancias en paralelo
// pierden incrementos (last-writer-wins) y los hard limits dejan de ser límites.
// Las variantes *_Unlocked son SOLO para callers que YA sostienen el lock.
function initTaskBudget(planPath, taskId) {
  return withPlanLock(planPath, () => initTaskBudgetUnlocked(planPath, taskId))
}

function consumeBudget(taskId, worktree) {
  const planPath = findPlanFile(worktree)
  if (!planPath) return null
  return withPlanLock(planPath, () => {
    let state = readBudget(planPath)
    if (!state.tasks[taskId]) {
      initTaskBudgetUnlocked(planPath, taskId)
      state = readBudget(planPath) // re-read: initTaskBudget escribe un budget fresco (recupera corrupción)
    }
    const t = state.tasks[taskId]
    t.toolCalls++
    t.lastActivity = Date.now()
    const elapsed = (t.lastActivity - t.startTime) / 60000
    const withinBudget = t.toolCalls <= BUDGET_LIMITS.maxToolCalls && elapsed <= BUDGET_LIMITS.maxDurationMinutes && t.consecutiveFails <= BUDGET_LIMITS.maxConsecutiveFails
    writeBudget(planPath, state)
    return { withinBudget, toolCalls: t.toolCalls, consecutiveFails: t.consecutiveFails, elapsedMinutes: Math.round(elapsed), limits: BUDGET_LIMITS, budgetCorrupted: !!state.budgetCorrupted }
  })
}

function budgetStatus(taskId, worktree) {
  const planPath = findPlanFile(worktree)
  if (!planPath) return null
  const state = readBudget(planPath)
  const budgetCorrupted = !!state.budgetCorrupted
  const t = state.tasks[taskId]
  if (!t) return { exists: false, budgetCorrupted }
  const elapsed = (Date.now() - t.startTime) / 60000
  return {
    exists: true, taskId, toolCalls: t.toolCalls, subAgentCalls: t.subAgentCalls,
    consecutiveFails: t.consecutiveFails, elapsedMinutes: Math.round(elapsed),
    withinBudget: t.toolCalls <= BUDGET_LIMITS.maxToolCalls && elapsed <= BUDGET_LIMITS.maxDurationMinutes && t.consecutiveFails <= BUDGET_LIMITS.maxConsecutiveFails,
    limits: BUDGET_LIMITS, budgetCorrupted,
  }
}

function budgetReset(taskId, worktree) {
  const planPath = findPlanFile(worktree)
  if (!planPath) return null
  return withPlanLock(planPath, () => {
    const state = readBudget(planPath)
    delete state.tasks[taskId]
    writeBudget(planPath, state)
    return { reset: true }
  })
}

// ---------- P2-05: Trace ID por tarea ----------
//
// Persistence choice: el traceId se persiste en el budget JSON del plan
// (`<plan>.budget.json` → `tasks[<taskId>].traceId`), que ya se escribe/lee de
// forma síncrona en cada tool. Sobrevive reinicios del server (el mapa
// en-memoria es solo cache). Adicionalmente viaja en cada registro del
// verify-log.jsonl y en las sesiones de session-tracking.ps1.

const traceIdByTask = new Map()

function getOrCreateTraceId(planPath, taskId) {
  // Caller debe sostener el plan lock (hoy: updateTaskStateCore) — usa la variante Unlocked.
  initTaskBudgetUnlocked(planPath, taskId)
  const state = readBudget(planPath)
  const t = state.tasks[taskId]
  if (!t.traceId) {
    t.traceId = randomUUID()
    writeBudget(planPath, state)
  }
  return t.traceId
}

function readTraceId(planPath, taskId) {
  try {
    const t = readBudget(planPath).tasks[taskId]
    return (t && t.traceId) || null
  } catch { return null }
}

function traceIdForTask(taskId) {
  if (traceIdByTask.has(taskId)) return traceIdByTask.get(taskId)
  try {
    const planPath = findPlanFile(PROJECT_ROOT)
    if (planPath) return readTraceId(planPath, taskId)
  } catch {}
  return null
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
  {
    planFile: z.string().optional().describe("Ruta al plan file. Si se omite, busca el más reciente en docs/plans/"),
    claim: z.boolean().optional().default(false).describe("Claim atómico: marca la tarea devuelta IN PROGRESS bajo lock (evita que 2 instancias hagan Discovery de la misma)"),
  },
  async ({ planFile, claim }) => {
    const worktree = PROJECT_ROOT
    const planPath = resolvePlan(planFile, worktree)
    if (!planPath) return { content: [{ type: "text", text: JSON.stringify({ error: "No plan file found in docs/plans/" }) }] }

    // Campaign ID write-on-read atómico: sin lock, dos instancias pueden pisar
    // el write y desincronizar el mtime que findPlanFile usa como "más reciente".
    let content = readFileSync(planPath, "utf-8")
    const locked = withPlanLock(planPath, () => {
      const { campaignId, content: updatedContent } = getOrCreateCampaignId(content)
      if (updatedContent === content) return { content }
      writeFileSync(planPath, updatedContent, "utf-8")
      return { content: updatedContent, campaignId }
    })
    content = locked.content
    const campaignId = locked.campaignId ?? extractCampaignId(content)
    const tasks = parseTasks(content)
    const pending = tasks.filter(t => t.state === "⬜ PENDING" || t.state === "⏳ IN PROGRESS")
    const completed = tasks.filter(t => t.state === "✅ COMPLETED").length
    const failed = tasks.filter(t => t.state === "❌ FAILED").length
    const gates = countGateResults(content)
    let nextTask = pending.length > 0 ? pending[0] : null

    // R2: claim atómico — marca IN PROGRESS dentro del flujo para que una segunda
    // instancia no arranque Discovery sobre la misma tarea.
    let claimResult = null
    if (nextTask && claim && nextTask.state === "⬜ PENDING") {
      try { claimResult = updateTaskStateCore(planPath, nextTask.id, "in-progress", null, worktree) } catch (e) { claimResult = { updated: false, error: e.message } }
      if (!claimResult.updated) {
        return { content: [{ type: "text", text: JSON.stringify({
          planFile: planPath, campaignId, hasTask: false, task: null,
          claimBlocked: claimResult,
          summary: { completed, failed, pending: pending.length, total: tasks.length },
        }) }] }
      }
      nextTask = { ...nextTask, state: "⏳ IN PROGRESS" }
    }

    const recitation = parseRecitation(content, nextTask?.id ?? null)

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
        autonomous: extractAutonomous(content),
        hasTask: nextTask !== null,
        task: nextTask,
        claimed: claimResult ? claimResult.updated : false,
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

// Consumo de un recurso de budget atómico bajo plan lock (antes: RMW sin lock,
// perdía incrementos con 2+ instancias en paralelo).
function bumpBudget(planPath, taskId, resource) {
  return withPlanLock(planPath, () => {
    // TSYS-06 C3/C7: capturar corrupción ANTES de que initTaskBudget la recupere silenciosamente.
    const budgetCorrupted = !!readBudget(planPath).budgetCorrupted
    initTaskBudgetUnlocked(planPath, taskId)
    const state = readBudget(planPath)
    const t = state.tasks[taskId]
    if (resource === "tool_call") t.toolCalls++
    else if (resource === "sub_agent") t.subAgentCalls++
    else if (resource === "fail") t.consecutiveFails++
    t.lastActivity = Date.now()
    writeBudget(planPath, state)
    const elapsed = (t.lastActivity - t.startTime) / 60000
    const withinBudget = t.toolCalls <= BUDGET_LIMITS.maxToolCalls && elapsed <= BUDGET_LIMITS.maxDurationMinutes && t.consecutiveFails <= BUDGET_LIMITS.maxConsecutiveFails
    return { consumed: resource, taskId, toolCalls: t.toolCalls, subAgentCalls: t.subAgentCalls, consecutiveFails: t.consecutiveFails, withinBudget, limits: BUDGET_LIMITS, budgetCorrupted }
  })
}

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
    let result
    try {
      result = bumpBudget(planPath, taskId, resource)
    } catch (e) {
      return { content: [{ type: "text", text: JSON.stringify({ error: `Plan lock failed: ${e.message}` }) }] }
    }
    return { content: [{ type: "text", text: JSON.stringify(result) }] }
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
    event: z.string().describe("Event name (e.g. 'task.started', 'campaign.completed', 'plan.adjust')"),
    campaignId: z.string().describe("Campaign ID"),
    data: z.record(z.any()).optional().default({}).describe("Arbitrary event payload"),
    taskId: z.string().optional().describe("ID de tarea para adjuntar su traceId al evento (P2-05)"),
    decision_reason: z.string().optional().describe("TSYS-09: motivo de la decisión (por qué se reabrió/cerró una tarea) — se persiste en el trace log"),
    pattern: z.string().optional().describe("TSYS-09: patrón detectado (ej: 'retry', 'reopen', 'scope-creep') — se persiste en el trace log"),
  },
  async ({ event, campaignId, data, taskId, decision_reason, pattern }) => {
    let traceId = null
    if (taskId) {
      traceId = traceIdForTask(taskId)
      if (traceId) data = { ...data, traceId }
    }
    // TSYS-09: tracing de decisiones — campos opcionales, no rompen el esquema del log.
    if (decision_reason) data = { ...data, decision_reason }
    if (pattern) data = { ...data, pattern }
    const entry = traceEmit(campaignId, event, data, PROJECT_ROOT)
    return { content: [{ type: "text", text: JSON.stringify({ emitted: true, event, campaignId, traceId: traceId || entry.traceId || null, decision_reason: decision_reason || null, pattern: pattern || null }) }] }
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
    // P3-04: reminder suave — decisiones técnicas inciertas se validan contra web/docs oficiales primero.
    const reminder = file === "decisions"
      ? "Recordatorio: si la decisión es técnica y hay incertidumbre, validala primero con websearch/webfetch contra documentación oficial o GitHub (Regla de Validación de AGENTS.md)."
      : undefined
    return { content: [{ type: "text", text: JSON.stringify({ written: true, file, date, line: line.trim(), ...(reminder ? { reminder } : {}) }) }] }
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
//
// TSYS-06 §6.2/§6.3 (C4/C6/C12): writes con checksum + atómicos y WIP check-and-set
// atómico. updateTaskStateCore corre TODA la lógica bajo withPlanLock (lock file
// exclusivo por plan): el scan WIP, el read→update y el write quedan serializados
// contra otros writers del MISMO plan file. Checksum sha1 del contenido original +
// re-read antes del write → si otro proceso cambió el archivo, el perdedor recibe
// `conflict:true`/`updated:false` con el estado ganador en el payload (C4).

function sha1(content) {
  return createHash("sha1").update(content).digest("hex")
}

function sleepSync(ms) {
  const sab = new Int32Array(new SharedArrayBuffer(4))
  Atomics.wait(sab, 0, 0, ms)
}

// ponytail: lock file exclusivo (O_EXCL) con retry corto y stale-detection por mtime.
// techo: serializa writers del MISMO archivo; la carrera cross-plan del claim WIP
// queda como spec del runner diferido (TSYS-06 Fase 4). Wait cap ~1s (antes 5s que
// congelaba el server entero): al agotarse, falla FAST con pid/ts del holder —
// diagnosticable con campaign_plan_lock_info en vez de debugging a ciegas.
function readLockHolder(lockPath) {
  try { return JSON.parse(readFileSync(lockPath, "utf-8")) } catch { return null }
}

function lockFileSync(lockPath, fn, maxWaitMs = 1000) {
  const deadline = Date.now() + maxWaitMs
  for (;;) {
    let fd = null
    try {
      fd = openSync(lockPath, "wx")
      writeSync(fd, JSON.stringify({ pid: process.pid, ts: Date.now() }))
      closeSync(fd)
      try {
        return fn()
      } finally {
        try { rmSync(lockPath, { force: true }) } catch {}
      }
    } catch (e) {
      if (e.code === "EEXIST") {
        // Lock stale de un crash previo (>10s sin liberar) → romperlo.
        try {
          const st = statSync(lockPath)
          if (Date.now() - st.mtimeMs > 10000) { rmSync(lockPath, { force: true }); continue }
        } catch {}
        if (Date.now() > deadline) {
          const holder = readLockHolder(lockPath)
          throw new Error(`Lock busy: ${lockPath} held by pid ${holder?.pid ?? "?"} since ${holder ? new Date(holder.ts).toISOString() : "?"} (waited ${maxWaitMs}ms). Reintentá o inspeccioná con campaign_plan_lock_info.`)
        }
        sleepSync(25)
        continue
      }
      throw e
    }
  }
}

function withPlanLock(planPath, fn) {
  return lockFileSync(`${planPath}.lock`, fn)
}

// Windows: AV/indexer puede sostener un handle sobre el plan → EPERM/EACCES
// transitorio en el rename. Retry corto en vez de fallar toda la actualización.
function renameWithRetry(from, to, attempts = 3) {
  for (let i = 0; ; i++) {
    try { renameSync(from, to); return } catch (e) {
      if ((e.code === "EPERM" || e.code === "EACCES") && i < attempts - 1) { sleepSync(50); continue }
      throw e
    }
  }
}

function detectConflict(currentContent, originalChecksum) {
  return sha1(currentContent) !== originalChecksum
}

// Core exportable (tests TSYS-06): read→regex-update→checksum-check→temp+rename bajo lock.
// Devuelve el payload del resultado; el handler agrega tracing/lessons post-write.
export function updateTaskStateCore(planPath, taskId, newState, recitationData, worktree) {
  return withPlanLock(planPath, () => {
    // P2-04: WIP hard-limit — convención one-task-at-a-time. El scan corre DENTRO del
    // lock (check-and-set atómico, C12): ningún claim puede pasar entre el scan y el write.
    if (newState === "in-progress") {
      const active = findInProgressTasks(worktree).filter(t => t.id !== taskId)
      if (active.length > 0) {
        const list = active.map(a => `${a.id} (${a.name}) en ${a.source}`).join(", ")
        const staleList = (active.stale || []).map(a => `${a.id} en ${a.source}`)
        return {
          updated: false,
          error: `No se puede iniciar la tarea ${taskId}: ya hay otra tarea en progreso (${list}). Convención one-task-at-a-time: completala o cerrála antes de arrancar otra.`,
          activeTasks: active,
          staleNotBlocking: staleList,
          wipBlocked: true,
        }
      }
    }

    const original = readFileSync(planPath, "utf-8")
    const originalChecksum = sha1(original)
    // Usar el contenido CON el Campaign ID insertado (antes se descartaba y
    // cada write generaba un UUID nuevo sin persistirlo).
    const { campaignId, content: withCampaignId } = getOrCreateCampaignId(original)
    // TSYS-09: capturar el estado previo para trazar el cambio (por qué se reabrió/cerró).
    const fromState = (() => {
      try { const t = parseTasks(original).find(t => t.id === taskId); return t ? t.state : null } catch { return null }
    })()

    let updated = updateState(withCampaignId, taskId, newState)
    if (recitationData) updated = updateRecitation(updated, { campaignId, ...recitationData })

    if (updated === original) {
      return { updated: false, warning: `Task ${taskId} not found or no changes needed`, campaignId }
    }

    // TSYS-06 §6.2 (C4): checksum check-and-set — si otro writer modificó el archivo
    // entre nuestro read y este punto, el update pierde y el perdedor ve el estado ganador.
    const current = readFileSync(planPath, "utf-8")
    if (detectConflict(current, originalChecksum)) {
      const currentTask = (() => { try { return parseTasks(current).find(t => t.id === taskId) || null } catch { return null } })()
      return {
        updated: false,
        conflict: true,
        warning: `Concurrent modification detected: plan file changed while updating task ${taskId}.`,
        taskId, newState, campaignId,
        currentState: currentTask ? currentTask.state : null,
      }
    }

    // P2-05: traceId por tarea — se genera/persiste al entrar en in-progress.
    let traceId = null
    if (newState === "in-progress") {
      traceId = getOrCreateTraceId(planPath, taskId)
      traceIdByTask.set(taskId, traceId)
    } else {
      traceId = traceIdForTask(taskId)
    }

    // TSYS-06 §6.2 (C6): write atómico — temp + rename con retry EPERM; un kill
    // entre write y rename deja el archivo original intacto (o el update completo).
    const tmp = `${planPath}.tmp`
    writeFileSync(tmp, updated, "utf-8")
    renameWithRetry(tmp, planPath)

    return { updated: true, taskId, newState, campaignId, planFile: planPath, traceId, fromState }
  })
}

server.tool(
  "campaign_update_task_state",
  {
    taskId: z.string().describe("ID de la tarea a actualizar (ej: '14', 'DRV-068')"),
    newState: z.enum(["completed", "failed", "in-progress", "pending"]).describe("Nuevo estado de la tarea"),
    planFile: z.string().optional().describe("Ruta al plan file. Si se omite, busca el más reciente."),
    decision_reason: z.string().optional().describe("TSYS-09: motivo del cambio de estado (por qué se reabrió/cerró) — se persiste en el trace log via plan.adjust"),
    pattern: z.string().optional().describe("TSYS-09: patrón detectado (ej: 'retry', 'reopen') — se persiste en el trace log via plan.adjust"),
    recitation: z.object({
      activeGoal: z.string().optional().describe("Objetivo activo actual"),
      lastAction: z.string().optional().describe("Qué se hizo en esta iteración"),
      result: z.string().optional().describe("Resultado (✅ o ❌)"),
      nextAction: z.string().optional().describe("Próxima acción a tomar"),
      contract: z.string().optional().describe("Contrato de validación cumplido"),
      nextTask: z.string().optional().describe("ID de la próxima tarea a ejecutar"),
    }).optional().describe("Datos estructurados de recitation"),
  },
  async ({ taskId, newState, planFile, decision_reason, pattern, recitation }) => {
    const worktree = PROJECT_ROOT
    const planPath = resolvePlan(planFile, worktree)
    if (!planPath) return { content: [{ type: "text", text: JSON.stringify({ error: "No plan file found" }) }] }

    const recitationData = recitation ? {
      taskId,
      activeGoal: recitation.activeGoal,
      status: newState,
      lastAction: recitation.lastAction,
      result: recitation.result,
      nextAction: recitation.nextAction,
      contract: recitation.contract,
      nextTask: recitation.nextTask,
    } : null

    let result
    try {
      result = updateTaskStateCore(planPath, taskId, newState, recitationData, worktree)
    } catch (e) {
      return { content: [{ type: "text", text: JSON.stringify({ updated: false, error: `Plan lock failed: ${e.message}` }) }] }
    }

    // Post-write side effects (best-effort; kill entre write y trace conserva el estado — §3.5).
    if (result.updated) {
      traceEmit(result.campaignId, `task.${newState}`, { taskId, newState, taskType: "unknown", traceId: result.traceId }, worktree)
      // TSYS-09: evento plan.adjust — registra cuándo un plan/tarea cambia de estado con el motivo.
      // Campos opcionales: no rompe el esquema del log (solo añade claves cuando existen).
      traceEmit(result.campaignId, "plan.adjust", {
        taskId, fromState: result.fromState, newState,
        decision_reason: decision_reason || null,
        pattern: pattern || null,
        traceId: result.traceId,
      }, worktree)
      if (newState === "completed" || newState === "failed") {
        try {
          const tasks = parseTasks(readFileSync(planPath, "utf-8"))
          const task = tasks.find(t => t.id === taskId)
          const note = task ? `Task ${taskId} (${task.name}) → ${newState} | Contract: ${task.contract || "none"}` : `Task ${taskId} → ${newState}`
          const memFile = join(MEMORY_DIR, "lessons.md")
          const date = new Date().toISOString().slice(0, 10)
          try { appendFileSync(memFile, `- ${date} | ${taskId} | ${note}\n`, "utf-8") } catch {}
        } catch {}
      }
    }

    return { content: [{ type: "text", text: JSON.stringify(result) }] }
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

    // EVAL-01: append every verify to the eval log for North Star metrics (docs/reports/pipeline-evals.md).
    // P2-05: cada registro incluye traceId de la tarea para trazabilidad.
    // P3-rem: skills (derivadas del plan vía detectType) + toolUsed (derivado del command) para correlación skill→primer intento.
    try {
      const logPath = join(TASK_SYSTEM, "enforcement", "verify-log.jsonl")
      const planPath = (() => { try { return findPlanFile(PROJECT_ROOT) } catch { return null } })()
      const skills = (() => {
        try {
          if (!planPath || !taskId) return []
          const task = parseTasks(readFileSync(planPath, "utf-8")).find(t => t.id === taskId || (t.name && (t.name.startsWith(taskId) || t.name.includes(taskId))))
          if (!task || !task.files) return []
          const typeInfo = detectType(task.files)
          return (typeInfo && typeInfo.skills) || []
        } catch { return [] }
      })()
      const toolUsed = (() => {
        const c = (command || "").trim()
        if (!c) return null
        const bin = c.split(/\s+/)[0].toLowerCase()
        if (bin.includes("cargo")) return "cargo-verify"
        if (bin.includes("pwsh") || bin.includes("powershell") || bin.endsWith(".ps1")) return "pwsh"
        if (bin === "node") return "node"
        if (bin === "npx") return "npx"
        if (bin === "npm") return "npm"
        if (bin.startsWith("python")) return "python"
        if (bin === "git") return "git"
        return bin || null
      })()
      // Rotación simple: log >5MB → verify-log-<ts>.jsonl (evita contexto
      // gigante para diagnose y append lento con los años).
      try { const st = statSync(logPath); if (st.size > 5 * 1024 * 1024) renameSync(logPath, logPath.replace(/\.jsonl$/, `-${Date.now()}.jsonl`)) } catch {}
      appendFileSync(logPath, JSON.stringify({
        ts: new Date().toISOString(), taskId: taskId || null, traceId: taskId ? traceIdForTask(taskId) : null, command,
        passed, exitCode, expectedExitCode, elapsed,
        summary, plan: planPath, skills, toolUsed,
      }) + "\n", "utf-8")
    } catch { /* eval logging must never break verify */ }

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

// Orden específico→genérico: `vantadb-python/src/lib.rs` debe ser python, no
// "multi" por el fallback `/src\//`. El patrón genérico va ÚLTIMO y gana el
// primer match. Para tareas genuinamente multi-dominio usar extraSkills.
const TYPE_PATTERNS = [
  { pattern: /vantadb-python\//, type: "python", label: "Python SDK", skills: ["source-driven-development"], checks: ["python -m pytest vantadb-python/tests/ -v"] },
  { pattern: /vantadb-ts\//, type: "typescript", label: "TypeScript SDK", skills: ["source-driven-development"], checks: ["npx tsc --noEmit", "npm test"] },
  { pattern: /web\/src\//, type: "frontend", label: "Web frontend", skills: ["frontend-ui-engineering", "design-taste-frontend"], checks: ["npx tsc --noEmit", "npm run lint"] },
  { pattern: /\.github\//, type: "devops", label: "CI/CD / DevOps", skills: ["ci-cd-and-automation", "doubt-driven-development"], checks: ["yamllint .github/"] },
  { pattern: /vantadb-server\//, type: "server", label: "HTTP server", skills: ["source-driven-development", "security-and-hardening"], checks: ["cargo check -p vantadb-server"] },
  { pattern: /docs\//, type: "docs", label: "Documentation", skills: ["writing-guidelines", "writing-plans"], checks: ["scripts/validate-docs-coverage.ps1"] },
  { pattern: /(^|[^a-z-])src\//, type: "rust", label: "Rust core", skills: ["source-driven-development", "doubt-driven-development", "ponytail"], checks: ["cargo check -p vantadb", "cargo fmt --check", "cargo clippy --workspace --all-targets --all-features -- -D warnings", "cargo nextest run --profile audit --workspace --build-jobs 2"] },
]

const ESTIMATE_MAP = { "🟢": { turns: "5-10", label: "Bajo" }, "🟡": { turns: "15-30", label: "Medio" }, "🔴": { turns: "30-60", label: "Alto" } }

function detectType(archivosClave) {
  if (!archivosClave || archivosClave.trim() === "") return { type: "unknown", label: "No detectable", skills: [], checks: [], estimate: null }

  const m = TYPE_PATTERNS.find(tp => tp.pattern.test(archivosClave))
  if (!m) return { type: "unknown", label: "No detectable", skills: ["campaign-executor"], checks: ["cargo check -p vantadb"], estimate: null }

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
    const skills = [...new Set([...(typeInfo.skills || []), "campaign-executor", "progreso", "ponytail", ...(extraSkills || [])])]
    const sortOrder = ["campaign-executor", "progreso", "ponytail"]
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
    const budgetCorrupted = !!budget.budgetCorrupted

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
        budgetCorrupted,
      }, null, 2) }],
    }
  },
)

const SERVER_START_TIME = Date.now()

// ---------- Tool 9: campaign_health_status ----------

server.tool(
  "campaign_health_status",
  {},
  async () => {
    const health = getHealth(PROJECT_ROOT)
    const uptime = Date.now() - SERVER_START_TIME
    return {
      content: [{
        type: "text",
        text: JSON.stringify({ ...health, serverLiveness: true, serverStartTime: SERVER_START_TIME, uptime }, null, 2),
      }],
    }
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

// Keywords específicos solamente: los ultra-genéricos ("add","new","find",
// "search") matcheaban media backlog y forzaban workflows equivocados. Si no hay
// señal específica → null → C0 genérica (comportamiento correcto por default).
const WORKFLOW_KEYWORDS = {
  "bug-fix": ["bug", "fix", "error", "crash", "panic", "incorrect", "wrong", "fails", "broken"],
  "feature-add": ["feature", "implement", "create", "integrate"],
  "refactor": ["refactor", "clean", "simplify", "rename", "extract", "inline", "split", "restructure"],
  "research": ["research", "investigate", "explore", "how does"],
  "nine-second-saloon": ["quick", "fast", "urgent", "hotfix", "emergency", "critical", "immediate", "ship"],
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
        fallback: !workflow ? "Use generic C0 state machine from iter-loop-tools.md" : undefined,
      }, null, 2) }],
    }
  },
)

// ---------- Tool 13: campaign_validate_command (cross-validate with PS1 script) ----------

server.tool(
  "campaign_validate_command",
  {
    command: z.string().describe("Command to validate"),
    type: z.enum(["shell", "file_path", "python", "code", "sql", "html", "text"]).optional().default("shell"),
    workspace: z.string().optional().describe("Workspace root"),
  },
  async ({ command, type, workspace }) => {
    const jsResult = validateOutput(command, type, workspace || PROJECT_ROOT)
    let ps1Result = null
    try {
      const psCmd = `& '${join(TASK_SYSTEM, "validation", "validate-output.ps1")}' -Content '${command.replace(/'/g, "''")}' -Type ${type}${workspace ? ` -Workspace '${workspace.replace(/'/g, "''")}'` : ""}`
      const out = execSync(psCmd, { encoding: "utf-8", timeout: 10000, shell: "pwsh" })
      ps1Result = JSON.parse(out.trim())
    } catch {}
    return {
      content: [{ type: "text", text: JSON.stringify({
        valid: jsResult.valid && (ps1Result ? ps1Result.valid : true),
        jsValidation: jsResult,
        ps1Validation: ps1Result,
        crossValid: !ps1Result || jsResult.valid === ps1Result.valid,
      }, null, 2) }],
    }
  },
)

// ---------- Tool 14: campaign_diagnose_pipeline ----------

server.tool(
  "campaign_diagnose_pipeline",
  {
    pipelinePath: z.string().optional().default("").describe("Path to pipeline root (default: project root)"),
  },
  async ({ pipelinePath }) => {
    const pyScript = join(TASK_SYSTEM, "self-modification", "performance_diagnosis.py")
    if (!existsSync(pyScript)) {
      return { content: [{ type: "text", text: JSON.stringify({ error: "Diagnosis script not found" }) }] }
    }
    try {
      const cmd = `python "${pyScript}" "${pipelinePath || PROJECT_ROOT}" 2>$null`
      const out = execSync(cmd, { encoding: "utf-8", timeout: 60000, shell: "pwsh" })
      const result = JSON.parse(out.trim())
      return { content: [{ type: "text", text: JSON.stringify(result, null, 2) }] }
    } catch (e) {
      return { content: [{ type: "text", text: JSON.stringify({ error: `Diagnosis failed: ${e.message}`, hint: "Install python + dependencies to use full diagnosis" }) }] }
    }
  },
)

// ---------- Tool 15: campaign_get_workflow ----------

server.tool(
  "campaign_get_workflow",
  {
    name: z.string().describe("Workflow name (bug-fix, feature-add, refactor, research, nine-second-saloon)"),
  },
  async ({ name }) => {
    const workflow = loadWorkflow(name)
    if (!workflow) {
      const available = Object.keys(WORKFLOW_KEYWORDS).filter(w => loadWorkflow(w))
      return { content: [{ type: "text", text: JSON.stringify({ error: `Workflow '${name}' not found`, available }, null, 2) }] }
    }
    return { content: [{ type: "text", text: JSON.stringify(workflow, null, 2) }] }
  },
)

// ---------- Tool 16: campaign_enforce_state ----------

// [SPEC -> RUNTIME] C0 enforcement extras (vantadb-lead 2026-08-05, recommendation A).
// Wired single-call checks: blocked_env, allowed_commands (VERIFY), write-operation classifier.
// max_edit_lines / max_files_per_state / read-dedup / context-budget remain [SPEC] (need per-session accumulators).
const BLOCKED_ENV_DEFAULT = ["API_KEY", "TOKEN", "SECRET", "PASSWORD", "REGISTRY_TOKEN", "AUTH"]
const VERIFY_COMMANDS = [
  "cargo", "just verify", "just check", "just ci", "pytest", "npm test", "npm run",
  "node --check", "node --test", "git diff", "git status", "git log", "rg", "grep",
  "npx", "python", "Get-ChildItem", "Test-Path", "dev-tools/verify", "scripts/",
]
const C0_CHECK_CONFIG = {
  PLAN: { blocked_env: BLOCKED_ENV_DEFAULT },
  ACT: { blocked_env: BLOCKED_ENV_DEFAULT },
  VERIFY: { blocked_env: BLOCKED_ENV_DEFAULT, allowed_commands: VERIFY_COMMANDS },
  COLLATERAL: { blocked_env: BLOCKED_ENV_DEFAULT },
  RESEARCH: { blocked_env: BLOCKED_ENV_DEFAULT },
  EVALUATE: { blocked_env: BLOCKED_ENV_DEFAULT },
  REVIEW: { blocked_env: BLOCKED_ENV_DEFAULT },
  ACCEPT: { blocked_env: BLOCKED_ENV_DEFAULT },
  CLOSE: { blocked_env: BLOCKED_ENV_DEFAULT },
  STALL: { blocked_env: BLOCKED_ENV_DEFAULT },
}

// Scan for $VAR / ${VAR} / $VARIABLE references matching blocked prefixes.
function checkBlockedEnv(cmd, prefixes) {
  if (!prefixes || !cmd) return null
  for (const pre of prefixes) {
    if (new RegExp(`\\$\\{?${pre}[A-Z_]*}?`).test(cmd)) {
      return `Command blocked: references environment variable matching '${pre}'.`
    }
  }
  return null
}

function checkAllowedCommands(cmd, prefixes) {
  if (!prefixes || !cmd) return null
  const c = cmd.trim()
  for (const p of prefixes) { if (c.startsWith(p)) return null }
  return `Command rejected: '${c.slice(0, 60)}' is not in the allowed commands for this state. Allowed prefixes: ${prefixes.join(", ")}`
}

// Write/destructive operations that bypass Edit/Write denial via Bash redirects.
const WRITE_OPS = [
  { re: /(\s|^)(tee|cp|mv|dd)\s/, cls: "FileWrite" },
  { re: /(\s|^)(sed\s+-i|awk\s+-i\s+inplace|perl\s+-pi|patch)\s/, cls: "FileModify" },
  { re: /(^|\s|2|1)>>?\s*\S/, cls: "FileWrite" },
  { re: /(\s|^)(rm|rmdir|shred|truncate)\s/, cls: "Destructive" },
  { re: /git\s+clean/, cls: "Destructive" },
]

function stateAllowsWrites(stateKey) {
  const a = getAllowedTools(stateKey).allowed || []
  return a.some(p => p === "edit" || p === "write" || p.startsWith("edit") || p.startsWith("write"))
}

function classifyBashWrite(cmd, stateKey) {
  if (!cmd || stateAllowsWrites(stateKey)) return null
  for (const seg of cmd.split(/&&|;|\||\r?\n/)) {
    for (const op of WRITE_OPS) {
      if (op.re.test(seg)) {
        const allow = getAllowedTools(stateKey).allowed.join(", ")
        return `Bash command blocked in state '${stateKey}': segment '${seg.trim().slice(0, 50)}' performs a ${op.cls} operation which requires Write/Edit in allowed_tools. Allowed: ${allow}`
      }
    }
  }
  return null
}

server.tool(
  "campaign_enforce_state",
  {
    state: z.string().describe("Current C0 state name"),
    toolName: z.string().describe("Tool to validate"),
    toolArgs: z.record(z.any()).optional().default({}).describe("Tool arguments for context-aware checks"),
  },
  async ({ state, toolName, toolArgs }) => {
    const stateKey = state.toUpperCase()
    const allowed = getAllowedTools(stateKey)
    const actionCheck = validateAction(stateKey, toolName)
    const checkConfig = C0_CHECK_CONFIG[stateKey] || {}

    const warnings = []
    const blocks = []

    if (!actionCheck.allowed) {
      blocks.push(`Tool '${toolName}' is not allowed in state '${stateKey}'`)
    }

    if (toolArgs.command && toolName.toLowerCase() === "bash") {
      const envBlock = checkBlockedEnv(toolArgs.command, checkConfig.blocked_env)
      if (envBlock) blocks.push(envBlock)

      const val = validateShellCommand(toolArgs.command)
      if (!val.valid) blocks.push(...val.errors)
      warnings.push(...val.warnings)

      if (checkConfig.allowed_commands) {
        const ac = checkAllowedCommands(toolArgs.command, checkConfig.allowed_commands)
        if (ac) blocks.push(ac)
      } else {
        const cls = classifyBashWrite(toolArgs.command, stateKey)
        if (cls) blocks.push(cls)
      }
    }

    if (toolArgs.filePath && toolName.toLowerCase().match(/edit|write/)) {
      const val = validateFilePath(toolArgs.filePath, PROJECT_ROOT)
      if (!val.valid) blocks.push(...val.errors)
    }

    return {
      content: [{ type: "text", text: JSON.stringify({
        allowed: blocks.length === 0,
        state: stateKey,
        tool: toolName,
        blocks,
        warnings,
        allowedTools: allowed,
        checkConfig,
        actionCheck,
      }, null, 2) }],
    }
  },
)

// ---------- Tool 17: campaign_session_track ----------

const SESSION_DIR = join(TASK_SYSTEM, "enforcement", "sessions")
if (!existsSync(SESSION_DIR)) {
  try { mkdirSync(SESSION_DIR, { recursive: true }) } catch {}
}

function sessionPath(sessionId) { return join(SESSION_DIR, `${sessionId.replace(/[<>:"/\\|?*]/g, "_")}.json`) }

function readSession(sessionId) {
  const p = sessionPath(sessionId)
  try { return JSON.parse(readFileSync(p, "utf-8")) } catch { return null }
}

function writeSession(sessionId, data) {
  writeFileSync(sessionPath(sessionId), JSON.stringify(data, null, 2), "utf-8")
}

server.tool(
  "campaign_session_track",
  {
    action: z.enum(["create", "get", "update", "list", "delete"]).describe("Session action"),
    sessionId: z.string().optional().describe("Session ID (required for create/get/update/delete)"),
    state: z.string().optional().describe("Current state (for create/update)"),
    context: z.record(z.any()).optional().default({}).describe("Session context data"),
  },
  async ({ action, sessionId, state, context }) => {
    switch (action) {
      case "create": {
        if (!sessionId) return { content: [{ type: "text", text: JSON.stringify({ error: "sessionId required" }) }] }
        const existing = readSession(sessionId)
        if (existing) return { content: [{ type: "text", text: JSON.stringify({ error: "Session exists", session: existing }) }] }
        const session = { sessionId, state: state || "PLAN", context, createdAt: new Date().toISOString(), updatedAt: new Date().toISOString(), iterationCount: 0, transitionCount: 0 }
        writeSession(sessionId, session)
        return { content: [{ type: "text", text: JSON.stringify({ created: true, session }) }] }
      }
      case "get": {
        if (!sessionId) return { content: [{ type: "text", text: JSON.stringify({ error: "sessionId required" }) }] }
        const session = readSession(sessionId)
        if (!session) return { content: [{ type: "text", text: JSON.stringify({ error: "Session not found" }) }] }
        return { content: [{ type: "text", text: JSON.stringify(session, null, 2) }] }
      }
      case "update": {
        if (!sessionId) return { content: [{ type: "text", text: JSON.stringify({ error: "sessionId required" }) }] }
        const p = sessionPath(sessionId)
        try {
          // RMW bajo lock: sin esto, dos sub-agentes en paralelo pierden
          // contextos entre sí y el iterationCount++ queda corto.
          return { content: [{ type: "text", text: JSON.stringify(lockFileSync(`${p}.lock`, () => {
            const s = readSession(sessionId)
            if (!s) return { error: "Session not found" }
            if (state) s.state = state
            if (context) s.context = { ...s.context, ...context }
            s.iterationCount++
            s.updatedAt = new Date().toISOString()
            writeSession(sessionId, s)
            return { updated: true, session: s }
          })) }] }
        } catch (e) {
          return { content: [{ type: "text", text: JSON.stringify({ error: `Session lock failed: ${e.message}` }) }] }
        }
      }
      case "list": {
        const sessions = []
        if (existsSync(SESSION_DIR)) {
          for (const f of readdirSync(SESSION_DIR).filter(f => f.endsWith(".json"))) {
            try { sessions.push(JSON.parse(readFileSync(join(SESSION_DIR, f), "utf-8"))) } catch {}
          }
        }
        return { content: [{ type: "text", text: JSON.stringify({ sessions }) }] }
      }
      case "delete": {
        if (!sessionId) return { content: [{ type: "text", text: JSON.stringify({ error: "sessionId required" }) }] }
        const p = sessionPath(sessionId)
        if (existsSync(p)) { rmSync(p, { force: true }) }
        return { content: [{ type: "text", text: JSON.stringify({ deleted: true, sessionId }) }] }
      }
      default:
        return { content: [{ type: "text", text: JSON.stringify({ error: "Unknown action" }) }] }
    }
  },
)

// ---------- Tool 18: campaign_state_snapshot ----------

server.tool(
  "campaign_state_snapshot",
  {
    state: z.string().describe("Current C0 state (PLAN/ACT/VERIFY/etc.)"),
    planFile: z.string().optional().describe("Path to plan file"),
    taskId: z.string().optional().describe("Current task ID"),
    notes: z.string().optional().describe("Free-form state notes"),
  },
  async ({ state, planFile, taskId, notes }) => {
    const file = join(PROJECT_ROOT, "docs", "pipeline-state.json")
    try {
      // RMW bajo lock: last-writer-wins entre instancias perdía campos.
      return { content: [{ type: "text", text: JSON.stringify(lockFileSync(`${file}.lock`, () => {
        let prev = {}
        try { prev = JSON.parse(readFileSync(file, "utf-8")) } catch {}
        const snap = {
          ...prev,
          lastSync: new Date().toISOString(),
          state: (state || prev.state || "").toUpperCase(),
          planFile: planFile || prev.planFile,
          taskId: taskId || prev.taskId,
          ...(notes !== undefined ? { notes } : {}),
        }
        try { writeFileSync(file, JSON.stringify(snap, null, 2), "utf-8") } catch (e) {
          return { saved: false, error: e.message }
        }
        return { saved: true, file, snapshot: snap }
      }), null, 2) }] }
    } catch (e) {
      return { content: [{ type: "text", text: JSON.stringify({ saved: false, error: `State lock failed: ${e.message}` }) }] }
    }
  },
)

// ---------- Tool 20: campaign_backlog_dedup ----------

server.tool(
  "campaign_backlog_dedup",
  {
    backlogPath: z.string().optional().default("docs/Backlog.md").describe("Path to backlog file"),
  },
  async ({ backlogPath }) => {
    const file = resolve(PROJECT_ROOT, backlogPath)
    if (!existsSync(file)) return { content: [{ type: "text", text: JSON.stringify({ error: `Backlog not found: ${file}` }) }] }

    const text = readFileSync(file, "utf-8")
    const rows = text.split("\n")
    const taskRows = []   // actual task rows: `ID` in first table cell

    for (const line of rows) {
      // task row: | `ID` | ... | (backticked ID in first cell)
      const taskMatch = line.match(/^\|\s*`([A-Z0-9]+-\d+)`\s*\|/)
      if (taskMatch) taskRows.push(taskMatch[1])
    }

    const dupRows = [...new Set(taskRows.filter((id, i) => taskRows.indexOf(id) !== i))]

    // Near-dupes: same normalized number AND similar prefix (Levenshtein <= 2).
    // Catches NUEVA-01 vs NUEVO-01; ignores DESKTOP-01 vs AUDIT-01 (different prefix).
    const lev = (a, b) => {
      const dp = Array.from({ length: a.length + 1 }, (_, i) => [i, ...Array(b.length).fill(0)])
      for (let j = 0; j <= b.length; j++) dp[0][j] = j
      for (let i = 1; i <= a.length; i++)
        for (let j = 1; j <= b.length; j++)
          dp[i][j] = Math.min(dp[i - 1][j] + 1, dp[i][j - 1] + 1, dp[i - 1][j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1))
      return dp[a.length][b.length]
    }
    const normNum = id => String(parseInt(id.match(/(\d+)$/)?.[1] || "0", 10))
    const prefix = id => id.replace(/-?\d+$/, "")
    const nearDupes = {}
    for (const id of taskRows) {
      const hits = taskRows.filter(x => x !== id && normNum(x) === normNum(id) && lev(prefix(x), prefix(id)) <= 2)
      if (hits.length) nearDupes[id] = hits
    }

    return {
      content: [{ type: "text", text: JSON.stringify({
        file,
        totalTaskRows: taskRows.length,
        distinctTaskRows: new Set(taskRows).size,
        duplicateTaskRows: dupRows,
        nearDuplicateIds: nearDupes,
        readOnly: true,
      }, null, 2) }],
    }
  },
)

// ---------- Tool 21: campaign_eval_summary (North Star medible) ----------
//
// Tasa de primer-intento por skill desde verify-log.jsonl. Convierte la North
// Star (>90% first-try) en número consultable, no aspiración.
server.tool(
  "campaign_eval_summary",
  { limit: z.number().optional().default(500).describe("Máx filas recientes a considerar") },
  async ({ limit }) => {
    const logPath = join(TASK_SYSTEM, "enforcement", "verify-log.jsonl")
    let rows = []
    try {
      for (const line of readFileSync(logPath, "utf-8").split("\n")) {
        if (!line.trim()) continue
        try { rows.push(JSON.parse(line)) } catch {}
      }
    } catch {}
    rows = rows.filter(r => r.taskId).slice(-limit)
    // ponytail: O(tasks×rows) por el filter anidado — fino a miles de filas; indexar si crece.
    const byTask = new Map()
    for (const r of rows) if (!byTask.has(r.taskId)) byTask.set(r.taskId, r.passed === true)
    const perSkill = {}
    let ok = 0
    for (const [taskId, firstPassed] of byTask) {
      if (firstPassed) ok++
      const taskRows = rows.filter(r => r.taskId === taskId)
      for (const s of [...new Set(taskRows.flatMap(r => r.skills || []))]) {
        ;(perSkill[s] ??= { tasks: 0, firstTry: 0 })
        perSkill[s].tasks++
        if (firstPassed) perSkill[s].firstTry++
      }
    }
    for (const s of Object.values(perSkill)) s.rate = s.tasks ? +(s.firstTry / s.tasks).toFixed(3) : null
    return { content: [{ type: "text", text: JSON.stringify({
      totalTasks: byTask.size,
      overallFirstTryRate: byTask.size ? +(ok / byTask.size).toFixed(3) : null,
      northStarTarget: 0.9,
      perSkill,
    }, null, 2) }] }
  },
)

// ---------- Tool 22: campaign_plan_lock_info (diagnóstico de locks) ----------
server.tool(
  "campaign_plan_lock_info",
  {},
  async () => {
    const dir = join(PROJECT_ROOT, "docs", "plans")
    const locks = []
    if (existsSync(dir)) {
      for (const f of readdirSync(dir).filter(f => f.endsWith(".lock"))) {
        const fp = join(dir, f)
        const holder = readLockHolder(fp)
        const ageMs = Date.now() - statSync(fp).mtimeMs
        locks.push({
          file: f,
          pid: holder?.pid ?? null,
          since: holder ? new Date(holder.ts).toISOString() : null,
          ageSeconds: Math.round(ageMs / 1000),
          stale: ageMs > 10000, // >10s: lockFileSync lo romperá en el próximo intento
        })
      }
    }
    return { content: [{ type: "text", text: JSON.stringify({ locks }, null, 2) }] }
  },
)

process.on('uncaughtException', (err) => {
  console.error('[campaign-server] Uncaught exception:', err)
})

process.on('unhandledRejection', (reason) => {
  console.error('[campaign-server] Unhandled rejection:', reason)
})

// ---------- start ----------
//
// TSYS-06-F1: exports para tests (node --test) — al importar el módulo NO se
// conecta el transporte stdio (isMain guard). Al correr como server (bun/node
// campaign-server.mjs) argv[1] == este archivo → conecta exactamente como antes.

export { readBudget, writeBudget, withPlanLock, findInProgressTasks, detectConflict, sha1, consumeBudget, budgetStatus, detectType, bumpBudget, findPlanFile }

const isMain = process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)
if (isMain) {
  try {
    const transport = new StdioServerTransport()
    await server.connect(transport)
  } catch (error) {
    console.error("[campaign-server] Fatal: Failed to connect transport:", error)
    process.exit(1)
  }
}
