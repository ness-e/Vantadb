export interface VantaErrorJSON {
  name: string;
  code: string;
  message: string;
  details?: unknown;
  timestamp: string;
}

const ERROR_CODES = {
  CLOSED: "CLOSED",
  WASM_ERROR: "WASM_ERROR",
  VALIDATION_ERROR: "VALIDATION_ERROR",
  NOT_FOUND: "NOT_FOUND",
  INVALID_ARGUMENT: "INVALID_ARGUMENT",
  CORRUPT: "CORRUPT",
  RESOURCE_LIMIT: "RESOURCE_LIMIT",
  TIMEOUT: "TIMEOUT",
  BUSY: "BUSY",
  IO_ERROR: "IO_ERROR",
} as const;

export type ErrorCode = (typeof ERROR_CODES)[keyof typeof ERROR_CODES];

export class VantaError extends Error {
  readonly code: string;
  readonly details?: unknown;
  readonly timestamp: Date;

  constructor(code: string, message: string, details?: unknown) {
    super(message);
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
  if (/node not found|not found:/i.test(message)) return "NOT_FOUND";
  if (
    /validation error|invalid input|vector dimension mismatch|duplicate node|node id collision|no vector stored|iql parse error/i.test(
      message,
    )
  ) {
    return "VALIDATION_ERROR";
  }
  if (/incompatible binary format|wal version mismatch|serialization error|schema error/i.test(message)) {
    return "CORRUPT";
  }
  if (/resource limit/i.test(message)) return "RESOURCE_LIMIT";
  if (/timed out after|timeout/i.test(message)) return "TIMEOUT";
  if (/database busy/i.test(message)) return "BUSY";
  if (/wal error|io error|backend error/i.test(message)) return "IO_ERROR";
  return "WASM_ERROR";
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
  );
}
