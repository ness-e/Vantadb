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
  if (!r || typeof r !== 'object') {
    throw new Error("_mapRecord: expected an object, got " + typeof r);
  }
  if (typeof r.namespace !== 'string') {
    throw new Error("_mapRecord: record missing required string field 'namespace'");
  }
  if (typeof r.key !== 'string') {
    throw new Error("_mapRecord: record missing required string field 'key'");
  }
  if (typeof r.payload !== 'string') {
    throw new Error("_mapRecord: record missing required string field 'payload'");
  }
  return r;
}

/**
 * VantaDB — The vector-graph database that thinks.
 * In-process TypeScript binding via WASM. Zero network overhead.
 *
 * Usage:
 *   const db = VantaDB.connect("./my_brain");
 *   db.put({ namespace: "ns", key: "k", payload: "Hello" });
 *   const record = db.get("ns", "k");
 *   db.close();
 */
export class VantaDB {
  private inner: WasmVantaDB;
  /** Whether this instance has been closed. Guards all public methods. */
  private _closed: boolean = false;

  private constructor(inner: WasmVantaDB) {
    this.inner = inner;
  }

  /// Connect to a VantaDB database.
  /// - path: filesystem path for persistent storage
  /// - If path is empty/":memory:" or omitted, opens in-memory engine
  static connect(path?: string): VantaDB {
    const inner = path && path !== ":memory:"
      ? WasmVantaDB.open(path)
      : new WasmVantaDB(null);
    return new VantaDB(inner);
  }

  /**
   * Create a new VantaDB instance with the given config.
   * @param config - Configuration options.
   *   Note: In WASM mode, `storage_path` is accepted but ignored (CODE-089) — the
   *   WASM backend always uses an in-memory engine. For persistent storage, use `connect()`.
   */
  static create(config?: VantaConfig): VantaDB {
    if (config?.storage_path) {
      console.warn(
        "VantaDB.create(): storage_path is ignored in WASM mode — the WASM backend always uses an in-memory engine."
      );
    }
    const inner = new WasmVantaDB(config ?? null);
    return new VantaDB(inner);
  }

  static open(path: string): VantaDB {
    const inner = WasmVantaDB.open(path);
    return new VantaDB(inner);
  }

  /**
   * Assert the instance is not yet closed.
   * @throws {Error} If close() has already been called.
   */
  private _assertOpen(): void {
    if (this._closed) {
      throw new Error("VantaDB instance is closed");
    }
  }

  /**
   * Close the database and release underlying WASM engine resources.
   *
   * After close(), all public methods throw. Unlike relying on WASM GC/finalization,
   * this explicitly marks the instance as closed and prevents accidental use-after-free.
   */
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

  put(input: {
    namespace: string;
    key: string;
    payload: string;
    metadata?: Record<string, any>;
    vector?: number[];
    ttl_ms?: number;
  }): MemoryRecord {
    this._assertOpen();
    const raw = this.inner.put(input);
    return _mapRecord(raw);
  }

  putBatch(
    inputs: Array<{
      namespace: string;
      key: string;
      payload: string;
      metadata?: Record<string, any>;
      vector?: number[];
      ttl_ms?: number;
    }>
  ): MemoryRecord[] {
    this._assertOpen();
    const records: any[] = this.inner.put_batch(inputs);
    for (let i = 0; i < records.length; i++) {
      records[i] = _mapRecord(records[i]);
    }
    return records as MemoryRecord[];
  }

  get(namespace: string, key: string): MemoryRecord | null {
    this._assertOpen();
    const raw = this.inner.get(namespace, key);
    return raw != null ? _mapRecord(raw) : null;
  }

  delete(namespace: string, key: string): boolean {
    this._assertOpen();
    return this.inner.delete(namespace, key);
  }

  listNamespaces(): string[] {
    this._assertOpen();
    return this.inner.list_namespaces();
  }

  list(
    namespace: string,
    options: ListOptions = {}
  ): MemoryListPage {
    this._assertOpen();
    const raw = this.inner.list(namespace, options);
    const items = raw.records ?? [];
    for (let i = 0; i < items.length; i++) {
      items[i] = _mapRecord(items[i]);
    }
    return {
      records: items,
      next_cursor: raw.next_cursor,
    };
  }

  private _buildSearchRequest(request: SearchRequest, explain?: boolean): any {
    return {
      namespace: request.namespace,
      query_vector: request.query_vector,
      filters: request.filters ?? {},
      text_query: request.text_query ?? null,
      top_k: request.top_k ?? 10,
      distance_metric: request.distance_metric ?? "Cosine",
      explain: explain ?? (request.explain ?? false),
    };
  }

  search(request: SearchRequest): SearchHit[] {
    this._assertOpen();
    const raw: any[] = this.inner.search(this._buildSearchRequest(request));
    return raw.map((hit: any) => ({
      record: _mapRecord(hit.record),
      distance: hit.score, // score from WASM is the L2/cosine distance
      explanation: hit.explanation ?? undefined,
    }));
  }

  searchVector(
    vector: number[],
    topK: number = 10
  ): { node_id: string; distance: number }[] {
    this._assertOpen();
    const raw: any[] = this.inner.search_vector(new Float32Array(vector), topK);
    return raw.map((hit: any) => ({
      node_id: hit.node_id,
      distance: hit.score, // score from WASM is the L2/cosine distance
    }));
  }

  explainSearch(request: SearchRequest): any {
    this._assertOpen();
    return this.inner.explain_memory_search(this._buildSearchRequest(request, true));
  }

  exportNamespace(path: string, namespace: string): ExportReport {
    this._assertOpen();
    return this.inner.export_namespace(path, namespace);
  }

  exportAll(path: string): ExportReport {
    this._assertOpen();
    return this.inner.export_all(path);
  }

  importRecords(records: any[]): ImportReport {
    this._assertOpen();
    return this.inner.import_records(records);
  }

  importFile(path: string): ImportReport {
    this._assertOpen();
    return this.inner.import_file(path);
  }

  rebuildIndex(): any {
    this._assertOpen();
    return this.inner.rebuild_index();
  }

  compactLayout(): bigint {
    this._assertOpen();
    return this.inner.compact_layout();
  }

  auditTextIndex(namespace?: string): any {
    this._assertOpen();
    return this.inner.audit_text_index(namespace ?? null);
  }

  auditTextIndexDeep(namespace?: string): any {
    this._assertOpen();
    return this.inner.audit_text_index_deep(namespace ?? null);
  }

  repairTextIndex(): any {
    this._assertOpen();
    return this.inner.repair_text_index();
  }

  flush(): void {
    this._assertOpen();
    this.inner.flush();
  }

  compactWal(): void {
    this._assertOpen();
    this.inner.compact_wal();
  }

  purgeExpired(): bigint {
    this._assertOpen();
    return this.inner.purge_expired();
  }

  operationalMetrics(): OperationalMetrics {
    this._assertOpen();
    return this.inner.operational_metrics();
  }

  query(query: string): QueryResult {
    this._assertOpen();
    return this.inner.query(query);
  }

  /**
   * Insert a graph node.
   *
   * For IDs > 2^53, use bigint — JavaScript Numbers lose integer precision
   * above 2^53 (CODE-090).
   */
  insertNode(
    id: number | bigint,
    content?: string,
    vector?: number[],
    fields: Record<string, any> = {}
  ): void {
    this._assertOpen();
    if (typeof id === 'number' && !Number.isSafeInteger(id)) {
      throw new Error(
        `insertNode: id ${id} is not a safe integer — JavaScript numbers lose precision above 2^53. Use bigint for large IDs.`
      );
    }
    this.inner.insert_node(
      BigInt(id),
      content ?? null,
      vector ? new Float32Array(vector) : null,
      fields
    );
  }

  getNode(id: number): NodeRecord | null {
    this._assertOpen();
    const raw = this.inner.get_node(BigInt(id));
    if (raw == null) return null;
    return raw;
  }

  deleteNode(id: number, reason: string = "deleted"): void {
    this._assertOpen();
    this.inner.delete_node(BigInt(id), reason);
  }

  addEdge(
    source: number,
    target: number,
    label: string = "",
    weight?: number
  ): void {
    this._assertOpen();
    this.inner.add_edge(BigInt(source), BigInt(target), label, weight ?? null);
  }

  graphBfs(roots: number[], maxDepth: number = 10): any {
    this._assertOpen();
    return this.inner.graph_bfs(new BigUint64Array(roots.map(BigInt)), maxDepth);
  }

  graphDfs(roots: number[], maxDepth: number = 10): any {
    this._assertOpen();
    return this.inner.graph_dfs(new BigUint64Array(roots.map(BigInt)), maxDepth);
  }

  graphTopologicalSort(roots: number[]): any {
    this._assertOpen();
    return this.inner.graph_topological_sort(new BigUint64Array(roots.map(BigInt)));
  }

  graphIsDag(roots: number[]): boolean {
    this._assertOpen();
    return this.inner.graph_is_dag(new BigUint64Array(roots.map(BigInt)));
  }

  generateSnippet(
    payload: string,
    query: string,
    withHighlighting: boolean = false
  ): string | undefined {
    this._assertOpen();
    return this.inner.generate_snippet(payload, query, withHighlighting) ?? undefined;
  }
}
