// VantaDB memory plugin (FIND-106, template_version 1.2.0, docs_verified 2026-11-19)
// WIRE-10: one-shot via `vanta-cli mcp-call` — no PowerShell, no MCP stdio bridge.
// Source: https://opencode.ai/docs/plugins/ + ../references/recall-policy.md (FIND-103)
// Install: copy to <project>/.opencode/plugins/ (or ~/.config/opencode/plugins/).
// Policy: SessionStart recall (top_k 5) -> per-message recall (verbatim) ->
// PreCompact save-state -> Stop auto-capture. Empty recall injects NOTHING.
// FIND-169 (1.2.0): `session.idle` auto-capture resolves a stable thread_id
// per OpenCode session (deterministic title, reused across restarts) instead
// of calling `thread_send` with no thread_id (deterministic -32602 no-op).

import { execFileSync } from "node:child_process";

const DB_PATH = process.env.VANTADB_DB_PATH ?? "<DB_PATH>";
const REPO = process.env.VANTADB_REPO ?? "<REPO>";

// sessionKey -> thread_id cache (plugin process lifetime). On restart the
// deterministic title is looked up via thread_list so idle capture reuses the
// same thread instead of creating one per process or per event.
const idleThreads = new Map();
// sessionKey -> last successful marker send (ms). Collapses duplicate
// delivery when a runtime dispatches session.idle to BOTH the `event` hook
// and the event-name key below.
const lastIdleSent = new Map();
const IDLE_DEDUPE_MS = 2000;

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

// ── session.idle auto-capture (FIND-169) ────────────────────────────────
//
// Current opencode delivers `session.idle` through the generic `event` hook
// as `{ event: { type: "session.idle", properties: { sessionID } } }`; older
// templates subscribed the event-name key directly. Both paths delegate to
// the same sender, deduped per session inside IDLE_DEDUPE_MS.

function sessionKeyOf(input) {
  const e = input?.event ?? input;
  return (
    e?.properties?.sessionID ??
    e?.properties?.session?.id ??
    e?.sessionID ??
    e?.session?.id ??
    e?.info?.id ??
    null
  );
}

function idleThreadTitle(sessionKey) {
  return `vanta-opencode-idle-${sessionKey ?? "default"}`;
}

// Find a thread by title, paging thread_list (count is the PAGE size —
// keep fetching while a full page comes back). Bounded: without a cap a
// pathological DB could page forever on the first idle event.
function findThreadByTitle(title) {
  const PAGE = 100;
  const MAX_PAGES = 10; // 1000 threads is far past any real usage
  for (let page = 0; page < MAX_PAGES; page++) {
    const listed = mcpCall("thread_list", { limit: PAGE, offset: page * PAGE });
    const threads = listed?.threads;
    if (!Array.isArray(threads)) return null; // DB unreachable / error shape
    const found = threads.find((t) => t?.title === title);
    if (found?.thread_id) return found.thread_id;
    if (threads.length < PAGE) return null; // exhausted
  }
  return null;
}

// Resolve (or create once) the stable thread for this session. Returns the
// thread_id as a u128 string, or null when the DB is unreachable — the hook
// then exits as a clean no-op (mcpCall already logged the failure).
function ensureIdleThreadId(sessionKey) {
  const key = sessionKey ?? "default";
  if (idleThreads.has(key)) return idleThreads.get(key);
  const title = idleThreadTitle(sessionKey);
  // Reuse the deterministic-titled thread across restarts: at most ONE
  // thread per OpenCode session exists, not one per plugin load.
  const found = findThreadByTitle(title);
  if (found) {
    idleThreads.set(key, found);
    return found;
  }
  const created = mcpCall("thread_create", { title });
  const id = created?.thread_id ?? null;
  if (id) idleThreads.set(key, id);
  return id;
}

function sendIdleMarker(input) {
  const sessionKey = sessionKeyOf(input);
  const key = sessionKey ?? "default";
  const now = Date.now();
  if (now - (lastIdleSent.get(key) ?? 0) < IDLE_DEDUPE_MS) return;
  let threadId = ensureIdleThreadId(sessionKey);
  if (!threadId) return; // clean no-op, error already logged by mcpCall
  const args = () => ({
    thread_id: threadId,
    role: "assistant",
    content: "session idle auto-capture",
  });
  let res = mcpCall("thread_send", args());
  if (res?.ok !== true) {
    // Cached thread was purged (TTL) or deleted: drop it, recreate, retry once.
    idleThreads.delete(key);
    threadId = ensureIdleThreadId(sessionKey);
    if (!threadId) return;
    res = mcpCall("thread_send", args());
  }
  if (res?.ok === true) lastIdleSent.set(key, now);
}

export const VantadbMemory = async () => ({
  // SessionStart recall: memory_recall(scope agent, top_k 5) -> additionalContext.
  "session.created": async (input, output) => {
    const res = mcpCall("memory_recall", {
      query: "session start context",
      scope: "agent",
      top_k: 5,
    });
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
      "VantaDB save-state: persist open turns via thread_send and/or scene_write before compaction.",
    );
  },
  // Stop auto-capture: proxy-turns auto-captured; direct turns need explicit thread_send.
  // NOTE: the idle event carries no transcript — this records a static marker only;
  // per-turn content capture happens in `tool.execute.before` above.
  // Kept as an event-name key for runtimes that dispatch by name; current
  // opencode routes the same event through `event` below.
  "session.idle": async (input) => {
    sendIdleMarker(input);
  },
  // Generic bus hook: current opencode delivers session.idle here as
  // { event: { type: "session.idle", properties: { sessionID } } }.
  event: async (input) => {
    if (input?.event?.type === "session.idle") {
      sendIdleMarker(input.event);
    }
  },
});
