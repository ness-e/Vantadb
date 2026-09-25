// VantaDB memory plugin (FIND-106, template_version 1.1.0, docs_verified 2026-09-17)
// WIRE-10: one-shot via `vanta-cli mcp-call` — no PowerShell, no MCP stdio bridge.
// Source: https://opencode.ai/docs/plugins/ + ../references/recall-policy.md (FIND-103)
// Install: copy to <project>/.opencode/plugins/ (or ~/.config/opencode/plugins/).
// Policy: SessionStart recall (top_k 5) -> per-message recall (verbatim) ->
// PreCompact save-state -> Stop auto-capture. Empty recall injects NOTHING.

import { execFileSync } from "node:child_process";

const DB_PATH = process.env.VANTADB_DB_PATH ?? "<DB_PATH>";
const REPO = process.env.VANTADB_REPO ?? "<REPO>";

// Resolve the CLI: explicit override first, then the repo debug build
// (campaign-verified binary; a stale global install masks drift — same rule
// as vanta-mcp-local.ps1), then PATH.
function cliCandidates() {
  const exe = process.platform === "win32" ? "vanta-cli.exe" : "vanta-cli";
  const cands = [];
  if (process.env.VANTADB_CLI) cands.push(process.env.VANTADB_CLI);
  if (REPO && !REPO.startsWith("<")) {
    cands.push(`${REPO}/target/debug/${exe}`);
    cands.push(`${REPO}/target/release/${exe}`);
  }
  cands.push(exe); // PATH (install.sh -> ~/.vanta/bin)
  return cands;
}

function mcpCall(tool, args) {
  // One-shot: `vanta-cli mcp-call` speaks MCP over piped stdio internally
  // and prints the tool `result` verbatim. Returns the structured payload
  // (structuredContent ?? parsed content[0].text) or null (null = inject
  // NOTHING, never fill the gap). Non-zero exit = fail-loud null.
  const argv = ["--db", DB_PATH, "mcp-call", "--tool", tool, "--args", JSON.stringify(args)];
  const errors = [];
  for (const bin of cliCandidates()) {
    try {
      const out = execFileSync(bin, argv, { timeout: 15000, encoding: "utf8" });
      const result = JSON.parse(out);
      if (result && typeof result === "object") {
        if (result.structuredContent && typeof result.structuredContent === "object") {
          return result.structuredContent;
        }
        const text = result?.content?.[0]?.text;
        if (typeof text === "string") {
          try {
            return JSON.parse(text);
          } catch {
            return { text };
          }
        }
      }
      return null;
    } catch (e) {
      errors.push(`${bin}: ${e?.message ?? e}`);
    }
  }
  console.error(`[vantadb-memory] mcp-call failed: ${errors.join(" | ")}`);
  return null;
}

export const VantadbMemory = async () => ({
  // SessionStart recall: memory_recall(scope agent, top_k 5) -> additionalContext.
  "session.created": async (input, output) => {
    const res = mcpCall("memory_recall", { query: "session start context", scope: "agent", top_k: 5 });
    const hits = res?.prepend_context ?? res?.recalled ?? [];
    if (Array.isArray(hits) && hits.length > 0) {
      output.additionalContext = (output.additionalContext ?? "") + hits.join("\n");
    } else if (typeof hits === "string" && hits.length > 0) {
      output.additionalContext = (output.additionalContext ?? "") + hits;
    }
    // Empty -> NOTHING (no filler).
  },
  // Per-message recall: keyed on verbatim prompt text via tool boundary.
  "tool.execute.before": async (input, output) => {
    const text = output?.args?.prompt ?? input?.prompt ?? null;
    if (typeof text === "string" && text.length > 0) {
      const res = mcpCall("memory_recall", { query: text, scope: "agent", top_k: 5 });
      const hits = res?.prepend_context ?? res?.recalled ?? [];
      if (Array.isArray(hits) && hits.length > 0) {
        output.additionalContext = (output.additionalContext ?? "") + hits.join("\n");
      } else if (typeof hits === "string" && hits.length > 0) {
        output.additionalContext = (output.additionalContext ?? "") + hits;
      }
    }
  },
  // PreCompact save-state: what isn't saved before compaction is gone.
  "experimental.session.compacting": async (input, output) => {
    output.context.push(
      "VantaDB save-state: persist open turns via thread_send and/or scene_write before compaction."
    );
  },
  // Stop auto-capture: proxy-turns auto-captured; direct turns need explicit thread_send.
  // NOTE: the idle event carries no transcript — this records a static marker only;
  // per-turn content capture happens in `tool.execute.before` above.
  "session.idle": async () => {
    mcpCall("thread_send", { role: "assistant", content: "session idle auto-capture" });
  },
});
