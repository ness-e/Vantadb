// SelectionBar (ESPACIO-02): barra de batch ops sobre la selección del lasso.
// Presentacional puro — la lógica vive en SpaceLens (handlers) y en el
// undoStore (borrado con snapshot). Estilo manga/neo-brutalista (D4): press,
// border-2, font-tech — consistente con el toolbar de SpaceLens.
export type SelectionBusy = "export" | "delete" | null;

interface SelectionBarProps {
  /** Registros seleccionados (derivados de `state.selected` en SpaceLens). */
  count: number;
  /** Operación en vuelo — deshabilita todos los botones. */
  busy: SelectionBusy;
  onExport: () => void;
  onDelete: () => void;
  onClear: () => void;
}

export default function SelectionBar({
  count,
  busy,
  onExport,
  onDelete,
  onClear,
}: SelectionBarProps) {
  if (count === 0) return null;

  const disabled = busy !== null;
  const btn =
    "press border-2 border-foreground bg-background px-2 py-1 text-[10px] font-semibold disabled:opacity-50";

  return (
    <div
      className="flex flex-wrap items-center gap-2 border-b-4 border-foreground bg-neon/10 px-4 py-2"
      role="group"
      aria-label="Acciones sobre la selección"
    >
      <span className="font-tech text-[10px] font-bold text-cyan-700" role="status">
        ● {count} {count === 1 ? "seleccionado" : "seleccionados"}
      </span>
      <button
        type="button"
        className={btn}
        disabled={disabled}
        onClick={onExport}
        title={`Exportar ${count} registro(s) como JSONL (importable 1:1)`}
      >
        {busy === "export" ? "…" : "⭳ exportar (n)"}
      </button>
      <button
        type="button"
        className={`${btn} bg-red-100 hover:bg-red-200`}
        disabled={disabled}
        onClick={onDelete}
        title={`Mover ${count} registro(s) a la papelera (Ctrl+Z deshace)`}
      >
        {busy === "delete" ? "…" : "✕ eliminar (n)"}
      </button>
      <button
        type="button"
        className={btn}
        disabled={disabled}
        onClick={onClear}
        title="Limpiar selección (sin borrar nada)"
      >
        ✕ limpiar
      </button>
    </div>
  );
}