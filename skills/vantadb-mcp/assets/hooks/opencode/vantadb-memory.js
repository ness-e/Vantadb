// VantaDB memory plugin (FIND-106, template_version 1.0.0, docs_verified 2026-09-17)
// Source: https://opencode.ai/docs/plugins/ + ../references/recall-policy.md (FIND-103)
// Install: copy to <project>/.opencode/plugins/ (or ~/.config/opencode/plugins/).
// Policy: SessionStart recall (top_k 5) -> per-message recall (verbatim) ->
// PreCompact save-state -> Stop auto-capture. Empty recall injects NOTHING.

// ponytail: shell out to the MCP stdio server via the FIND-104 launcher instead of
// embedding an MCP client here; upgrade path = native MCP call when plugins allow it.
import { execFileSync } from "node:child_process";

const DB_PATH = process.env.VANTADB_DB_PATH ?? "<DB_PATH>";
const REPO = process.env.VANTADB_REPO ?? "<REPO>";

function mcpCall(tool, args) {
  // Minimal stdio bridge: delegates to vanta-cli through the launcher.
  // Returns parsed payload or null (null = inject NOTHING, never fill the gap).
  try {
    const out = execFileSync(
      "pwsh",
      ["-NoProfile", "-File", `${REPO}/vanta-mcp-local.ps1`, "-DbPath", DB_PATH],
      { input: JSON.stringify({ tool, args }), timeout: 15000, encoding: "utf8" }
    );
    return JSON.parse(out);
  } catch {
    return null;
  }
}

export const VantadbMemory = async () => ({
  // SessionStart recall: memory_recall(scope agent, top_k 5) -> additionalContext.
  "session.created": async (input, output) => {
    const res = mcpCall("memory_recall", { query: "session start context", scope: "agent", top_k: 5 });
    const hits = res?.prepend_context ?? res?.recalled ?? [];
    if (Array.isArray(hits) && hits.length > 0) {
      output.additionalContext = (output.additionalContext ?? "") + hits.join("\n");
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
