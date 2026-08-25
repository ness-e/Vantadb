// Metadata tab (VS-06): editor KV con tipo inferido de VantaValue
// (str/int/flt/bool/date/lst/nil). Filas add/remove; validación y serialización
// viven en `rowsToMetadata` (shared.ts) — el tab es presentacional.
import { MetaRow, MetaType } from "./shared";
import { TriangleAlert } from "lucide-react";

interface Props {
  rows: MetaRow[];
  setRows: (rows: MetaRow[]) => void;
}

const TYPES: MetaType[] = ["str", "int", "flt", "bool", "date", "lst", "nil"];

function updateRow(rows: MetaRow[], i: number, patch: Partial<MetaRow>): MetaRow[] {
  return rows.map((r, idx) => (idx === i ? { ...r, ...patch } : r));
}

export default function MetadataTab({ rows, setRows }: Props) {
  // Keys duplicadas (después de trim) → highlight + bloqueo de guardado.
  const counts = new Map<string, number>();
  for (const r of rows) {
    const k = r.key.trim();
    if (k) counts.set(k, (counts.get(k) ?? 0) + 1);
  }
  const isDup = (k: string) => (counts.get(k.trim()) ?? 0) > 1;

  function addRow() {
    setRows([...rows, { key: "", type: "str", raw: "" }]);
  }
  function removeRow(i: number) {
    setRows(rows.filter((_, idx) => idx !== i));
  }

  if (rows.length === 0) {
    return (
      <div>
        <p className="font-tech text-[11px] text-muted-foreground">
          sin metadata — agregá una fila
        </p>
        <button
          type="button"
          onClick={addRow}
          className="press mt-3 border-2 border-foreground bg-background px-3 py-1.5 text-xs font-semibold"
        >
          + AGREGAR FILA
        </button>
      </div>
    );
  }

  return (
    <div>
      <div className="space-y-1.5">
        {rows.map((row, i) => (
          <div key={i} className="grid grid-cols-[64px_1fr_1fr_24px] gap-1.5 items-center">
            <select
              value={row.type}
              onChange={(e) => setRows(updateRow(rows, i, { type: e.target.value as MetaType }))}
              aria-label={`Tipo de ${row.key || `fila ${i + 1}`}`}
              className="border-2 border-foreground bg-background px-1 py-1 font-tech text-[10px]"
            >
              {TYPES.map((t) => (
                <option key={t} value={t}>
                  {t}
                </option>
              ))}
            </select>
            <span className="relative block">
              <input
                value={row.key}
                onChange={(e) => setRows(updateRow(rows, i, { key: e.target.value }))}
                placeholder="key"
                aria-label="Key de metadata"
                aria-invalid={isDup(row.key) || undefined}
                title={isDup(row.key) ? "Key duplicada — el guardado se bloquea hasta corregirla" : undefined}
                className={`w-full border-2 bg-background px-1.5 py-1 pr-5 font-mono text-[11px] ${
                  isDup(row.key) ? "border-neon text-foreground" : "border-foreground"
                }`}
              />
              {/* VS-18/P15: dup no es solo borde neon — ícono alerta + title (AA:
                  texto foreground, borde neon como acento no-texto 3:1). */}
              {isDup(row.key) && (
                <span
                  aria-hidden="true"
                  className="pointer-events-none absolute right-1 top-1/2 -translate-y-1/2 leading-none"
                >
                  <TriangleAlert className="h-3 w-3" strokeWidth={2.5} />
                </span>
              )}
            </span>
            {row.type === "nil" ? (
              <span className="border-2 border-dashed border-foreground px-1.5 py-1 font-tech text-[10px] text-muted-foreground">
                null
              </span>
            ) : row.type === "bool" ? (
              <select
                value={row.raw}
                onChange={(e) => setRows(updateRow(rows, i, { raw: e.target.value }))}
                aria-label="Valor booleano"
                className="border-2 border-foreground bg-background px-1 py-1 font-tech text-[11px]"
              >
                <option value="true">true</option>
                <option value="false">false</option>
              </select>
            ) : (
              <input
                value={row.raw}
                onChange={(e) => setRows(updateRow(rows, i, { raw: e.target.value }))}
                placeholder={row.type === "lst" ? '["a","b"]' : row.type === "date" ? "AAAA-MM-DDTHH:mm" : "valor"}
                aria-label={`Valor de ${row.key || `fila ${i + 1}`}`}
                type={row.type === "int" || row.type === "flt" ? "number" : row.type === "date" ? "datetime-local" : "text"}
                step={row.type === "int" ? 1 : "any"}
                className="border-2 border-foreground bg-background px-1.5 py-1 font-mono text-[11px]"
              />
            )}
            <button
              type="button"
              onClick={() => removeRow(i)}
              aria-label={`Quitar fila ${row.key || i + 1}`}
              className="press flex h-6 w-6 items-center justify-center border-2 border-foreground text-xs"
            >
              ✕
            </button>
          </div>
        ))}
      </div>
      <button
        type="button"
        onClick={addRow}
        className="press mt-3 border-2 border-foreground bg-background px-3 py-1.5 text-xs font-semibold"
      >
        + AGREGAR FILA
      </button>
    </div>
  );
}