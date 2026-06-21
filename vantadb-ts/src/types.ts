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
  created_at_ms: number;
  updated_at_ms: number;
  version: number;
  node_id: number;
  vector?: number[];
  expires_at_ms?: number;
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
  score: number;
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
  id: number;
  fields: Record<string, VantaValue>;
  vector?: number[];
  vector_dimensions: number;
  edges: EdgeRecord[];
  confidence_score: number;
  importance: number;
  hits: number;
  last_accessed: number;
  epoch: number;
  tier: "Hot" | "Cold";
  is_alive: boolean;
}

export interface EdgeRecord {
  target: number;
  label: string;
  weight: number;
}

export interface QueryResult {
  Read?: NodeRecord[];
  Write?: { affected_nodes: number; message: string; node_id?: number };
  StaleContext?: { node_id: number };
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
  startup_ms: number;
  wal_replay_ms: number;
  wal_records_replayed: number;
  ann_rebuild_ms: number;
  ann_rebuild_scanned_nodes: number;
  process_rss_bytes: number;
  process_virtual_bytes: number;
  hnsw_nodes_count: number;
  hnsw_logical_bytes: number;
  volatile_cache_entries: number;
  volatile_cache_cap_bytes: number;
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
