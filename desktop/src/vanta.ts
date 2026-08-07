// Typed bridge over the Tauri v2 IPC commands (DESK-03/06).
// Every type mirrors the serde DTOs in desktop/src-tauri/src/connections/types.rs.
// Command names must NOT be renamed — they're wired in src-tauri/src/lib.rs.
import { invoke } from "@tauri-apps/api/core";

// --- Enums (serde rename_all = "snake_case") ---------------------------------
export type Capability = "native" | "http" | "mcp" | "node" | "python" | "wasm";
export type ConnectionStatus = "connected" | "disconnected" | "error";
export type HealthStatus = "healthy" | "degraded" | "unhealthy";

// --- DTOs --------------------------------------------------------------------
export interface HealthReport {
  status: HealthStatus;
  backend: string;
  latency_ms: number;
  checked_at_ms: number;
  message?: string | null;
}

export interface ConnectionInfo {
  id: string;
  name: string;
  via: Capability;
  status: ConnectionStatus;
  description?: string | null;
}

export interface IngestItem {
  id?: string;
  text: string;
  /** Expands to `default` on the Rust side when omitted. */
  namespace?: string;
  embedding?: number[];
  /** Arbitrary JSON-able values. */
  metadata?: Record<string, unknown>;
}

export interface SearchQuery {
  query: string;
  top_k?: number;
  namespace?: string;
  filters?: Record<string, unknown>;
  embedding?: number[];
}

export interface SearchResult {
  id: string;
  namespace: string;
  text: string;
  /** Relevance, higher is better (backend-defined). */
  score: number;
  metadata?: Record<string, unknown>;
}

export interface MemoryRecord {
  id: string;
  namespace: string;
  text: string;
  embedding?: unknown;
  metadata?: Record<string, unknown>;
  created_at_ms?: number | null;
}

/** `ServerClientConfig` wire shape: `timeout` is a serde `Duration` (secs+nanos). */
export interface ServerClientConfig {
  url: string;
  port: number;
  token?: string;
  timeout?: { secs: number; nanos?: number };
}

// --- Error handling ----------------------------------------------------------
// Rust `#[non_exhaustive] VantaError` is externally tagged, so the rejected
// value can be `{ Native: "..." }`, `{ Http: { kind, message, status } }`, etc.
// Tauri v2 may also wrap it as `{ message, code }`. Handle all shapes lazily.
function firstString(v: unknown): string {
  if (typeof v === "string") return v;
  if (v && typeof v === "object") {
    const entries = Object.values(v as Record<string, unknown>);
    for (const e of entries) {
      if (typeof e === "string") return e;
      if (e && typeof e === "object") {
        const s = pickMessage(e);
        if (s) return s;
      }
    }
  }
  return "";
}

function pickMessage(e: unknown): string {
  if (!e || typeof e !== "object") return "";
  const r = e as Record<string, unknown>;
  return typeof r.message === "string" ? r.message : "";
}

/** Best-effort human-readable message from any VantaError shape. */
export function vantaErrorMessage(err: unknown): string {
  if (typeof err === "string") return err;
  if (err && typeof err === "object") {
    const r = err as Record<string, unknown>;
    const msg = pickMessage(r) || pickMessage(r.message) || firstString(r);
    if (msg) return msg;
  }
  return String(err ?? "unknown error");
}

// --- Command wrappers ---------------------------------------------------------
export function health(): Promise<HealthReport> {
  return invoke<HealthReport>("vanta_health");
}

export function connectNative(path: string): Promise<ConnectionInfo> {
  return invoke<ConnectionInfo>("vanta_connect", { target: { via: "native", path } });
}

export function connectServer(cfg: ServerClientConfig): Promise<ConnectionInfo> {
  return invoke<ConnectionInfo>("vanta_connect", {
    target: {
      via: "server",
      config: { ...cfg, timeout: cfg.timeout ?? { secs: 15, nanos: 0 } },
    },
  });
}

export function disconnect(id: string): Promise<void> {
  return invoke<void>("vanta_disconnect", { id });
}

/** Returns `[id, info]` pairs straight from Rust. */
export function listConnections(): Promise<[string, ConnectionInfo][]> {
  return invoke<[string, ConnectionInfo][]>("vanta_list_connections");
}

export function setActive(id: string): Promise<void> {
  return invoke<void>("vanta_set_active", { id });
}

export function ingest(records: IngestItem[]): Promise<string[]> {
  return invoke<string[]>("vanta_ingest", { records });
}

export function ingestBatch(records: IngestItem[]): Promise<string[]> {
  return invoke<string[]>("vanta_ingest_batch", { records });
}

export function search(query: SearchQuery): Promise<SearchResult[]> {
  return invoke<SearchResult[]>("vanta_search", { query });
}

export function get(key: string, namespace?: string): Promise<MemoryRecord> {
  return invoke<MemoryRecord>("vanta_get", { key, namespace });
}

export function remove(key: string, namespace?: string): Promise<void> {
  return invoke<void>("vanta_delete", { key, namespace });
}

export function list(opts?: { namespace?: string; limit?: number }): Promise<MemoryRecord[]> {
  return invoke<MemoryRecord[]>("vanta_list", {
    namespace: opts?.namespace,
    limit: opts?.limit,
  });
}