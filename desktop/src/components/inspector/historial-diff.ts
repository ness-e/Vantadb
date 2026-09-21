// Lógica pura del tab HISTORIAL (VS-14): diff entre dos versiones de un
// registro — payload (line-diff LCS), metadata (KV añadido/quitado/cambiado) y
// vector (dimensión/norma + "cambió" sí/no). React-free a propósito: el
// self-check (desktop/scripts/selfcheck-vs14.ts) corre este módulo en node puro
// (type-stripping nativo, node >= 23.6) contra un fixture de 3 versiones en la
// MISMA forma que devuelve el bridge (`versions`/`getVersion` de vanta.ts).
import type { MemoryRecord } from "../../vanta";

// --- Payload: line-diff ------------------------------------------------------

export type DiffLineKind = "ctx" | "add" | "del";

export interface DiffLine {
  kind: DiffLineKind;
  text: string;
}

/** Line-diff por LCS (Myers-lite): líneas comunes = ctx, solo en `before` =
 * del, solo en `after` = add. Payloads de memoria son chicos; O(n·m) alcanza.
 * `""` no produce líneas (ni ctx ni add/del). */
export function diffPayload(before: string, after: string): DiffLine[] {
  const a = before === "" ? [] : before.split("\n");
  const b = after === "" ? [] : after.split("\n");
  const n = a.length;
  const m = b.length;
  const dp: number[][] = Array.from({ length: n + 1 }, () => new Array(m + 1).fill(0));
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      dp[i][j] = a[i] === b[j] ? dp[i + 1][j + 1] + 1 : Math.max(dp[i + 1][j], dp[i][j + 1]);
    }
  }
  const out: DiffLine[] = [];
  let i = 0;
  let j = 0;
  while (i < n && j < m) {
    if (a[i] === b[j]) {
      out.push({ kind: "ctx", text: a[i] });
      i++;
      j++;
    } else if (dp[i + 1][j] >= dp[i][j + 1]) {
      out.push({ kind: "del", text: a[i] });
      i++;
    } else {
      out.push({ kind: "add", text: b[j] });
      j++;
    }
  }
  while (i < n) out.push({ kind: "del", text: a[i++] });
  while (j < m) out.push({ kind: "add", text: b[j++] });
  return out;
}

// --- Metadata: KV diff -------------------------------------------------------

export interface MetaDiff {
  /** Keys presentes solo en `after`. */
  added: string[];
  /** Keys presentes solo en `before`. */
  removed: string[];
  /** Keys en ambas con valor distinto (comparación JSON-plana del wire). */
  changed: Array<{ key: string; before: unknown; after: unknown }>;
}

export function diffMetadata(
  before: Record<string, unknown> | null | undefined,
  after: Record<string, unknown> | null | undefined,
): MetaDiff {
  const b = before ?? {};
  const a = after ?? {};
  const keys = new Set([...Object.keys(b), ...Object.keys(a)]);
  const added: string[] = [];
  const removed: string[] = [];
  const changed: Array<{ key: string; before: unknown; after: unknown }> = [];
  for (const key of keys) {
    const inB = Object.prototype.hasOwnProperty.call(b, key);
    const inA = Object.prototype.hasOwnProperty.call(a, key);
    if (inA && !inB) added.push(key);
    else if (inB && !inA) removed.push(key);
    else if (JSON.stringify(b[key]) !== JSON.stringify(a[key])) {
      changed.push({ key, before: b[key], after: a[key] });
    }
  }
  return { added, removed, changed };
}

// --- Vector ------------------------------------------------------------------

export interface VecSummary {
  dim: number;
  norm: number;
}

/** Dimensión + norma L2 del vector denso; `null` si ausente o vacío. */
export function vecSummary(v: number[] | null | undefined): VecSummary | null {
  if (!v || v.length === 0) return null;
  let sum = 0;
  for (const x of v) sum += x * x;
  return { dim: v.length, norm: Math.sqrt(sum) };
}

function vecsEqual(a: number[], b: number[]): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (Math.abs(a[i] - b[i]) > 1e-6) return false;
  }
  return true;
}

/** "¿El vector cambió entre versiones?" — cambió si aparece/desaparece, si la
 * dimensión difiere, o si algún valor difiere (ε=1e-6). La norma sola no
 * alcanza (dos vectores distintos pueden compartir norma); la UI la muestra
 * como encoding redundante, no como criterio. */
export function vectorChanged(
  a: number[] | null | undefined,
  b: number[] | null | undefined,
): boolean {
  if (!a || !b) return Boolean(a) !== Boolean(b);
  return !vecsEqual(a, b);
}

// --- Composición para el tab -------------------------------------------------

export interface VersionDiff {
  payload: DiffLine[];
  metadata: MetaDiff;
  vecA: VecSummary | null;
  vecB: VecSummary | null;
  vectorChanged: boolean;
}

/** Diff `a` → `b` (git-style: qué cambió de la versión base a la comparada). */
export function diffVersions(a: MemoryRecord, b: MemoryRecord): VersionDiff {
  return {
    payload: diffPayload(a.text, b.text),
    metadata: diffMetadata(a.metadata, b.metadata),
    vecA: vecSummary(a.vector),
    vecB: vecSummary(b.vector),
    vectorChanged: vectorChanged(a.vector, b.vector),
  };
}