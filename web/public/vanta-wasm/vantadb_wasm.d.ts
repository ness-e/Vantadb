declare namespace wasm_bindgen {
    /* tslint:disable */
    /* eslint-disable */

    /**
     * The main VantaDB handle exposed to JavaScript via `wasm_bindgen`.
     */
    export class VantaDB {
        free(): void;
        [Symbol.dispose](): void;
        /**
         * Add a directed edge between two graph nodes with an optional weight
         * and creation timestamp (Unix ms).
         */
        add_edge(source_id: bigint, target_id: bigint, label: string, weight?: number | null, created_at_ms?: bigint | null): void;
        /**
         * Run a text index consistency audit for an optional namespace.
         */
        audit_text_index(namespace?: string | null): any;
        /**
         * Run a deep text index consistency audit for an optional namespace.
         */
        audit_text_index_deep(namespace?: string | null): any;
        /**
         * Bulk-import records from a binary .vdbdump file.
         * Returns a report object with total_records, batches_committed, duration_ms.
         */
        bulk_import(path: string): any;
        /**
         * Bulk-import records from binary bytes (.vdbdump format).
         * Accepts a Uint8Array and returns a report object.
         */
        bulk_import_bytes(data: Uint8Array): any;
        /**
         * Return the capabilities object describing supported features.
         */
        capabilities(): any;
        /**
         * Close the database and release underlying engine resources.
         * After close, the VantaDB handle should not be used for further operations.
         * This does NOT free the JS wrapper object — callers should drop references
         * after close to allow WASM GC to reclaim the wrapper.
         */
        close(): void;
        /**
         * Compact the storage layout and return the number of freed bytes.
         */
        compact_layout(): bigint;
        /**
         * Compact the write-ahead log.
         */
        compact_wal(): void;
        /**
         * Open VantaDB with IndexedDB-based persistent storage (fallback when OPFS is unavailable).
         */
        static connect_idb(path: string): Promise<VantaDB>;
        /**
         * Open VantaDB with OPFS-based persistent storage in the browser.
         */
        static connect_persistent(path: string): Promise<VantaDB>;
        /**
         * Delete a single record by namespace and key. Returns whether a record was deleted.
         */
        delete(namespace: string, key: string): boolean;
        /**
         * Delete persisted state from IndexedDB.
         */
        delete_idb(): Promise<void>;
        /**
         * Delete a graph node by ID with an associated reason string.
         */
        delete_node(id: bigint, reason: string): void;
        /**
         * Run a search with explanation metadata for debugging scoring.
         */
        explain_memory_search(request: any): any;
        /**
         * Export all records across all namespaces to the given path.
         */
        export_all(path: string): any;
        /**
         * Export all records in a namespace to a JSON file at the given path.
         */
        export_namespace(path: string, namespace: string): any;
        /**
         * Flush all pending writes to disk.
         */
        flush(): void;
        /**
         * Generate a text snippet with optional highlighting for a given query.
         */
        generate_snippet(payload: string, text_query: string, with_highlighting: boolean): string | undefined;
        /**
         * Retrieve a single record by namespace and key.
         */
        get(namespace: string, key: string): any;
        /**
         * Retrieve a graph node by its numeric ID.
         */
        get_node(id: bigint): any;
        /**
         * Perform a breadth-first traversal from the given root node IDs.
         */
        graph_bfs(roots: BigUint64Array, max_depth: number, direction: string): any;
        /**
         * Perform a depth-first traversal from the given root node IDs.
         */
        graph_dfs(roots: BigUint64Array, max_depth: number, direction: string): any;
        /**
         * Return whether the subgraph reachable from the given roots forms a DAG.
         */
        graph_is_dag(roots: BigUint64Array): boolean;
        /**
         * Compute a topological sort order starting from the given root node IDs.
         */
        graph_topological_sort(roots: BigUint64Array): any;
        /**
         * Import records from a JSON file at the given path.
         */
        import_file(path: string): any;
        /**
         * Import records from a JS array of memory record objects.
         */
        import_records(records: any): any;
        /**
         * Insert a graph node with optional content, vector, and fields.
         */
        insert_node(id: bigint, content: string | null | undefined, vector: Float32Array | null | undefined, fields: any): void;
        /**
         * List records in a namespace with optional filters, limit, and cursor pagination.
         */
        list(namespace: string, options: any): any;
        /**
         * Return all namespaces as a JS array of strings.
         */
        list_namespaces(): any;
        /**
         * Restore all records from OPFS storage into memory.
         */
        load(): Promise<void>;
        /**
         * Restore all records from IndexedDB storage into memory.
         */
        load_idb(): Promise<void>;
        /**
         * Create a new VantaDB instance from an optional WASM config object.
         */
        constructor(config_val?: any | null);
        /**
         * Open VantaDB at the given storage path.
         */
        static open(path: string): VantaDB;
        /**
         * Return operational metrics as a JS object with stringified large numbers.
         */
        operational_metrics(): any;
        /**
         * Purge all expired records and return the number removed.
         */
        purge_expired(): bigint;
        /**
         * Insert or update a single memory record from a JS object.
         */
        put(input: any): any;
        /**
         * Insert or update multiple memory records from a JS array.
         */
        put_batch(inputs: any): any;
        /**
         * Execute a raw DSL query string and return the result.
         */
        query(query: string): any;
        /**
         * Rebuild the HNSW index and return a rebuild report.
         */
        rebuild_index(): any;
        /**
         * Paginated HNSW rebuild from text records.
         *
         * Iterates through memory records in batches (max 1000) using the
         * cursor-based list() API to prevent OOM on large namespaces.
         */
        reindex_hnsw_from_text(namespace: string, page_size?: number | null): any;
        /**
         * Repair the text index and return a repair report.
         */
        repair_text_index(): any;
        /**
         * Persist all in-memory records to OPFS storage.
         */
        save(): Promise<void>;
        /**
         * Persist all in-memory records to IndexedDB storage.
         */
        save_idb(): Promise<void>;
        /**
         * Search memory records by vector similarity with optional filters and text query.
         */
        search(request: any): any;
        /**
         * Search nodes by raw vector without namespace scoping.
         */
        search_vector(vector: Float32Array, top_k: number): any;
    }

}
declare type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

declare interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_vantadb_free: (a: number, b: number) => void;
    readonly vantadb_add_edge: (a: number, b: bigint, c: bigint, d: number, e: number, f: number, g: number, h: bigint) => [number, number];
    readonly vantadb_audit_text_index: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_audit_text_index_deep: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_bulk_import: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_bulk_import_bytes: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_capabilities: (a: number) => [number, number, number];
    readonly vantadb_close: (a: number) => [number, number];
    readonly vantadb_compact_layout: (a: number) => [bigint, number, number];
    readonly vantadb_compact_wal: (a: number) => [number, number];
    readonly vantadb_connect_idb: (a: number, b: number) => any;
    readonly vantadb_connect_persistent: (a: number, b: number) => any;
    readonly vantadb_delete: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly vantadb_delete_idb: (a: number) => any;
    readonly vantadb_delete_node: (a: number, b: bigint, c: number, d: number) => [number, number];
    readonly vantadb_explain_memory_search: (a: number, b: any) => [number, number, number];
    readonly vantadb_export_all: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_export_namespace: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly vantadb_flush: (a: number) => [number, number];
    readonly vantadb_generate_snippet: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly vantadb_get: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly vantadb_get_node: (a: number, b: bigint) => [number, number, number];
    readonly vantadb_graph_bfs: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly vantadb_graph_dfs: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly vantadb_graph_is_dag: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_graph_topological_sort: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_import_file: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_import_records: (a: number, b: any) => [number, number, number];
    readonly vantadb_insert_node: (a: number, b: bigint, c: number, d: number, e: number, f: number, g: any) => [number, number];
    readonly vantadb_list: (a: number, b: number, c: number, d: any) => [number, number, number];
    readonly vantadb_list_namespaces: (a: number) => [number, number, number];
    readonly vantadb_load: (a: number) => any;
    readonly vantadb_load_idb: (a: number) => any;
    readonly vantadb_new: (a: number) => [number, number, number];
    readonly vantadb_open: (a: number, b: number) => [number, number, number];
    readonly vantadb_operational_metrics: (a: number) => [number, number, number];
    readonly vantadb_purge_expired: (a: number) => [bigint, number, number];
    readonly vantadb_put: (a: number, b: any) => [number, number, number];
    readonly vantadb_put_batch: (a: number, b: any) => [number, number, number];
    readonly vantadb_query: (a: number, b: number, c: number) => [number, number, number];
    readonly vantadb_rebuild_index: (a: number) => [number, number, number];
    readonly vantadb_reindex_hnsw_from_text: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly vantadb_repair_text_index: (a: number) => [number, number, number];
    readonly vantadb_save: (a: number) => any;
    readonly vantadb_save_idb: (a: number) => any;
    readonly vantadb_search: (a: number, b: any) => [number, number, number];
    readonly vantadb_search_vector: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly wasm_bindgen__convert__closures_____invoke__he5211670e4cc5c11: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen__convert__closures_____invoke__h0bf0d27f3df94d79: (a: number, b: number, c: any, d: any) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_destroy_closure: (a: number, b: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
declare function wasm_bindgen (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
