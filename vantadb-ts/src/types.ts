export type Value =
  | { String: string }
  | { Int: number }
  | { Float: number }
  | { Bool: boolean }
  | { Null: null }
  | { ListString: string[] }
  | { ListInt: number[] }
  | { ListFloat: number[] }
  | { ListBool: boolean[] };

export type Metadata = Record<string, Value>;

/** Plain JS value accepted as metadata/filter input (normalized internally to `Value`). */
export type FlatValue = string | number | boolean | null;

/**
 * Metadata/filters as provided by callers: plain JS values (preferred,
 * e.g. `{ lang: "en" }`) or the tagged wire form (backward compat,
 * e.g. `{ lang: { String: "en" } }`). Records returned by the engine
 * always use the tagged `Metadata` form.
 */
export type MetadataInput = Record<string, FlatValue | Value>;

export interface MemoryInput {
  namespace: string;
  key: string;
  payload: string;
  metadata?: MetadataInput;
  vector?: number[];
  /** Sparse term-weight vector (e.g. raw-keyword weights). Sparse vectors
   * participate in sparse-dot search alongside the dense vector. Wire shape:
   * `Record<u32-index, weight>` — empty / undefined skips sparse search. */
  sparse_vector?: Record<number, number>;
  ttl_ms?: number;
}

export interface MemoryRecord {
  namespace: string;
  key: string;
  payload: string;
  metadata: Metadata;
  created_at_ms: string | number;
  updated_at_ms: string | number;
  version: string | number;
  node_id: string | number;
  // PERF-08: the WASM layer now emits record vectors as a zero-copy Float32Array
  // (previously a number[] built via serde_wasm_bindgen). Both are indexable/
  // iterable; consumers must accept either form.
  vector?: Float32Array | number[];
  expires_at_ms?: string | number;
}

export interface ListOptions {
  filters?: MetadataInput;
  limit?: number;
  /** Opaque page cursor: the WASM backend emits decimal strings (policy
   * string-u64) while the native backend emits numbers; both accept either
   * form back (FIND-125 resync against `vantadb-wasm/src/lib.rs:200-229`). */
  cursor?: string | number;
}

export interface MemoryListPage {
  records: MemoryRecord[];
  /** Opaque next cursor (decimal string from WASM, number from native);
   * absent on the last page. Feed back into `ListOptions.cursor`. */
  next_cursor?: string | number;
}

// TS-09 (breaking, 0.7.0): every multi-parameter call takes a single input
// object — `put`/`search` already did; `get`/`delete`/`deleteByFilter`/
// `list`/`count`/`supersede`/`searchMulti`/`searchVector`/`similarToKey` now
// do too. Single-scalar methods (graph ids, `putBatch` arrays) stay positional.

/** Input for `get`. */
export interface GetInput {
  namespace: string;
  key: string;
}

/** Input for `delete`. */
export interface DeleteInput {
  namespace: string;
  key: string;
}

/** Input for `deleteByFilter`. */
export interface DeleteByFilterInput {
  namespace: string;
  filter: FilterItem[];
}

/** Input for `list`: namespace plus the usual page options. */
export interface ListInput extends ListOptions {
  namespace: string;
}

/** Input for `count`. */
export interface CountInput {
  namespace: string;
  filters?: FilterItem[];
}

/** Input for `supersede`. */
export interface SupersedeInput {
  namespace: string;
  oldKey: string;
  newKey: string;
}

/** Input for `searchVector`. */
export interface SearchVectorInput {
  vector: number[];
  topK?: number;
}

/** Input for `similarToKey`. */
export interface SimilarToKeyInput {
  namespace: string;
  key: string;
  topK?: number;
}

export interface SearchRequest {
  namespace: string;
  query_vector: number[];
  filters?: MetadataInput;
  text_query?: string;
  top_k?: number;
  distance_metric?: "Cosine" | "Euclidean";
  explain?: boolean;
  /** When true, hide records marked as superseded (ADR-028). Default false:
   * superseded records remain searchable for backward compatibility. */
  exclude_superseded?: boolean;
}

/** A list of namespaces to search in batch. Used by `searchMulti`. */
export interface BatchSearchRequest extends Omit<SearchRequest, "namespace"> {
  /** Namespaces to search independently; results are merged and capped at top_k. */
  namespaces: string[];
}

export interface SearchHit {
  record: MemoryRecord;
  /** L2 distance (or cosine distance) between the query vector and this hit's record vector.
   * Lower values indicate higher similarity. This is a distance, not a similarity score. */
  distance: number;
  explanation?: SearchExplanationHit;
}

export interface SearchExplanationHit {
  identity: string;
  score: number;
  snippet?: string;
  matched_tokens: string[];
  matched_phrases: string[];
}

export interface NodeInput {
  id: number;
  content?: string;
  vector?: number[];
  fields: Record<string, FlatValue | Value>;
}

export interface NodeRecord {
  id: string;
  fields: Record<string, Value>;
  // PERF-08: zero-copy Float32Array from the WASM layer (was number[]).
  vector?: Float32Array | number[];
  vector_dimensions: number;
  edges: EdgeRecord[];
  confidence_score: number;
  importance: number;
  hits: number;
  last_accessed: string;
  epoch: number;
  tier: "Hot" | "Cold";
  is_alive: boolean;
}

export interface EdgeRecord {
  /**
   * Target node id. The WASM layer serializes u128 ids as strings
   * (JS number-safety); the SDK normalizes them to bigint on read.
   * Both forms are accepted so hand-constructed fixtures round-trip.
   */
  target: string | bigint;
  label: string;
  weight: number;
}

export interface QueryResult {
  Read?: NodeRecord[];
  Write?: { affected_nodes: number; message: string; node_id?: string };
  StaleContext?: { node_id: string };
}

export interface ExportReport {
  records_exported: number;
  namespaces: string[];
  path: string;
  duration_ms: number;
}

/** Filter operators (PascalCase — wire-compatible with the core `FilterOp`). */
export type FilterOp = "Eq" | "Neq" | "Gt" | "Gte" | "Lt" | "Lte";

/** Single AND-combined filter item for export/delete operations. */
export interface FilterItem {
  field: string;
  op: FilterOp;
  /** Plain JS value (preferred) or tagged wire form (backward compat). */
  value: FlatValue | Value;
}

export interface ImportReport {
  inserted: number;
  updated: number;
  skipped: number;
  errors: number;
  duration_ms: number;
}

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
  /** Absent (undefined) when the OS/backend reports no value — the WASM
   * layer serializes `Option<String>` as absent, never `null`
   * (FIND-125 resync against `JsOperationalMetrics`,
   * `vantadb-wasm/src/lib.rs:269-318`). */
  mmap_resident_bytes?: string;
  volatile_cache_entries: string;
  volatile_cache_cap_bytes: string;
  jemalloc_allocated_bytes?: string;
  jemalloc_active_bytes?: string;
  jemalloc_metadata_bytes?: string;
  jemalloc_resident_bytes?: string;
  jemalloc_mapped_bytes?: string;
  jemalloc_retained_bytes?: string;
  /** Cumulative NaN/Inf→0.0 sanitizations on outgoing payloads (WSM-12). */
  nan_sanitization_count: string;
  /** Cumulative metadata fields dropped from outgoing records (WSM-11). */
  metadata_drop_count: string;
}

export interface Capabilities {
  runtime_profile: string;
  persistence: boolean;
  vector_search: boolean;
  iql_queries: boolean;
  read_only: boolean;
}

export interface Config {
  storage_path?: string;
  read_only?: boolean;
  rss_threshold?: number;
  memory_limit?: number;
}

/**
 * Result of a BFS, DFS, filtered-traversal or topological-sort traversal.
 *
 * Wire format (real, verified 2026-08-28 against `src/sdk/graph.rs` and
 * `vantadb-wasm/src/lib.rs:1552-1578`): the WASM binding returns `Vec<u128>` and
 * `serde_wasm_bindgen 0.6` with default options serializes each `u128` as a
 * `BigInt` (because `u128` exceeds `Number.MAX_SAFE_INTEGER`). Consumers
 * receive a `bigint[]` of node IDs in traversal order — NOT the fictional
 * `{ visited, levels, path }` / `{ visited, order, has_cycle }` shape that
 * older revisions of this file advertised.
 *
 * `@deprecated` aliases `GraphBfsResult` / `GraphDfsResult` / `GraphTopologicalSortResult`
 * are kept as `bigint[]` for source compatibility but the old field shape is gone.
 * This is a breaking change for code that read `result.visited` / `result.levels` / etc.
 * — iterate the array or index into it instead.
 */
export type GraphBfsResult = bigint[];

/** @deprecated Use `bigint[]` directly. See `GraphBfsResult` for the wire shape. */
export type GraphDfsResult = bigint[];

/** @deprecated Use `bigint[]` directly. `has_cycle` is no longer exposed in the wire. */
export type GraphTopologicalSortResult = bigint[];

/** Optional edge label/time filter for `graphFilteredTraversal` (GRAFO-01). */
export interface GraphTraversalFilter {
  /** Only follow edges whose label id is in this set. Empty = no label filter. */
  labels?: number[];
  /** Inclusive [from_ms, to_ms] window on edge creation time. Absent = no filter. */
  time_range?: [number, number] | null;
}

/** Degree centrality entry for a graph node (GRAFO-01). */
export interface GraphDegreeEntry {
  /** Node id as a string (u128 ids exceed JS safe integers). */
  id: string;
  in_degree: number;
  out_degree: number;
}
