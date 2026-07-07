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
  mmap_resident_bytes: string | null;
  volatile_cache_entries: string;
  volatile_cache_cap_bytes: string;
  jemalloc_allocated_bytes: string | null;
  jemalloc_active_bytes: string | null;
  jemalloc_metadata_bytes: string | null;
  jemalloc_resident_bytes: string | null;
  jemalloc_mapped_bytes: string | null;
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

export interface GraphBfsResult {
  visited: number[];
  levels: Record<string, number>;
  path: number[][];
}

export interface GraphDfsResult {
  visited: number[];
  order: number[];
  has_cycle: boolean;
}

export interface GraphTopologicalSortResult {
  sorted: number[];
  has_cycle: boolean;
}
