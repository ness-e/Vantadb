// Typed bridge over the console's IPC transport (DESK-03/06, WEB-00).
// Every type mirrors the serde DTOs in desktop/src-tauri/src/connections/types.rs.
// Command names must NOT be renamed — they're wired in src-tauri/src/lib.rs.
// All calls go through the pluggable transport (Tauri IPC today, HTTP in
// WEB-04) so the React components stay backend-agnostic.
import { transport, TauriBackend } from "./transport.ts";

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
  /** When true, each result carries a per-hit score breakdown (`explanation`). */
  explain?: boolean;
}

export interface SearchResult {
  id: string;
  namespace: string;
  text: string;
  /** Relevance, higher is better (backend-defined). */
  score: number;
  metadata?: Record<string, unknown>;
  /** Per-hit score breakdown when the search ran with `explain: true`. */
  explanation?: ExplanationHit | null;
}

/** Mirrors the Rust `ExplanationHit` wire DTO (VS-CORE-03): per-hit BM25/RRF breakdown. */
export interface ExplanationHit {
  /** Unique identity string (`namespace\0key`) of the matched record. */
  identity: string;
  /** Combined relevance score for this hit. */
  score: number;
  /** Text snippet surrounding the matched query terms, if available. */
  snippet?: string | null;
  /** Query tokens that matched in this record. */
  matched_tokens: string[];
  /** Query phrases that matched in this record. */
  matched_phrases: string[];
  /** Per-term BM25 scoring breakdown. */
  bm25_terms: Bm25Term[];
  /** Rank of this hit in the text-only result set, if applicable. */
  rrf_text_rank?: number | null;
  /** Rank of this hit in the vector-only result set, if applicable. */
  rrf_vector_rank?: number | null;
}

/** Per-term BM25 scoring decomposition (mirror of `Bm25Term` wire DTO). */
export interface Bm25Term {
  /** The query term token. */
  token: string;
  /** Term frequency in the matched document. */
  tf: number;
  /** Document frequency across the namespace. */
  df: number;
  /** Total length (in tokens) of the matched document. */
  doc_len: number;
  /** BM25 score contribution for this term. */
  contribution: number;
}

export interface MemoryRecord {
  id: string;
  namespace: string;
  text: string;
  /** Optional embedding vector (dense). */
  vector?: number[] | null;
  metadata?: Record<string, unknown>;
  created_at_ms?: number | null;
  /** Last-update time as unix milliseconds. */
  updated_at_ms?: number | null;
  /** Monotonic version counter. */
  version?: number | null;
  /** Deterministic node id derived from namespace and key (string on the wire). */
  node_id?: string | null;
  /** Optional sparse term-weight vector (dimension → coefficient). */
  sparse_vector?: Record<string, number> | null;
  /** Absolute unix-ms expiry; null/absent means the record never expires. */
  expires_at_ms?: number | null;
}

/** A page of records with the cursor for the next page (VS-CORE-01). */
export interface ListPage {
  records: MemoryRecord[];
  /** Zero-based cursor for the next page; null/absent means last page. */
  next_cursor?: number | null;
}

/** Per-namespace record statistics (VS-CORE-02). Mirrors the Rust `NamespaceStats` DTO. */
export interface NamespaceStats {
  /** Total records in the namespace, including expired (not-yet-purged) ones. */
  count: number;
  /** Records expiring within the window (core default: 24h). */
  expiring_soon: number;
  /** Records already past their expiry (still present until purged). */
  expired: number;
}

/** Namespace → stats map (mirror of `NamespaceStatsMap`). */
export type NamespaceStatsMap = Record<string, NamespaceStats>;

/** `ServerClientConfig` wire shape: `timeout` is a serde `Duration` (secs+nanos). */
export interface ServerClientConfig {
  url: string;
  port: number;
  token?: string;
  timeout?: { secs: number; nanos?: number };
}

/** A single audit-log entry (VS-12). Mirrors `src/audit.rs` `AuditEvent`. */
export interface AuditEvent {
  /** ISO 8601 UTC timestamp (e.g. `2026-08-02T12:34:56Z`). */
  timestamp: string;
  /** Operation name: `put`, `delete`, `put_batch`, ... */
  op: string;
  namespace: string;
  /** Target record key, or `"N/A"` for operations without a single key. */
  key: string;
  /** `"ok"` or `"err"`. */
  outcome: string;
  /** Optional reason (e.g. the delete reason). */
  reason?: string | null;
}

/** A page of audit events, newest first, with the cursor for the next page. */
export interface AuditPage {
  events: AuditEvent[];
  /** Offset for the next older page; null/absent means last page. */
  next_cursor?: number | null;
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
  return transport.call<HealthReport>("vanta_health");
}

export function connectNative(path: string): Promise<ConnectionInfo> {
  return transport.call<ConnectionInfo>("vanta_connect", { target: { via: "native", path } });
}

export function connectServer(cfg: ServerClientConfig): Promise<ConnectionInfo> {
  return transport.call<ConnectionInfo>("vanta_connect", {
    target: {
      via: "server",
      config: { ...cfg, timeout: cfg.timeout ?? { secs: 15, nanos: 0 } },
    },
  });
}

export function disconnect(id: string): Promise<void> {
  return transport.call<void>("vanta_disconnect", { id });
}

/** Returns `[id, info]` pairs straight from Rust. */
export function listConnections(): Promise<[string, ConnectionInfo][]> {
  return transport.call<[string, ConnectionInfo][]>("vanta_list_connections");
}

export function setActive(id: string): Promise<void> {
  return transport.call<void>("vanta_set_active", { id });
}

export function ingest(records: IngestItem[]): Promise<string[]> {
  return transport.call<string[]>("vanta_ingest", { records });
}

export function ingestBatch(records: IngestItem[]): Promise<string[]> {
  return transport.call<string[]>("vanta_ingest_batch", { records });
}

export function search(query: SearchQuery): Promise<SearchResult[]> {
  return transport.call<SearchResult[]>("vanta_search", { query });
}

export function get(key: string, namespace?: string): Promise<MemoryRecord> {
  return transport.call<MemoryRecord>("vanta_get", { key, namespace });
}

/** Fetch a record as it was at a specific version (VS-CORE-07). Only the
 * embedded (native) connection implements version history. */
export function getVersion(
  key: string,
  version: number,
  namespace?: string,
): Promise<MemoryRecord> {
  return transport.call<MemoryRecord>("vanta_get_version", { key, version, namespace });
}

/** List every retained version of a record, ascending v1..vN (VS-CORE-07).
 * Only the embedded (native) connection implements version history. */
export function versions(key: string, namespace?: string): Promise<MemoryRecord[]> {
  return transport.call<MemoryRecord[]>("vanta_versions", { key, namespace });
}

export function remove(key: string, namespace?: string): Promise<void> {
  return transport.call<void>("vanta_delete", { key, namespace });
}

/** Upsert a record by key (create or replace), optionally pinning an absolute
 * unix-ms expiry. Returns the stored record. */
export function vantaPut(params: {
  namespace?: string;
  key: string;
  payload: string;
  metadata?: Record<string, unknown>;
  expires_at_ms?: number;
}): Promise<MemoryRecord> {
  return transport.call<MemoryRecord>("vanta_put", {
    namespace: params.namespace,
    key: params.key,
    payload: params.payload,
    metadata: params.metadata,
    expires_at_ms: params.expires_at_ms,
  });
}

/**
 * @deprecated Use `listPage` (VS-CORE-01) — kept so legacy components keep
 * receiving the bare record array. Returns only the first page's records.
 */
export function list(opts?: { namespace?: string; limit?: number }): Promise<MemoryRecord[]> {
  return listPage(opts).then((p) => p.records);
}

/** Paginated list: one page of records plus the cursor for the next page.
 * Pass `cursor` from a previous `next_cursor` to continue; a page with
 * `next_cursor: null` is the last one. */
export function listPage(opts?: {
  namespace?: string;
  limit?: number;
  cursor?: number;
}): Promise<ListPage> {
  return transport.call<ListPage>("vanta_list", {
    namespace: opts?.namespace,
    limit: opts?.limit,
    cursor: opts?.cursor,
  });
}

// --- IQL (VS-CORE-06) ------------------------------------------------------------
// Discriminated union mirroring the Rust `VantaQueryResult` wire DTO
// (externally tagged: the variant name is the discriminant).

export interface QueryWrite {
  affected_nodes: number;
  message: string;
  /** Node id as a string (u128 ids exceed JS safe integers). */
  node_id?: string | null;
}

export interface QueryStaleContext {
  node_id: string;
}

export type VantaQueryResult =
  | { Read: MemoryRecord[] }
  | { Write: QueryWrite }
  | { StaleContext: QueryStaleContext };

/** Execute an IQL statement against the active connection (VS-CORE-06).
 * Rejects with `unsupported` when the active connection has no IQL endpoint
 * (only the native/embedded transport implements it). */
export function queryIql(iql: string): Promise<VantaQueryResult> {
  return transport.call<VantaQueryResult>("vanta_query", { iql });
}

/** IQL editor autocomplete candidates for the token being typed (VS-CORE-06).
 * Pure string shim — keywords + identifiers already in the statement. */
export function iqlAutocomplete(prefix: string): Promise<string[]> {
  return transport.call<string[]>("vanta_iql_autocomplete", { prefix });
}

// --- Export (VS-CORE-04) ------------------------------------------------------
// Mirrors `MemoryFilterItem` / `ExportReport` in
// desktop/src-tauri/src/connections/types.rs. `op` stays PascalCase on the
// wire ("Eq"/"Neq"/"Gt"/...) — structurally compatible with
// `filters-core.ts`'s `VantaFilterItem` (query builder output).

export interface MemoryFilterItem {
  field: string;
  op: "Eq" | "Neq" | "Gt" | "Lt" | "Gte" | "Lte";
  /** JSON-able value (untagged — the bridge converts to the core `VantaValue`). */
  value: unknown;
}

export interface ExportReport {
  records_exported: number;
  namespaces: string[];
  path: string;
  duration_ms: number;
}

/** Export a namespace to a JSONL file (VS-CORE-04). Pass `filter` (AND-combined
 * metadata items, e.g. from the query builder) to export only matching records;
 * omit it to export the full namespace. */
export function exportNamespace(opts: {
  namespace: string;
  path: string;
  filter?: MemoryFilterItem[];
}): Promise<ExportReport> {
  return transport.call<ExportReport>("vanta_export_namespace", {
    namespace: opts.namespace,
    path: opts.path,
    filter: opts.filter ?? null,
  });
}

/** Delete every record in a namespace matching an AND-combined metadata filter
 * (VS-CORE-05). Returns the number of records deleted.
 *
 * The core rejects an empty filter to prevent accidental full-namespace
 * deletion — the rejection propagates as a rejected Promise; surface it to the
 * user (batch-delete confirmation lives in OP-02). */
export function deleteByFilter(opts: {
  namespace: string;
  filter: MemoryFilterItem[];
}): Promise<number> {
  return transport.call<number>("vanta_delete_by_filter", {
    namespace: opts.namespace,
    filter: opts.filter,
  });
}

// --- Graph (GRAFO-01) ------------------------------------------------------------
// Mirrors the wire DTOs in desktop/src-tauri/src/connections/types.rs. Node ids
// are strings (u128 ids exceed JS safe integers). `degree` is populated by
// `graphDegree`; traversals leave it at 0.

export interface VantaGraphNodeInfo {
  id: string;
  /** Display label (content/text payload, id fallback). */
  label: string;
  /** Grouping key for coloring (namespace or node type), when known. */
  group?: string | null;
  /** In+out degree centrality (0 when not computed). */
  degree?: number;
}

export interface VantaGraphEdgeInfo {
  source: string;
  target: string;
  label?: string | null;
  weight?: number | null;
}

export interface VantaGraphTraversalResult {
  nodes: VantaGraphNodeInfo[];
  edges: VantaGraphEdgeInfo[];
}

export type GraphDirection = "Forward" | "Reverse" | "Both";

/** Breadth-first graph traversal from root node ids (GRAFO-01). `direction`
 * controls which edges are followed; `limit` caps the result (default 50). */
export function graphBfs(opts: {
  roots: string[];
  maxDepth: number;
  direction?: GraphDirection;
  limit?: number;
}): Promise<VantaGraphTraversalResult> {
  return transport.call<VantaGraphTraversalResult>("vanta_graph_bfs", {
    roots: opts.roots,
    maxDepth: opts.maxDepth,
    direction: opts.direction ?? "Forward",
    limit: opts.limit ?? null,
  });
}

/** Depth-first graph traversal from root node ids (GRAFO-01). */
export function graphDfs(opts: {
  roots: string[];
  maxDepth: number;
  direction?: GraphDirection;
  limit?: number;
}): Promise<VantaGraphTraversalResult> {
  return transport.call<VantaGraphTraversalResult>("vanta_graph_dfs", {
    roots: opts.roots,
    maxDepth: opts.maxDepth,
    direction: opts.direction ?? "Forward",
    limit: opts.limit ?? null,
  });
}

/** Degree centrality (in+out) for every node in a namespace (GRAFO-01). An
 * empty/unknown namespace resolves to an empty array, not an error. */
export function graphDegree(opts: {
  namespace: string;
  limit?: number;
}): Promise<VantaGraphNodeInfo[]> {
  return transport.call<VantaGraphNodeInfo[]>("vanta_graph_degree", {
    namespace: opts.namespace,
    limit: opts.limit ?? null,
  });
}

// --- Metrics (ADMIN-01/04/05) ---------------------------------------------------
// Subset of `VantaOperationalMetrics` consumed by the dashboard grid (KPI cards
// + later live dashboard). Rust serializes every u64 field; we only declare
// the ones the UI reads. `mmap_resident_bytes` is `Option<u64>` → null on wire.
export interface OperationalMetrics {
  process_rss_bytes: number;
  records_imported: number;
  import_errors: number;
  text_lexical_queries: number;
  text_candidates_scored: number;
  planner_hybrid_queries: number;
  planner_text_only_queries: number;
  planner_vector_only_queries: number;
  derived_prefix_scans: number;
  derived_full_scan_fallbacks: number;
  startup_ms: number;
  wal_replay_ms: number;
  wal_records_replayed: number;
  ann_rebuild_ms: number;
  derived_rebuild_ms: number;
  text_index_rebuild_ms: number;
  text_postings_written: number;
  text_index_repairs: number;
  text_consistency_audits: number;
  text_consistency_audit_failures: number;
  mmap_resident_bytes: number | null;
  hnsw_logical_bytes: number;
  hnsw_nodes_count: number;
}

/** Point-in-time operational metrics snapshot (ADMIN-01). */
export function metrics(): Promise<OperationalMetrics> {
  return transport.call<OperationalMetrics>("vanta_metrics");
}

/** Per-namespace record statistics (VS-CORE-02). `count` includes expired
 * (not-yet-purged) records. Pass `expiringSoonWindowMs` to override the core's
 * 24h default. Rejects with `unsupported` when the active connection has no
 * stats endpoint (fall back to a client-side `list()` count). */
export function namespaceStats(expiringSoonWindowMs?: number): Promise<NamespaceStatsMap> {
  return transport.call<NamespaceStatsMap>("vanta_namespace_stats", {
    expiring_soon_window_ms: expiringSoonWindowMs ?? null,
  });
}

/** Audit-log events from the active connection, newest first (VS-12).
 * Rejects with `unsupported: audit log no configurado` when the active
 * connection has no audit log configured. */
export function auditEvents(opts?: {
  namespace?: string;
  op?: string;
  outcome?: string;
  limit?: number;
  cursor?: number;
}): Promise<AuditPage> {
  return transport.call<AuditPage>("vanta_audit_events", {
    namespace: opts?.namespace,
    op: opts?.op,
    outcome: opts?.outcome,
    limit: opts?.limit,
    cursor: opts?.cursor,
  });
}

// --- Deep links `vanta://` (VS-16) ---------------------------------------------
// Raw `vanta://` URLs are CLI-arg / OS-scheme input — treated as untrusted
// (official deep-link docs warn fake links can be passed as plain args).

/** Rust-emitted event name for `vanta://` arrivals while the app runs (VS-16).
 * Payload: `string[]` of raw URLs (mirrors `DEEP_LINK_EVENT` in lib.rs). */
export const DEEP_LINK_EVENT = "vanta-deep-link";

/** Parsed `vanta://` URL (VS-16). `query` preserves the raw query string so
 * callers decide how to interpret it (semantic query, key lookup, …). */
export interface VantaDeepLink {
  namespace: string | null;
  key: string | null;
  query: string | null;
}

/** Strictly parse a raw `vanta://…` URL (VS-16).
 * Grammar: `vanta://[namespace][/key][?query=…]` — empty segments collapse to
 * null. Returns null for anything that isn't `vanta://`-prefixed, so callers
 * can safely ignore fake/invalid links (official security caution). */
export function parseVantaUrl(raw: string): VantaDeepLink | null {
  const rest = raw.trim();
  if (!rest.startsWith("vanta://")) return null;
  const afterScheme = rest.slice("vanta://".length);
  const qIdx = afterScheme.indexOf("?");
  const rawPath = (qIdx === -1 ? afterScheme : afterScheme.slice(0, qIdx)).replace(/\/+$/, "");
  const queryPart = qIdx === -1 ? "" : afterScheme.slice(qIdx + 1);
  // Split first, then percent-decode each segment — decoding the whole path
  // before splitting would break `%2F` (encoded "/") inside a segment.
  const rawSegs = rawPath.split("/").filter((s) => s.length > 0);
  let segs: string[];
  try {
    segs = rawSegs.map((s) => decodeURIComponent(s));
  } catch {
    return null; // malformed percent-encoding → invalid link
  }
  const namespace = segs[0] ?? null;
  const key = segs[1] ?? null;
  const query =
    queryPart.length > 0 ? new URLSearchParams(queryPart).get("query") : null;
  if (!namespace && !key && query === null) return null;
  return { namespace, key, query };
}

/** Drain `vanta://` URLs buffered while the frontend was loading (VS-16).
 * Safe to call at startup — returns [] when nothing is pending.
 * Tauri-only: deep links are an OS-scheme/IPC feature, so this is a no-op
 * (empty array) on any other transport (web/WASM, WEB-04+). */
export function takeDeepLink(): Promise<string[]> {
  if (transport instanceof TauriBackend) {
    return transport.call<string[]>("vanta_deep_link_take");
  }
  return Promise.resolve([]);
}