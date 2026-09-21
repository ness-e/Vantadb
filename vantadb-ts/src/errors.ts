/**
 * Wire shape of a {@link DbError} as produced by {@link DbError.toJSON}.
 *
 * Mirrors the Python `error_to_dict()` dict (`docs/api/ERROR_HANDLING.md` §5.2)
 * for cross-binding log correlation.
 */
export interface ErrorJSON {
  name: string;
  code: string;
  message: string;
  details?: unknown;
  timestamp: string;
}

/**
 * Canonical cross-binding error codes (ERR-TS-01). The VALUES are the ten
 * `VANTADB_*` strings emitted by `Error::code()` in the Rust core
 * (`docs/api/ERROR_HANDLING.md` §1.1) — previously TS/WASM carried the
 * unprefixed forms on the wire, which was a documented drift; the keys keep
 * their readable unprefixed names for TS consumers (`ERROR_CODES.BUSY`).
 * BREAKING (0.x): `err.code` string values now carry the `VANTADB_` prefix.
 */
export const ERROR_CODES = {
  CLOSED: "VANTADB_CLOSED",
  WASM_ERROR: "VANTADB_WASM_ERROR",
  VALIDATION_ERROR: "VANTADB_VALIDATION_ERROR",
  NOT_FOUND: "VANTADB_NOT_FOUND",
  INVALID_ARGUMENT: "VANTADB_INVALID_ARGUMENT",
  CORRUPT: "VANTADB_CORRUPT",
  RESOURCE_LIMIT: "VANTADB_RESOURCE_LIMIT",
  TIMEOUT: "VANTADB_TIMEOUT",
  BUSY: "VANTADB_BUSY",
  IO_ERROR: "VANTADB_IO_ERROR",
} as const;

/** Union of the canonical `VANTADB_*` code strings in {@link ERROR_CODES}. */
export type ErrorCode = (typeof ERROR_CODES)[keyof typeof ERROR_CODES];

/**
 * Canonical VantaDB error (ERR-TS-01).
 *
 * Carries a `VANTADB_*` {@link DbError.code} (see `docs/api/ERROR_HANDLING.md` §1.1),
 * an optional `details` payload, and a creation `timestamp`. Serialized `name`
 * stays `"VantaError"` (AST-004, asserted by tests).
 *
 * @example
 * ```ts
 * throw new DbError(ERROR_CODES.NOT_FOUND, "node not found: 42");
 * ```
 */
export class DbError extends Error {
  readonly code: string;
  readonly details?: unknown;
  readonly timestamp: Date;

  /**
   * Create a `DbError`.
   *
   * @param code - Canonical `VANTADB_*` code (use {@link ERROR_CODES}).
   * @param message - Human-readable message (prefixed with context by {@link wrapWasmError}).
   * @param details - Optional structured payload (preserved verbatim in {@link DbError.toJSON}).
   * @param options - Forwarded to `Error` (`options.cause` preserves the original chain).
   */
  constructor(code: string, message: string, details?: unknown, options?: ErrorOptions) {
    // `options.cause` preserves the original error chain (ERR-TS-01, §4.3).
    super(message, options);
    // Serialized `name` stays "VantaError" (AST-004): it is asserted by tests
    // and mirrored in the toJSON wire shape consumed by Python `error_to_dict`.
    this.name = "VantaError";
    this.code = code;
    this.details = details;
    this.timestamp = new Date();
  }

  /**
   * Serialize this error to its {@link ErrorJSON} wire shape.
   *
   * @returns Plain JSON object (`name`/`code`/`message`/`timestamp`, plus
   * `details` when defined) consumed by Python `error_to_dict`.
   */
  toJSON(): ErrorJSON {
    const json: ErrorJSON = {
      name: this.name,
      code: this.code,
      message: this.message,
      timestamp: this.timestamp.toISOString(),
    };
    if (this.details !== undefined) {
      json.details = this.details;
    }
    return json;
  }
}

/** Codes that may legitimately arrive on an error thrown by the WASM binding. */
const KNOWN_CODES: ReadonlySet<string> = new Set(Object.values(ERROR_CODES));

interface WasmErrorLike extends Error {
  /** Structured code attached by `vantadb-wasm`'s `to_js_err` (FIND-10). */
  code?: unknown;
}

/**
 * Classify a WASM-binding error message into a `DbError` code.
 *
 * Fallback used when the thrown error carries no structured `code` property
 * (e.g. a `vantadb-wasm` pkg build predating FIND-10). The core flattens
 * `Error` to its Display string at the wasm boundary, so the prefixes
 * mirror the variant messages in `src/error.rs` (corrupt / not-found /
 * validation are the contract classes).
 */
export function classifyWasmError(message: string): ErrorCode {
  if (/node not found|not found:/i.test(message)) return ERROR_CODES.NOT_FOUND;
  if (
    /validation error|invalid input|vector dimension mismatch|duplicate node|node id collision|no vector stored|iql parse error/i.test(
      message,
    )
  ) {
    return ERROR_CODES.VALIDATION_ERROR;
  }
  if (/incompatible binary format|wal version mismatch|serialization error|schema error/i.test(message)) {
    return ERROR_CODES.CORRUPT;
  }
  if (/resource limit/i.test(message)) return ERROR_CODES.RESOURCE_LIMIT;
  if (/timed out after|timeout/i.test(message)) return ERROR_CODES.TIMEOUT;
  if (/database busy/i.test(message)) return ERROR_CODES.BUSY;
  if (/wal error|io error|backend error/i.test(message)) return ERROR_CODES.IO_ERROR;
  return ERROR_CODES.WASM_ERROR;
}

/**
 * Wrap any value thrown by the WASM binding into a {@link DbError}.
 *
 * `DbError` inputs pass through untouched. Otherwise the structured `code`
 * attached by `vantadb-wasm`'s `to_js_err` (FIND-10) wins when known;
 * message-prefix classification ({@link classifyWasmError}) covers older
 * pkg builds; `WASM_ERROR` is the final fallback.
 *
 * @param e - Thrown value (Error, WASM error, or anything).
 * @param context - Operation label prefixed to the message (`"<context>: <message>"`).
 * @returns A `DbError` with `cause` set to the original value.
 */
export function wrapWasmError(e: unknown, context: string): DbError {
  if (e instanceof DbError) return e;
  const message = e instanceof Error ? e.message : String(e);
  const details = e instanceof Error
    ? { name: e.name, stack: e.stack }
    : { original: e };
  // Prefer the structured code attached by the wasm binding; fall back to
  // message-prefix classification (older pkg builds) and finally WASM_ERROR.
  const code = e instanceof Error
    && typeof (e as WasmErrorLike).code === "string"
    && KNOWN_CODES.has((e as WasmErrorLike).code as string)
    ? (e as WasmErrorLike).code as string
    : classifyWasmError(message);
  return new DbError(
    code,
    `${context}: ${message}`,
    details,
    { cause: e },
  );
}
