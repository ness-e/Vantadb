import { DbError, ERROR_CODES } from "./errors.js";

import type {
  FilterInput,
  FilterItem,
  FilterSpec,
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

/** A filter item after normalization: `value` is always the tagged `Value` wire form. */
export interface NormalizedFilterItem {
  field: string;
  op: FilterItem["op"];
  value: Value;
}

/** Same normalization for AND-combined filter items (returns wire-ready values). */
export function normalizeFilterItems(
  items: FilterItem[],
): NormalizedFilterItem[] {
  return items.map((item) => ({
    field: item.field,
    op: item.op,
    value: normalizeValue(item.value),
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
    case "DateTime":
      if (typeof payload !== "string") {
        throw new DbError(
          ERROR_CODES.VALIDATION_ERROR,
          "normalizeValue: { DateTime } payload must be an RFC 3339 string",
        );
      }
      return { DateTime: payload };
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
    case "ListDateTime":
      if (
        !Array.isArray(payload) ||
        !payload.every((e: unknown) => typeof e === "string")
      ) {
        throw new DbError(
          ERROR_CODES.VALIDATION_ERROR,
          "normalizeValue: { ListDateTime } payload must be an RFC 3339 string[]",
        );
      }
      return { ListDateTime: payload as string[] };
    default:
      throw new DbError(
        ERROR_CODES.VALIDATION_ERROR,
        `normalizeValue: unrecognized tagged value ${JSON.stringify(tag)}`,
      );
  }
}

export function normalizeValue(v: unknown): Value {
  if (v === null) return { Null: null };
  if (v instanceof Date) {
    // py↔js parity: Python accepts `datetime` objects; JS accepts `Date`
    // instances (wire form is the RFC 3339 tagged `DateTime`).
    if (Number.isNaN(v.getTime())) {
      throw new DbError(
        ERROR_CODES.VALIDATION_ERROR,
        "normalizeValue: Date must be a valid date",
      );
    }
    return { DateTime: v.toISOString() };
  }
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

/** Canonical `$op` → wire `FilterOp` map (same set as Python/MCP/CLI). */
const FILTER_OPS: Record<string, FilterItem["op"]> = {
  $eq: "Eq",
  $neq: "Neq",
  $gt: "Gt",
  $gte: "Gte",
  $lt: "Lt",
  $lte: "Lte",
};

/**
 * Normalize the canonical cross-SDK `$op` filter DSL (interchangeable with
 * Python `py_dict_to_filter_ops`, MCP `parse_filter_ops` and the CLI) into the
 * native `FilterItem[]` wire:
 *
 * - `{field: value}` → implicit `$eq`;
 * - `{field: {$gte: v, $lt: v2}}` → one item per `$op` key (AND-combined).
 *
 * An unknown `$op` throws `VALIDATION_ERROR` (never ignored silently — same
 * error contract as the other transports). Values go through
 * {@link normalizeValue}; tagged values (`{String: "x"}`) are treated as
 * equality values, not operator objects.
 */
export function normalizeFilterInput(input: FilterInput): NormalizedFilterItem[] {
  if (input === null || typeof input !== "object" || Array.isArray(input)) {
    throw new DbError(
      ERROR_CODES.VALIDATION_ERROR,
      "normalizeFilterInput: expected a filter object (field → value / {$op: value})",
    );
  }
  const out: NormalizedFilterItem[] = [];
  for (const [field, spec] of Object.entries(input)) {
    if (
      spec !== null &&
      typeof spec === "object" &&
      !Array.isArray(spec) &&
      !(spec instanceof Date)
    ) {
      const obj = spec as Record<string, unknown>;
      if (Object.keys(obj).some((k) => k.startsWith("$"))) {
        for (const [opKey, val] of Object.entries(obj)) {
          const op = FILTER_OPS[opKey];
          if (op === undefined) {
            throw new DbError(
              ERROR_CODES.VALIDATION_ERROR,
              `normalizeFilterInput: unknown filter operator '${opKey}' for field '${field}'. Supported: $eq, $neq, $gt, $gte, $lt, $lte`,
            );
          }
          out.push({ field, op, value: normalizeValue(val) });
        }
        continue;
      }
    }
    out.push({ field, op: "Eq", value: normalizeValue(spec) });
  }
  return out;
}

/**
 * Accept either filter form at the public boundary: the native
 * `FilterItem[]` wire or the canonical `$op` DSL. `undefined`/`null` (and an
 * empty filter) yields `[]` — callers keep their existing empty-filter
 * semantics (e.g. `count` counts all; `deleteByFilter` rejects in core).
 */
export function toFilterItems(spec: FilterSpec | undefined | null): NormalizedFilterItem[] {
  if (spec === undefined || spec === null) return [];
  return Array.isArray(spec) ? normalizeFilterItems(spec) : normalizeFilterInput(spec);
}
