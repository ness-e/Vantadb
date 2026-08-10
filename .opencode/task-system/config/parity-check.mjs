#!/usr/bin/env node
// EVAL-02 — parity check for the C0 state machine definitions.
// Ensures every state in state-tools.mjs (canonical, consumed at runtime) appears in:
//   - prompts/iter-loop-tools.md   (prose spec, State Machine section)
//   - skills/campaign-executor/SKILL.md (diagram)
// Fails (exit 1) if any state is missing, so the "do NOT diverge them" header is enforced.
import { readFileSync } from "node:fs"
import { resolve, join, dirname } from "node:path"
import { pathToFileURL } from "node:url"
import { fileURLToPath } from "node:url"

const __dirname = dirname(fileURLToPath(import.meta.url))
const TASK_SYSTEM = resolve(__dirname, "..")
const ROOT = resolve(TASK_SYSTEM, "..", "..")

const { STATE_TOOLS } = await import(pathToFileURL(join(TASK_SYSTEM, "config", "state-tools.mjs")).href)
const canonical = Object.keys(STATE_TOOLS)
const targets = {
  "prompts/iter-loop-tools.md": join(TASK_SYSTEM, "prompts", "iter-loop-tools.md"),
  "skills/campaign-executor/SKILL.md": join(ROOT, ".opencode", "skills", "campaign-executor", "SKILL.md"),
}

let failed = false
for (const [label, path] of Object.entries(targets)) {
  const body = readFileSync(path, "utf-8")
  const missing = canonical.filter(s => !new RegExp(`\\b${s}\\b`).test(body))
  if (missing.length) {
    failed = true
    console.error(`❌ ${label} — faltan estados: ${missing.join(", ")}`)
  } else {
    console.log(`✅ ${label} — ${canonical.length} estados presentes`)
  }
}

if (failed) {
  console.error("\n❌ Paridad C0 rota. state-tools.mjs es la fuente canónica; actualizá la spec prose (iter-loop-tools.md) y el diagrama (SKILL.md).")
  process.exit(1)
}
console.log(`\n✅ Paridad C0 OK — ${canonical.length} estados (${canonical.join(", ")}) en todas las fuentes.`)