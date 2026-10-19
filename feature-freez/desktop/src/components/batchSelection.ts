// Batch selection ops (OP-02): selección múltiple del grid por key
// `${namespace}:${id}` — mismo keying que getRowId de TanStack (DataExplorer).
// Módulo puro (React-free) para testear toggle/select-all sin RTL.
import type { MemoryRecord } from "../vanta";

/** Key estable de fila para la selección (coincide con getRowId del grid). */
export function rowKey(record: MemoryRecord): string {
  return `${record.namespace}:${record.id}`;
}

/** Toggle de un id dentro de la selección → NUEVO Set (immutable). */
export function toggleId(selected: ReadonlySet<string>, id: string): Set<string> {
  const next = new Set(selected);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  return next;
}

/** Select-all sobre ids dados: si ya están todos seleccionados, limpia; si
 * falta alguno, selecciona todos. Devuelve NUEVO Set. Aplica a la "página
 * actual" (filas cargadas), no al namespace completo. */
export function selectAll(selected: ReadonlySet<string>, ids: readonly string[]): Set<string> {
  return ids.every((id) => selected.has(id)) ? new Set() : new Set(ids);
}