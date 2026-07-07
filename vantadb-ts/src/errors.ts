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

export function wrapWasmError(e: unknown, context: string): VantaError {
  if (e instanceof VantaError) return e;
  const message = e instanceof Error ? e.message : String(e);
  const details = e instanceof Error
    ? { name: e.name, stack: e.stack }
    : { original: e };
  return new VantaError(
    "WASM_ERROR",
    `${context}: ${message}`,
    details,
  );
}
