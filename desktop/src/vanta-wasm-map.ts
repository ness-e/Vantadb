// Command → WASM method mapping for the standalone browser transport (WASM-02).
//
// Every `vanta_*` command wrapper in vanta.ts resolves here to a method on the
// `VantaDB` class exposed by `vantadb-wasm/pkg` (wasm-bindgen). The wrapper is
// thin (1:1 with the generated API); this file adapts the WASM wire to the
// desktop DTOs the same way vanta-http-map.ts does for REST — components read
// `record.text`/`record.id`, the WASM wire sends `key`/`payload` with
// VantaValue-tagged metadata, and the shared adapters from vanta-http-map.ts
// bridge the two.
//
// Commands with NO wire-compatible WASM method (multi-connection, version
// history, audit log, graph DTOs, export-to-path) are rejected with a
// descriptive error — never an invented call (WEB-04 pattern).
import type { VantaDB } from "../../vantadb-wasm/pkg/vantadb_wasm.js";
import type {
  HealthReport,
  IngestItem,
  ListPage,
  MemoryFilterItem,
  MemoryRecord,
  OperationalMetrics,
  SearchQuery,
} from "./vanta";
import {
  filterToWire,
  genId,
  ingestToInput,
  recordFromSdk,
  searchHitFromSdk,
  searchToRequest,
} from "./vanta-http-map.ts";

export interface WasmMapping {
  /** Invoke the mapped WASM method and adapt the result to the desktop DTO. */
  run: (db: VantaDB, args: Record<string, unknown>) => unknown;
  /** Mutating commands persist to OPFS/IDB after running (differential save). */
  persist?: boolean;
}

/** Commands with no wire-compatible WASM method (documented divergence). */
const unsupported = (reason: string) => reason;

const mappings: Record<string, WasmMapping | string> = {
  // --- Health ---
  vanta_health: {
    run: (db) => {
      // Engine liveness probe: metrics resolve only while the DB is open.
      db.operational_metrics();
      return {
        status: "healthy",
        backend: "wasm",
        latency_ms: 0,
        checked_at_ms: Date.now(),
      } satisfies HealthReport;
    },
  },

  // --- Multi-connection is Tauri-only: WASM owns one implicit persistent DB.
  vanta_connect: unsupported(
    "multi-connection management is Tauri-only; the WASM transport owns one implicit database",
  ),
  vanta_disconnect: unsupported(
    "multi-connection management is Tauri-only; the WASM transport owns one implicit database",
  ),
  vanta_list_connections: unsupported(
    "multi-connection management is Tauri-only; the WASM transport owns one implicit database",
  ),
  vanta_set_active: unsupported(
    "multi-connection management is Tauri-only; the WASM transport owns one implicit database",
  ),

  // --- Records ---
  vanta_ingest: {
    persist: true,
    run: (db, args) => {
      const records = (args.records ?? []) as IngestItem[];
      const inputs = records.map((item) => ingestToInput(item, item.id ?? genId()));
      return (db.put_batch(inputs) as Record<string, unknown>[]).map((r) => r.key as string);
    },
  },
  vanta_ingest_batch: {
    persist: true,
    run: (db, args) => {
      const records = (args.records ?? []) as IngestItem[];
      const inputs = records.map((item) => ingestToInput(item, item.id ?? genId()));
      return (db.put_batch(inputs) as Record<string, unknown>[]).map((r) => r.key as string);
    },
  },
  vanta_search: {
    run: (db, args) => {
      const hits = db.search(searchToRequest(args.query as SearchQuery)) as Record<string, unknown>[];
      return (hits ?? []).map((h) => searchHitFromSdk(h));
    },
  },
  vanta_get: {
    run: (db, args) => {
      const ns = (args.namespace as string) ?? "";
      const rec = db.get(ns, args.key as string);
      if (rec == null) {
        throw new Error(`record not found: ${ns}/${args.key}`);
      }
      return wasmRecordFromSdk(rec as Record<string, unknown>);
    },
  },
  vanta_get_version: unsupported("version history is native-only; the WASM get() has no version parameter"),
  vanta_versions: unsupported("version history is native-only; the WASM binding has no versions() method"),
  vanta_delete: {
    persist: true,
    run: (db, args) => {
      db.delete((args.namespace as string) ?? "", args.key as string);
    },
  },
  vanta_put: {
    persist: true,
    run: (db, args) => {
      const expires = args.expires_at_ms as number | undefined;
      const input = ingestToInput(
        {
          id: args.key as string,
          text: args.payload as string,
          namespace: args.namespace as string,
          metadata: args.metadata as Record<string, unknown>,
        },
        args.key as string,
      );
      input.ttl_ms = expires ? Math.max(0, expires - Date.now()) : null;
      return wasmRecordFromSdk(db.put(input) as Record<string, unknown>);
    },
  },
  vanta_list: {
    run: (db, args) => {
      const opts: Record<string, unknown> = {};
      if (args.limit != null) opts.limit = args.limit;
      if (args.cursor != null) opts.cursor = args.cursor;
      const page = db.list((args.namespace as string) ?? "", opts) as {
        records?: Record<string, unknown>[];
        next_cursor?: number | null;
      };
      return {
        records: (page.records ?? []).map((r) => wasmRecordFromSdk(r)),
        next_cursor: page.next_cursor ?? null,
      } satisfies ListPage;
    },
  },
  vanta_delete_by_filter: {
    persist: true,
    // wasm-bindgen maps u64 → JS bigint; the desktop surface is a number.
    run: (db, args) =>
      Number(db.delete_by_filter(args.namespace as string, filterToWire(args.filter as MemoryFilterItem[]))),
  },

  // --- IQL (VS-CORE-06): the WASM binding's query() resolves reads against
  // the graph-store, not the memory record store — `SELECT * FROM ns` returns
  // `{Read: []}` even with records present (verified against the engine). A
  // silent-empty read is worse than an honest degradation, so IQL is
  // unsupported until the engine's wasm query path surfaces memory records.
  vanta_query: unsupported(
    "IQL queries against the WASM binding resolve reads against the graph store, not memory records (SELECT returns empty despite records existing); requires engine work before wiring the IQL panel",
  ),
  vanta_iql_autocomplete: unsupported(
    "IQL autocomplete is a server-side shim; the WASM binding has no autocomplete method",
  ),

  // --- Export (VS-CORE-04): export_namespace(path, ...) writes to a storage
  // path; in the browser there is no user-visible file path (Tauri save
  // dialog / server filesystem) — the desktop export flow cannot be satisfied.
  vanta_export_namespace: unsupported(
    "export writes to a filesystem path (Tauri save dialog / server); the browser WASM transport has no file path",
  ),

  // --- Graph (GRAFO-01): the WASM binding returns visited node ids
  // (Vec<u128>) or in/out degree counts — the desktop surface requires the
  // {nodes, edges} DTO. No wire-compatible mapping.
  vanta_graph_bfs: unsupported(
    "WASM graph_bfs returns visited node ids (Vec<u128>), not the desktop {nodes, edges} DTO",
  ),
  vanta_graph_dfs: unsupported(
    "WASM graph_dfs returns visited node ids (Vec<u128>), not the desktop {nodes, edges} DTO",
  ),
  vanta_graph_degree: unsupported(
    "WASM graph_degree is root-based ({id, in_degree, out_degree}); the desktop surface is namespace-based ({id, label, group, degree})",
  ),

  // --- Metrics (ADMIN-01): operational_metrics() stringifies u64 fields for
  // JS Number safety; the desktop interface is numeric — coerce on read.
  vanta_metrics: {
    run: (db) => metricsFromWasm(db.operational_metrics() as Record<string, unknown>),
  },
  // vanta_namespace_stats (VS-CORE-02): no per-namespace stats method on the
  // WASM binding — callers fall back to a client-side list() count (vanta.ts).
  vanta_namespace_stats: unsupported(
    "per-namespace stats are server/native-only; the WASM binding has no stats method (fall back to list() counts)",
  ),

  // --- Audit (VS-12) ---
  vanta_audit_events: unsupported("audit log is server/native-only; the WASM binding has no audit method"),
};

/**
 * WASM wire record → desktop MemoryRecord. The binding stringifies u64
 * timestamps (memory_record_to_js uses .to_string()) and serializes the dense
 * vector as a Float32Array — coerce both to the numeric desktop surface that
 * recordFromSdk's casts only pretend to guarantee.
 */
function wasmRecordFromSdk(r: Record<string, unknown>): MemoryRecord {
  const rec = recordFromSdk(r);
  return {
    ...rec,
    created_at_ms: rec.created_at_ms == null ? null : Number(rec.created_at_ms),
    updated_at_ms: rec.updated_at_ms == null ? null : Number(rec.updated_at_ms),
    version: rec.version == null ? null : Number(rec.version),
    expires_at_ms: rec.expires_at_ms == null ? null : Number(rec.expires_at_ms),
    vector: r.vector instanceof Float32Array ? Array.from(r.vector) : rec.vector,
  };
}

/** WASM operational_metrics (u64 → decimal string) → desktop numeric interface. */
function metricsFromWasm(m: Record<string, unknown>): OperationalMetrics {
  return Object.fromEntries(
    Object.entries(m).map(([k, v]) => [k, v == null ? null : Number(v)]),
  ) as unknown as OperationalMetrics;
}

/**
 * Resolve a `vanta_*` command to its WASM mapping.
 * @throws Error with a descriptive message when the command has no wire-
 * compatible WASM method (connection management, graph DTOs, export, ...).
 */
export function getWasmMapping(cmd: string): WasmMapping {
  const m = mappings[cmd];
  if (m === undefined) {
    throw new Error(`[vanta] ${cmd}: no WASM method in vanta-wasm-map.ts`);
  }
  if (typeof m === "string") {
    throw new Error(`[vanta] ${cmd}: ${m}`);
  }
  return m;
}

/** Commands that map to a real WASM method (for coverage tests). */
export function wasmMappedCommands(): string[] {
  return Object.entries(mappings)
    .filter(([, v]) => typeof v === "object")
    .map(([cmd]) => cmd);
}

/** Commands rejected as wire-incompatible (documented divergences). */
export function wasmUnsupportedCommands(): string[] {
  return Object.entries(mappings)
    .filter(([, v]) => typeof v === "string")
    .map(([cmd]) => cmd);
}