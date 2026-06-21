import { VantaDB as WasmVantaDB } from "vantadb-wasm";

import type {
  Capabilities,
  ExportReport,
  ImportReport,
  ListOptions,
  MemoryListPage,
  MemoryRecord,
  NodeRecord,
  OperationalMetrics,
  QueryResult,
  SearchHit,
  SearchRequest,
  VantaConfig,
} from "./types.js";

function _toNum(val: bigint): number {
  return Number(val);
}

function _mapRecord(r: any): MemoryRecord {
  return {
    ...r,
    node_id: _toNum(r.node_id),
    created_at_ms: _toNum(r.created_at_ms),
    updated_at_ms: _toNum(r.updated_at_ms),
    version: _toNum(r.version),
    expires_at_ms: r.expires_at_ms != null ? _toNum(r.expires_at_ms) : undefined,
  };
}

export class VantaDB {
  private inner: WasmVantaDB;

  private constructor(inner: WasmVantaDB) {
    this.inner = inner;
  }

  static create(config?: VantaConfig): VantaDB {
    const inner = new WasmVantaDB(config ?? null);
    return new VantaDB(inner);
  }

  static open(path: string): VantaDB {
    const inner = WasmVantaDB.open(path);
    return new VantaDB(inner);
  }

  close(): void {
    this.inner.free();
  }

  capabilities(): Capabilities {
    const raw = this.inner.capabilities();
    return {
      runtime_profile: raw.runtime_profile,
      persistence: raw.persistence,
      vector_search: raw.vector_search,
      iql_queries: raw.iql_queries,
      read_only: raw.read_only,
    };
  }

  async put(input: {
    namespace: string;
    key: string;
    payload: string;
    metadata?: Record<string, any>;
    vector?: number[];
    ttl_ms?: number;
  }): Promise<MemoryRecord> {
    const raw = this.inner.put(input);
    return _mapRecord(raw);
  }

  async putBatch(
    inputs: Array<{
      namespace: string;
      key: string;
      payload: string;
      metadata?: Record<string, any>;
      vector?: number[];
      ttl_ms?: number;
    }>
  ): Promise<MemoryRecord[]> {
    const records: any[] = this.inner.put_batch(inputs);
    return records.map(_mapRecord);
  }

  async get(namespace: string, key: string): Promise<MemoryRecord | null> {
    const raw = this.inner.get(namespace, key);
    return raw != null ? _mapRecord(raw) : null;
  }

  async delete(namespace: string, key: string): Promise<boolean> {
    return this.inner.delete(namespace, key);
  }

  async listNamespaces(): Promise<string[]> {
    return this.inner.list_namespaces();
  }

  async list(
    namespace: string,
    options: ListOptions = {}
  ): Promise<MemoryListPage> {
    const raw = this.inner.list(namespace, options);
    return {
      records: (raw.records ?? []).map(_mapRecord),
      next_cursor: raw.next_cursor,
    };
  }

  async search(request: SearchRequest): Promise<SearchHit[]> {
    const raw: any[] = this.inner.search({
      namespace: request.namespace,
      query_vector: request.query_vector,
      filters: request.filters ?? {},
      text_query: request.text_query ?? null,
      top_k: request.top_k ?? 10,
      distance_metric: request.distance_metric ?? "Cosine",
      explain: request.explain ?? false,
    });
    return raw.map((hit: any) => ({
      record: _mapRecord(hit.record),
      score: hit.score,
      explanation: hit.explanation ?? undefined,
    }));
  }

  async searchVector(vector: number[], topK: number = 10): Promise<SearchHit[]> {
    const raw: any[] = this.inner.search_vector(new Float32Array(vector), topK);
    return raw.map((hit: any) => ({
      record: _mapRecord(hit.record),
      score: hit.score,
    }));
  }

  async explainSearch(request: SearchRequest): Promise<any> {
    return this.inner.explain_memory_search({
      namespace: request.namespace,
      query_vector: request.query_vector,
      filters: request.filters ?? {},
      text_query: request.text_query ?? null,
      top_k: request.top_k ?? 10,
      distance_metric: request.distance_metric ?? "Cosine",
      explain: true,
    });
  }

  async exportNamespace(path: string, namespace: string): Promise<ExportReport> {
    const raw = this.inner.export_namespace(path, namespace);
    return {
      records_exported: _toNum(raw.records_exported),
      namespaces: raw.namespaces,
      path: raw.path,
      duration_ms: _toNum(raw.duration_ms),
    };
  }

  async exportAll(path: string): Promise<ExportReport> {
    const raw = this.inner.export_all(path);
    return {
      records_exported: _toNum(raw.records_exported),
      namespaces: raw.namespaces,
      path: raw.path,
      duration_ms: _toNum(raw.duration_ms),
    };
  }

  async importRecords(records: any[]): Promise<ImportReport> {
    const raw = this.inner.import_records(records);
    return {
      inserted: _toNum(raw.inserted),
      updated: _toNum(raw.updated),
      skipped: _toNum(raw.skipped),
      errors: _toNum(raw.errors),
      duration_ms: _toNum(raw.duration_ms),
    };
  }

  async importFile(path: string): Promise<ImportReport> {
    const raw = this.inner.import_file(path);
    return {
      inserted: _toNum(raw.inserted),
      updated: _toNum(raw.updated),
      skipped: _toNum(raw.skipped),
      errors: _toNum(raw.errors),
      duration_ms: _toNum(raw.duration_ms),
    };
  }

  async rebuildIndex(): Promise<any> {
    return this.inner.rebuild_index();
  }

  async compactLayout(): Promise<number> {
    return _toNum(this.inner.compact_layout());
  }

  async auditTextIndex(namespace?: string): Promise<any> {
    return this.inner.audit_text_index(namespace ?? null);
  }

  async auditTextIndexDeep(namespace?: string): Promise<any> {
    return this.inner.audit_text_index_deep(namespace ?? null);
  }

  async repairTextIndex(): Promise<any> {
    return this.inner.repair_text_index();
  }

  async flush(): Promise<void> {
    this.inner.flush();
  }

  async compactWal(): Promise<void> {
    this.inner.compact_wal();
  }

  async purgeExpired(): Promise<number> {
    return _toNum(this.inner.purge_expired());
  }

  async operationalMetrics(): Promise<OperationalMetrics> {
    return this.inner.operational_metrics();
  }

  async query(query: string): Promise<QueryResult> {
    return this.inner.query(query);
  }

  async insertNode(
    id: number,
    content?: string,
    vector?: number[],
    fields: Record<string, any> = {}
  ): Promise<void> {
    this.inner.insert_node(
      BigInt(id),
      content ?? null,
      vector ? new Float32Array(vector) : null,
      fields
    );
  }

  async getNode(id: number): Promise<NodeRecord | null> {
    const raw = this.inner.get_node(BigInt(id));
    if (raw == null) return null;
    return {
      ...raw,
      id: _toNum(raw.id),
      vector_dimensions: _toNum(raw.vector_dimensions),
      hits: _toNum(raw.hits),
      last_accessed: _toNum(raw.last_accessed),
      epoch: _toNum(raw.epoch),
    };
  }

  async deleteNode(id: number, reason: string = "deleted"): Promise<void> {
    this.inner.delete_node(BigInt(id), reason);
  }

  async addEdge(
    source: number,
    target: number,
    label: string = "",
    weight?: number
  ): Promise<void> {
    this.inner.add_edge(BigInt(source), BigInt(target), label, weight ?? null);
  }

  async graphBfs(roots: number[], maxDepth: number = 10): Promise<any> {
    return this.inner.graph_bfs(new BigUint64Array(roots.map(BigInt)), maxDepth);
  }

  async graphDfs(roots: number[], maxDepth: number = 10): Promise<any> {
    return this.inner.graph_dfs(new BigUint64Array(roots.map(BigInt)), maxDepth);
  }

  async graphTopologicalSort(roots: number[]): Promise<any> {
    return this.inner.graph_topological_sort(new BigUint64Array(roots.map(BigInt)));
  }

  async graphIsDag(roots: number[]): Promise<boolean> {
    return this.inner.graph_is_dag(new BigUint64Array(roots.map(BigInt)));
  }

  async generateSnippet(
    payload: string,
    query: string,
    withHighlighting: boolean = false
  ): Promise<string | undefined> {
    return this.inner.generate_snippet(payload, query, withHighlighting) ?? undefined;
  }
}
