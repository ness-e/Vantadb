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
 * Canonical cross-binding error codes (ERR-TS-01, FFI-02). The VALUES are the
 * `VANTADB_*` strings emitted by `Error::code()` in the Rust core
 * (`docs/api/ERROR_HANDLING.md` §1.1).
 */
export const ERROR_CODES = {
  CLOSED: "VANTADB_CLOSED",
  WASM_ERROR: "VANTADB_WASM_ERROR",
  VALIDATION_ERROR: "VANTADB_VALIDATION_ERROR",
  NOT_FOUND: "VANTADB_NOT_FOUND",
  INVALID_ARGUMENT: "VANTADB_INVALID_ARGUMENT",
  CORRUPT: "VANTADB_CORRUPT",
  SERIALIZATION: "VANTADB_SERIALIZATION",
  RESOURCE_LIMIT: "VANTADB_RESOURCE_LIMIT",
  QUOTA_EXCEEDED: "VANTADB_QUOTA_EXCEEDED",
  TIMEOUT: "VANTADB_TIMEOUT",
  BUSY: "VANTADB_BUSY",
  IO_ERROR: "VANTADB_IO_ERROR",
} as const;

/** Union of the canonical `VANTADB_*` code strings in {@link ERROR_CODES}. */
export type ErrorCode = (typeof ERROR_CODES)[keyof typeof ERROR_CODES];

/**
 * Canonical FFI error code constants (numerical and string variants).
 */
export const VantaErrorCode = {
  Io: 1,
  Corrupt: 2,
  Serialization: 3,
  DimensionMismatch: 4,
  NotFound: 5,
  InvalidInput: 6,
  TransactionConflict: 7,
  Closed: 8,
  Encryption: 9,
  QuotaExceeded: 10,
  Internal: 99,
  // String representations
  ERR_IO: "ERR_IO",
  ERR_CORRUPT: "ERR_CORRUPT",
  ERR_SERIALIZATION: "ERR_SERIALIZATION",
  ERR_DIMENSION_MISMATCH: "ERR_DIMENSION_MISMATCH",
  ERR_NOT_FOUND: "ERR_NOT_FOUND",
  ERR_INVALID_INPUT: "ERR_INVALID_INPUT",
  ERR_TXN_CONFLICT: "ERR_TXN_CONFLICT",
  ERR_CLOSED: "ERR_CLOSED",
  ERR_ENCRYPTION: "ERR_ENCRYPTION",
  ERR_QUOTA_EXCEEDED: "ERR_QUOTA_EXCEEDED",
  ERR_INTERNAL: "ERR_INTERNAL",
} as const;

export type VantaErrorCode =
  | "CLOSED"
  | "WASM_ERROR"
  | "VALIDATION_ERROR"
  | "NOT_FOUND"
  | "INVALID_ARGUMENT"
  | "CORRUPT"
  | "SERIALIZATION"
  | "RESOURCE_LIMIT"
  | "QUOTA_EXCEEDED"
  | "TIMEOUT"
  | "BUSY"
  | "IO_ERROR"
  | ErrorCode;

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
export const KNOWN_CODES: ReadonlySet<string> = new Set([
  ...Object.values(ERROR_CODES),
  "ERR_IO",
  "ERR_CORRUPT",
  "ERR_SERIALIZATION",
  "ERR_DIMENSION_MISMATCH",
  "ERR_NOT_FOUND",
  "ERR_INVALID_INPUT",
  "ERR_TXN_CONFLICT",
  "ERR_CLOSED",
  "ERR_ENCRYPTION",
  "ERR_QUOTA_EXCEEDED",
  "ERR_INTERNAL",
]);

interface WasmErrorLike extends Error {
  /** Structured code attached by `vantadb-wasm`'s `to_js_err` (FIND-10). */
  code?: unknown;
}

/** Map structured or raw code into canonical ERROR_CODES string. */
export function mapCodeToCanonical(code: string): ErrorCode {
  switch (code) {
    case "ERR_IO":
    case "VANTADB_IO_ERROR":
      return ERROR_CODES.IO_ERROR;
    case "ERR_CORRUPT":
    case "VANTADB_CORRUPT":
      return ERROR_CODES.CORRUPT;
    case "ERR_SERIALIZATION":
    case "VANTADB_SERIALIZATION":
    case "SERIALIZATION":
      return ERROR_CODES.SERIALIZATION;
    case "ERR_DIMENSION_MISMATCH":
    case "VANTADB_DIMENSION_MISMATCH":
      return ERROR_CODES.VALIDATION_ERROR;
    case "ERR_NOT_FOUND":
    case "VANTADB_NOT_FOUND":
    case "NOT_FOUND":
      return ERROR_CODES.NOT_FOUND;
    case "ERR_INVALID_INPUT":
    case "VANTADB_INVALID_INPUT":
    case "VANTADB_VALIDATION_ERROR":
    case "VANTADB_INVALID_ARGUMENT":
    case "INVALID_ARGUMENT":
      return ERROR_CODES.INVALID_ARGUMENT;
    case "ERR_TXN_CONFLICT":
    case "VANTADB_TXN_CONFLICT":
    case "VANTADB_BUSY":
    case "BUSY":
      return ERROR_CODES.BUSY;
    case "ERR_CLOSED":
    case "VANTADB_CLOSED":
    case "CLOSED":
      return ERROR_CODES.CLOSED;
    case "ERR_ENCRYPTION":
    case "VANTADB_ENCRYPTION":
      return ERROR_CODES.VALIDATION_ERROR;
    case "ERR_QUOTA_EXCEEDED":
    case "VANTADB_QUOTA_EXCEEDED":
    case "QUOTA_EXCEEDED":
    case "VANTADB_RESOURCE_LIMIT":
    case "RESOURCE_LIMIT":
      return ERROR_CODES.QUOTA_EXCEEDED;
    case "ERR_INTERNAL":
    case "VANTADB_WASM_ERROR":
      return ERROR_CODES.WASM_ERROR;
    default:
      if ((Object.values(ERROR_CODES) as readonly string[]).includes(code)) {
        return code as ErrorCode;
      }
      return ERROR_CODES.WASM_ERROR;
  }
}

/**
 * Classify a binding error message into a `DbError` code.
 *
 * Ensures serialization errors map to SERIALIZATION (never CORRUPT).
 */
export function classifyWasmError(message: string): ErrorCode {
  if (/quota exceeded|quota_exceeded/i.test(message)) return ERROR_CODES.QUOTA_EXCEEDED;
  if (/serialization error|failed to serialize/i.test(message)) return ERROR_CODES.SERIALIZATION;
  if (/node not found|not found:/i.test(message)) return ERROR_CODES.NOT_FOUND;
  if (
    /validation error|invalid input|vector dimension mismatch|duplicate node|node id collision|no vector stored|iql parse error/i.test(
      message,
    )
  ) {
    return ERROR_CODES.VALIDATION_ERROR;
  }
  if (/incompatible binary format|wal version mismatch|schema error/i.test(message)) {
    return ERROR_CODES.CORRUPT;
  }
  if (/resource limit/i.test(message)) return ERROR_CODES.RESOURCE_LIMIT;
  if (/timed out after|timeout/i.test(message)) return ERROR_CODES.TIMEOUT;
  if (/database busy/i.test(message)) return ERROR_CODES.BUSY;
  if (/wal error|io error|backend error/i.test(message)) return ERROR_CODES.IO_ERROR;
  if (/database is closing|closed/i.test(message)) return ERROR_CODES.CLOSED;
  return ERROR_CODES.WASM_ERROR;
}

export const NATIVE_CODE_PREFIX =
  /^(?:\[ERR(?:_CODE)?:([A-Z0-9_]+)\]\s*)?(?:(VANTADB_[A-Z0-9_]+):\s*)?([\s\S]*)$/;

/**
 * Wrap any native/WASM binding error into a uniform {@link DbError}.
 *
 * Checks structured `[ERR_CODE:...]` or `.code` property first, avoiding
 * fragile regex message heuristics. Never classifies serialization as corrupt.
 */
export function fromNativeError(e: unknown, context: string = "native"): DbError {
  if (e instanceof DbError) return e;
  const rawMessage = e instanceof Error ? e.message : String(e);
  const details = e instanceof Error ? { name: e.name, stack: e.stack } : { original: e };

  let code: string | undefined;

  // 1. Structured .code property
  if (e instanceof Error && typeof (e as WasmErrorLike).code === "string") {
    const rawCode = (e as WasmErrorLike).code as string;
    code = mapCodeToCanonical(rawCode);
  }

  // 2. Structured prefix in message: [ERR_CODE:<code>] or VANTADB_<code>:
  let cleanMessage = rawMessage;
  const match = NATIVE_CODE_PREFIX.exec(rawMessage);
  if (match) {
    if (!code && match[1]) {
      code = mapCodeToCanonical(match[1]);
    }
    if (!code && match[2] && (Object.values(ERROR_CODES) as readonly string[]).includes(match[2])) {
      code = match[2];
    }
    if (match[1] || match[2]) {
      cleanMessage = match[3];
    }
  }

  // 3. Fallback message classification
  if (!code || code === ERROR_CODES.WASM_ERROR) {
    const fallbackCode = classifyWasmError(cleanMessage);
    if (fallbackCode !== ERROR_CODES.WASM_ERROR || !code) {
      code = fallbackCode;
    }
  }

  return new DbError(code, `${context}: ${cleanMessage}`, details, { cause: e });
}

/**
 * Wrap any value thrown by the WASM binding into a {@link DbError}.
 */
export function wrapWasmError(e: unknown, context: string): DbError {
  return fromNativeError(e, context);
}
