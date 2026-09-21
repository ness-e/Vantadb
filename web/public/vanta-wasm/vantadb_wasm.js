let wasm_bindgen = (function(exports) {
    let script_src;
    if (typeof document !== 'undefined' && document.currentScript !== null) {
        script_src = new URL(document.currentScript.src, location.href).toString();
    }

    /**
     * The main VantaDB handle exposed to JavaScript via `wasm_bindgen`.
     */
    class VantaDB {
        static __wrap(ptr) {
            const obj = Object.create(VantaDB.prototype);
            obj.__wbg_ptr = ptr;
            VantaDBFinalization.register(obj, obj.__wbg_ptr, obj);
            return obj;
        }
        __destroy_into_raw() {
            const ptr = this.__wbg_ptr;
            this.__wbg_ptr = 0;
            VantaDBFinalization.unregister(this);
            return ptr;
        }
        free() {
            const ptr = this.__destroy_into_raw();
            wasm.__wbg_vantadb_free(ptr, 0);
        }
        /**
         * Add a directed edge between two graph nodes with an optional weight
         * and creation timestamp (Unix ms).
         * @param {bigint} source_id
         * @param {bigint} target_id
         * @param {string} label
         * @param {number | null} [weight]
         * @param {bigint | null} [created_at_ms]
         */
        add_edge(source_id, target_id, label, weight, created_at_ms) {
            const ptr0 = passStringToWasm0(label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_add_edge(this.__wbg_ptr, source_id, target_id, ptr0, len0, isLikeNone(weight) ? Number.MAX_SAFE_INTEGER : Math.fround(weight), !isLikeNone(created_at_ms), isLikeNone(created_at_ms) ? BigInt(0) : created_at_ms);
            if (ret[1]) {
                throw takeFromExternrefTable0(ret[0]);
            }
        }
        /**
         * Run a text index consistency audit for an optional namespace.
         * @param {string | null} [namespace]
         * @returns {any}
         */
        audit_text_index(namespace) {
            var ptr0 = isLikeNone(namespace) ? 0 : passStringToWasm0(namespace, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len0 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_audit_text_index(this.__wbg_ptr, ptr0, len0);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Run a deep text index consistency audit for an optional namespace.
         * @param {string | null} [namespace]
         * @returns {any}
         */
        audit_text_index_deep(namespace) {
            var ptr0 = isLikeNone(namespace) ? 0 : passStringToWasm0(namespace, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len0 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_audit_text_index_deep(this.__wbg_ptr, ptr0, len0);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Bulk-import records from a binary .vdbdump file.
         * Returns a report object with total_records, batches_committed, duration_ms.
         * @param {string} path
         * @returns {any}
         */
        bulk_import(path) {
            const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_bulk_import(this.__wbg_ptr, ptr0, len0);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Bulk-import records from binary bytes (.vdbdump format).
         * Accepts a Uint8Array and returns a report object.
         * @param {Uint8Array} data
         * @returns {any}
         */
        bulk_import_bytes(data) {
            const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_bulk_import_bytes(this.__wbg_ptr, ptr0, len0);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Return the capabilities object describing supported features.
         * @returns {any}
         */
        capabilities() {
            const ret = wasm.vantadb_capabilities(this.__wbg_ptr);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Close the database and release underlying engine resources.
         * After close, the VantaDB handle should not be used for further operations.
         * This does NOT free the JS wrapper object — callers should drop references
         * after close to allow WASM GC to reclaim the wrapper.
         */
        close() {
            const ret = wasm.vantadb_close(this.__wbg_ptr);
            if (ret[1]) {
                throw takeFromExternrefTable0(ret[0]);
            }
        }
        /**
         * Compact the storage layout and return the number of freed bytes.
         * @returns {bigint}
         */
        compact_layout() {
            const ret = wasm.vantadb_compact_layout(this.__wbg_ptr);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return BigInt.asUintN(64, ret[0]);
        }
        /**
         * Compact the write-ahead log.
         */
        compact_wal() {
            const ret = wasm.vantadb_compact_wal(this.__wbg_ptr);
            if (ret[1]) {
                throw takeFromExternrefTable0(ret[0]);
            }
        }
        /**
         * Open VantaDB with IndexedDB-based persistent storage (fallback when OPFS is unavailable).
         * @param {string} path
         * @returns {Promise<VantaDB>}
         */
        static connect_idb(path) {
            const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_connect_idb(ptr0, len0);
            return ret;
        }
        /**
         * Open VantaDB with OPFS-based persistent storage in the browser.
         * @param {string} path
         * @returns {Promise<VantaDB>}
         */
        static connect_persistent(path) {
            const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_connect_persistent(ptr0, len0);
            return ret;
        }
        /**
         * Delete a single record by namespace and key. Returns whether a record was deleted.
         * @param {string} namespace
         * @param {string} key
         * @returns {boolean}
         */
        delete(namespace, key) {
            const ptr0 = passStringToWasm0(namespace, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_delete(this.__wbg_ptr, ptr0, len0, ptr1, len1);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return ret[0] !== 0;
        }
        /**
         * Delete persisted state from IndexedDB.
         * @returns {Promise<void>}
         */
        delete_idb() {
            const ret = wasm.vantadb_delete_idb(this.__wbg_ptr);
            return ret;
        }
        /**
         * Delete a graph node by ID with an associated reason string.
         * @param {bigint} id
         * @param {string} reason
         */
        delete_node(id, reason) {
            const ptr0 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_delete_node(this.__wbg_ptr, id, ptr0, len0);
            if (ret[1]) {
                throw takeFromExternrefTable0(ret[0]);
            }
        }
        /**
         * Run a search with explanation metadata for debugging scoring.
         * @param {any} request
         * @returns {any}
         */
        explain_memory_search(request) {
            const ret = wasm.vantadb_explain_memory_search(this.__wbg_ptr, request);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Export all records across all namespaces to the given path.
         * @param {string} path
         * @returns {any}
         */
        export_all(path) {
            const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_export_all(this.__wbg_ptr, ptr0, len0);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Export all records in a namespace to a JSON file at the given path.
         * @param {string} path
         * @param {string} namespace
         * @returns {any}
         */
        export_namespace(path, namespace) {
            const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(namespace, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_export_namespace(this.__wbg_ptr, ptr0, len0, ptr1, len1);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Flush all pending writes to disk.
         */
        flush() {
            const ret = wasm.vantadb_flush(this.__wbg_ptr);
            if (ret[1]) {
                throw takeFromExternrefTable0(ret[0]);
            }
        }
        /**
         * Generate a text snippet with optional highlighting for a given query.
         * @param {string} payload
         * @param {string} text_query
         * @param {boolean} with_highlighting
         * @returns {string | undefined}
         */
        generate_snippet(payload, text_query, with_highlighting) {
            const ptr0 = passStringToWasm0(payload, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(text_query, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_generate_snippet(this.__wbg_ptr, ptr0, len0, ptr1, len1, with_highlighting);
            let v3;
            if (ret[0] !== 0) {
                v3 = getStringFromWasm0(ret[0], ret[1]).slice();
                wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
            }
            return v3;
        }
        /**
         * Retrieve a single record by namespace and key.
         * @param {string} namespace
         * @param {string} key
         * @returns {any}
         */
        get(namespace, key) {
            const ptr0 = passStringToWasm0(namespace, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_get(this.__wbg_ptr, ptr0, len0, ptr1, len1);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Retrieve a graph node by its numeric ID.
         * @param {bigint} id
         * @returns {any}
         */
        get_node(id) {
            const ret = wasm.vantadb_get_node(this.__wbg_ptr, id);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Perform a breadth-first traversal from the given root node IDs.
         * @param {BigUint64Array} roots
         * @param {number} max_depth
         * @param {string} direction
         * @returns {any}
         */
        graph_bfs(roots, max_depth, direction) {
            const ptr0 = passArray64ToWasm0(roots, wasm.__wbindgen_malloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(direction, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_graph_bfs(this.__wbg_ptr, ptr0, len0, max_depth, ptr1, len1);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Perform a depth-first traversal from the given root node IDs.
         * @param {BigUint64Array} roots
         * @param {number} max_depth
         * @param {string} direction
         * @returns {any}
         */
        graph_dfs(roots, max_depth, direction) {
            const ptr0 = passArray64ToWasm0(roots, wasm.__wbindgen_malloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(direction, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_graph_dfs(this.__wbg_ptr, ptr0, len0, max_depth, ptr1, len1);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Return whether the subgraph reachable from the given roots forms a DAG.
         * @param {BigUint64Array} roots
         * @returns {boolean}
         */
        graph_is_dag(roots) {
            const ptr0 = passArray64ToWasm0(roots, wasm.__wbindgen_malloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_graph_is_dag(this.__wbg_ptr, ptr0, len0);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return ret[0] !== 0;
        }
        /**
         * Compute a topological sort order starting from the given root node IDs.
         * @param {BigUint64Array} roots
         * @returns {any}
         */
        graph_topological_sort(roots) {
            const ptr0 = passArray64ToWasm0(roots, wasm.__wbindgen_malloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_graph_topological_sort(this.__wbg_ptr, ptr0, len0);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Import records from a JSON file at the given path.
         * @param {string} path
         * @returns {any}
         */
        import_file(path) {
            const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_import_file(this.__wbg_ptr, ptr0, len0);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Import records from a JS array of memory record objects.
         * @param {any} records
         * @returns {any}
         */
        import_records(records) {
            const ret = wasm.vantadb_import_records(this.__wbg_ptr, records);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Insert a graph node with optional content, vector, and fields.
         * @param {bigint} id
         * @param {string | null | undefined} content
         * @param {Float32Array | null | undefined} vector
         * @param {any} fields
         */
        insert_node(id, content, vector, fields) {
            var ptr0 = isLikeNone(content) ? 0 : passStringToWasm0(content, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len0 = WASM_VECTOR_LEN;
            var ptr1 = isLikeNone(vector) ? 0 : passArrayF32ToWasm0(vector, wasm.__wbindgen_malloc);
            var len1 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_insert_node(this.__wbg_ptr, id, ptr0, len0, ptr1, len1, fields);
            if (ret[1]) {
                throw takeFromExternrefTable0(ret[0]);
            }
        }
        /**
         * List records in a namespace with optional filters, limit, and cursor pagination.
         * @param {string} namespace
         * @param {any} options
         * @returns {any}
         */
        list(namespace, options) {
            const ptr0 = passStringToWasm0(namespace, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_list(this.__wbg_ptr, ptr0, len0, options);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Return all namespaces as a JS array of strings.
         * @returns {any}
         */
        list_namespaces() {
            const ret = wasm.vantadb_list_namespaces(this.__wbg_ptr);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Restore all records from OPFS storage into memory.
         * @returns {Promise<void>}
         */
        load() {
            const ret = wasm.vantadb_load(this.__wbg_ptr);
            return ret;
        }
        /**
         * Restore all records from IndexedDB storage into memory.
         * @returns {Promise<void>}
         */
        load_idb() {
            const ret = wasm.vantadb_load_idb(this.__wbg_ptr);
            return ret;
        }
        /**
         * Create a new VantaDB instance from an optional WASM config object.
         * @param {any | null} [config_val]
         */
        constructor(config_val) {
            const ret = wasm.vantadb_new(isLikeNone(config_val) ? 0 : addToExternrefTable0(config_val));
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            this.__wbg_ptr = ret[0];
            VantaDBFinalization.register(this, this.__wbg_ptr, this);
            return this;
        }
        /**
         * Open VantaDB at the given storage path.
         * @param {string} path
         * @returns {VantaDB}
         */
        static open(path) {
            const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_open(ptr0, len0);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return VantaDB.__wrap(ret[0]);
        }
        /**
         * Return operational metrics as a JS object with stringified large numbers.
         * @returns {any}
         */
        operational_metrics() {
            const ret = wasm.vantadb_operational_metrics(this.__wbg_ptr);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Purge all expired records and return the number removed.
         * @returns {bigint}
         */
        purge_expired() {
            const ret = wasm.vantadb_purge_expired(this.__wbg_ptr);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return BigInt.asUintN(64, ret[0]);
        }
        /**
         * Insert or update a single memory record from a JS object.
         * @param {any} input
         * @returns {any}
         */
        put(input) {
            const ret = wasm.vantadb_put(this.__wbg_ptr, input);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Insert or update multiple memory records from a JS array.
         * @param {any} inputs
         * @returns {any}
         */
        put_batch(inputs) {
            const ret = wasm.vantadb_put_batch(this.__wbg_ptr, inputs);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Execute a raw DSL query string and return the result.
         * @param {string} query
         * @returns {any}
         */
        query(query) {
            const ptr0 = passStringToWasm0(query, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_query(this.__wbg_ptr, ptr0, len0);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Rebuild the HNSW index and return a rebuild report.
         * @returns {any}
         */
        rebuild_index() {
            const ret = wasm.vantadb_rebuild_index(this.__wbg_ptr);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Paginated HNSW rebuild from text records.
         *
         * Iterates through memory records in batches (max 1000) using the
         * cursor-based list() API to prevent OOM on large namespaces.
         * @param {string} namespace
         * @param {number | null} [page_size]
         * @returns {any}
         */
        reindex_hnsw_from_text(namespace, page_size) {
            const ptr0 = passStringToWasm0(namespace, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_reindex_hnsw_from_text(this.__wbg_ptr, ptr0, len0, isLikeNone(page_size) ? Number.MAX_SAFE_INTEGER : (page_size) >>> 0);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Repair the text index and return a repair report.
         * @returns {any}
         */
        repair_text_index() {
            const ret = wasm.vantadb_repair_text_index(this.__wbg_ptr);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Persist all in-memory records to OPFS storage.
         * @returns {Promise<void>}
         */
        save() {
            const ret = wasm.vantadb_save(this.__wbg_ptr);
            return ret;
        }
        /**
         * Persist all in-memory records to IndexedDB storage.
         * @returns {Promise<void>}
         */
        save_idb() {
            const ret = wasm.vantadb_save_idb(this.__wbg_ptr);
            return ret;
        }
        /**
         * Search memory records by vector similarity with optional filters and text query.
         * @param {any} request
         * @returns {any}
         */
        search(request) {
            const ret = wasm.vantadb_search(this.__wbg_ptr, request);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
        /**
         * Search nodes by raw vector without namespace scoping.
         * @param {Float32Array} vector
         * @param {number} top_k
         * @returns {any}
         */
        search_vector(vector, top_k) {
            const ptr0 = passArrayF32ToWasm0(vector, wasm.__wbindgen_malloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.vantadb_search_vector(this.__wbg_ptr, ptr0, len0, top_k);
            if (ret[2]) {
                throw takeFromExternrefTable0(ret[1]);
            }
            return takeFromExternrefTable0(ret[0]);
        }
    }
    if (Symbol.dispose) VantaDB.prototype[Symbol.dispose] = VantaDB.prototype.free;
    exports.VantaDB = VantaDB;
    function __wbg_get_imports() {
        const import0 = {
            __proto__: null,
            __wbg_Error_92b29b0548f8b746: function(arg0, arg1) {
                const ret = Error(getStringFromWasm0(arg0, arg1));
                return ret;
            },
            __wbg_Number_9a4e0ecb0fa16705: function(arg0) {
                const ret = Number(arg0);
                return ret;
            },
            __wbg_String_8564e559799eccda: function(arg0, arg1) {
                const ret = String(arg1);
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg___wbindgen_bigint_get_as_i64_d968e41184ae354f: function(arg0, arg1) {
                const v = arg1;
                const ret = typeof(v) === 'bigint' ? v : undefined;
                getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
            },
            __wbg___wbindgen_boolean_get_fa956cfa2d1bd751: function(arg0) {
                const v = arg0;
                const ret = typeof(v) === 'boolean' ? v : undefined;
                return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
            },
            __wbg___wbindgen_debug_string_c25d447a39f5578f: function(arg0, arg1) {
                const ret = debugString(arg1);
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg___wbindgen_in_aca499c5de7ff5e5: function(arg0, arg1) {
                const ret = arg0 in arg1;
                return ret;
            },
            __wbg___wbindgen_is_bigint_2f76dc55065b4273: function(arg0) {
                const ret = typeof(arg0) === 'bigint';
                return ret;
            },
            __wbg___wbindgen_is_function_1ff95bcc5517c252: function(arg0) {
                const ret = typeof(arg0) === 'function';
                return ret;
            },
            __wbg___wbindgen_is_null_ea9085d691f535d3: function(arg0) {
                const ret = arg0 === null;
                return ret;
            },
            __wbg___wbindgen_is_object_a27215656b807791: function(arg0) {
                const val = arg0;
                const ret = typeof(val) === 'object' && val !== null;
                return ret;
            },
            __wbg___wbindgen_is_string_ea5e6cc2e4141dfe: function(arg0) {
                const ret = typeof(arg0) === 'string';
                return ret;
            },
            __wbg___wbindgen_is_undefined_c05833b95a3cf397: function(arg0) {
                const ret = arg0 === undefined;
                return ret;
            },
            __wbg___wbindgen_jsval_eq_e659fcf7b0e32763: function(arg0, arg1) {
                const ret = arg0 === arg1;
                return ret;
            },
            __wbg___wbindgen_jsval_loose_eq_db4c3b15f63fc170: function(arg0, arg1) {
                const ret = arg0 == arg1;
                return ret;
            },
            __wbg___wbindgen_number_get_394265ed1e1b84ee: function(arg0, arg1) {
                const obj = arg1;
                const ret = typeof(obj) === 'number' ? obj : undefined;
                getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
            },
            __wbg___wbindgen_string_get_b0ca35b86a603356: function(arg0, arg1) {
                const obj = arg1;
                const ret = typeof(obj) === 'string' ? obj : undefined;
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg___wbindgen_throw_344f42d3211c4765: function(arg0, arg1) {
                throw new Error(getStringFromWasm0(arg0, arg1));
            },
            __wbg__wbg_cb_unref_fffb441def202758: function(arg0) {
                arg0._wbg_cb_unref();
            },
            __wbg_apply_3ac86a26fdb56c05: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = arg0.apply(arg1, arg2);
                return ret;
            }, arguments); },
            __wbg_buffer_54b87055582c8a81: function(arg0) {
                const ret = arg0.buffer;
                return ret;
            },
            __wbg_call_8a2dd23819f8a60a: function() { return handleError(function (arg0, arg1) {
                const ret = arg0.call(arg1);
                return ret;
            }, arguments); },
            __wbg_call_a6e5c5dce5018821: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = arg0.call(arg1, arg2);
                return ret;
            }, arguments); },
            __wbg_done_89b2b13e91a60321: function(arg0) {
                const ret = arg0.done;
                return ret;
            },
            __wbg_entries_015dc610cd81ede0: function(arg0) {
                const ret = Object.entries(arg0);
                return ret;
            },
            __wbg_error_a6fa202b58aa1cd3: function(arg0, arg1) {
                let deferred0_0;
                let deferred0_1;
                try {
                    deferred0_0 = arg0;
                    deferred0_1 = arg1;
                    console.error(getStringFromWasm0(arg0, arg1));
                } finally {
                    wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
                }
            },
            __wbg_getRandomValues_3f44b700395062e5: function() { return handleError(function (arg0, arg1) {
                globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
            }, arguments); },
            __wbg_get_507a50627bffa49b: function(arg0, arg1) {
                const ret = arg0[arg1 >>> 0];
                return ret;
            },
            __wbg_get_78f252d074a84d0b: function() { return handleError(function (arg0, arg1) {
                const ret = Reflect.get(arg0, arg1);
                return ret;
            }, arguments); },
            __wbg_get_c7eb1f358a7654df: function() { return handleError(function (arg0, arg1) {
                const ret = Reflect.get(arg0, arg1);
                return ret;
            }, arguments); },
            __wbg_get_unchecked_6e0ad6d2a41b06f6: function(arg0, arg1) {
                const ret = arg0[arg1 >>> 0];
                return ret;
            },
            __wbg_get_with_ref_key_6412cf3094599694: function(arg0, arg1) {
                const ret = arg0[arg1];
                return ret;
            },
            __wbg_instanceof_ArrayBuffer_4480b9e0068a8adb: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof ArrayBuffer;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Map_e5b5e3db98422fcc: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof Map;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Promise_4cb210c0b8f8c959: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof Promise;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Uint8Array_309b927aaf7a3fc7: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof Uint8Array;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_isArray_0677c962b281d01a: function(arg0) {
                const ret = Array.isArray(arg0);
                return ret;
            },
            __wbg_isSafeInteger_04f36e4056f1b851: function(arg0) {
                const ret = Number.isSafeInteger(arg0);
                return ret;
            },
            __wbg_iterator_6f722e4a93058b71: function() {
                const ret = Symbol.iterator;
                return ret;
            },
            __wbg_length_1f0964f4a5e2c6d8: function(arg0) {
                const ret = arg0.length;
                return ret;
            },
            __wbg_length_370319915dc99107: function(arg0) {
                const ret = arg0.length;
                return ret;
            },
            __wbg_log_0c201ade58bb55e1: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
                let deferred0_0;
                let deferred0_1;
                try {
                    deferred0_0 = arg0;
                    deferred0_1 = arg1;
                    console.log(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3), getStringFromWasm0(arg4, arg5), getStringFromWasm0(arg6, arg7));
                } finally {
                    wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
                }
            },
            __wbg_log_ce2c4456b290c5e7: function(arg0, arg1) {
                let deferred0_0;
                let deferred0_1;
                try {
                    deferred0_0 = arg0;
                    deferred0_1 = arg1;
                    console.log(getStringFromWasm0(arg0, arg1));
                } finally {
                    wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
                }
            },
            __wbg_mark_b4d943f3bc2d2404: function(arg0, arg1) {
                performance.mark(getStringFromWasm0(arg0, arg1));
            },
            __wbg_measure_84362959e621a2c1: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                let deferred0_0;
                let deferred0_1;
                let deferred1_0;
                let deferred1_1;
                try {
                    deferred0_0 = arg0;
                    deferred0_1 = arg1;
                    deferred1_0 = arg2;
                    deferred1_1 = arg3;
                    performance.measure(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
                } finally {
                    wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
                    wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
                }
            }, arguments); },
            __wbg_new_227d7c05414eb861: function() {
                const ret = new Error();
                return ret;
            },
            __wbg_new_32b398fb48b6d94a: function() {
                const ret = new Array();
                return ret;
            },
            __wbg_new_7796ffc7ed656783: function() {
                const ret = new Map();
                return ret;
            },
            __wbg_new_b667d279fd5aa943: function(arg0, arg1) {
                const ret = new Error(getStringFromWasm0(arg0, arg1));
                return ret;
            },
            __wbg_new_cd45aabdf6073e84: function(arg0) {
                const ret = new Uint8Array(arg0);
                return ret;
            },
            __wbg_new_da52cf8fe3429cb2: function() {
                const ret = new Object();
                return ret;
            },
            __wbg_new_typed_1824d93f294193e5: function(arg0, arg1) {
                try {
                    var state0 = {a: arg0, b: arg1};
                    var cb0 = (arg0, arg1) => {
                        const a = state0.a;
                        state0.a = 0;
                        try {
                            return wasm_bindgen__convert__closures_____invoke__h0bf0d27f3df94d79(a, state0.b, arg0, arg1);
                        } finally {
                            state0.a = a;
                        }
                    };
                    const ret = new Promise(cb0);
                    return ret;
                } finally {
                    state0.a = 0;
                }
            },
            __wbg_new_with_length_e6785c33c8e4cce8: function(arg0) {
                const ret = new Uint8Array(arg0 >>> 0);
                return ret;
            },
            __wbg_next_6dbf2c0ac8cde20f: function(arg0) {
                const ret = arg0.next;
                return ret;
            },
            __wbg_next_71f2aa1cb3d1e37e: function() { return handleError(function (arg0) {
                const ret = arg0.next();
                return ret;
            }, arguments); },
            __wbg_now_86c0d4ba3fa605b8: function() {
                const ret = Date.now();
                return ret;
            },
            __wbg_now_e7c6795a7f81e10f: function(arg0) {
                const ret = arg0.now();
                return ret;
            },
            __wbg_performance_3fcf6e32a7e1ed0a: function(arg0) {
                const ret = arg0.performance;
                return ret;
            },
            __wbg_prototypesetcall_4770620bbe4688a0: function(arg0, arg1, arg2) {
                Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
            },
            __wbg_push_d2ae3af0c1217ae6: function(arg0, arg1) {
                const ret = arg0.push(arg1);
                return ret;
            },
            __wbg_queueMicrotask_0ab5b2d2393e99b9: function(arg0) {
                const ret = arg0.queueMicrotask;
                return ret;
            },
            __wbg_queueMicrotask_6a09b7bc46549209: function(arg0) {
                queueMicrotask(arg0);
            },
            __wbg_resolve_2191a4dfe481c25b: function(arg0) {
                const ret = Promise.resolve(arg0);
                return ret;
            },
            __wbg_set_4d7dd76f3dae2926: function(arg0, arg1, arg2) {
                arg0.set(getArrayU8FromWasm0(arg1, arg2));
            },
            __wbg_set_575dd786d51585f8: function(arg0, arg1, arg2) {
                const ret = arg0.set(arg1, arg2);
                return ret;
            },
            __wbg_set_6be42768c690e380: function(arg0, arg1, arg2) {
                arg0[arg1] = arg2;
            },
            __wbg_set_8535240470bf2500: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = Reflect.set(arg0, arg1, arg2);
                return ret;
            }, arguments); },
            __wbg_set_8a16b38e4805b298: function(arg0, arg1, arg2) {
                arg0[arg1 >>> 0] = arg2;
            },
            __wbg_stack_3b0d974bbf31e44f: function(arg0, arg1) {
                const ret = arg1.stack;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
                getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
            },
            __wbg_static_accessor_GLOBAL_4ef717fb391d88b7: function() {
                const ret = typeof global === 'undefined' ? null : global;
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_static_accessor_GLOBAL_THIS_8d1badc68b5a74f4: function() {
                const ret = typeof globalThis === 'undefined' ? null : globalThis;
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_static_accessor_SELF_146583524fe1469b: function() {
                const ret = typeof self === 'undefined' ? null : self;
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_static_accessor_WINDOW_f2829a2234d7819e: function() {
                const ret = typeof window === 'undefined' ? null : window;
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_then_16d107c451e9905d: function(arg0, arg1, arg2) {
                const ret = arg0.then(arg1, arg2);
                return ret;
            },
            __wbg_then_6ec10ae38b3e92f7: function(arg0, arg1) {
                const ret = arg0.then(arg1);
                return ret;
            },
            __wbg_value_a5d5488a9589444a: function(arg0) {
                const ret = arg0.value;
                return ret;
            },
            __wbg_vantadb_new: function(arg0) {
                const ret = VantaDB.__wrap(arg0);
                return ret;
            },
            __wbindgen_cast_0000000000000001: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 578, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__he5211670e4cc5c11);
                return ret;
            },
            __wbindgen_cast_0000000000000002: function(arg0) {
                // Cast intrinsic for `F64 -> Externref`.
                const ret = arg0;
                return ret;
            },
            __wbindgen_cast_0000000000000003: function(arg0) {
                // Cast intrinsic for `I64 -> Externref`.
                const ret = arg0;
                return ret;
            },
            __wbindgen_cast_0000000000000004: function(arg0, arg1) {
                // Cast intrinsic for `Ref(String) -> Externref`.
                const ret = getStringFromWasm0(arg0, arg1);
                return ret;
            },
            __wbindgen_cast_0000000000000005: function(arg0, arg1) {
                // Cast intrinsic for `U128 -> Externref`.
                const ret = (BigInt.asUintN(64, arg0) | (BigInt.asUintN(64, arg1) << BigInt(64)));
                return ret;
            },
            __wbindgen_cast_0000000000000006: function(arg0) {
                // Cast intrinsic for `U64 -> Externref`.
                const ret = BigInt.asUintN(64, arg0);
                return ret;
            },
            __wbindgen_init_externref_table: function() {
                const table = wasm.__wbindgen_externrefs;
                const offset = table.grow(4);
                table.set(0, undefined);
                table.set(offset + 0, undefined);
                table.set(offset + 1, null);
                table.set(offset + 2, true);
                table.set(offset + 3, false);
            },
        };
        return {
            __proto__: null,
            "./vantadb_wasm_bg.js": import0,
        };
    }

    function wasm_bindgen__convert__closures_____invoke__he5211670e4cc5c11(arg0, arg1, arg2) {
        const ret = wasm.wasm_bindgen__convert__closures_____invoke__he5211670e4cc5c11(arg0, arg1, arg2);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }

    function wasm_bindgen__convert__closures_____invoke__h0bf0d27f3df94d79(arg0, arg1, arg2, arg3) {
        wasm.wasm_bindgen__convert__closures_____invoke__h0bf0d27f3df94d79(arg0, arg1, arg2, arg3);
    }

    const VantaDBFinalization = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(ptr => wasm.__wbg_vantadb_free(ptr, 1));

    function addToExternrefTable0(obj) {
        const idx = wasm.__externref_table_alloc();
        wasm.__wbindgen_externrefs.set(idx, obj);
        return idx;
    }

    const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(state => wasm.__wbindgen_destroy_closure(state.a, state.b));

    function debugString(val) {
        // primitive types
        const type = typeof val;
        if (type == 'number' || type == 'boolean' || val == null) {
            return  `${val}`;
        }
        if (type == 'string') {
            return `"${val}"`;
        }
        if (type == 'symbol') {
            const description = val.description;
            if (description == null) {
                return 'Symbol';
            } else {
                return `Symbol(${description})`;
            }
        }
        if (type == 'function') {
            const name = val.name;
            if (typeof name == 'string' && name.length > 0) {
                return `Function(${name})`;
            } else {
                return 'Function';
            }
        }
        // objects
        if (Array.isArray(val)) {
            const length = val.length;
            let debug = '[';
            if (length > 0) {
                debug += debugString(val[0]);
            }
            for(let i = 1; i < length; i++) {
                debug += ', ' + debugString(val[i]);
            }
            debug += ']';
            return debug;
        }
        // Test for built-in
        const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
        let className;
        if (builtInMatches && builtInMatches.length > 1) {
            className = builtInMatches[1];
        } else {
            // Failed to match the standard '[object ClassName]'
            return toString.call(val);
        }
        if (className == 'Object') {
            // we're a user defined class or Object
            // JSON.stringify avoids problems with cycles, and is generally much
            // easier than looping through ownProperties of `val`.
            try {
                return 'Object(' + JSON.stringify(val) + ')';
            } catch (_) {
                return 'Object';
            }
        }
        // errors
        if (val instanceof Error) {
            return `${val.name}: ${val.message}\n${val.stack}`;
        }
        // TODO we could test for more things here, like `Set`s and `Map`s.
        return className;
    }

    function getArrayU8FromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
    }

    let cachedBigUint64ArrayMemory0 = null;
    function getBigUint64ArrayMemory0() {
        if (cachedBigUint64ArrayMemory0 === null || cachedBigUint64ArrayMemory0.byteLength === 0) {
            cachedBigUint64ArrayMemory0 = new BigUint64Array(wasm.memory.buffer);
        }
        return cachedBigUint64ArrayMemory0;
    }

    let cachedDataViewMemory0 = null;
    function getDataViewMemory0() {
        if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
            cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
        }
        return cachedDataViewMemory0;
    }

    let cachedFloat32ArrayMemory0 = null;
    function getFloat32ArrayMemory0() {
        if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
            cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
        }
        return cachedFloat32ArrayMemory0;
    }

    function getStringFromWasm0(ptr, len) {
        return decodeText(ptr >>> 0, len);
    }

    let cachedUint8ArrayMemory0 = null;
    function getUint8ArrayMemory0() {
        if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
            cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
        }
        return cachedUint8ArrayMemory0;
    }

    function handleError(f, args) {
        try {
            return f.apply(this, args);
        } catch (e) {
            const idx = addToExternrefTable0(e);
            wasm.__wbindgen_exn_store(idx);
        }
    }

    function isLikeNone(x) {
        return x === undefined || x === null;
    }

    function makeMutClosure(arg0, arg1, f) {
        const state = { a: arg0, b: arg1, cnt: 1 };
        const real = (...args) => {

            // First up with a closure we increment the internal reference
            // count. This ensures that the Rust closure environment won't
            // be deallocated while we're invoking it.
            state.cnt++;
            const a = state.a;
            state.a = 0;
            try {
                return f(a, state.b, ...args);
            } finally {
                state.a = a;
                real._wbg_cb_unref();
            }
        };
        real._wbg_cb_unref = () => {
            if (--state.cnt === 0) {
                wasm.__wbindgen_destroy_closure(state.a, state.b);
                state.a = 0;
                CLOSURE_DTORS.unregister(state);
            }
        };
        CLOSURE_DTORS.register(real, state, state);
        return real;
    }

    function passArray64ToWasm0(arg, malloc) {
        const ptr = malloc(arg.length * 8, 8) >>> 0;
        getBigUint64ArrayMemory0().set(arg, ptr / 8);
        WASM_VECTOR_LEN = arg.length;
        return ptr;
    }

    function passArray8ToWasm0(arg, malloc) {
        const ptr = malloc(arg.length * 1, 1) >>> 0;
        getUint8ArrayMemory0().set(arg, ptr / 1);
        WASM_VECTOR_LEN = arg.length;
        return ptr;
    }

    function passArrayF32ToWasm0(arg, malloc) {
        const ptr = malloc(arg.length * 4, 4) >>> 0;
        getFloat32ArrayMemory0().set(arg, ptr / 4);
        WASM_VECTOR_LEN = arg.length;
        return ptr;
    }

    function passStringToWasm0(arg, malloc, realloc) {
        if (realloc === undefined) {
            const buf = cachedTextEncoder.encode(arg);
            const ptr = malloc(buf.length, 1) >>> 0;
            getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
            WASM_VECTOR_LEN = buf.length;
            return ptr;
        }

        let len = arg.length;
        let ptr = malloc(len, 1) >>> 0;

        const mem = getUint8ArrayMemory0();

        let offset = 0;

        for (; offset < len; offset++) {
            const code = arg.charCodeAt(offset);
            if (code > 0x7F) break;
            mem[ptr + offset] = code;
        }
        if (offset !== len) {
            if (offset !== 0) {
                arg = arg.slice(offset);
            }
            ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
            const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
            const ret = cachedTextEncoder.encodeInto(arg, view);

            offset += ret.written;
            ptr = realloc(ptr, len, offset, 1) >>> 0;
        }

        WASM_VECTOR_LEN = offset;
        return ptr;
    }

    function takeFromExternrefTable0(idx) {
        const value = wasm.__wbindgen_externrefs.get(idx);
        wasm.__externref_table_dealloc(idx);
        return value;
    }

    let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
    cachedTextDecoder.decode();
    function decodeText(ptr, len) {
        return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
    }

    const cachedTextEncoder = new TextEncoder();

    if (!('encodeInto' in cachedTextEncoder)) {
        cachedTextEncoder.encodeInto = function (arg, view) {
            const buf = cachedTextEncoder.encode(arg);
            view.set(buf);
            return {
                read: arg.length,
                written: buf.length
            };
        };
    }

    let WASM_VECTOR_LEN = 0;

    let wasmModule, wasmInstance, wasm;
    function __wbg_finalize_init(instance, module) {
        wasmInstance = instance;
        wasm = instance.exports;
        wasmModule = module;
        cachedBigUint64ArrayMemory0 = null;
        cachedDataViewMemory0 = null;
        cachedFloat32ArrayMemory0 = null;
        cachedUint8ArrayMemory0 = null;
        wasm.__wbindgen_start();
        return wasm;
    }

    async function __wbg_load(module, imports) {
        if (typeof Response === 'function' && module instanceof Response) {
            if (typeof WebAssembly.instantiateStreaming === 'function') {
                try {
                    return await WebAssembly.instantiateStreaming(module, imports);
                } catch (e) {
                    const validResponse = module.ok && expectedResponseType(module.type);

                    if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                        console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                    } else { throw e; }
                }
            }

            const bytes = await module.arrayBuffer();
            return await WebAssembly.instantiate(bytes, imports);
        } else {
            const instance = await WebAssembly.instantiate(module, imports);

            if (instance instanceof WebAssembly.Instance) {
                return { instance, module };
            } else {
                return instance;
            }
        }

        function expectedResponseType(type) {
            switch (type) {
                case 'basic': case 'cors': case 'default': return true;
            }
            return false;
        }
    }

    function initSync(module) {
        if (wasm !== undefined) return wasm;


        if (module !== undefined) {
            if (Object.getPrototypeOf(module) === Object.prototype) {
                ({module} = module)
            } else {
                console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
            }
        }

        const imports = __wbg_get_imports();
        if (!(module instanceof WebAssembly.Module)) {
            module = new WebAssembly.Module(module);
        }
        const instance = new WebAssembly.Instance(module, imports);
        return __wbg_finalize_init(instance, module);
    }

    async function __wbg_init(module_or_path) {
        if (wasm !== undefined) return wasm;


        if (module_or_path !== undefined) {
            if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
                ({module_or_path} = module_or_path)
            } else {
                console.warn('using deprecated parameters for the initialization function; pass a single object instead')
            }
        }

        if (module_or_path === undefined && script_src !== undefined) {
            module_or_path = script_src.replace(/\.js$/, "_bg.wasm");
        }
        const imports = __wbg_get_imports();

        if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
            module_or_path = fetch(module_or_path);
        }

        const { instance, module } = await __wbg_load(await module_or_path, imports);

        return __wbg_finalize_init(instance, module);
    }

    return Object.assign(__wbg_init, { initSync }, exports);
})({ __proto__: null });
