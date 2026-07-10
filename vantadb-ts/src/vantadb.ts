import { VantaDB as WasmVantaDB } from "vantadb-wasm";

import { VantaError, wrapWasmError } from "./errors.js";

import type {
  Capabilities,
  ExportReport,
  GraphBfsResult,
  GraphDfsResult,
  GraphTopologicalSortResult,
  ImportReport,
  ListOptions,
  MemoryInput,
  MemoryListPage,
  MemoryRecord,
  NodeRecord,
  OperationalMetrics,
  QueryResult,
  SearchHit,
  SearchRequest,
  VantaConfig,
  VantaValue,
} from "./types.js";

function _mapRecord(r: unknown): MemoryRecord {
  if (!r || typeof r !== "object") {
    throw new VantaError(
      "VALIDATION_ERROR",
      "_mapRecord: expected an object, got " + typeof r,
    );
  }
  const obj = r as Record<string, unknown>;
  if (typeof obj.namespace !== "string") {
    throw new VantaError(
      "VALIDATION_ERROR",
      "_mapRecord: record missing required string field 'namespace'",
    );
  }
  if (typeof obj.key !== "string") {
    throw new VantaError(
      "VALIDATION_ERROR",
      "_mapRecord: record missing required string field 'key'",
    );
  }
  if (typeof obj.payload !== "string") {
    throw new VantaError(
      "VALIDATION_ERROR",
      "_mapRecord: record missing required string field 'payload'",
    );
  }
  return r as MemoryRecord;
}

export class VantaDB {
  private inner: WasmVantaDB;
  private _closed: boolean = false;

  private constructor(inner: WasmVantaDB) {
    this.inner = inner;
  }

  /**
   * Connect to a VantaDB database with persistent storage.
   *
   * @param path - Filesystem path for persistent storage. Omit or pass `":memory:"` for in-memory.
   * @returns A new VantaDB instance.
   * @throws {VantaError} If the WASM engine fails to initialise.
   *
   * @example
   * ```ts
   * // In-memory
   * const db = VantaDB.connect();
   * // Persistent
   * const db = VantaDB.connect("./my_brain");
   * ```
   */
  static connect(path?: string): VantaDB {
    try {
      const inner = path && path !== ":memory:"
        ? WasmVantaDB.open(path)
        : new WasmVantaDB(null);
      return new VantaDB(inner);
    } catch (e) {
      throw wrapWasmError(e, "connect");
    }
  }

  /**
   * Create a new VantaDB instance with the given config.
   *
   * Note: In WASM mode, `storage_path` is accepted but ignored — the
   * WASM backend always uses an in-memory engine. For persistent storage, use `connect()`.
   *
   * @param config - Optional configuration.
   * @returns A new VantaDB instance.
   * @throws {VantaError} If the WASM engine fails to initialise.
   *
   * @example
   * ```ts
   * const db = VantaDB.create({ memory_limit: 1073741824 });
   * ```
   */
  static create(config?: VantaConfig): VantaDB {
    if (config?.storage_path) {
      console.warn(
        "VantaDB.create(): storage_path is ignored in WASM mode — the WASM backend always uses an in-memory engine.",
      );
    }
    try {
      const inner = new WasmVantaDB(config ?? null);
      return new VantaDB(inner);
    } catch (e) {
      throw wrapWasmError(e, "create");
    }
  }

  /**
   * Open a persistent VantaDB database at the given path.
   *
   * @param path - Filesystem path to the database.
   * @returns A new VantaDB instance.
   * @throws {VantaError} If the WASM engine fails to open the database.
   *
   * @example
   * ```ts
   * const db = VantaDB.open("./my_brain");
   * ```
   */
  static open(path: string): VantaDB {
    try {
      const inner = WasmVantaDB.open(path);
      return new VantaDB(inner);
    } catch (e) {
      throw wrapWasmError(e, "open");
    }
  }

  private _assertOpen(): void {
    if (this._closed) {
      throw new VantaError("CLOSED", "VantaDB instance is closed");
    }
  }

  /**
   * Close the database and release underlying WASM engine resources.
   *
   * After close(), all public methods throw VantaError with code "CLOSED".
   * Calling close() multiple times is safe (no-op on subsequent calls).
   *
   * @throws {VantaError} If the WASM engine fails during close.
   *
   * @example
   * ```ts
   * db.close();
   * ```
   */
  close(): void {
    if (this._closed) return;
    try {
      this.inner.close();
    } catch (e) {
      throw wrapWasmError(e, "close");
    } finally {
      this._closed = true;
    }
  }

  /**
   * Get the capabilities of the underlying WASM engine.
   *
   * @returns The engine capabilities.
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * const caps = db.capabilities();
   * console.log(caps.vector_search); // true
   * ```
   */
  capabilities(): Capabilities {
    this._assertOpen();
    try {
      const raw = this.inner.capabilities();
      return {
        runtime_profile: raw.runtime_profile,
        persistence: raw.persistence,
        vector_search: raw.vector_search,
        iql_queries: raw.iql_queries,
        read_only: raw.read_only,
      };
    } catch (e) {
      throw wrapWasmError(e, "capabilities");
    }
  }

  /**
   * Store a memory record.
   *
   * @param input - The memory record to store.
   * @returns The stored record with system-generated fields populated.
   * @throws {VantaError} If the namespace or key is empty, or if the instance is closed.
   *
   * @example
   * ```ts
   * const record = db.put({
   *   namespace: "docs",
   *   key: "welcome",
   *   payload: "Hello, world!",
   *   metadata: { source: { type: "String", value: "manual" } },
   *   vector: [0.1, 0.2, 0.3],
   * });
   * console.log(record.version); // "1"
   * ```
   */
  put(input: MemoryInput): MemoryRecord {
    this._assertOpen();
    try {
      const raw = this.inner.put(input);
      return _mapRecord(raw);
    } catch (e) {
      throw wrapWasmError(e, "put");
    }
  }

  /**
   * Store multiple memory records in a single batch operation.
   *
   * @param inputs - Array of memory records to store.
   * @returns Array of stored records in the same order as the input.
   * @throws {VantaError} If any input is invalid, or if the instance is closed.
   *
   * @example
   * ```ts
   * const records = db.putBatch([
   *   { namespace: "docs", key: "a", payload: "first" },
   *   { namespace: "docs", key: "b", payload: "second" },
   * ]);
   * ```
   */
  putBatch(inputs: MemoryInput[]): MemoryRecord[] {
    this._assertOpen();
    try {
      const records = this.inner.put_batch(inputs) as unknown[];
      for (let i = 0; i < records.length; i++) {
        records[i] = _mapRecord(records[i]);
      }
      return records as MemoryRecord[];
    } catch (e) {
      throw wrapWasmError(e, "putBatch");
    }
  }

  /**
   * Retrieve a memory record by namespace and key.
   *
   * @param namespace - The namespace.
   * @param key - The record key.
   * @returns The record if found, or null if it does not exist.
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * const record = db.get("docs", "welcome");
   * if (record) console.log(record.payload);
   * ```
   */
  get(namespace: string, key: string): MemoryRecord | null {
    this._assertOpen();
    try {
      const raw = this.inner.get(namespace, key);
      return raw != null ? _mapRecord(raw) : null;
    } catch (e) {
      throw wrapWasmError(e, "get");
    }
  }

  /**
   * Delete a memory record by namespace and key.
   *
   * @param namespace - The namespace.
   * @param key - The record key.
   * @returns true if the record was deleted, false if it did not exist.
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * const deleted = db.delete("docs", "welcome");
   * ```
   */
  delete(namespace: string, key: string): boolean {
    this._assertOpen();
    try {
      return this.inner.delete(namespace, key);
    } catch (e) {
      throw wrapWasmError(e, "delete");
    }
  }

  /**
   * List all namespaces in the database.
   *
   * @returns Array of namespace strings.
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * const namespaces = db.listNamespaces();
   * ```
   */
  listNamespaces(): string[] {
    this._assertOpen();
    try {
      return this.inner.list_namespaces();
    } catch (e) {
      throw wrapWasmError(e, "listNamespaces");
    }
  }

  /**
   * List memory records in a namespace with pagination.
   *
   * @param namespace - The namespace to list.
   * @param options - Pagination options (limit, cursor, filters).
   * @returns A page of records with an optional cursor for continuation.
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * const page = db.list("docs", { limit: 10 });
   * while (page.records.length) {
   *   for (const r of page.records) console.log(r.key);
   *   if (!page.next_cursor) break;
   *   page = db.list("docs", { limit: 10, cursor: page.next_cursor });
   * }
   * ```
   */
  list(namespace: string, options: ListOptions = {}): MemoryListPage {
    this._assertOpen();
    try {
      const raw = this.inner.list(namespace, options);
      const items: unknown[] = raw.records ?? [];
      for (let i = 0; i < items.length; i++) {
        items[i] = _mapRecord(items[i]);
      }
      return {
        records: items as MemoryRecord[],
        next_cursor: raw.next_cursor,
      };
    } catch (e) {
      throw wrapWasmError(e, "list");
    }
  }

  private _buildSearchRequest(request: SearchRequest, explain?: boolean): Record<string, unknown> {
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

  /**
   * Search for memory records by vector similarity, with optional text + hybrid search.
   *
   * @param request - The search request parameters.
   * @returns Array of search hits ordered by relevance (closest first).
   * @throws {VantaError} If the instance is closed or the search fails.
   *
   * @example
   * ```ts
   * const hits = db.search({
   *   namespace: "docs",
   *   query_vector: [0.1, 0.2, 0.3],
   *   top_k: 5,
   * });
   * for (const hit of hits) {
   *   console.log(hit.record.payload, hit.distance);
   * }
   * ```
   */
  search(request: SearchRequest): SearchHit[] {
    this._assertOpen();
    try {
      const raw = this.inner.search(this._buildSearchRequest(request)) as unknown[];
      return raw.map((hit: unknown) => {
        const h = hit as Record<string, unknown>;
        return {
          record: _mapRecord(h.record),
          distance: h.score as number,
          explanation: (h.explanation ?? undefined) as SearchHit["explanation"],
        };
      });
    } catch (e) {
      throw wrapWasmError(e, "search");
    }
  }

  /**
   * Search for graph nodes by vector similarity (low-level API).
   *
   * @param vector - Query vector (number array or Float32Array).
   * @param topK - Maximum number of results (default: 10).
   * @returns Array of results with node IDs and distances.
   * @throws {VantaError} If the instance is closed or the vector is invalid.
   *
   * @example
   * ```ts
   * const results = db.searchVector([0.1, 0.2, 0.3, 0.4], 5);
   * for (const r of results) {
   *   console.log(r.node_id, r.distance);
   * }
   * ```
   */
  searchVector(
    vector: number[],
    topK: number = 10,
  ): { node_id: string; distance: number }[] {
    this._assertOpen();
    try {
      const raw: unknown[] = this.inner.search_vector(new Float32Array(vector), topK);
      return raw.map((hit: unknown) => {
        const h = hit as Record<string, unknown>;
        return {
          node_id: h.node_id as string,
          distance: h.score as number,
        };
      });
    } catch (e) {
      throw wrapWasmError(e, "searchVector");
    }
  }

  /**
   * Execute a search and return detailed explanation metadata.
   *
   * @param request - The search request parameters.
   * @returns Raw explanation object from the engine.
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * const explanation = db.explainSearch({
   *   namespace: "docs",
   *   query_vector: [0.1, 0.2, 0.3],
   *   text_query: "hello",
   * });
   * console.log(explanation.route);
   * ```
   */
  explainSearch(request: SearchRequest): Record<string, unknown> {
    this._assertOpen();
    try {
      return this.inner.explain_memory_search(
        this._buildSearchRequest(request, true),
      );
    } catch (e) {
      throw wrapWasmError(e, "explainSearch");
    }
  }

  /**
   * Export all records in a namespace to a file.
   *
   * @param path - Output file path.
   * @param namespace - Namespace to export.
   * @returns Export report with counts and timing.
   * @throws {VantaError} If the instance is closed or the export fails.
   *
   * @example
   * ```ts
   * const report = db.exportNamespace("./export.jsonl", "docs");
   * console.log(report.records_exported);
   * ```
   */
  exportNamespace(path: string, namespace: string): ExportReport {
    this._assertOpen();
    try {
      return this.inner.export_namespace(path, namespace);
    } catch (e) {
      throw wrapWasmError(e, "exportNamespace");
    }
  }

  /**
   * Export all records across all namespaces to a file.
   *
   * @param path - Output file path.
   * @returns Export report with counts and timing.
   * @throws {VantaError} If the instance is closed or the export fails.
   *
   * @example
   * ```ts
   * const report = db.exportAll("./backup.jsonl");
   * ```
   */
  exportAll(path: string): ExportReport {
    this._assertOpen();
    try {
      return this.inner.export_all(path);
    } catch (e) {
      throw wrapWasmError(e, "exportAll");
    }
  }

  /**
   * Import records from an array.
   *
   * @param records - Array of memory record inputs to import.
   * @returns Import report with counts and timing.
   * @throws {VantaError} If the instance is closed or the import fails.
   *
   * @example
   * ```ts
   * const report = db.importRecords([
   *   { namespace: "docs", key: "a", payload: "hello" },
   * ]);
   * console.log(report.inserted);
   * ```
   */
  importRecords(records: MemoryInput[]): ImportReport {
    this._assertOpen();
    try {
      return this.inner.import_records(records);
    } catch (e) {
      throw wrapWasmError(e, "importRecords");
    }
  }

  /**
   * Import records from a JSONL file.
   *
   * @param path - Path to the JSONL file.
   * @returns Import report with counts and timing.
   * @throws {VantaError} If the instance is closed or the file cannot be read.
   *
   * @example
   * ```ts
   * const report = db.importFile("./backup.jsonl");
   * ```
   */
  importFile(path: string): ImportReport {
    this._assertOpen();
    try {
      return this.inner.import_file(path);
    } catch (e) {
      throw wrapWasmError(e, "importFile");
    }
  }

  /**
   * Rebuild the ANN index from scratch.
   *
   * @returns Engine-specific rebuild result.
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * const result = db.rebuildIndex();
   * ```
   */
  rebuildIndex(): unknown {
    this._assertOpen();
    try {
      return this.inner.rebuild_index();
    } catch (e) {
      throw wrapWasmError(e, "rebuildIndex");
    }
  }

  /**
   * Compact the internal storage layout to reclaim space.
   *
   * @returns Number of bytes reclaimed (bigint).
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * const reclaimed = db.compactLayout();
   * ```
   */
  compactLayout(): bigint {
    this._assertOpen();
    try {
      return this.inner.compact_layout();
    } catch (e) {
      throw wrapWasmError(e, "compactLayout");
    }
  }

  /**
   * Audit the text index for consistency.
   *
   * @param namespace - Optional namespace to scope the audit.
   * @returns Audit report.
   * @throws {VantaError} If the instance is closed.
   */
  auditTextIndex(namespace?: string): unknown {
    this._assertOpen();
    try {
      return this.inner.audit_text_index(namespace ?? null);
    } catch (e) {
      throw wrapWasmError(e, "auditTextIndex");
    }
  }

  /**
   * Deep audit of the text index with detailed diagnostics.
   *
   * @param namespace - Optional namespace to scope the audit.
   * @returns Detailed audit report.
   * @throws {VantaError} If the instance is closed.
   */
  auditTextIndexDeep(namespace?: string): unknown {
    this._assertOpen();
    try {
      return this.inner.audit_text_index_deep(namespace ?? null);
    } catch (e) {
      throw wrapWasmError(e, "auditTextIndexDeep");
    }
  }

  /**
   * Repair the text index if inconsistencies are detected.
   *
   * @returns Repair report.
   * @throws {VantaError} If the instance is closed.
   */
  repairTextIndex(): unknown {
    this._assertOpen();
    try {
      return this.inner.repair_text_index();
    } catch (e) {
      throw wrapWasmError(e, "repairTextIndex");
    }
  }

  /**
   * Flush all pending writes to storage.
   *
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * db.flush();
   * ```
   */
  flush(): void {
    this._assertOpen();
    try {
      this.inner.flush();
    } catch (e) {
      throw wrapWasmError(e, "flush");
    }
  }

  /**
   * Compact the write-ahead log (WAL) to reclaim space.
   *
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * db.compactWal();
   * ```
   */
  compactWal(): void {
    this._assertOpen();
    try {
      this.inner.compact_wal();
    } catch (e) {
      throw wrapWasmError(e, "compactWal");
    }
  }

  /**
   * Purge all expired records (those past their TTL).
   *
   * @returns Number of records purged (bigint).
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * const purged = db.purgeExpired();
   * ```
   */
  purgeExpired(): bigint {
    this._assertOpen();
    try {
      return this.inner.purge_expired();
    } catch (e) {
      throw wrapWasmError(e, "purgeExpired");
    }
  }

  /**
   * Get operational metrics from the engine.
   *
   * @returns Current operational metrics.
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * const m = db.operationalMetrics();
   * console.log(m.hnsw_nodes_count);
   * ```
   */
  operationalMetrics(): OperationalMetrics {
    this._assertOpen();
    try {
      return this.inner.operational_metrics();
    } catch (e) {
      throw wrapWasmError(e, "operationalMetrics");
    }
  }

  /**
   * Execute an IQL (Intelligence Query Language) query against the graph.
   *
   * @param query - IQL query string (LISP-like syntax).
   * @returns Query result containing nodes or write confirmation.
   * @throws {VantaError} If the instance is closed or the query is invalid.
   *
   * @example
   * ```ts
   * const result = db.query("(entity :id 1)");
   * if (result.Read) console.log(result.Read.length, "nodes found");
   * ```
   */
  query(query: string): QueryResult {
    this._assertOpen();
    try {
      return this.inner.query(query);
    } catch (e) {
      throw wrapWasmError(e, "query");
    }
  }

  /**
   * Insert a graph node.
   *
   * For IDs > 2^53, use bigint — JavaScript Numbers lose integer precision
   * above 2^53.
   *
   * @param id - Node ID (number or bigint).
   * @param content - Optional content string.
   * @param vector - Optional embedding vector.
   * @param fields - Optional typed metadata fields.
   * @throws {VantaError} If the ID is not a safe integer, or if the instance is closed.
   *
   * @example
   * ```ts
   * db.insertNode(1, "root", [0.1, 0.2], { tag: { type: "String", value: "important" } });
   * ```
   */
  insertNode(
    id: number | bigint,
    content?: string,
    vector?: number[],
    fields: Record<string, VantaValue> = {},
  ): void {
    this._assertOpen();
    if (typeof id === "number" && !Number.isSafeInteger(id)) {
      throw new VantaError(
        "INVALID_ARGUMENT",
        `insertNode: id ${id} is not a safe integer — JavaScript numbers lose precision above 2^53. Use bigint for large IDs.`,
      );
    }
    try {
      this.inner.insert_node(
        BigInt(id),
        content ?? null,
        vector ? new Float32Array(vector) : null,
        fields,
      );
    } catch (e) {
      throw wrapWasmError(e, "insertNode");
    }
  }

  /**
   * Retrieve a graph node by ID.
   *
   * @param id - Node ID.
   * @returns The node record if found, or null if it does not exist.
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * const node = db.getNode(1);
   * if (node) console.log(node.edges.length, "edges");
   * ```
   */
  getNode(id: number): NodeRecord | null {
    this._assertOpen();
    try {
      const raw = this.inner.get_node(BigInt(id));
      if (raw == null) return null;
      return raw as NodeRecord;
    } catch (e) {
      throw wrapWasmError(e, "getNode");
    }
  }

  /**
   * Delete a graph node.
   *
   * @param id - Node ID.
   * @param reason - Deletion reason (default: "deleted").
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * db.deleteNode(1, "no longer needed");
   * ```
   */
  deleteNode(id: number, reason: string = "deleted"): void {
    this._assertOpen();
    try {
      this.inner.delete_node(BigInt(id), reason);
    } catch (e) {
      throw wrapWasmError(e, "deleteNode");
    }
  }

  /**
   * Add a directed edge between two graph nodes.
   *
   * @param source - Source node ID.
   * @param target - Target node ID.
   * @param label - Edge label (default: "").
   * @param weight - Optional edge weight.
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * db.addEdge(1, 2, "knows", 0.8);
   * ```
   */
  addEdge(
    source: number,
    target: number,
    label: string = "",
    weight?: number,
  ): void {
    this._assertOpen();
    try {
      this.inner.add_edge(BigInt(source), BigInt(target), label, weight ?? null);
    } catch (e) {
      throw wrapWasmError(e, "addEdge");
    }
  }

  /**
   * Perform a breadth-first search (BFS) traversal of the graph.
   *
   * @param roots - Array of root node IDs to start from.
   * @param maxDepth - Maximum traversal depth (default: 10).
   * @returns BFS result with visited nodes and levels.
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * const result = db.graphBfs([1, 2], 3);
   * console.log(result.visited);
   * ```
   */
  graphBfs(roots: number[], maxDepth: number = 10): GraphBfsResult {
    this._assertOpen();
    try {
      return this.inner.graph_bfs(
        new BigUint64Array(roots.map(BigInt)),
        maxDepth,
      ) as GraphBfsResult;
    } catch (e) {
      throw wrapWasmError(e, "graphBfs");
    }
  }

  /**
   * Perform a depth-first search (DFS) traversal of the graph.
   *
   * @param roots - Array of root node IDs to start from.
   * @param maxDepth - Maximum traversal depth (default: 10).
   * @returns DFS result with visited nodes and order.
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * const result = db.graphDfs([1], 5);
   * ```
   */
  graphDfs(roots: number[], maxDepth: number = 10): GraphDfsResult {
    this._assertOpen();
    try {
      return this.inner.graph_dfs(
        new BigUint64Array(roots.map(BigInt)),
        maxDepth,
      ) as GraphDfsResult;
    } catch (e) {
      throw wrapWasmError(e, "graphDfs");
    }
  }

  /**
   * Perform a topological sort on the graph starting from the given roots.
   *
   * @param roots - Array of root node IDs.
   * @returns Topological sort result.
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * const result = db.graphTopologicalSort([1]);
   * if (result.has_cycle) console.warn("Graph has a cycle!");
   * ```
   */
  graphTopologicalSort(roots: number[]): GraphTopologicalSortResult {
    this._assertOpen();
    try {
      return this.inner.graph_topological_sort(
        new BigUint64Array(roots.map(BigInt)),
      ) as GraphTopologicalSortResult;
    } catch (e) {
      throw wrapWasmError(e, "graphTopologicalSort");
    }
  }

  /**
   * Check if the subgraph reachable from the given roots is a DAG (acyclic).
   *
   * @param roots - Array of root node IDs.
   * @returns true if the graph is a DAG (no cycles detected).
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * const isDag = db.graphIsDag([1]);
   * ```
   */
  graphIsDag(roots: number[]): boolean {
    this._assertOpen();
    try {
      return this.inner.graph_is_dag(new BigUint64Array(roots.map(BigInt)));
    } catch (e) {
      throw wrapWasmError(e, "graphIsDag");
    }
  }

  /**
   * Generate a text snippet with highlighting around query terms.
   *
   * @param payload - The source text to generate a snippet from.
   * @param query - The query string to highlight.
   * @param withHighlighting - If true, wrap matching terms in highlighting markers.
   * @returns The generated snippet, or undefined if snippet generation is not available.
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * const snippet = db.generateSnippet(
   *   "VantaDB is a vector database for AI agents",
   *   "vector database",
   *   true
   * );
   * ```
   */
  generateSnippet(
    payload: string,
    query: string,
    withHighlighting: boolean = false,
  ): string | undefined {
    this._assertOpen();
    try {
      return this.inner.generate_snippet(payload, query, withHighlighting) ?? undefined;
    } catch (e) {
      throw wrapWasmError(e, "generateSnippet");
    }
  }
}

export { VantaError } from "./errors.js";
export {
  isMemoryRecord,
  isSearchHit,
  isNodeRecord,
  isValidVantaValue,
  isVantaMetadata,
  isValidVector,
  validateVector,
} from "./guards.js";
