import { useEffect, useState } from "react";

/**
 * HelpPanel (FIND-25) — first-run usage guide: shortcuts + surfaces, in user
 * language. Opens with the "?" key; Esc or click closes it.
 */

export type HelpTab = "general" | "proxy";

const SHORTCUTS: Array<[string, string]> = [
  ["⌘K / Ctrl+K", "Paleta de comandos"],
  ["Ctrl+Z", "Deshacer (papelera)"],
  ["F1", "Ayuda general (esta guía)"],
  ["F2", "Ayuda proxy / ajustes (contextual)"],
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
  ["ACTIVIDAD", "Audit log: actividad y timeline de cambios"],
  ["MEMORIA", "Escenas con heat, persona, skills y generation log"],
  ["PROXY", "Dashboard proxy local: TurnReports, sesiones, write-back, rate-limit"],
  ["AJUSTES", "Perfiles de conexión, defaults de búsqueda e idioma"],
];

export function HelpPanel({ onClose, initialTab = "general" }: { onClose: () => void; initialTab?: HelpTab }) {
  const [tab, setTab] = useState<HelpTab>(initialTab);
  useEffect(() => setTab(initialTab), [initialTab]);
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

        <div className="mt-4 flex gap-2" role="tablist" aria-label="Secciones de ayuda">
          <button
            role="tab"
            aria-selected={tab === "general"}
            onClick={() => setTab("general")}
            className={`press border-2 border-foreground px-3 py-1 text-xs font-semibold ${tab === "general" ? "bg-foreground text-background" : "bg-background"}`}
          >
            General (F1)
          </button>
          <button
            role="tab"
            aria-selected={tab === "proxy"}
            onClick={() => setTab("proxy")}
            className={`press border-2 border-foreground px-3 py-1 text-xs font-semibold ${tab === "proxy" ? "bg-foreground text-background" : "bg-background"}`}
          >
            Proxy / Ajustes (F2)
          </button>
        </div>

        {tab === "proxy" ? (
          <>
            <h3 className="mt-5 font-[family-name:var(--font-space-mono)] text-xs uppercase tracking-widest text-[var(--muted-foreground)]">
              Proxy local
            </h3>
            <ul className="mt-2 space-y-1.5 text-sm text-[var(--foreground)]">
              <li>• <span className="font-bold">TurnReports:</span> registro de turnos del upstream LLM (request/response + latencia)</li>
              <li>• <span className="font-bold">Sesiones activas:</span> conexiones proxy abiertas y su estado</li>
              <li>• <span className="font-bold">Cola write-back:</span> escrituras pendientes hacia VantaDB</li>
              <li>• <span className="font-bold">Rate-limit:</span> límites del upstream y backoff</li>
              <li className="text-xs text-[var(--muted-foreground)]">Configurá la URL del proxy en AJUSTES → Proxy. F2 abre este tab contextual.</li>
            </ul>
            <h3 className="mt-5 font-[family-name:var(--font-space-mono)] text-xs uppercase tracking-widest text-[var(--muted-foreground)]">
              Ajustes
            </h3>
            <ul className="mt-2 space-y-1.5 text-sm text-[var(--foreground)]">
              <li>• Perfiles de conexión (native / server + Bearer)</li>
              <li>• Defaults de búsqueda (topK) y preferencias de workspace</li>
              <li>• Selector de tema claro/oscuro</li>
            </ul>
          </>
        ) : null}
        {tab === "general" && (
          <>
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
          </>
        )}
      </div>
    </div>
  );
}
