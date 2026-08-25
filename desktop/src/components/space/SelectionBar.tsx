// SelectionBar (ESPACIO-02): barra de batch ops sobre la selección del lasso.
// Presentacional puro — la lógica vive en SpaceLens (handlers) y en el
// undoStore (borrado con snapshot). Estilo manga/neo-brutalista (D4): press,
// border-2, font-tech — consistente con el toolbar de SpaceLens.
// UX-09: el borrado usa el patrón inline de 2 pasos (TrashLens/DeleteButton):
// primer click arma ("¿BORRAR N?"), segundo ejecuta; ✕ cancela. Adiós
// window.confirm nativo (rompía el lenguaje de la app y el teclado).
import { useEffect, useState } from "react";
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
  // UX-09: confirmación inline armada (patrón DeleteButton/TrashLens).
  const [armed, setArmed] = useState(false);
  useEffect(() => {
    if (count === 0 || busy === "delete") setArmed(false);
  }, [count, busy]);

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
      {armed ? (
        <>
          <button
            type="button"
            className={`${btn} bg-red-100 font-bold hover:bg-red-200`}
            disabled={disabled}
            onClick={() => {
              setArmed(false);
              onDelete();
            }}
            title="Confirmar borrado (Ctrl+Z deshace)"
          >
            {busy === "delete" ? "…" : `¿BORRAR ${count}?`}
          </button>
          <button
            type="button"
            className={btn}
            disabled={disabled}
            onClick={() => setArmed(false)}
            aria-label="Cancelar borrado"
          >
            ✕
          </button>
        </>
      ) : (
        <button
          type="button"
          className={`${btn} bg-red-100 hover:bg-red-200`}
          disabled={disabled}
          onClick={() => setArmed(true)}
          title={`Mover ${count} registro(s) a la papelera (Ctrl+Z deshace)`}
        >
          {busy === "delete" ? "…" : "✕ eliminar (n)"}
        </button>
      )}
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