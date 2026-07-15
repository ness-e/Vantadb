import type {
  MemoryRecord,
  NodeRecord,
  SearchHit,
  VantaValue,
  VantaMetadata,
} from "./types.js";

export function isMemoryRecord(r: unknown): r is MemoryRecord {
  if (r === null || typeof r !== "object") return false;
  const obj = r as Record<string, unknown>;
  return (
    typeof obj.namespace === "string" &&
    typeof obj.key === "string" &&
    typeof obj.payload === "string" &&
    typeof obj.version === "string" &&
    typeof obj.node_id === "string" &&
    typeof obj.created_at_ms === "string" &&
    typeof obj.updated_at_ms === "string"
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

const VALID_VANTA_TYPES = [
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

export function isValidVantaValue(v: unknown): v is VantaValue {
  if (v === null || typeof v !== "object") return false;
  const obj = v as Record<string, unknown>;
  const keys = Object.keys(obj);
  if (keys.length !== 1) return false;
  const type = keys[0];
  if (!(VALID_VANTA_TYPES as readonly string[]).includes(type)) return false;
  if (type === "Null") return obj[type] === null || obj[type] === undefined;
  return true;
}

export function isVantaMetadata(m: unknown): m is VantaMetadata {
  if (m === null || typeof m !== "object") return false;
  return Object.values(m).every(isValidVantaValue);
}

export function isValidVector(v: unknown): v is number[] {
  if (!Array.isArray(v)) return false;
  if (v.length === 0) return false;
  return v.every((n) => typeof n === "number" && isFinite(n));
}

export function validateVector(v: unknown): asserts v is Float32Array {
  if (!Array.isArray(v)) {
    throw new TypeError(
      "validateVector: expected an array, got " + typeof v,
    );
  }
  if (v.length === 0) {
    throw new RangeError("validateVector: vector cannot be empty");
  }
  for (let i = 0; i < v.length; i++) {
    if (typeof v[i] !== "number" || !isFinite(v[i])) {
      throw new TypeError(
        `validateVector: invalid or non-finite element at index ${i}`,
      );
    }
  }
}
