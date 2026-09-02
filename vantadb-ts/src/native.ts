import { VantaError, ERROR_CODES, classifyWasmError } from "./errors.js";
import { isMemoryRecord } from "./guards.js";

import type {
  Capabilities,
  ListOptions,
  MemoryInput,
  MemoryListPage,
  MemoryRecord,
  SearchHit,
  SearchRequest,
  VantaMetadataInput,
} from "./types.js";

import type { SearchRequest as NativeSearchRequest } from "vantadb-node";

/**
 * Options accepted by `NativeVantaDB.connect()`.
 *
 * Mirrors the subset of `VantaConfig` that matters for the native backend:
 * `read_only` opens the database without write access, `memory_limit` caps the
 * engine memory budget (bytes).
 */
export interface NativeConnectOptions {
  read_only?: boolean;
  memory_limit?: number;
}

/**
 * Wrap an error thrown by the native binding in a uniform `VantaError`, the
 * same way `wrapWasmError` does for the WASM backend. Errors that already are
 * `VantaError` pass through untouched.
 *
 * `vantadb-node` prefixes engine errors with the canonical stable code
 * (`"VANTADB_NOT_FOUND: Node not found: 7"`, ERR-TS-01); when the prefix is
 * present and belongs to the 10-code contract it becomes `err.code` and is
 * stripped from the human message. Otherwise we fall back to the same
 * message-prefix classifier the WASM path uses — there is no separate
 * `NATIVE_ERROR` code.
 */
const NATIVE_CODE_PREFIX = /^(VANTADB_[A-Z0-9_]+): ([\s\S]*)$/;

export function wrapNativeError(e: unknown, context: string): VantaError {
  if (e instanceof VantaError) return e;
  const message = e instanceof Error ? e.message : String(e);
  const details = e instanceof Error
    ? { name: e.name, stack: e.stack }
    : { original: e };
  const match = NATIVE_CODE_PREFIX.exec(message);
  if (match && (Object.values(ERROR_CODES) as readonly string[]).includes(match[1])) {
    return new VantaError(match[1], `${context}: ${match[2]}`, details, { cause: e });
  }
  return new VantaError(classifyWasmError(message), `${context}: ${message}`, details, { cause: e });
}

/**
 * Convert caller-provided metadata (plain values or tagged) to the strict
 * tagged form expected by the native binding (`VantaMetadata`).
 * Accepts `VantaMetadataInput` where values can be plain JS primitives
 * (string, number, boolean, null) or already-tagged `VantaValue`.
 * The native binding uses `'Null'` string literal for null values.
 */
function normalizeMetadataForNative(
  input: VantaMetadataInput | undefined,
): import("vantadb-node").VantaMetadata | undefined {
  if (input === undefined) return undefined;
  const out: import("vantadb-node").VantaMetadata = {};
  for (const [k, v] of Object.entries(input)) {
    if (v === null) {
      out[k] = 'Null';
    } else if (typeof v === "string") {
      out[k] = { String: v };
    } else if (typeof v === "number") {
      // Detect integer vs float
      if (Number.isInteger(v)) out[k] = { Int: v };
      else out[k] = { Float: v };
    } else if (typeof v === "boolean") {
      out[k] = { Bool: v };
    } else if (typeof v === "object" && v !== null) {
      // Assume already a tagged VantaValue (e.g., { String: "..." })
      // Check if it matches native VantaValue variants
      const vv = v as Record<string, unknown>;
      if ('String' in vv && typeof vv.String === 'string') out[k] = { String: vv.String };
      else if ('Int' in vv && typeof vv.Int === 'number') out[k] = { Int: vv.Int };
      else if ('Float' in vv && typeof vv.Float === 'number') out[k] = { Float: vv.Float };
      else if ('Bool' in vv && typeof vv.Bool === 'boolean') out[k] = { Bool: vv.Bool };
      else if ('DateTime' in vv && typeof vv.DateTime === 'string') out[k] = { DateTime: vv.DateTime };
      else if ('ListString' in vv && Array.isArray(vv.ListString)) out[k] = { ListString: vv.ListString };
      else if ('ListInt' in vv && Array.isArray(vv.ListInt)) out[k] = { ListInt: vv.ListInt };
      else if ('ListFloat' in vv && Array.isArray(vv.ListFloat)) out[k] = { ListFloat: vv.ListFloat };
      else if ('ListBool' in vv && Array.isArray(vv.ListBool)) out[k] = { ListBool: vv.ListBool };
      else if ('ListDateTime' in vv && Array.isArray(vv.ListDateTime)) out[k] = { ListDateTime: vv.ListDateTime };
      else if ('Null' in vv) out[k] = 'Null';
      else {
        throw new VantaError(
          ERROR_CODES.VALIDATION_ERROR,
          `normalizeMetadataForNative: unrecognized tagged value for key "${k}"`,
        );
      }
    } else {
      throw new VantaError(
        ERROR_CODES.VALIDATION_ERROR,
        `normalizeMetadataForNative: unsupported value type for key "${k}": ${typeof v}`,
      );
    }
  }
  return out;
}

function _mapRecord(r: unknown): MemoryRecord {
  if (!r || typeof r !== "object") {
    throw new VantaError(
      ERROR_CODES.VALIDATION_ERROR,
      "_mapRecord: expected an object, got " + typeof r,
    );
  }
  if (!isMemoryRecord(r)) {
    throw new VantaError(
      ERROR_CODES.VALIDATION_ERROR,
      "_mapRecord: invalid MemoryRecord structure or missing required fields",
    );
  }
  return r;
}

/**
 * Node.js native backend for VantaDB, powered by the `vantadb-node` napi-rs
 * binding. This is the **additional** backend to the browser WASM wrapper
 * (`vantadb.ts`): it provides real filesystem persistence (fjall/WAL/fsync)
 * which WASM cannot.
 *
 * API notes (isomorphic with `VantaDB` in `vantadb.ts`):
 * - Same method names, input shapes and output shapes for the exposed subset
 *   (connect/close/flush/capabilities/put/putBatch/get/delete/list/
 *   listNamespaces/search).
 * - Methods are **async**: the native engine runs on background threads
 *   (`spawn_blocking`), so the JS thread is never blocked.
 * - `connect` is an async factory; the WASM wrapper's sync `connect` has no
 *   native equivalent.
 *
 * Platform fallback: the native `.node` binary is platform-specific. If it is
 * not present for the current platform (e.g. a browser bundle), calling any
 * method throws a `VantaError` with a canonical `VANTADB_*` code and a clear
 * message — browsers should use the WASM wrapper (`vantadb.ts`) instead.
 */
export class NativeVantaDB {
  private inner: import("vantadb-node").VantaDb;
  private _closed: boolean = false;

  private constructor(inner: import("vantadb-node").VantaDb) {
    this.inner = inner;
  }

  // async + await so that ASYNC rejections of the inner promise are caught
  // and wrapped too — a bare try/catch around a returned promise only sees
  // synchronous throws (TS-02).
  private async _native<T>(method: string, fn: () => Promise<T> | T): Promise<T> {
    try {
      return await fn();
    } catch (e) {
      throw wrapNativeError(e, method);
    }
  }

  /**
   * Connect to a VantaDB database.
   *
   * Unlike the WASM wrapper, `path` defaults to `":memory:"` (in-memory engine)
   * and any real filesystem path opens a **persistent** database (fjall backend
   * with WAL/fsync) — the key differential vs WASM.
   *
   * @param path - Filesystem path for persistent storage, or `":memory:"` for
   *   in-memory. Defaults to `":memory:"`.
   * @param options - Optional `{ read_only?, memory_limit? }`.
   * @returns A new NativeVantaDB instance.
   * @throws {VantaError} If the native binding is unavailable for this platform
   *   or the engine fails to open.
   */
  static async connect(path: string = ":memory:", options?: NativeConnectOptions): Promise<NativeVantaDB> {
    try {
      const { VantaDb } = await import("vantadb-node");
      const inner = await VantaDb.connect(path, options ?? undefined);
      return new NativeVantaDB(inner);
    } catch (e) {
      throw wrapNativeError(
        e,
        "connect (is the native binding built? run `npm run build` in vantadb-node)",
      );
    }
  }

  private _assertOpen(): void {
    if (this._closed) {
      throw new VantaError(ERROR_CODES.CLOSED, "NativeVantaDB instance is closed");
    }
  }

  /**
   * Close the database. Pending writes are flushed first. Awaits the native
   * `close()`, which sets the durability barrier: new operations are rejected
   * and every in-flight operation (including fire-and-forget `put`s whose
   * background task had not yet run) is drained before the engine is flushed.
   * Safe to call multiple times (no-op on subsequent calls).
   */
  async close(): Promise<void> {
    if (this._closed) return;
    try {
      await this.inner.close();
    } catch (e) {
      throw wrapNativeError(e, "close");
    } finally {
      this._closed = true;
    }
  }

  /** Flush the WAL and memory-mapped files to disk. */
  async flush(): Promise<void> {
    this._assertOpen();
    await this._native("flush", () => this.inner.flush());
  }

  /** Get the capabilities of the native engine. */
  async capabilities(): Promise<Capabilities> {
    this._assertOpen();
    return this._native("capabilities", () => {
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
   */
  async put(input: MemoryInput): Promise<MemoryRecord> {
    this._assertOpen();
    return this._native("put", async () => {
      const wire = {
        namespace: input.namespace,
        key: input.key,
        payload: input.payload,
        vector: input.vector,
        ttl_ms: input.ttl_ms,
        metadata: input.metadata !== undefined ? normalizeMetadataForNative(input.metadata) : undefined,
      };
      return _mapRecord(await this.inner.put(wire));
    });
  }

  /**
   * Store multiple memory records in a single batch operation.
   *
   * @param inputs - Array of memory records to store.
   * @returns Array of stored records in the same order as the input.
   */
  async putBatch(inputs: MemoryInput[]): Promise<MemoryRecord[]> {
    this._assertOpen();
    return this._native("putBatch", async () => {
      const normalized = inputs.map((i) => ({
        namespace: i.namespace,
        key: i.key,
        payload: i.payload,
        vector: i.vector,
        ttl_ms: i.ttl_ms,
        metadata: i.metadata !== undefined ? normalizeMetadataForNative(i.metadata) : undefined,
      }));
      const records: unknown[] = await this.inner.putBatch(normalized);
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
   */
  async get(namespace: string, key: string): Promise<MemoryRecord | null> {
    this._assertOpen();
    return this._native("get", async () => {
      const raw = await this.inner.get(namespace, key);
      return raw != null ? _mapRecord(raw) : null;
    });
  }

  /**
   * Delete a memory record by namespace and key.
   *
   * @returns true if the record was deleted, false if it did not exist.
   */
  async delete(namespace: string, key: string): Promise<boolean> {
    this._assertOpen();
    return this._native("delete", () => this.inner.delete(namespace, key));
  }

  /** List all namespaces that contain at least one memory record. */
  async listNamespaces(): Promise<string[]> {
    this._assertOpen();
    return this._native("listNamespaces", () => this.inner.listNamespaces());
  }

  /**
   * List memory records in a namespace with optional filters and cursor
   * pagination.
   *
   * @param namespace - The namespace to list.
   * @param options - Pagination options (limit, cursor, filters).
   * @returns A page of records with an optional cursor for continuation.
   */
  async list(namespace: string, options: ListOptions = {}): Promise<MemoryListPage> {
    this._assertOpen();
    return this._native("list", async () => {
      const wire = {
        filters: options.filters !== undefined ? normalizeMetadataForNative(options.filters) : undefined,
        limit: options.limit,
        cursor: options.cursor,
      };
      const raw = await this.inner.list(namespace, wire);
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

  private _buildSearchRequest(request: SearchRequest, explain?: boolean): NativeSearchRequest {
    // Pass the request through untouched. A zero-norm cosine query is rejected
    // by the core with ERR-028 (src/sdk/search/mod.rs) and surfaces as
    // VantaError — this layer is glue, not a place for search decisions
    // (api-contract.md R-8). vantadb.ts (WASM) does the same, so both backends
    // are aligned.
    return {
      namespace: request.namespace,
      query_vector: request.query_vector,
      filters: request.filters !== undefined ? normalizeMetadataForNative(request.filters) : undefined,
      text_query: request.text_query ?? undefined,
      top_k: request.top_k ?? 10,
      distance_metric: request.distance_metric ?? "Cosine",
      explain: explain ?? (request.explain ?? false),
    };
  }

  /**
   * Search for memory records by vector similarity, with optional text +
   * hybrid search.
   *
   * @param request - The search request parameters.
   * @returns Array of search hits ordered by relevance (closest first).
   */
  async search(request: SearchRequest): Promise<SearchHit[]> {
    this._assertOpen();
    return this._native("search", async () => {
      const raw: unknown[] = await this.inner.search(this._buildSearchRequest(request));
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
}
