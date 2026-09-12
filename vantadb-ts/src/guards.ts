import { DbError, ERROR_CODES } from "./errors.js";

import type {
  MemoryRecord,
  NodeRecord,
  SearchHit,
  SearchRequest,
  Value,
  Metadata,
} from "./types.js";

export function isMemoryRecord(r: unknown): r is MemoryRecord {
  if (r === null || typeof r !== "object") return false;
  const obj = r as Record<string, unknown>;
  return (
    typeof obj.namespace === "string" &&
    typeof obj.key === "string" &&
    typeof obj.payload === "string" &&
    (typeof obj.version === "string" || typeof obj.version === "number") &&
    (typeof obj.node_id === "string" || typeof obj.node_id === "number") &&
    (typeof obj.created_at_ms === "string" ||
      typeof obj.created_at_ms === "number") &&
    (typeof obj.updated_at_ms === "string" ||
      typeof obj.updated_at_ms === "number")
  );
}

export function isSearchHit(h: unknown): h is SearchHit {
  if (h === null || typeof h !== "object") return false;
  const obj = h as Record<string, unknown>;
  return isMemoryRecord(obj.record) && typeof obj.distance === "number";
}

export function isNodeRecord(n: unknown): n is NodeRecord {
  if (n === null || typeof n !== "object") return false;
  const obj = n as Record<string, unknown>;
  return (
    typeof obj.id === "string" &&
    typeof obj.vector_dimensions === "number" &&
    Array.isArray(obj.edges) &&
    typeof obj.confidence_score === "number" &&
    typeof obj.importance === "number" &&
    typeof obj.hits === "number" &&
    typeof obj.last_accessed === "string" &&
    typeof obj.epoch === "number" &&
    (obj.tier === "Hot" || obj.tier === "Cold") &&
    typeof obj.is_alive === "boolean"
  );
}

const VALID_VALUE_TYPES = [
  "String",
  "Int",
  "Float",
  "Bool",
  "Null",
  "ListString",
  "ListInt",
  "ListFloat",
  "ListBool",
] as const;

export function isValidValue(v: unknown): v is Value {
  if (v === null || typeof v !== "object") return false;
  const obj = v as Record<string, unknown>;
  const keys = Object.keys(obj);
  if (keys.length !== 1) return false;
  const type = keys[0];
  if (!(VALID_VALUE_TYPES as readonly string[]).includes(type)) return false;
  if (type === "Null") return obj[type] === null || obj[type] === undefined;
  return true;
}

export function isMetadata(m: unknown): m is Metadata {
  if (m === null || typeof m !== "object") return false;
  return Object.values(m).every(isValidValue);
}

export function isValidVector(v: unknown): v is number[] {
  if (!Array.isArray(v)) return false;
  if (v.length === 0) return false;
  return v.every((n) => typeof n === "number" && isFinite(n));
}

/**
 * Validate a vector input. Accepts a plain `number[]` (copied downstream
 * into a `Float32Array`) or a `Float32Array` (passed through zero-copy —
 * prefer it on hot paths). Throws `DbError` with the canonical
 * `VANTADB_VALIDATION_ERROR` code (ERR-TS-01 — previously raw
 * `TypeError`/`RangeError`, which bypassed the uniform error contract).
 * BREAKING for callers catching `TypeError` by name: catch `DbError` and
 * check `.code` instead.
 */
export function validateVector(v: unknown): asserts v is number[] | Float32Array {
  if (!Array.isArray(v) && !(v instanceof Float32Array)) {
    throw new DbError(
      ERROR_CODES.VALIDATION_ERROR,
      "validateVector: expected an array, got " + typeof v,
    );
  }
  if (v.length === 0) {
    throw new DbError(
      ERROR_CODES.VALIDATION_ERROR,
      "validateVector: vector cannot be empty",
    );
  }
  for (let i = 0; i < v.length; i++) {
    if (typeof v[i] !== "number" || !isFinite(v[i])) {
      throw new DbError(
        ERROR_CODES.VALIDATION_ERROR,
        `validateVector: invalid or non-finite element at index ${i}`,
      );
    }
  }
}

/**
 * Map a raw engine record to `MemoryRecord`, validating its shape.
 * Single shared definition used by both the WASM (`vantadb.ts`) and native
 * (`native.ts`) backends. The error strings are snapshot-covered — do not
 * reword them.
 */
export function _mapRecord(r: unknown): MemoryRecord {
  if (!r || typeof r !== "object") {
    throw new DbError(
      ERROR_CODES.VALIDATION_ERROR,
      "_mapRecord: expected an object, got " + typeof r,
    );
  }
  if (!isMemoryRecord(r)) {
    throw new DbError(
      ERROR_CODES.VALIDATION_ERROR,
      "_mapRecord: invalid MemoryRecord structure or missing required fields",
    );
  }
  return r;
}

/**
 * Backend-neutral core of a search request, shared by the WASM (`vantadb.ts`)
 * and native (`native.ts`) builders. Each backend spreads this base and adds
 * its own wire encoding for `filters`/`text_query` (null-vs-undefined and
 * tagged-metadata shapes differ per binding) — so the base stays wire-neutral
 * and must NOT gain backend-specific fields.
 */
export interface SearchRequestBase {
  namespace: string;
  query_vector: number[];
  top_k: number;
  distance_metric: "Cosine" | "Euclidean";
  explain: boolean;
}

/**
 * Validate the backend-neutral core of a search request (D5a).
 * Previously every field flowed through with `??` defaults and no checks,
 * so `""`, `[]`, `NaN` or unknown metric strings reached the engine and
 * failed there with worse errors (or silently misbehaved). Now malformed
 * input throws `DbError` with `VALIDATION_ERROR` at the TS frontier —
 * the same contract `validateVector` and `native.ts` enforce.
 */
export function buildSearchRequestBase(
  request: SearchRequest,
  explain?: boolean,
): SearchRequestBase {
  if (request === null || typeof request !== "object") {
    throw new DbError(
      ERROR_CODES.VALIDATION_ERROR,
      `buildSearchRequestBase: expected a SearchRequest object, got ${request === null ? "null" : typeof request}`,
    );
  }
  if (typeof request.namespace !== "string" || request.namespace.length === 0) {
    throw new DbError(
      ERROR_CODES.VALIDATION_ERROR,
      "buildSearchRequestBase: namespace must be a non-empty string",
    );
  }
  if (
    !Array.isArray(request.query_vector) ||
    request.query_vector.length === 0
  ) {
    throw new DbError(
      ERROR_CODES.VALIDATION_ERROR,
      "buildSearchRequestBase: query_vector must be a non-empty array",
    );
  }
  for (let i = 0; i < request.query_vector.length; i++) {
    const n: unknown = request.query_vector[i];
    if (typeof n !== "number" || !Number.isFinite(n)) {
      throw new DbError(
        ERROR_CODES.VALIDATION_ERROR,
        `buildSearchRequestBase: invalid or non-finite query_vector element at index ${i}`,
      );
    }
  }
  const top_k = request.top_k ?? 10;
  if (
    typeof top_k !== "number" ||
    !Number.isFinite(top_k) ||
    !Number.isInteger(top_k) ||
    top_k < 0
  ) {
    throw new DbError(
      ERROR_CODES.VALIDATION_ERROR,
      "buildSearchRequestBase: top_k must be an integer >= 0",
    );
  }
  const distance_metric = request.distance_metric ?? "Cosine";
  if (distance_metric !== "Cosine" && distance_metric !== "Euclidean") {
    throw new DbError(
      ERROR_CODES.VALIDATION_ERROR,
      "buildSearchRequestBase: distance_metric must be Cosine or Euclidean",
    );
  }
  const effExplain = explain ?? (request.explain ?? false);
  if (typeof effExplain !== "boolean") {
    throw new DbError(
      ERROR_CODES.VALIDATION_ERROR,
      "buildSearchRequestBase: explain must be a boolean",
    );
  }
  return {
    namespace: request.namespace,
    query_vector: request.query_vector,
    top_k,
    distance_metric,
    explain: effExplain,
  };
}
