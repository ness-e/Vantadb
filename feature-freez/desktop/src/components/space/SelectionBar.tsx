// SelectionBar (ESPACIO-02): barra de batch ops sobre la selección del lasso.
// Presentacional puro — la lógica vive en SpaceLens (handlers) y en el
// undoStore (borrado con snapshot). Estilo manga/neo-brutalista (D4): press,
// border-2, font-tech — consistente con el toolbar de SpaceLens.
// UX-09: el borrado usa el patrón inline de 2 pasos (TrashLens/DeleteButton):
// primer click arma ("¿BORRAR N?"), segundo ejecuta; ✕ cancela. Adiós
// window.confirm nativo (rompía el lenguaje de la app y el teclado).
import { useEffect, useState } from "react";
import { tp, tt, type DesktopLang } from "../../i18n";
import { connectionPrefs } from "../../store/connections";
export type SelectionBusy = "export" | "delete" | null;

interface SelectionBarProps {
  /** Registros seleccionados (derivados de `state.selected` en SpaceLens). */
  count: number;
  /** Operación en vuelo — deshabilita todos los botones. */
  busy: SelectionBusy;
  onExport: () => void;
  onDelete: () => void;
  onClear: () => void;
  lang?: DesktopLang;
}

export default function SelectionBar({
  count,
  busy,
  onExport,
  onDelete,
  onClear,
  lang = connectionPrefs.get().lang ?? "es",
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
      aria-label={tt(lang, "space.sel.groupAria", "Acciones sobre la selección")}
    >
      <span className="font-tech text-[10px] font-bold text-cyan-700" role="status">
        ● {count === 1
          ? tp(lang, "space.sel.selectedOne", "{n} seleccionado", { n: String(count) })
          : tp(lang, "space.sel.selected", "{n} seleccionado(s)", { n: String(count) })}
      </span>
      <button
        type="button"
        className={btn}
        disabled={disabled}
        onClick={onExport}
        title={tp(lang, "space.sel.exportTitle", "Exportar {n} registro(s) como JSONL (importable 1:1)", { n: String(count) })}
      >
        {busy === "export" ? "…" : tt(lang, "space.sel.export", "⭳ exportar (n)")}
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
            title={tt(lang, "space.sel.confirmTitle", "Confirmar borrado (Ctrl+Z deshace)")}
          >
            {busy === "delete" ? "…" : tp(lang, "space.sel.deleteConfirm", "¿BORRAR {n}?", { n: String(count) })}
          </button>
          <button
            type="button"
            className={btn}
            disabled={disabled}
            onClick={() => setArmed(false)}
            aria-label={tt(lang, "space.sel.cancelAria", "Cancelar borrado")}
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
          title={tp(lang, "space.sel.deleteTitle", "Mover {n} registro(s) a la papelera (Ctrl+Z deshace)", { n: String(count) })}
        >
          {busy === "delete" ? "…" : tt(lang, "space.sel.delete", "✕ eliminar (n)")}
        </button>
      )}
      <button
        type="button"
        className={btn}
        disabled={disabled}
        onClick={onClear}
        title={tt(lang, "space.sel.clearTitle", "Limpiar selección (sin borrar nada)")}
      >
        {tt(lang, "space.sel.clear", "✕ limpiar")}
      </button>
    </div>
  );
}