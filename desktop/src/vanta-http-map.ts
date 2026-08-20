// Command → REST endpoint mapping for the web transport (WEB-04).
//
// Every `vanta_*` command wrapper in vanta.ts resolves here to a real endpoint
// of the embedded server's `/api/v2/*` surface. Source of truth for paths and
// wire shapes: src/cli_server.rs (route table + handlers). Request/response
// adaptation mirrors the desktop Tauri bridge
// (desktop/src-tauri/src/connections/native.rs) so the React components keep
// receiving the exact desktop DTOs — components read `record.text`/`record.id`,
// the SDK wire sends `key`/`payload`, and this file bridges the two.
//
// Commands with NO wire-compatible REST endpoint (multi-connection only)
// are rejected with a descriptive error — never an invented path.
// vanta.ts must NOT change signature (its callers are the 33 components using
// `../vanta`).
import type {
  IngestItem,
  ListPage,
  MemoryFilterItem,
  MemoryRecord,
  SearchQuery,
  SearchResult,
  VantaQueryResult,
} from "./vanta";

// --- Wire adaptation (mirror of native.rs to_vanta_value/from_vanta_value) ---

/** Plain JSON → SDK `VantaValue` (externally tagged: `{"String":"x"}`). */
function toVantaValue(v: unknown): unknown {
  if (v === null) return { Null: null };
  if (typeof v === "boolean") return { Bool: v };
  if (typeof v === "number") return Number.isInteger(v) ? { Int: v } : { Float: v };
  if (typeof v === "string") return { String: v };
  if (Array.isArray(v)) {
    if (v.every((x) => typeof x === "string")) return { ListString: v };
    if (v.every((x) => typeof x === "boolean")) return { ListBool: v };
    if (v.every((x) => Number.isInteger(x))) return { ListInt: v };
    if (v.every((x) => typeof x === "number")) return { ListFloat: v };
    return { String: JSON.stringify(v) };
  }
  // Objects (and anything else) collapse to their JSON string, like the bridge.
  return { String: JSON.stringify(v) };
}

/** SDK `VantaValue` (tagged) → plain JSON (mirror of native.rs from_vanta_value). */
export function fromVantaValue(v: unknown): unknown {
  if (v && typeof v === "object") {
    const entry = Object.entries(v as Record<string, unknown>)[0];
    if (entry) return entry[1];
  }
  return v;
}

function mapValues(
  o: Record<string, unknown> | undefined,
  f: (v: unknown) => unknown,
): Record<string, unknown> {
  return Object.fromEntries(Object.entries(o ?? {}).map(([k, v]) => [k, f(v)]));
}

/** SDK `VantaMemoryRecord` → desktop `MemoryRecord` (native.rs record_to_memory). */
export function recordFromSdk(r: Record<string, unknown>): MemoryRecord {
  return {
    id: r.key as string,
    namespace: r.namespace as string,
    text: r.payload as string,
    vector: (r.vector as number[] | null) ?? null,
    metadata: mapValues(r.metadata as Record<string, unknown> | undefined, fromVantaValue),
    created_at_ms: (r.created_at_ms as number | null) ?? null,
    updated_at_ms: (r.updated_at_ms as number | null) ?? null,
    version: (r.version as number | null) ?? null,
    node_id: r.node_id != null ? String(r.node_id) : null,
    sparse_vector: (r.sparse_vector as Record<string, number> | null) ?? null,
    expires_at_ms: (r.expires_at_ms as number | null) ?? null,
  };
}

/** SDK `VantaMemorySearchHit` → desktop `SearchResult`. */
export function searchHitFromSdk(hit: Record<string, unknown>): SearchResult {
  const rec = (hit.record ?? {}) as Record<string, unknown>;
  return {
    id: rec.key as string,
    namespace: rec.namespace as string,
    text: rec.payload as string,
    score: hit.score as number,
    metadata: mapValues(rec.metadata as Record<string, unknown> | undefined, fromVantaValue),
    explanation: (hit.explanation ?? null) as SearchResult["explanation"],
  };
}

/** Desktop `IngestItem` → SDK `VantaMemoryInput` (native.rs ingest_to_input). */
export function ingestToInput(item: IngestItem, key: string): Record<string, unknown> {
  return {
    namespace: item.namespace ?? "",
    key,
    payload: item.text,
    metadata: mapValues(item.metadata, toVantaValue),
    vector: item.embedding ?? null,
    sparse_vector: null,
    ttl_ms: null,
  };
}

let idSeq = 0;
/** Bridge `gen_id` equivalent: `rec_{now_ms}_{seq}` (native.rs:127). */
export function genId(): string {
  const now = typeof performance !== "undefined" ? performance.timeOrigin + performance.now() : Date.now();
  return `rec_${Math.floor(now)}_${idSeq++}`;
}

/** Desktop `SearchQuery` → SDK `VantaMemorySearchRequest` (native.rs search_request). */
export function searchToRequest(q: SearchQuery): Record<string, unknown> {
  const text = q.query.trim();
  return {
    namespace: q.namespace ?? "",
    query_vector: q.embedding ?? [],
    query_sparse: null,
    filters: mapValues(q.filters, toVantaValue),
    text_query: text.length > 0 ? text : null,
    top_k: q.top_k ?? 10,
    distance_metric: "Cosine",
    explain: q.explain ?? false,
  };
}

/** Desktop `MemoryFilterItem[]` → SDK `VantaMemoryFilter` (tagged values). */
export function filterToWire(filter: MemoryFilterItem[]): Record<string, unknown>[] {
  return filter.map((f) => ({ field: f.field, op: f.op, value: toVantaValue(f.value) }));
}

/** IQL NodeDTO (relational is plain FieldValue JSON, NOT VantaValue-tagged). */
function nodeDtoToMemoryRecord(n: Record<string, unknown>): MemoryRecord {
  const relational = (n.relational ?? {}) as Record<string, unknown>;
  const id = String(n.id);
  const text =
    ["__vanta_payload", "text", "content"]
      .map((k) => relational[k])
      .find((v): v is string => typeof v === "string") ?? "";
  const namespace =
    typeof relational.__vanta_namespace === "string" ? relational.__vanta_namespace : "";
  const metadata = Object.fromEntries(
    Object.entries(relational).filter(([k]) => !k.startsWith("__vanta_")),
  );
  return { id, namespace, text, metadata, node_id: id };
}

/** `QueryResponse` → desktop `VantaQueryResult` (IQL; cli_server.rs execute_query). */
function queryResultFromResponse(r: Record<string, unknown>): VantaQueryResult {
  if (r.success === false) throw new Error(typeof r.data === "string" ? r.data : "query failed");
  if (Array.isArray(r.nodes)) {
    return { Read: r.nodes.map((n) => nodeDtoToMemoryRecord(n as Record<string, unknown>)) };
  }
  const data = typeof r.data === "string" ? r.data : "";
  const mutated = /^Mutated (\d+) nodes: (.*)$/s.exec(data);
  if (mutated) {
    return {
      Write: {
        affected_nodes: Number(mutated[1]),
        message: mutated[2],
        node_id: r.node_id != null ? String(r.node_id) : null,
      },
    };
  }
  // StaleContext (data starts with "STALE_CONTEXT").
  return { StaleContext: { node_id: String(r.node_id) } };
}

// --- Mapping table -------------------------------------------------------------

export interface HttpMapping {
  method: string;
  path: (args: Record<string, unknown>) => string;
  /** Query params; undefined/empty values are dropped. */
  query?: (args: Record<string, unknown>) => Record<string, unknown> | undefined;
  /** JSON body; absent when undefined. */
  body?: (args: Record<string, unknown>) => unknown;
  /** Response adaptation (SDK wire → desktop DTO). Defaults to passthrough. */
  transform?: (data: unknown) => unknown;
}

/** Commands with no wire-compatible REST endpoint (documented divergence). */
const unsupported = (reason: string) => reason;

const mappings: Record<string, HttpMapping | string> = {
  // --- Health ---
  vanta_health: {
    method: "GET",
    path: () => "/api/v2/health",
    // HealthReportV2 serializes identically to the desktop HealthReport.
  },

  // --- Multi-connection is Tauri-only: web mode = one implicit server connection.
  vanta_connect: unsupported(
    "multi-connection management is Tauri-only; the web transport talks to the embedded server directly",
  ),
  vanta_disconnect: unsupported(
    "multi-connection management is Tauri-only; the web transport talks to the embedded server directly",
  ),
  vanta_list_connections: unsupported(
    "multi-connection management is Tauri-only; the web transport talks to the embedded server directly",
  ),
  vanta_set_active: unsupported(
    "multi-connection management is Tauri-only; the web transport talks to the embedded server directly",
  ),

  // --- Records ---
  vanta_ingest: {
    method: "POST",
    path: () => "/api/v2/records/batch",
    body: (args) => {
      const records = (args.records ?? []) as IngestItem[];
      return records.map((item) => ingestToInput(item, item.id ?? genId()));
    },
    transform: (data) =>
      ((data as Record<string, unknown>[]) ?? []).map((r) => r.key ?? String(r.id)),
  },
  vanta_ingest_batch: {
    method: "POST",
    path: () => "/api/v2/records/batch",
    body: (args) => {
      const records = (args.records ?? []) as IngestItem[];
      return records.map((item) => ingestToInput(item, item.id ?? genId()));
    },
    transform: (data) =>
      ((data as Record<string, unknown>[]) ?? []).map((r) => r.key ?? String(r.id)),
  },
  vanta_search: {
    method: "POST",
    path: () => "/api/v2/search",
    body: (args) => searchToRequest(args.query as SearchQuery),
    // REST-04: el server devuelve {records, next_cursor}; el desktop search()
    // es one-shot (firma de vanta.ts intacta) → unwrap de records.
    transform: (data) => {
      const page = (data ?? {}) as { records?: Record<string, unknown>[] };
      return (page.records ?? []).map((h) =>
        searchHitFromSdk(h as Record<string, unknown>),
      );
    },
  },
  vanta_get: {
    method: "GET",
    path: (args) => {
      const ns = encodeURIComponent((args.namespace as string) ?? "");
      const key = encodeURIComponent(args.key as string);
      return `/api/v2/records/${ns}/${key}`;
    },
    transform: (data) => recordFromSdk(data as Record<string, unknown>),
  },
  vanta_get_version: {
    method: "GET",
    path: (args) => {
      const ns = encodeURIComponent((args.namespace as string) ?? "");
      const key = encodeURIComponent(args.key as string);
      return `/api/v2/records/${ns}/${key}/versions`;
    },
    query: (args) => ({ version: args.version }),
    transform: (data) => recordFromSdk(data as Record<string, unknown>),
  },
  vanta_versions: {
    method: "GET",
    path: (args) => {
      const ns = encodeURIComponent((args.namespace as string) ?? "");
      const key = encodeURIComponent(args.key as string);
      return `/api/v2/records/${ns}/${key}/versions`;
    },
    transform: (data) =>
      ((data as Record<string, unknown>[]) ?? []).map((r) =>
        recordFromSdk(r as Record<string, unknown>),
      ),
  },
  vanta_put: {
    method: "POST",
    path: () => "/api/v2/records",
    body: (args) => {
      const expires = args.expires_at_ms as number | undefined;
      const input = ingestToInput(
        { id: args.key as string, text: args.payload as string, namespace: args.namespace as string, metadata: args.metadata as Record<string, unknown> },
        args.key as string,
      );
      // Core `put` takes a relative ttl; convert absolute unix-ms → ttl (native.rs:459).
      input.ttl_ms = expires ? Math.max(0, expires - Date.now()) : null;
      return input;
    },
    transform: (data) => recordFromSdk(data as Record<string, unknown>),
  },
  vanta_delete: {
    method: "DELETE",
    path: (args) => {
      const ns = encodeURIComponent((args.namespace as string) ?? "");
      const key = encodeURIComponent(args.key as string);
      return `/api/v2/records/${ns}/${key}`;
    },
    transform: () => undefined,
  },
  vanta_list: {
    method: "GET",
    path: () => "/api/v2/list",
    query: (args) => ({
      namespace: args.namespace,
      limit: args.limit,
      cursor: args.cursor,
      filter_ops: args.filter_ops,
    }),
    transform: (data) => {
      const page = (data ?? {}) as { records?: Record<string, unknown>[]; next_cursor?: number | null };
      return {
        records: (page.records ?? []).map((r) => recordFromSdk(r)),
        next_cursor: page.next_cursor ?? null,
      } as ListPage;
    },
  },
  vanta_delete_by_filter: {
    method: "DELETE",
    path: () => "/api/v2/records",
    query: (args) => ({
      namespace: args.namespace,
      filter: JSON.stringify(filterToWire(args.filter as MemoryFilterItem[])),
    }),
    transform: (data) => ((data as Record<string, unknown>).deleted as number) ?? 0,
  },

  // --- IQL (VS-CORE-06) ---
  vanta_query: {
    method: "POST",
    path: () => "/api/v2/query",
    body: (args) => ({ query: args.iql as string }),
    transform: (data) => queryResultFromResponse(data as Record<string, unknown>),
  },
  vanta_iql_autocomplete: {
    method: "GET",
    path: () => "/api/v2/autocomplete",
    query: (args) => ({ prefix: args.prefix }),
  },

  // --- Export (VS-CORE-04) ---
  vanta_export_namespace: {
    method: "POST",
    path: () => "/api/v2/export",
    body: (args) => {
      const filter = args.filter as MemoryFilterItem[] | null | undefined;
      return {
        path: args.path,
        namespace: args.namespace,
        filter: filter && filter.length > 0 ? filterToWire(filter) : null,
      };
    },
    // VantaExportReport serializes identically to the desktop ExportReport.
  },

  // --- Graph (GRAFO-01 / REST-03): the graph_v2 endpoints mirror the desktop
  // DTOs (desktop/src-tauri/src/connections/types.rs) with u128 node/edge ids
  // as decimal strings, so ids above u64::MAX survive the JSON wire (the
  // legacy /api/v2/graph/* endpoints return bare u128 values the browser
  // cannot parse). Responses are passthrough: GraphTraversalDTO ↔
  // VantaGraphTraversalResult, GraphNodeDTO[] ↔ VantaGraphNodeInfo[].
  vanta_graph_bfs: {
    method: "POST",
    path: () => "/api/v2/graph/v2/bfs",
    body: (args) => ({
      roots: args.roots,
      max_depth: args.maxDepth,
      direction: ((args.direction as string) ?? "Forward").toLowerCase(),
      limit: args.limit ?? null,
    }),
  },
  vanta_graph_dfs: {
    method: "POST",
    path: () => "/api/v2/graph/v2/dfs",
    body: (args) => ({
      roots: args.roots,
      max_depth: args.maxDepth,
      direction: ((args.direction as string) ?? "Forward").toLowerCase(),
      limit: args.limit ?? null,
    }),
  },
  vanta_graph_degree: {
    method: "POST",
    path: () => "/api/v2/graph/v2/degree",
    body: (args) => ({
      namespace: args.namespace,
      limit: args.limit ?? null,
    }),
  },

  // --- Metrics (ADMIN-01 / REST-02): /api/v2/metrics returns
  // { metrics: VantaOperationalMetrics, namespaces: {...} }; the desktop
  // OperationalMetrics interface is a subset of `metrics`.
  vanta_metrics: {
    method: "GET",
    path: () => "/api/v2/metrics",
    transform: (data) => (data as { metrics: unknown }).metrics,
  },
  // vanta_namespace_stats (VS-CORE-02) reuses the same endpoint — the
  // per-namespace half of the payload is the NamespaceStatsMap.
  vanta_namespace_stats: {
    method: "GET",
    path: () => "/api/v2/metrics",
    transform: (data) => (data as { namespaces: unknown }).namespaces,
  },

  // --- Audit (VS-12) ---
  vanta_audit_events: {
    method: "GET",
    path: () => "/api/v2/audit",
    query: (args) => ({
      namespace: args.namespace,
      op: args.op,
      outcome: args.outcome,
      limit: args.limit,
      cursor: args.cursor,
    }),
    // AuditPageV2 {events, next_cursor} serializes identically to AuditPage.
  },
};

/**
 * Resolve a `vanta_*` command to its HTTP mapping.
 * @throws Error with a descriptive message when the command has no wire-
 * compatible REST endpoint (connection management).
 */
export function getHttpMapping(cmd: string): HttpMapping {
  const m = mappings[cmd];
  if (m === undefined) {
    throw new Error(`[vanta] ${cmd}: no REST endpoint in vanta-http-map.ts`);
  }
  if (typeof m === "string") {
    throw new Error(`[vanta] ${cmd}: ${m}`);
  }
  return m;
}

/** Commands that resolve to a real endpoint (for coverage tests). */
export function mappedCommands(): string[] {
  return Object.entries(mappings)
    .filter(([, v]) => typeof v === "object")
    .map(([cmd]) => cmd);
}

/** Commands rejected as wire-incompatible (documented divergences). */
export function unsupportedCommands(): string[] {
  return Object.entries(mappings)
    .filter(([, v]) => typeof v === "string")
    .map(([cmd]) => cmd);
}