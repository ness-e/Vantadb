import { DbError, ERROR_CODES } from "./errors.js";

import type {
  FlatValue,
  FilterItem,
  Metadata,
  MetadataInput,
  Value,
} from "./types.js";

/**
 * Normalize caller-provided metadata to the tagged wire form the Rust engine
 * expects (externally-tagged serde enum: `{"String": "x"}`, `{"Int": 1}`, ...).
 *
 * Plain JS values map by type: string → String, boolean → Bool, integer → Int,
 * other number → Float, null → Null. Already-tagged values pass through
 * untouched (backward compat).
 */
export function normalizeMetadata(
  m?: MetadataInput | null,
): Metadata | undefined {
  if (m === undefined || m === null) return undefined;
  const out: Record<string, Value> = {};
  for (const [k, v] of Object.entries(m)) {
    out[k] = normalizeValue(v);
  }
  return out;
}

/** Same normalization for AND-combined filter items. */
export function normalizeFilterItems(
  items: FilterItem[],
): FilterItem[] {
  return items.map((item) => ({
    ...item,
    value: normalizeValue(item.value) as FlatValue | Value,
  }));
}

/**
 * Validate an already-tagged wire value and return its canonical copy.
 * Mirrors the strictness of `normalizeMetadataForNative` (`native.ts`):
 * anything that is not exactly one known tag with a correctly-typed,
 * finite payload throws `DbError` with `VALIDATION_ERROR` instead of
 * flowing unchecked into the engine.
 */
function assertTaggedValue(v: Record<string, unknown>): Value {
  const keys = Object.keys(v);
  if (keys.length !== 1) {
    throw new DbError(
      ERROR_CODES.VALIDATION_ERROR,
      `normalizeValue: tagged value must have exactly one key, got ${keys.length}`,
    );
  }
  const tag = keys[0];
  const payload: unknown = v[tag];
  switch (tag) {
    case "String":
      if (typeof payload !== "string") {
        throw new DbError(
          ERROR_CODES.VALIDATION_ERROR,
          "normalizeValue: { String } payload must be a string",
        );
      }
      return { String: payload };
    case "Int":
      if (
        typeof payload !== "number" ||
        !Number.isFinite(payload) ||
        !Number.isInteger(payload)
      ) {
        throw new DbError(
          ERROR_CODES.VALIDATION_ERROR,
          "normalizeValue: { Int } payload must be a finite integer",
        );
      }
      return { Int: payload };
    case "Float":
      if (typeof payload !== "number" || !Number.isFinite(payload)) {
        throw new DbError(
          ERROR_CODES.VALIDATION_ERROR,
          "normalizeValue: { Float } payload must be a finite number",
        );
      }
      return { Float: payload };
    case "Bool":
      if (typeof payload !== "boolean") {
        throw new DbError(
          ERROR_CODES.VALIDATION_ERROR,
          "normalizeValue: { Bool } payload must be a boolean",
        );
      }
      return { Bool: payload };
    case "Null":
      if (payload !== null && payload !== undefined) {
        throw new DbError(
          ERROR_CODES.VALIDATION_ERROR,
          "normalizeValue: { Null } payload must be null",
        );
      }
      return { Null: null };
    case "ListString":
      if (
        !Array.isArray(payload) ||
        !payload.every((e: unknown) => typeof e === "string")
      ) {
        throw new DbError(
          ERROR_CODES.VALIDATION_ERROR,
          "normalizeValue: { ListString } payload must be a string[]",
        );
      }
      return { ListString: payload as string[] };
    case "ListInt":
      if (
        !Array.isArray(payload) ||
        !payload.every(
          (e: unknown) =>
            typeof e === "number" &&
            Number.isFinite(e) &&
            Number.isInteger(e),
        )
      ) {
        throw new DbError(
          ERROR_CODES.VALIDATION_ERROR,
          "normalizeValue: { ListInt } payload must be an integer[]",
        );
      }
      return { ListInt: payload as number[] };
    case "ListFloat":
      if (
        !Array.isArray(payload) ||
        !payload.every(
          (e: unknown) => typeof e === "number" && Number.isFinite(e),
        )
      ) {
        throw new DbError(
          ERROR_CODES.VALIDATION_ERROR,
          "normalizeValue: { ListFloat } payload must be a number[]",
        );
      }
      return { ListFloat: payload as number[] };
    case "ListBool":
      if (
        !Array.isArray(payload) ||
        !payload.every((e: unknown) => typeof e === "boolean")
      ) {
        throw new DbError(
          ERROR_CODES.VALIDATION_ERROR,
          "normalizeValue: { ListBool } payload must be a boolean[]",
        );
      }
      return { ListBool: payload as boolean[] };
    default:
      throw new DbError(
        ERROR_CODES.VALIDATION_ERROR,
        `normalizeValue: unrecognized tagged value ${JSON.stringify(tag)}`,
      );
  }
}

export function normalizeValue(v: unknown): Value {
  if (v === null) return { Null: null };
  switch (typeof v) {
    case "string":
      return { String: v };
    case "boolean":
      return { Bool: v };
    case "number":
      if (!Number.isFinite(v)) {
        throw new DbError(
          ERROR_CODES.VALIDATION_ERROR,
          "normalizeValue: number must be finite",
        );
      }
      return Number.isInteger(v) ? { Int: v } : { Float: v };
    default: {
      // Already in tagged wire form ({ String, Int, Float, ... }) — validate
      // strictly instead of casting blindly (D5a).
      if (typeof v !== "object" || Array.isArray(v)) {
        throw new DbError(
          ERROR_CODES.VALIDATION_ERROR,
          `normalizeValue: unsupported value type: ${Array.isArray(v) ? "array" : typeof v}`,
        );
      }
      return assertTaggedValue(v as Record<string, unknown>);
    }
  }
}
