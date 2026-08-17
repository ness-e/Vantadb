import { VantaDB as WasmVantaDB } from "vantadb-wasm";

import { VantaError, wrapWasmError } from "./errors.js";
import { isMemoryRecord } from "./guards.js";

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
  if (!isMemoryRecord(r)) {
    throw new VantaError(
      "VALIDATION_ERROR",
      "_mapRecord: invalid MemoryRecord structure or missing required fields",
    );
  }
  return r;
}

export class VantaDB {
  private inner: WasmVantaDB;
  private _closed: boolean = false;

  private constructor(inner: WasmVantaDB) {
    this.inner = inner;
  }

  /** Wraps a WASM call with a uniform error boundary. */
  private _wasm<T>(method: string, fn: () => T): T {
    try {
      return fn();
    } catch (e) {
      throw wrapWasmError(e, method);
    }
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

  // NOTE: static factory methods (connect/create/open) cannot use _wasm() since
  // the instance does not yet exist. They keep their own try-catch.

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
        "VantaDB.create(): storage_path is ignored unless a persistent backend is connected via connect_persistent(), connect_idb(), or connect_worker().",
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
    return this._wasm("capabilities", () => {
      const raw = this.inner.capabilities();
      return {
        runtime_profile: raw.runtime_profile,
        persistence: raw.persistence,
        vector_search: raw.vector_search,
        iql_queries: raw.iql_queries,
        read_only: raw.read_only,
      };
    });
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
    return this._wasm("put", () => _mapRecord(this.inner.put(input)));
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
    return this._wasm("putBatch", () => {
      const records = this.inner.put_batch(inputs) as unknown[];
      for (let i = 0; i < records.length; i++) {
        records[i] = _mapRecord(records[i]);
      }
      return records as MemoryRecord[];
    });
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
    return this._wasm("get", () => {
      const raw = this.inner.get(namespace, key);
      return raw != null ? _mapRecord(raw) : null;
    });
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
    return this._wasm("delete", () => this.inner.delete(namespace, key));
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
    return this._wasm("listNamespaces", () => this.inner.list_namespaces());
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
    return this._wasm("list", () => {
      const raw = this.inner.list(namespace, options);
      const items: unknown[] = raw.records ?? [];
      for (let i = 0; i < items.length; i++) {
        items[i] = _mapRecord(items[i]);
      }
      return {
        records: items as MemoryRecord[],
        next_cursor: raw.next_cursor,
      };
    });
  }

  private _buildSearchRequest(request: SearchRequest, explain?: boolean): Record<string, unknown> {
    // ERR-028 (AUDREP-55): a zero-norm cosine query vector is undefined
    // (cosine = 0/0). The core rejects it with VantaError::InvalidInput
    // (src/sdk/search/mod.rs) and that error surfaces here via the WASM
    // binding — this layer is glue and must NOT make search decisions
    // (api-contract.md R-8). Pass the request through untouched, like
    // native.ts, so both backends behave identically.
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
    return this._wasm("search", () => {
      const raw = this.inner.search(this._buildSearchRequest(request)) as unknown[];
      return raw.map((hit: unknown) => {
        const h = hit as Record<string, unknown>;
        return {
          record: _mapRecord(h.record),
          distance: h.score as number,
          explanation: (h.explanation ?? undefined) as SearchHit["explanation"],
        };
      });
    });
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
    return this._wasm("searchVector", () => {
      const raw: unknown[] = this.inner.search_vector(new Float32Array(vector), topK);
      return raw.map((hit: unknown) => {
        const h = hit as Record<string, unknown>;
        return {
          node_id: h.node_id as string,
          distance: h.score as number,
        };
      });
    });
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
    return this._wasm("explainSearch", () =>
      this.inner.explain_memory_search(this._buildSearchRequest(request, true)),
    );
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
    return this._wasm("exportNamespace", () => this.inner.export_namespace(path, namespace));
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
    return this._wasm("exportAll", () => this.inner.export_all(path));
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
    return this._wasm("importRecords", () => this.inner.import_records(records));
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
    return this._wasm("importFile", () => this.inner.import_file(path));
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
    return this._wasm("rebuildIndex", () => this.inner.rebuild_index());
  }

  /**
   * Rebuild the HNSW vector index from stored vectors with pagination.
   *
   * Paginates through memory records in batches (max 1000) using the
   * cursor-based `list()` API to prevent OOM on large namespaces.
   *
   * @param namespace - The namespace to rebuild.
   * @param pageSize - Batch size (default 1000, max 1000).
   * @returns A rebuild report with scanned and indexed counts.
   * @throws {VantaError} If the instance is closed.
   *
   * @example
   * ```ts
   * const report = db.reindexHnswFromText("docs");
   * console.log(report.scanned_nodes, "nodes reindexed");
   * ```
   */
  reindexHnswFromText(namespace: string, pageSize: number = 1000): unknown {
    this._assertOpen();
    return this._wasm("reindexHnswFromText", () => this.inner.reindex_hnsw_from_text(namespace, pageSize));
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
    return this._wasm("compactLayout", () => this.inner.compact_layout());
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
    return this._wasm("auditTextIndex", () => this.inner.audit_text_index(namespace ?? null));
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
    return this._wasm("auditTextIndexDeep", () => this.inner.audit_text_index_deep(namespace ?? null));
  }

  /**
   * Repair the text index if inconsistencies are detected.
   *
   * @returns Repair report.
   * @throws {VantaError} If the instance is closed.
   */
  repairTextIndex(): unknown {
    this._assertOpen();
    return this._wasm("repairTextIndex", () => this.inner.repair_text_index());
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
    this._wasm("flush", () => this.inner.flush());
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
    this._wasm("compactWal", () => this.inner.compact_wal());
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
    return this._wasm("purgeExpired", () => this.inner.purge_expired());
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
    return this._wasm("operationalMetrics", () => this.inner.operational_metrics());
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
    return this._wasm("query", () => this.inner.query(query));
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
    this._wasm("insertNode", () =>
      this.inner.insert_node(
        String(id),
        content ?? null,
        vector ? new Float32Array(vector) : null,
        fields,
      ),
    );
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
    return this._wasm("getNode", () => {
      const raw = this.inner.get_node(String(id));
      if (raw == null) return null;
      const node = raw as NodeRecord;
      // WASM serializes u128 edge targets as strings; expose them as bigint
      // (the SDK contract — see integration.test.ts "graph operations").
      node.edges = node.edges.map((e) => ({
        ...e,
        target: typeof e.target === "string" ? BigInt(e.target) : e.target,
      }));
      return node;
    });
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
    this._wasm("deleteNode", () => this.inner.delete_node(String(id), reason));
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
    createdAtMs?: number,
  ): void {
    this._assertOpen();
    this._wasm("addEdge", () =>
      this.inner.add_edge(
        String(source),
        String(target),
        label,
        weight ?? null,
        createdAtMs != null ? BigInt(createdAtMs) : null,
      ),
    );
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
  graphBfs(
    roots: number[],
    maxDepth: number = 10,
    direction: "Forward" | "Reverse" | "Both" = "Forward",
  ): GraphBfsResult {
    this._assertOpen();
    return this._wasm("graphBfs", () =>
      this.inner.graph_bfs(roots.map(String), maxDepth, direction) as GraphBfsResult,
    );
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
  graphDfs(
    roots: number[],
    maxDepth: number = 10,
    direction: "Forward" | "Reverse" | "Both" = "Forward",
  ): GraphDfsResult {
    this._assertOpen();
    return this._wasm("graphDfs", () =>
      this.inner.graph_dfs(roots.map(String), maxDepth, direction) as GraphDfsResult,
    );
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
    return this._wasm("graphTopologicalSort", () =>
      this.inner.graph_topological_sort(
        roots.map(String),
      ) as GraphTopologicalSortResult,
    );
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
    return this._wasm("graphIsDag", () =>
      this.inner.graph_is_dag(roots.map(String)),
    );
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
    return this._wasm("generateSnippet", () =>
      this.inner.generate_snippet(payload, query, withHighlighting) ?? undefined,
    );
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
export * from "./native.js";
