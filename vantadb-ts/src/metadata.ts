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

export function normalizeValue(v: unknown): Value {
  if (v === null) return { Null: null };
  switch (typeof v) {
    case "string":
      return { String: v };
    case "boolean":
      return { Bool: v };
    case "number":
      return Number.isInteger(v) ? { Int: v } : { Float: v };
    default:
      // Already in tagged wire form ({ String, Int, Float, ... }) — pass through.
      return v as Value;
  }
}
