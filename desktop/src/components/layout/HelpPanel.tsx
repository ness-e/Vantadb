import { useEffect } from "react";

/**
 * HelpPanel (FIND-25) — first-run usage guide: shortcuts + surfaces, in user
 * language. Opens with the "?" key; Esc or click closes it.
 */

const SHORTCUTS: Array<[string, string]> = [
  ["⌘K / Ctrl+K", "Paleta de comandos"],
  ["Ctrl+Z", "Deshacer (papelera)"],
  ["?", "Esta ayuda"],
];

const SURFACES: Array<[string, string]> = [
  ["RESUMEN", "Vista general: KPIs, namespaces y actividad reciente"],
  ["MEMORIAS", "Explorar, crear y editar registros de memoria"],
  ["BÚSQUEDA", "Búsqueda híbrida (vector + texto) sobre la selección"],
  ["ÍNDICES", "Estado y mantenimiento de índices vectoriales/texto"],
  ["CONSOLIDAR", "Detectar y fusionar duplicados"],
  ["IQL", "Consultas estructuradas al grafo/memoria"],
  ["ESPACIO", "Mapa visual de similitud entre vectores"],
  ["PAPELERA", "Registros borrados: restaurar o purgar"],
];

export function HelpPanel({ onClose }: { onClose: () => void }) {
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    // FIND-25: lightweight in-app usage guide (first-run + "?" reference).
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      onClick={onClose}
    >
      <div
        className="max-h-[80vh] w-[min(640px,92vw)] overflow-y-auto border-4 border-black bg-[var(--background)] p-6 shadow-[8px_8px_0_0_#000]"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-start justify-between">
          <h2 className="font-[family-name:var(--font-anton)] text-2xl uppercase tracking-wide text-[var(--foreground)]">
            Guía rápida
          </h2>
          <button
            aria-label="Cerrar ayuda"
            onClick={onClose}
            className="h-7 w-7 border border-black/30 text-[var(--foreground)] hover:bg-black/10"
          >
            ✕
          </button>
        </div>

        <h3 className="mt-5 font-[family-name:var(--font-space-mono)] text-xs uppercase tracking-widest text-[var(--muted-foreground)]">
          Atajos
        </h3>
        <ul className="mt-2 space-y-1.5">
          {SHORTCUTS.map(([k, d]) => (
            <li key={k} className="flex items-center gap-3 text-sm text-[var(--foreground)]">
              <kbd className="min-w-28 border border-black/40 px-2 py-0.5 text-center font-[family-name:var(--font-space-mono)] text-xs">
                {k}
              </kbd>
              {d}
            </li>
          ))}
        </ul>

        <h3 className="mt-6 font-[family-name:var(--font-space-mono)] text-xs uppercase tracking-widest text-[var(--muted-foreground)]">
          Superficies
        </h3>
        <ul className="mt-2 space-y-1.5">
          {SURFACES.map(([name, d]) => (
            <li key={name} className="text-sm text-[var(--foreground)]">
              <span className="font-[family-name:var(--font-space-mono)] text-xs font-bold uppercase tracking-wide">
                {name}
              </span>{" "}
              — {d}
            </li>
          ))}
        </ul>

        <p className="mt-6 border-t border-black/20 pt-3 text-xs text-[var(--muted-foreground)]">
          Tip: presioná <span className="font-[family-name:var(--font-space-mono)]">?</span> en
          cualquier momento para volver a esta guía.
        </p>
      </div>
    </div>
  );
}
