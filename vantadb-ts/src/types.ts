export type VantaValue =
  | { type: "String"; value: string }
  | { type: "Int"; value: number }
  | { type: "Float"; value: number }
  | { type: "Bool"; value: boolean }
  | { type: "Null" }
  | { type: "ListString"; value: string[] }
  | { type: "ListInt"; value: number[] }
  | { type: "ListFloat"; value: number[] }
  | { type: "ListBool"; value: boolean[] };

export type VantaMetadata = Record<string, VantaValue>;

export interface MemoryInput {
  namespace: string;
  key: string;
  payload: string;
  metadata?: VantaMetadata;
  vector?: number[];
  ttl_ms?: number;
}

export interface MemoryRecord {
  namespace: string;
  key: string;
  payload: string;
  metadata: VantaMetadata;
  created_at_ms: string;
  updated_at_ms: string;
  version: string;
  node_id: string;
  vector?: number[];
  expires_at_ms?: string;
}

export interface ListOptions {
  filters?: VantaMetadata;
  limit?: number;
  cursor?: number;
}

export interface MemoryListPage {
  records: MemoryRecord[];
  next_cursor?: number;
}

export interface SearchRequest {
  namespace: string;
  query_vector: number[];
  filters?: VantaMetadata;
  text_query?: string;
  top_k?: number;
  distance_metric?: "Cosine" | "Euclidean";
  explain?: boolean;
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
  fields: Record<string, VantaValue>;
}

export interface NodeRecord {
  id: string;
  fields: Record<string, VantaValue>;
  vector?: number[];
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
  target: string;
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

export interface ImportReport {
  inserted: number;
  updated: number;
  skipped: number;
  errors: number;
  duration_ms: number;
}

export interface OperationalMetrics {
  /** Milliseconds elapsed during engine startup (stringified u64). */
  startup_ms: string;
  /** Milliseconds spent replaying the WAL on database open. */
  wal_replay_ms: string;
  /** Number of WAL records replayed during startup. */
  wal_records_replayed: string;
  /** Milliseconds spent rebuilding the ANN index after startup. */
  ann_rebuild_ms: string;
  /** Number of nodes scanned during ANN index rebuild. */
  ann_rebuild_scanned_nodes: string;
  /** Milliseconds for derived index (memory) rebuild. */
  derived_rebuild_ms: string;
  /** Milliseconds for text index rebuild. */
  text_index_rebuild_ms: string;
  /** Number of posting entries written during text index build. */
  text_postings_written: string;
  /** Number of text index repair operations performed. */
  text_index_repairs: string;
  /** Number of lexical text queries executed. */
  text_lexical_queries: string;
  /** Cumulative milliseconds spent on lexical text queries. */
  text_lexical_query_ms: string;
  /** Total text candidates scored during hybrid search. */
  text_candidates_scored: string;
  /** Number of text index consistency audits performed. */
  text_consistency_audits: string;
  /** Number of consistency audit failures detected. */
  text_consistency_audit_failures: string;
  /** Cumulative milliseconds for hybrid (text + vector) queries. */
  hybrid_query_ms: string;
  /** Number of candidates fused during hybrid search. */
  hybrid_candidates_fused: string;
  /** Queries routed through the hybrid (text+vector) planner. */
  planner_hybrid_queries: string;
  /** Queries routed as text-only by the query planner. */
  planner_text_only_queries: string;
  /** Queries routed as vector-only by the query planner. */
  planner_vector_only_queries: string;
  /** Total records exported across all export operations. */
  records_exported: string;
  /** Total records imported across all import operations. */
  records_imported: string;
  /** Number of import errors encountered. */
  import_errors: string;
  /** Number of prefix scans on the derived index. */
  derived_prefix_scans: string;
  /** Number of fallbacks to full scan on the derived index. */
  derived_full_scan_fallbacks: string;
  /** Resident set size in bytes (process physical memory usage). */
  process_rss_bytes: string;
  /** Virtual memory size in bytes. */
  process_virtual_bytes: string;
  /** Number of nodes currently in the HNSW graph. */
  hnsw_nodes_count: string;
  /** Logical memory consumed by the HNSW graph in bytes. */
  hnsw_logical_bytes: string;
  /** Resident bytes in memory-mapped regions, or null if unavailable. */
  mmap_resident_bytes: string | null;
  /** Number of entries in the volatile metadata cache. */
  volatile_cache_entries: string;
  /** Capacity of the volatile metadata cache in bytes. */
  volatile_cache_cap_bytes: string;
  /** Bytes allocated by jemalloc, or null if jemalloc not in use. */
  jemalloc_allocated_bytes: string | null;
  /** Active bytes tracked by jemalloc, or null. */
  jemalloc_active_bytes: string | null;
  /** Metadata bytes used by jemalloc bookkeeping, or null. */
  jemalloc_metadata_bytes: string | null;
  /** Resident bytes reported by jemalloc, or null. */
  jemalloc_resident_bytes: string | null;
  /** Mapped memory regions tracked by jemalloc, or null. */
  jemalloc_mapped_bytes: string | null;
  /** Retained memory in jemalloc caches, or null. */
  jemalloc_retained_bytes: string | null;
}

export interface Capabilities {
  runtime_profile: string;
  persistence: boolean;
  vector_search: boolean;
  iql_queries: boolean;
  read_only: boolean;
}

export interface VantaConfig {
  storage_path?: string;
  read_only?: boolean;
  rss_threshold?: number;
  memory_limit?: number;
}
