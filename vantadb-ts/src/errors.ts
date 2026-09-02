export interface VantaErrorJSON {
  name: string;
  code: string;
  message: string;
  details?: unknown;
  timestamp: string;
}

/**
 * Canonical cross-binding error codes (ERR-TS-01). The VALUES are the ten
 * `VANTADB_*` strings emitted by `VantaError::code()` in the Rust core
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

export type ErrorCode = (typeof ERROR_CODES)[keyof typeof ERROR_CODES];

export class VantaError extends Error {
  readonly code: string;
  readonly details?: unknown;
  readonly timestamp: Date;

  constructor(code: string, message: string, details?: unknown, options?: ErrorOptions) {
    // `options.cause` preserves the original error chain (ERR-TS-01, §4.3).
    super(message, options);
    this.name = "VantaError";
    this.code = code;
    this.details = details;
    this.timestamp = new Date();
  }

  toJSON(): VantaErrorJSON {
    const json: VantaErrorJSON = {
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
 * Classify a WASM-binding error message into a `VantaError` code.
 *
 * Fallback used when the thrown error carries no structured `code` property
 * (e.g. a `vantadb-wasm` pkg build predating FIND-10). The core flattens
 * `VantaError` to its Display string at the wasm boundary, so the prefixes
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

export function wrapWasmError(e: unknown, context: string): VantaError {
  if (e instanceof VantaError) return e;
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
  return new VantaError(
    code,
    `${context}: ${message}`,
    details,
    { cause: e },
  );
}
