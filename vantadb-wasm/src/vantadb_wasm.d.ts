/* tslint:disable */
/* eslint-disable */
/**
 * Hand-written TypeScript declarations for `vantadb-wasm`.
 *
 * This file overrides the .d.ts that wasm-pack auto-generates from the
 * `#[wasm_bindgen]` Rust signatures. wasm-bindgen types most cross-boundary
 * values as `any` (limitation of the macro), which makes the npm package
 * unusable in TypeScript projects without a wrapper layer.
 *
 * The hand-written version below mirrors the runtime contract from
 * `vantadb-wasm/src/lib.rs` (the source of truth) so consumers get:
 *  - named input interfaces (`MemoryRecordInput`, `SearchRequest`, …)
 *  - named output interfaces (`MemoryRecord`, `SearchHit`, …)
 *  - typed enums for the few string-flagged arguments
 *  - JSDoc on every method (copied from the Rust doc comments)
 *
 * Build wiring: `dev-tools/build-wasm-types.mjs` replaces the generated
 * `export class VantaDB` block in `pkg/vantadb_wasm.d.ts` with the
 * contents of this file after `wasm-pack build`. The generated `InitInput`,
 * `InitOutput`, and `SyncInitInput` types are preserved unchanged because
 * they map 1:1 to the wasm-bindgen runtime glue.
 *
 * Source of truth: `vantadb-wasm/src/lib.rs`. When the Rust signatures
 * change, update this file in the same PR.
 */

// ─────────────────────────────────────────────────────────────────────────────
// WebAssembly init types (preserved from generated .d.ts verbatim — these are
// the runtime glue wasm-bindgen expects and must not be edited)
// ─────────────────────────────────────────────────────────────────────────────

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

// `InitOutput` is the FFI glue wasm-bindgen exposes after `initSync` /
// `__wbg_init`. Consumers do NOT touch this directly — they use the high-
// level `VantaDB` class. The pointer-based signatures here are 1:1 with the
// generated WebAssembly bindings and use `unknown` (not `any`) for the few
// JsValue slots, so the file stays type-safe without `any`. The bindings
// accept `unknown` at the call site because `unknown` is the top type.
export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_vantadb_free: (a: number, b: number) => void;
    readonly vantadb_add_edge: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: bigint) => [number, number];
    readonly vantadb_audit_text_index: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_audit_text_index_deep: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_bulk_import: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_bulk_import_bytes: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_capabilities: (a: number) => [number, number, number];
    readonly vantadb_close: (a: number) => [number, number];
    readonly vantadb_compact_layout: (a: number) => [bigint, number, number];
    readonly vantadb_compact_wal: (a: number) => [number, number];
    readonly vantadb_connect_idb: (a: number, b: number) => [number, number, number];
    readonly vantadb_connect_persistent: (a: number, b: number) => [number, number, number];
    readonly vantadb_count: (a: number, b: number, c: number, d: number, e: number) => [bigint, number, number];
    readonly vantadb_delete: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly vantadb_delete_by_filter: (a: number, b: number, c: number, d: number, e: number) => [bigint, number, number];
    readonly vantadb_delete_idb: (a: number) => [number, number, number];
    readonly vantadb_delete_node: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly vantadb_disable_auto_save: (a: number) => void;
    readonly vantadb_enable_auto_save: (a: number) => void;
    readonly vantadb_explain_memory_search: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_export_all: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_export_namespace: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly vantadb_export_namespace_filtered: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly vantadb_flush: (a: number) => [number, number];
    readonly vantadb_generate_snippet: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly vantadb_get: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly vantadb_get_node: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_graph_bfs: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly vantadb_graph_degree: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_graph_dfs: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly vantadb_graph_filtered_traversal: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number];
    readonly vantadb_graph_is_dag: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_graph_topological_sort: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_import_file: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_import_records: (a: number, b: number) => [number, number, number];
    readonly vantadb_insert_node: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number];
    readonly vantadb_is_auto_save_enabled: (a: number) => number;
    readonly vantadb_list: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly vantadb_list_namespaces: (a: number) => [number, number, number];
    readonly vantadb_load: (a: number) => [number, number, number];
    readonly vantadb_load_idb: (a: number) => [number, number, number];
    readonly vantadb_new: (a: number) => [number, number, number];
    readonly vantadb_open: (a: number, b: number) => [number, number, number];
    readonly vantadb_operational_metrics: (a: number) => [number, number, number];
    readonly vantadb_purge_expired: (a: number) => [bigint, number, number];
    readonly vantadb_put: (a: number, b: number) => [number, number, number];
    readonly vantadb_put_batch: (a: number, b: number) => [number, number, number];
    readonly vantadb_query: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_rebuild_index: (a: number) => [number, number, number];
    readonly vantadb_reindex_hnsw_from_text: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly vantadb_remove_edge: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
    readonly vantadb_repair_text_index: (a: number) => [number, number, number];
    readonly vantadb_save: (a: number) => [number, number, number];
    readonly vantadb_save_idb: (a: number) => [number, number, number];
    readonly vantadb_search: (a: number, b: number) => [number, number, number];
    readonly vantadb_search_multi: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_search_vector: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly vantadb_similar_to_key: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly vantadb_supersede: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
    readonly vantadb_try_auto_save: (a: number) => [number, number, number];
    readonly wasm_bindgen__convert__closures_____invoke__he5211670e4cc5c11: (a: number, b: number, c: unknown) => [number, number];
    readonly wasm_bindgen__convert__closures_____invoke__h0bf0d27f3df94d79: (a: number, b: number, c: unknown, d: unknown) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_destroy_closure: (a: number, b: number) => void;
    readonly __externref_table_dealloc: () => number;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

// ─────────────────────────────────────────────────────────────────────────────
// Domain types (the public surface for TypeScript consumers)
// ─────────────────────────────────────────────────────────────────────────────

/** WasmConfig accepted by the constructor (mirrors `WasmConfig` in `lib.rs`). */
export interface VantaConfigInput {
    /** Storage path (default: `"vantadb_data"`). For OPFS/IDB backends, this is the database name. */
    storage_path?: string;
    /** Open in read-only mode. */
    read_only?: boolean;
    /** RSS threshold (default: 0.80). */
    rss_threshold?: number;
    /** Optional memory limit in bytes. */
    memory_limit?: number;
}

/** One record stored in a memory namespace. Returned by `get`, `put`, `list`, `search`. */
export interface MemoryRecord {
    /** Namespace the record lives in. */
    namespace: string;
    /** Unique key within the namespace. */
    key: string;
    /** Text payload (always present). */
    payload: string;
    /** Unix-ms creation timestamp as a decimal string (policy string-u64). */
    created_at_ms: string;
    /** Unix-ms last-update timestamp as a decimal string. */
    updated_at_ms: string;
    /** Monotonic version counter as a decimal string. */
    version: string;
    /** Internal u128 node id as a decimal string. */
    node_id: string;
    /** Optional dense embedding vector (Float32Array on the wire, sanitized NaN/Inf → 0). */
    vector?: Float32Array;
    /** Optional TTL expiry as a decimal string (only present if set). */
    expires_at_ms?: string;
    /** Arbitrary metadata key-value pairs. Values are JSON-shaped. */
    metadata: Record<string, MetadataValue>;
}

/** A JSON-serializable metadata value. */
export type MetadataValue =
    | string
    | number
    | boolean
    | null
    | MetadataValue[]
    | { [k: string]: MetadataValue };

/** Input shape for `put`. Mirrors `MemoryInput` in `lib.rs`. */
export interface MemoryRecordInput {
    namespace: string;
    key: string;
    payload: string;
    metadata?: Record<string, MetadataValue>;
    /** Dense embedding vector. wasm-bindgen accepts `Float32Array` or a plain `number[]`. */
    vector?: Float32Array | number[] | null;
    /** Optional sparse term-weight vector. */
    sparse_vector?: SparseVector | null;
    /** Optional TTL as Unix-ms. */
    ttl_ms?: bigint | number | null;
}

/** Sparse term-weight vector: dimension id → weight. */
export type SparseVector = Record<string, number>;

/** AND-combined metadata filter. Mirrors `VantaMemoryFilterItem` from the core. */
export type MetadataFilter = MetadataFilterItem[];

/** One filter predicate inside a `MetadataFilter`. */
export interface MetadataFilterItem {
    /** Field name to test. */
    field: string;
    /** Comparison operator. */
    op: FilterOp;
    /** Value to compare against (same shape as metadata values). */
    value: MetadataValue;
}

/** Comparison operator for a `MetadataFilterItem`. */
export type FilterOp = "Eq" | "Neq" | "Gt" | "Lt" | "Gte" | "Lte";

/** Options for `list`. Mirrors `ListOptions` in `lib.rs`. */
export interface ListOptionsInput {
    /** AND-combined metadata filter; omit to list all records. */
    filters?: Record<string, MetadataValue>;
    /** Page size (default: 100). */
    limit?: number;
    /** Opaque cursor from a previous page's `next_cursor`; omit for first page. */
    cursor?: string | number | null;
}

/** Page returned by `list`. */
export interface ListPage {
    records: MemoryRecord[];
    /** Opaque next cursor (decimal string), absent on the last page. */
    next_cursor?: string;
}

/** Request shape for `search`. Mirrors `SearchRequest` in `lib.rs`. */
export interface SearchRequestInput {
    namespace: string;
    /** Query dense embedding vector. */
    query_vector: Float32Array | number[];
    /** Optional metadata filter. */
    filters?: Record<string, MetadataValue>;
    /** Hide records already superseded by another record. */
    exclude_superseded?: boolean;
    /** Optional text query for BM25 hybrid search. */
    text_query?: string;
    /** Top-K results to return (default: 10, hard cap: 1000). */
    top_k?: number;
    /** Distance metric: `"Cosine"` (default) or `"Euclidean"`. */
    distance_metric?: "Cosine" | "Euclidean";
    /** Include per-term explanation metadata in each hit. */
    explain?: boolean;
}

/** One hit returned by `search`. */
export interface SearchHit {
    record: MemoryRecord;
    /** Score (cosine similarity or negative distance depending on `distance_metric`). */
    score: number;
    /** Optional explanation (present when `explain: true` in the request). */
    explanation?: SearchExplanation;
}

/** Per-term explanation for a search hit (BSAFE-01 / hybrid RRF). */
export interface SearchExplanation {
    score: number;
    bm25_terms: Bm25TermExplanation[];
    vector_score?: number;
    text_score?: number;
}

export interface Bm25TermExplanation {
    term: string;
    contribution: number;
}

/** A graph node record (returned by `get_node`). */
export interface NodeRecord {
    /** u128 node id as a decimal string. */
    id: string;
    /** Free-form content associated with the node. */
    content?: string | null;
    /** Optional dense embedding vector. */
    vector?: number[];
    /** Vector dimensionality (0 if no vector). */
    vector_dimensions: number;
    /** Arbitrary structured fields. */
    fields: Record<string, MetadataValue>;
    /** Outgoing edges. */
    edges: EdgeRecord[];
    confidence_score: number;
    importance: number;
    hits: number;
    /** Last-accessed timestamp as a decimal string (policy string-u64). */
    last_accessed: string;
    epoch: number;
    tier: "Hot" | "Warm" | "Cold";
    is_alive: boolean;
}

/** One edge of a graph node. */
export interface EdgeRecord {
    target_id: string;
    label: string;
    weight: number;
    created_at_ms: string;
}

/** Input for `insert_node`. */
export interface NodeInsertInput {
    id: string;
    content?: string | null;
    vector?: Float32Array | number[] | null;
    fields?: Record<string, MetadataValue>;
}

/** Capabilities reported by `capabilities()`. */
export interface VantaCapabilities {
    /** True when the engine is backed by durable storage (OPFS/IDB/Worker). */
    persistence: boolean;
    /** Engine backend in use. */
    backend: string;
    /** Vector search support. */
    vector_search: boolean;
    /** IQL query support. */
    iql_queries: boolean;
    /** True if the engine is opened in read-only mode. */
    read_only: boolean;
}

/** Operational metrics returned by `operational_metrics()`. All numbers are decimal strings. */
export interface OperationalMetrics {
    startup_ms: string;
    wal_replay_ms: string;
    wal_records_replayed: string;
    ann_rebuild_ms: string;
    ann_rebuild_scanned_nodes: string;
    derived_rebuild_ms: string;
    text_index_rebuild_ms: string;
    text_postings_written: string;
    text_index_repairs: string;
    text_lexical_queries: string;
    text_lexical_query_ms: string;
    text_candidates_scored: string;
    text_consistency_audits: string;
    text_consistency_audit_failures: string;
    hybrid_query_ms: string;
    hybrid_candidates_fused: string;
    planner_hybrid_queries: string;
    planner_text_only_queries: string;
    planner_vector_only_queries: string;
    records_exported: string;
    records_imported: string;
    import_errors: string;
    derived_prefix_scans: string;
    derived_full_scan_fallbacks: string;
    process_rss_bytes: string;
    process_virtual_bytes: string;
    hnsw_nodes_count: string;
    hnsw_logical_bytes: string;
    mmap_resident_bytes?: string;
    volatile_cache_entries: string;
    volatile_cache_cap_bytes: string;
    jemalloc_allocated_bytes?: string;
    jemalloc_active_bytes?: string;
    jemalloc_metadata_bytes?: string;
    jemalloc_resident_bytes?: string;
    jemalloc_mapped_bytes?: string;
    jemalloc_retained_bytes?: string;
}

/** Report returned by import / bulk-import operations. */
export interface ImportReport {
    records_imported: number;
    records_skipped?: number;
    errors: { key?: string; message: string }[];
}

/** Report returned by `export_*` operations. */
export interface ExportReport {
    records_exported: number;
    output_path: string;
    bytes_written: number;
}

/** Report returned by `rebuild_index` / `reindex_hnsw_from_text`. */
export interface RebuildReport {
    scanned_nodes: number;
    indexed_vectors: number;
    skipped_tombstones: number;
    duration_ms: number;
}

/** Report returned by `audit_text_index` / `audit_text_index_deep` / `repair_text_index`. */
export interface AuditReport {
    /** True when the audit ran without finding drift. */
    passed: boolean;
    scanned_namespaces: number;
    scanned_postings: number;
    duration_ms: number;
    /** Drift details (only on failure / deep audit). */
    drift?: {
        namespace: string;
        key: string;
        reason: string;
    }[];
    /** `status` follows the engine's taxonomy (`ok` / `repair_recommended` / `repair_failed`). */
    status: "ok" | "repair_recommended" | "repair_failed";
}

/** Edge-label/time filter for `graph_filtered_traversal`. */
export interface GraphTraversalFilter {
    /** Only follow edges whose label id is in this set; empty/undefined = no label filter. */
    labels?: number[];
    /** Inclusive `[from_ms, to_ms]` window on edge creation time; absent = no filter. */
    time_range?: [number, number];
}

/** One entry returned by `graph_degree`. */
export interface GraphDegreeEntry {
    /** u128 node id as a decimal string. */
    id: string;
    in_degree: number;
    out_degree: number;
}

/** Traversal direction string accepted by graph traversal entry points. */
export type TraversalDirectionStr = "Forward" | "Reverse" | "Both";

/** IQL query result (returned by `query`). Shape mirrors the engine's `VantaQueryResult`. */
export type IqlResult =
    | { kind: "Read"; rows: IqlRow[] }
    | { kind: "Write"; affected_nodes: number; message: string; node_id?: string }
    | { kind: "StaleContext"; node_id: string };

export type IqlRow = Record<string, MetadataValue>;

/** Snippet returned by `generate_snippet` (already optionally highlighted). */
export type Snippet = string;

// ─────────────────────────────────────────────────────────────────────────────
// Main class
// ─────────────────────────────────────────────────────────────────────────────

/**
 * The main VantaDB handle exposed to JavaScript via `wasm_bindgen`.
 *
 * @example
 * ```ts
 * import { VantaDB } from "vantadb-wasm";
 *
 * // In-memory
 * const db = new VantaDB({ storage_path: "my-app" });
 * await db.put({ namespace: "notes", key: "k1", payload: "hello" });
 * const rec = await db.get("notes", "k1");
 *
 * // Persistent (OPFS in browser, IndexedDB fallback)
 * const persistent = await VantaDB.connect_persistent("my-app");
 * ```
 */
export class VantaDB {
    /** Free the wasm-bindgen wrapper (use `Symbol.dispose` or call directly). */
    free(): void;
    /** ES explicit resource management hook. */
    [Symbol.dispose](): void;

    // ── Construction & lifecycle ────────────────────────────────────────────

    /**
     * Create a new in-memory VantaDB instance from an optional config.
     * @param config_val Optional {@link VantaConfigInput}.
     */
    constructor(config_val?: VantaConfigInput | null);

    /**
     * Open VantaDB at the given storage path. Synchronous (in-memory by default).
     */
    static open(path: string): VantaDB;

    /**
     * Open VantaDB with OPFS-based persistent storage in the browser.
     *
     * NOTE: this call rejects with a descriptive `Error` if OPFS is unavailable
     * (e.g. `navigator.storage.getDirectory` rejects). The previous silent
     * in-memory fallback was removed; verify availability with
     * `VantaDB.connect_idb` if OPFS might be blocked.
     */
    static connect_persistent(path: string): Promise<VantaDB>;

    /**
     * Open VantaDB with IndexedDB-based persistent storage (fallback when OPFS
     * is unavailable). Always async.
     */
    static connect_idb(path: string): Promise<VantaDB>;

    /**
     * Open VantaDB with a Web-Worker–backed OPFS layer. The caller must
     * register `spawnOpfsWorker` on the global scope (from
     * `vantadb-wasm/src/opfs_bridge.js`) before calling this.
     */
    static connect_worker(path: string): Promise<VantaDB>;

    /**
     * Close the database and release underlying engine resources. After close
     * the handle should not be used for further operations. This does NOT free
     * the JS wrapper — drop references after close to allow WASM GC.
     */
    close(): void;

    /** Worker proxy: read a file from the connected worker. */
    worker_read(path: string): Promise<Uint8Array | null>;
    /** Worker proxy: write a file from the connected worker. */
    worker_write(path: string, data: Uint8Array): Promise<void>;
    /** Worker proxy: delete a file from the connected worker. */
    worker_delete(path: string): Promise<void>;

    // ── Persistence (OPFS / IDB) ────────────────────────────────────────────

    /**
     * Persist in-memory records to OPFS storage using differential writes.
     * Only records changed since the last successful `save` are serialized;
     * if nothing changed the file write is skipped entirely (PERF-08).
     */
    save(): Promise<void>;

    /**
     * Persist in-memory records to IndexedDB storage using differential writes.
     * Only records changed since the last successful `save_idb` are
     * serialized; if nothing changed the file write is skipped (PERF-08).
     */
    save_idb(): Promise<void>;

    /**
     * Restore all records from OPFS storage into memory.
     */
    load(): Promise<void>;

    /**
     * Restore all records from IndexedDB storage into memory.
     */
    load_idb(): Promise<void>;

    /** Delete persisted state from IndexedDB. */
    delete_idb(): Promise<void>;

    // ── Auto-save glue ──────────────────────────────────────────────────────

    /** Enable auto-save on `visibilitychange` / `pagehide` events. */
    enable_auto_save(): void;
    /** Disable auto-save. */
    disable_auto_save(): void;
    /** True iff auto-save is currently enabled. */
    is_auto_save_enabled(): boolean;

    /**
     * Attempt an auto-save if there are unsaved changes AND auto-save is enabled.
     * Returns `true` if a save was attempted, `false` if skipped.
     */
    try_auto_save(): Promise<boolean>;

    // ── Memory CRUD & search ───────────────────────────────────────────────

    /**
     * Insert or update a single memory record.
     * @returns the stored `MemoryRecord` (with assigned `node_id` / `version` / timestamps).
     */
    put(input: MemoryRecordInput): MemoryRecord;

    /**
     * Insert or update multiple memory records in one call (max 100,000).
     */
    put_batch(inputs: MemoryRecordInput[]): MemoryRecord[];

    /** Retrieve a single record by namespace and key, or `null` if not present. */
    get(namespace: string, key: string): MemoryRecord | null;

    /** Delete a single record. Returns `true` if a record was actually deleted. */
    delete(namespace: string, key: string): boolean;

    /** Return all namespaces as a `string[]`. */
    list_namespaces(): string[];

    /**
     * List records in a namespace with optional filters, limit, and cursor pagination.
     * @returns a {@link ListPage} with `records` and optional `next_cursor`.
     */
    list(namespace: string, options?: ListOptionsInput | null): ListPage;

    /**
     * Search memory records by vector similarity with optional filters and text query.
     */
    search(request: SearchRequestInput): SearchHit[];

    /**
     * Search nodes by raw vector without namespace scoping.
     * Returns one `{node_id, score}` entry per result (u128 ids as decimal strings).
     */
    search_vector(vector: Float32Array | number[], top_k: number): { node_id: string; score: number }[];

    /**
     * Search namespace-scoped memory records by vector similarity from an
     * existing key, without supplying a query vector. The source record is
     * excluded from the results.
     */
    similar_to_key(namespace: string, key: string, top_k: number): SearchHit[];

    /**
     * Search across multiple namespaces in a single call. Results are merged
     * and sorted by descending score, capped at `request.top_k` globally.
     * The `namespace` field on the request is ignored — pass `namespaces` instead.
     */
    search_multi(namespaces: string[], request: SearchRequestInput): SearchHit[];

    /**
     * Run a search with explanation metadata for debugging scoring.
     * Returns the raw engine explanation object.
     */
    explain_memory_search(request: SearchRequestInput): SearchExplanation;

    // ── Filters / counts / mutation ─────────────────────────────────────────

    /**
     * Count records in a namespace, optionally matching a metadata filter.
     * Pass an empty array or `null` to count every record in the namespace.
     */
    count(namespace: string, filter: MetadataFilter | null): bigint;

    /**
     * Delete all records in a namespace matching a metadata filter.
     * The core rejects an empty filter to prevent accidental full-namespace
     * deletion — that error propagates unchanged.
     */
    delete_by_filter(namespace: string, filter: MetadataFilter): bigint;

    /**
     * Mark an existing record as superseded by another existing record.
     * Returns an error if either key is missing, `old_key == new_key`, or the
     * old record is already superseded.
     */
    supersede(namespace: string, old_key: string, new_key: string): void;

    // ── Import / export ────────────────────────────────────────────────────

    /** Export all records in a namespace to a JSON file at the given path. */
    export_namespace(path: string, namespace: string): ExportReport;

    /** Export records matching a metadata filter to a JSON file. */
    export_namespace_filtered(path: string, namespace: string, filter: MetadataFilter): ExportReport;

    /** Export all records across all namespaces to the given path. */
    export_all(path: string): ExportReport;

    /** Import records from a JS array of `MemoryRecord` objects. */
    import_records(records: MemoryRecord[]): ImportReport;

    /** Import records from a JSON file at the given path. */
    import_file(path: string): ImportReport;

    /**
     * Bulk-import records from a binary `.vdbdump` file.
     * @returns an `ImportReport` with `records_imported`, errors, and duration.
     */
    bulk_import(path: string): ImportReport;

    /**
     * Bulk-import records from binary bytes (`.vdbdump` format).
     */
    bulk_import_bytes(data: Uint8Array): ImportReport;

    // ── Index maintenance ──────────────────────────────────────────────────

    /** Rebuild the HNSW index and return a rebuild report. */
    rebuild_index(): RebuildReport;

    /**
     * Paginated HNSW rebuild from text records. Iterates through memory
     * records in batches (max 1000) using the cursor-based `list()` API to
     * prevent OOM on large namespaces.
     */
    reindex_hnsw_from_text(namespace: string, page_size?: number | null): RebuildReport;

    /** Compact the storage layout. Returns the number of freed bytes. */
    compact_layout(): bigint;

    /** Compact the write-ahead log. */
    compact_wal(): void;

    /** Run a text-index consistency audit for an optional namespace. */
    audit_text_index(namespace?: string | null): AuditReport;

    /** Run a DEEP text-index consistency audit (slower; verifies postings + term frequencies). */
    audit_text_index_deep(namespace?: string | null): AuditReport;

    /** Repair the text index and return a repair report. */
    repair_text_index(): AuditReport;

    /**
     * Flush engine-internal buffers.
     *
     * NOTE: this is NOT a durability guarantee in the browser — the engine
     * backend here may be purely in-memory. Persisted state only becomes
     * durable after an explicit `save()` / `save_idb()` call. Emits a
     * console warning when no persistent backend is attached.
     */
    flush(): void;

    /** Purge all expired records (TTL) and return the number removed. */
    purge_expired(): bigint;

    // ── Metrics / capabilities / IQL ───────────────────────────────────────

    /** Return a {@link VantaCapabilities} object describing supported features. */
    capabilities(): VantaCapabilities;

    /** Return operational metrics. All large numbers are stringified (policy string-u64). */
    operational_metrics(): OperationalMetrics;

    /** Execute a raw IQL query string and return the result. */
    query(query: string): IqlResult;

    /**
     * Generate a text snippet with optional highlighting for a given query.
     * Returns `undefined` when no query terms match the payload.
     */
    generate_snippet(payload: string, text_query: string, with_highlighting: boolean): Snippet | undefined;

    // ── Graph ──────────────────────────────────────────────────────────────

    /** Insert a graph node with optional content, vector, and fields. */
    insert_node(id: string, content: string | null | undefined, vector: Float32Array | number[] | null | undefined, fields: Record<string, MetadataValue> | null): void;

    /** Retrieve a graph node by its numeric id (decimal string). */
    get_node(id: string): NodeRecord | null;

    /** Delete a graph node by id with an associated reason string. */
    delete_node(id: string, reason: string): void;

    /**
     * Add a directed edge between two graph nodes.
     * `weight` and `created_at_ms` are optional; `created_at_ms` uses policy
     * string-u64 (a `bigint` in JS).
     */
    add_edge(source_id: string, target_id: string, label: string, weight?: number | null, created_at_ms?: bigint | null): void;

    /**
     * Remove all edges between two graph nodes with the given label
     * (both directions).
     */
    remove_edge(source_id: string, target_id: string, label: string): void;

    /**
     * Perform a breadth-first traversal from the given root node ids
     * (decimal strings).
     * @returns the visited node ids in BFS order.
     */
    graph_bfs(roots: string[], max_depth: number, direction: TraversalDirectionStr): string[];

    /** Perform a depth-first traversal from the given root node ids. */
    graph_dfs(roots: string[], max_depth: number, direction: TraversalDirectionStr): string[];

    /** Compute a topological sort order starting from the given root node ids. */
    graph_topological_sort(roots: string[]): string[];

    /** True iff the subgraph reachable from the given roots forms a DAG. */
    graph_is_dag(roots: string[]): boolean;

    /**
     * Breadth-first traversal with optional edge label/time filtering.
     * Pass `null` for `filter` to disable both filters.
     */
    graph_filtered_traversal(roots: string[], max_depth: number, direction: TraversalDirectionStr, filter: GraphTraversalFilter | null): string[];

    /**
     * Degree centrality (in/out counts) for the subgraph reachable from the
     * given root node ids. Returns one entry per node with u128 ids as strings.
     */
    graph_degree(roots: string[]): GraphDegreeEntry[];
}

// ─────────────────────────────────────────────────────────────────────────────
// Init functions (preserved from generated .d.ts verbatim — these are the
// runtime glue wasm-bindgen expects and must not be edited)
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Instantiates the given `module`, which can either be bytes or a precompiled `WebAssembly.Module`.
 *
 * @param module - Passing `SyncInitInput` directly is deprecated.
 * @returns the {@link InitOutput} for the module.
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is `{RequestInfo}` or `{URL}`, makes a request and for
 * everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param module_or_path - Passing `InitInput` directly is deprecated.
 * @returns a `Promise` that resolves to the {@link InitOutput}.
 */
export default function __wbg_init(module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
