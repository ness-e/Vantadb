// Filters core (VS-07): lógica pura de filtros compuestos por metadata.
//
// Separado de FiltersBuilder.tsx para que el shell NO pague react-querybuilder
// en el bundle inicial (mismo patrón lazy del Inspector en VS-03): el UI pesa
// ~200 kB y solo se abre on-demand; estas funciones son las que el shell usa
// en cada render (inferir campos, evaluar resultados).
//
// Tipos y semántica espejan al core `src/sdk/types.rs`:
//   - VantaMemoryFilter = Vec<VantaMemoryFilterItem{field, op, value}> con
//     ops Eq/Neq/Gt/Lt/Gte/Lte (combina con AND).
//   - El wire del bridge desktop (`SearchQuery.filters`) solo admite el map
//     plano Eq → VS-07 aplica el árbol completo client-side (evaluateQuery).
import type { RuleGroupType, RuleType } from "react-querybuilder";

export type MetaFieldType = "string" | "int" | "float" | "bool" | "datetime";

export interface MetaField {
  name: string;
  type: MetaFieldType;
}

/** Un ítem del `VantaMemoryFilter` del core (src/sdk/types.rs:107-127). */
export interface VantaFilterItem {
  field: string;
  op: "Eq" | "Neq" | "Gt" | "Lt" | "Gte" | "Lte";
  value: unknown;
}

/** Query inicial: sin restricciones (AND vacío). */
export const EMPTY_QUERY: RuleGroupType = { combinator: "and", rules: [] };

// --- Inferencia de tipos de metadata ----------------------------------------

function inferType(v: unknown): MetaFieldType | null {
  if (typeof v === "boolean") return "bool";
  if (typeof v === "number") return Number.isInteger(v) ? "int" : "float";
  if (typeof v === "string" && /^\d{4}-\d{2}-\d{2}T/.test(v) && !Number.isNaN(Date.parse(v))) {
    return "datetime";
  }
  if (typeof v === "string") return "string";
  return null; // null/arrays/objects → no son campos filtrables
}

/** Infiere el esquema de metadata desde registros reales (unión de campos;
 * primer tipo concreto que se ve por campo; datetime si el string es RFC3339).
 * ponytail: heuristic, no schema registry — swap cuando el core exponga tipos. */
export function inferMetaFields(
  records: Array<{ metadata?: Record<string, unknown> | null }>,
): MetaField[] {
  const fields = new Map<string, MetaFieldType>();
  for (const rec of records) {
    for (const [name, value] of Object.entries(rec.metadata ?? {})) {
      if (fields.has(name)) continue;
      const t = inferType(value);
      if (t) fields.set(name, t);
    }
  }
  return [...fields.entries()]
    .map(([name, type]) => ({ name, type }))
    .sort((a, b) => a.name.localeCompare(b.name));
}

// --- Serialización a VantaMemoryFilter ---------------------------------------

const OP_TO_FILTER: Record<string, VantaFilterItem["op"]> = {
  "=": "Eq",
  "!=": "Neq",
  "<": "Lt",
  ">": "Gt",
  "<=": "Lte",
  ">=": "Gte",
};

/**
 * Serializa el árbol de reglas a `VantaMemoryFilter` (lista AND). Cada hoja
 * (field, op, value) → {field, op, value} con ops mapeadas 1:1 a VantaFilterOp.
 * Reglas sin campo/op o con valor vacío se omiten (son inválidas en el builder).
 * NOTA: el core combina con AND — los grupos OR del builder se aplanan; para
 * preservar OR usá `evaluateQuery()` en el lado de la UI.
 */
export function toVantaMemoryFilter(query: RuleGroupType): VantaFilterItem[] {
  const items: VantaFilterItem[] = [];
  const walk = (g: RuleGroupType): void => {
    for (const r of g.rules) {
      if ("rules" in r) {
        walk(r as RuleGroupType);
        continue;
      }
      const rule = r as RuleType;
      const op = OP_TO_FILTER[rule.operator];
      const empty = rule.value === "" || rule.value === undefined || rule.value === null;
      if (rule.field && op && !empty) {
        items.push({ field: rule.field, op, value: rule.value });
      }
    }
  };
  walk(query);
  return items;
}

// --- Evaluación client-side del árbol completo (AND/OR) -----------------------

/** Tipo de comparación inferido del valor REAL (no del schema declarado). */
function cmpKind(v: unknown): "number" | "string" | "bool" | "datetime" | null {
  if (typeof v === "boolean") return "bool";
  if (typeof v === "number") return "number";
  if (typeof v === "string") return /^\d{4}-\d{2}-\d{2}T/.test(v) ? "datetime" : "string";
  return null;
}

/** Compara dos valores; devuelve -1/0/1 o null si no son comparables. */
function compareValues(a: unknown, b: unknown): number | null {
  const kind = cmpKind(a) ?? cmpKind(b);
  if (kind === null) return null;
  if (kind === "bool") {
    if (typeof a !== "boolean" || typeof b !== "boolean") return null;
    return a === b ? 0 : a ? 1 : -1;
  }
  if (kind === "number") {
    const na = Number(a);
    const nb = Number(b);
    if (!Number.isFinite(na) || !Number.isFinite(nb)) return null;
    return na < nb ? -1 : na > nb ? 1 : 0;
  }
  if (kind === "datetime") {
    const ta = Date.parse(String(a));
    const tb = Date.parse(String(b));
    if (Number.isNaN(ta) || Number.isNaN(tb)) return null;
    return ta < tb ? -1 : ta > tb ? 1 : 0;
  }
  const sa = String(a);
  const sb = String(b);
  return sa < sb ? -1 : sa > sb ? 1 : 0;
}

function compareRule(actual: unknown, op: string, want: unknown): boolean {
  // Listas: matchea si ALGÚN elemento cumple (core aplana con to_index_values).
  if (Array.isArray(actual)) return actual.some((v) => compareRule(v, op, want));
  const c = compareValues(actual, want);
  if (c === null) return false;
  switch (op) {
    case "=":
      return c === 0;
    case "!=":
      return c !== 0;
    case "<":
      return c < 0;
    case ">":
      return c > 0;
    case "<=":
      return c <= 0;
    case ">=":
      return c >= 0;
    default:
      return false;
  }
}

/**
 * Evalúa el árbol de reglas (AND/OR, anidado) contra el metadata de un
 * registro. Un grupo vacío = sin restricción (true). Un campo ausente NO
 * matchea. Una hoja con valor vacío = inválida → no matchea.
 */
export function evaluateQuery(query: RuleGroupType, metadata: Record<string, unknown>): boolean {
  if (!query.rules.length) return true;
  const every = query.combinator !== "or";
  const verdicts: boolean[] = [];
  for (const r of query.rules) {
    if ("rules" in r) {
      verdicts.push(evaluateQuery(r as RuleGroupType, metadata));
      continue;
    }
    const rule = r as RuleType;
    const v = rule.value;
    const empty = v === undefined || v === null || (typeof v === "string" && v.trim() === "");
    verdicts.push(empty ? false : compareRule(metadata[rule.field], rule.operator, v));
  }
  return every ? verdicts.every(Boolean) : verdicts.some(Boolean);
}