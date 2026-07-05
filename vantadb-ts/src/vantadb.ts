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

function _mapRecord(r: any): MemoryRecord {
  return r;
}

export class VantaDB {
  private inner: WasmVantaDB;
  private _closed: boolean = false;

  private constructor(inner: WasmVantaDB) {
    this.inner = inner;
  }

  /// Connect to a VantaDB database.
  /// - path: filesystem path for persistent storage
  /// - If path is empty/":memory:" or omitted, opens in-memory engine
  static async connect(path?: string): Promise<VantaDB> {
    const inner = path && path !== ":memory:"
      ? WasmVantaDB.open(path)
      : new WasmVantaDB(null);
    return new VantaDB(inner);
  }

  static create(config?: VantaConfig): VantaDB {
    const inner = new WasmVantaDB(config ?? null);
    return new VantaDB(inner);
  }

  static open(path: string): VantaDB {
    const inner = WasmVantaDB.open(path);
    return new VantaDB(inner);
  }

  private _assertOpen(): void {
    if (this._closed) {
      throw new Error("VantaDB instance is closed");
    }
  }

  close(): void {
    if (this._closed) return;
    this.inner.close();
    this._closed = true;
  }

  capabilities(): Capabilities {
    this._assertOpen();
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
    this._assertOpen();
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
    this._assertOpen();
    const records: any[] = this.inner.put_batch(inputs);
    return records.map(_mapRecord);
  }

  async get(namespace: string, key: string): Promise<MemoryRecord | null> {
    this._assertOpen();
    const raw = this.inner.get(namespace, key);
    return raw != null ? _mapRecord(raw) : null;
  }

  async delete(namespace: string, key: string): Promise<boolean> {
    this._assertOpen();
    return this.inner.delete(namespace, key);
  }

  async listNamespaces(): Promise<string[]> {
    this._assertOpen();
    return this.inner.list_namespaces();
  }

  async list(
    namespace: string,
    options: ListOptions = {}
  ): Promise<MemoryListPage> {
    this._assertOpen();
    const raw = this.inner.list(namespace, options);
    return {
      records: (raw.records ?? []).map(_mapRecord),
      next_cursor: raw.next_cursor,
    };
  }

  async search(request: SearchRequest): Promise<SearchHit[]> {
    this._assertOpen();
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

  async searchVector(
    vector: number[],
    topK: number = 10
  ): Promise<{ node_id: string; score: number }[]> {
    this._assertOpen();
    const raw: any[] = this.inner.search_vector(new Float32Array(vector), topK);
    return raw.map((hit: any) => ({
      node_id: hit.node_id,
      score: hit.score,
    }));
  }

  async explainSearch(request: SearchRequest): Promise<any> {
    this._assertOpen();
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
    this._assertOpen();
    return this.inner.export_namespace(path, namespace);
  }

  async exportAll(path: string): Promise<ExportReport> {
    this._assertOpen();
    return this.inner.export_all(path);
  }

  async importRecords(records: any[]): Promise<ImportReport> {
    this._assertOpen();
    return this.inner.import_records(records);
  }

  async importFile(path: string): Promise<ImportReport> {
    this._assertOpen();
    return this.inner.import_file(path);
  }

  async rebuildIndex(): Promise<any> {
    this._assertOpen();
    return this.inner.rebuild_index();
  }

  async compactLayout(): Promise<bigint> {
    this._assertOpen();
    return this.inner.compact_layout();
  }

  async auditTextIndex(namespace?: string): Promise<any> {
    this._assertOpen();
    return this.inner.audit_text_index(namespace ?? null);
  }

  async auditTextIndexDeep(namespace?: string): Promise<any> {
    this._assertOpen();
    return this.inner.audit_text_index_deep(namespace ?? null);
  }

  async repairTextIndex(): Promise<any> {
    this._assertOpen();
    return this.inner.repair_text_index();
  }

  async flush(): Promise<void> {
    this._assertOpen();
    this.inner.flush();
  }

  async compactWal(): Promise<void> {
    this._assertOpen();
    this.inner.compact_wal();
  }

  async purgeExpired(): Promise<bigint> {
    this._assertOpen();
    return this.inner.purge_expired();
  }

  async operationalMetrics(): Promise<OperationalMetrics> {
    this._assertOpen();
    return this.inner.operational_metrics();
  }

  async query(query: string): Promise<QueryResult> {
    this._assertOpen();
    return this.inner.query(query);
  }

  async insertNode(
    id: number,
    content?: string,
    vector?: number[],
    fields: Record<string, any> = {}
  ): Promise<void> {
    this._assertOpen();
    this.inner.insert_node(
      BigInt(id),
      content ?? null,
      vector ? new Float32Array(vector) : null,
      fields
    );
  }

  async getNode(id: number): Promise<NodeRecord | null> {
    this._assertOpen();
    const raw = this.inner.get_node(BigInt(id));
    if (raw == null) return null;
    return raw;
  }

  async deleteNode(id: number, reason: string = "deleted"): Promise<void> {
    this._assertOpen();
    this.inner.delete_node(BigInt(id), reason);
  }

  async addEdge(
    source: number,
    target: number,
    label: string = "",
    weight?: number
  ): Promise<void> {
    this._assertOpen();
    this.inner.add_edge(BigInt(source), BigInt(target), label, weight ?? null);
  }

  async graphBfs(roots: number[], maxDepth: number = 10): Promise<any> {
    this._assertOpen();
    return this.inner.graph_bfs(new BigUint64Array(roots.map(BigInt)), maxDepth);
  }

  async graphDfs(roots: number[], maxDepth: number = 10): Promise<any> {
    this._assertOpen();
    return this.inner.graph_dfs(new BigUint64Array(roots.map(BigInt)), maxDepth);
  }

  async graphTopologicalSort(roots: number[]): Promise<any> {
    this._assertOpen();
    return this.inner.graph_topological_sort(new BigUint64Array(roots.map(BigInt)));
  }

  async graphIsDag(roots: number[]): Promise<boolean> {
    this._assertOpen();
    return this.inner.graph_is_dag(new BigUint64Array(roots.map(BigInt)));
  }

  async generateSnippet(
    payload: string,
    query: string,
    withHighlighting: boolean = false
  ): Promise<string | undefined> {
    this._assertOpen();
    return this.inner.generate_snippet(payload, query, withHighlighting) ?? undefined;
  }
}
